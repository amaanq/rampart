//! Passkey / Webauthn flows via `webauthn-rs` high-level API.
//!
//! sign_count regression is logged as a warning rather than treated as
//! an attack (synced passkeys legitimately non-monotonic).

use anyhow::{Context, Result};
use rampart_codegen::queries::{users, webauthn};
use rand::TryRngCore;
use time::{Duration, OffsetDateTime};
use webauthn_rs::prelude::*;

pub fn build(cfg: &crate::config::Config) -> Result<Webauthn> {
    let rp_id = cfg.webauthn_rp_id.as_str();
    let origin = url::Url::parse(&cfg.public_origin).context("public_origin parse")?;
    WebauthnBuilder::new(rp_id, &origin)?
        .rp_name("rampart")
        .build()
        .context("webauthn build")
}

fn new_ceremony_id() -> Vec<u8> {
    let mut buf = [0u8; 16];
    rand::rngs::OsRng
        .try_fill_bytes(&mut buf)
        .expect("OsRng must not fail");
    buf.to_vec()
}

pub async fn save_registration_state(
    pool: &deadpool_postgres::Pool,
    user_id: i64,
    state: &PasskeyRegistration,
) -> Result<Vec<u8>> {
    let id = new_ceremony_id();
    let blob = serde_json::to_vec(state)?;
    let expires = OffsetDateTime::now_utc() + Duration::minutes(10);
    let c = pool.get().await?;
    webauthn::ceremony_insert_register()
        .bind(&c, &id, &Some(user_id), &blob, &expires)
        .await?;
    Ok(id)
}

pub async fn save_authentication_state(
    pool: &deadpool_postgres::Pool,
    user_id: Option<i64>,
    state: &PasskeyAuthentication,
) -> Result<Vec<u8>> {
    let id = new_ceremony_id();
    let blob = serde_json::to_vec(state)?;
    let expires = OffsetDateTime::now_utc() + Duration::minutes(10);
    let c = pool.get().await?;
    webauthn::ceremony_insert_auth()
        .bind(&c, &id, &user_id, &blob, &expires)
        .await?;
    Ok(id)
}

pub async fn load_registration_state(
    pool: &deadpool_postgres::Pool,
    id: &[u8],
    user_id: i64,
) -> Result<PasskeyRegistration> {
    let c = pool.get().await?;
    let id_vec = id.to_vec();
    let blob = webauthn::ceremony_consume_register()
        .bind(&c, &id_vec, &user_id)
        .opt()
        .await?
        .context("registration ceremony not found or expired")?;
    Ok(serde_json::from_slice(&blob)?)
}

pub async fn load_authentication_state(
    pool: &deadpool_postgres::Pool,
    id: &[u8],
) -> Result<PasskeyAuthentication> {
    let c = pool.get().await?;
    let id_vec = id.to_vec();
    let blob = webauthn::ceremony_consume_auth()
        .bind(&c, &id_vec)
        .opt()
        .await?
        .context("authentication ceremony not found or expired")?;
    Ok(serde_json::from_slice(&blob)?)
}

pub async fn load_passkeys_for_user(
    pool: &deadpool_postgres::Pool,
    user_id: i64,
) -> Result<Vec<Passkey>> {
    let c = pool.get().await?;
    let blobs = webauthn::credentials_for_user()
        .bind(&c, &user_id)
        .all()
        .await?;
    let mut out = Vec::with_capacity(blobs.len());
    for blob in blobs {
        if let Ok(p) = serde_json::from_slice::<Passkey>(&blob) {
            out.push(p);
        }
    }
    Ok(out)
}

pub async fn load_passkeys_for_email(
    pool: &deadpool_postgres::Pool,
    email: &str,
) -> Result<(i64, Vec<Passkey>)> {
    let c = pool.get().await?;
    let Some(user_id) = users::by_email_id().bind(&c, &email).opt().await? else {
        anyhow::bail!("no such user");
    };
    let passkeys = load_passkeys_for_user(pool, user_id).await?;
    if passkeys.is_empty() {
        anyhow::bail!("no passkeys registered for this user");
    }
    Ok((user_id, passkeys))
}

pub async fn insert_credential(
    pool: &deadpool_postgres::Pool,
    user_id: i64,
    name: &str,
    passkey: &Passkey,
) -> Result<()> {
    let blob = serde_json::to_vec(passkey)?;
    let cred_id: Vec<u8> = passkey.cred_id().as_ref().to_vec();
    let c = pool.get().await?;
    webauthn::credential_insert()
        .bind(&c, &user_id, &cred_id, &blob, &name)
        .await?;
    Ok(())
}

/// Persist the updated Passkey blob and sign counter together after a
/// successful authentication.
pub async fn update_credential_after_auth(
    pool: &deadpool_postgres::Pool,
    cred_id: &[u8],
    auth_result: &webauthn_rs::prelude::AuthenticationResult,
) -> Result<()> {
    let new_counter = auth_result.counter();
    let mut c = pool.get().await?;
    let txn = c.transaction().await?;

    let cred_id_vec = cred_id.to_vec();
    let row = webauthn::credential_for_update()
        .bind(&txn, &cred_id_vec)
        .opt()
        .await?
        .context("credential not found for blob update")?;
    let prev_count = row.sign_count;
    let blob = row.credential_blob;

    // WebAuthn L2 §6.1.2: when the authenticator supports a sign counter
    // (ever returns a non-zero value), `new <= stored` signals a cloned
    // authenticator. Synced passkeys report counter=0 forever so only
    // enforce when both sides are non-zero.
    if prev_count > 0 && new_counter > 0 && (new_counter as i32) <= prev_count {
        tracing::warn!(
            prev = prev_count,
            new = new_counter,
            "webauthn sign_count rollback — possible cloned authenticator"
        );
        return Err(anyhow::anyhow!("sign_count rollback"));
    }

    let new_blob = match serde_json::from_slice::<Passkey>(&blob) {
        Ok(mut pk) => {
            pk.update_credential(auth_result);
            match serde_json::to_vec(&pk) {
                Ok(b) => Some(b),
                Err(e) => {
                    tracing::warn!(error = ?e, "re-serialize Passkey failed; counter-only update");
                    None
                }
            }
        }
        Err(e) => {
            tracing::warn!(error = ?e, "deserialize Passkey failed; counter-only update");
            None
        }
    };

    let counter_i32 = new_counter as i32;
    if let Some(blob) = new_blob {
        webauthn::credential_update_blob_and_count()
            .bind(&txn, &counter_i32, &blob, &cred_id_vec)
            .await?;
    } else {
        webauthn::credential_update_count_only()
            .bind(&txn, &counter_i32, &cred_id_vec)
            .await?;
    }

    txn.commit().await?;
    Ok(())
}
