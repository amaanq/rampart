//! Passkey / Webauthn flows via `webauthn-rs` high-level API.
//!
//! `sign_count` regression is logged as a warning rather than treated as
//! an attack (synced passkeys legitimately non-monotonic).

use anyhow::{
   Context as _,
   Result,
};
use rampart_codegen::queries::{
   users,
   webauthn,
};
use rand::{
   TryRng as _,
   rngs::SysRng,
};
use time::{
   Duration,
   OffsetDateTime,
};
use webauthn_rs::prelude::*;

use crate::config::Config;

pub fn build(cfg: &Config) -> Result<Webauthn> {
   let rp_id = cfg.webauthn_rp_id.as_str();
   let origin = url::Url::parse(&cfg.public_origin).context("public_origin parse")?;
   WebauthnBuilder::new(rp_id, &origin)?
      .rp_name("rampart")
      .build()
      .context("webauthn build")
}

pub fn user_handle(user_id: i64) -> Uuid {
   let mut hasher = hmac_sha256::Hash::new();
   hasher.update(b"rampart-user-");
   #[expect(
      clippy::big_endian_bytes,
      reason = "stable cross-platform user handle derivation"
   )]
   hasher.update(user_id.to_be_bytes());
   let digest = hasher.finalize();
   let bytes: [u8; 16] = digest[..16].try_into().unwrap();
   Uuid::from_bytes(bytes)
}

fn new_ceremony_id() -> Vec<u8> {
   let mut buf = [0_u8; 16];
   SysRng
      .try_fill_bytes(&mut buf)
      .expect("SysRng must not fail");
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
   let conn = pool.get().await?;
   webauthn::ceremony_insert_register()
      .bind(&conn, &id, &Some(user_id), &blob, &expires)
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
   let conn = pool.get().await?;
   webauthn::ceremony_insert_auth()
      .bind(&conn, &id, &user_id, &blob, &expires)
      .await?;
   Ok(id)
}

pub async fn save_discoverable_authentication_state(
   pool: &deadpool_postgres::Pool,
   state: &DiscoverableAuthentication,
) -> Result<Vec<u8>> {
   let id = new_ceremony_id();
   let blob = serde_json::to_vec(state)?;
   let expires = OffsetDateTime::now_utc() + Duration::minutes(10);
   let conn = pool.get().await?;
   webauthn::ceremony_insert_auth()
      .bind(&conn, &id, &None, &blob, &expires)
      .await?;
   Ok(id)
}

pub async fn load_registration_state(
   pool: &deadpool_postgres::Pool,
   id: &[u8],
   user_id: i64,
) -> Result<PasskeyRegistration> {
   let conn = pool.get().await?;
   let id_vec = id.to_vec();
   let blob = webauthn::ceremony_consume_register()
      .bind(&conn, &id_vec, &user_id)
      .opt()
      .await?
      .context("registration ceremony not found or expired")?;
   Ok(serde_json::from_slice(&blob)?)
}

pub async fn load_authentication_state(
   pool: &deadpool_postgres::Pool,
   id: &[u8],
) -> Result<PasskeyAuthentication> {
   let conn = pool.get().await?;
   let id_vec = id.to_vec();
   let blob = webauthn::ceremony_consume_auth()
      .bind(&conn, &id_vec)
      .opt()
      .await?
      .context("authentication ceremony not found or expired")?;
   Ok(serde_json::from_slice(&blob)?)
}

pub async fn load_discoverable_authentication_state(
   pool: &deadpool_postgres::Pool,
   id: &[u8],
) -> Result<DiscoverableAuthentication> {
   let conn = pool.get().await?;
   let id_vec = id.to_vec();
   let blob = webauthn::ceremony_consume_auth()
      .bind(&conn, &id_vec)
      .opt()
      .await?
      .context("authentication ceremony not found or expired")?;
   Ok(serde_json::from_slice(&blob)?)
}

pub async fn load_passkeys_for_user(
   pool: &deadpool_postgres::Pool,
   user_id: i64,
) -> Result<Vec<Passkey>> {
   let conn = pool.get().await?;
   let blobs = webauthn::credentials_for_user()
      .bind(&conn, &user_id)
      .all()
      .await?;
   let mut out = Vec::with_capacity(blobs.len());
   for blob in blobs {
      if let Ok(passkey) = serde_json::from_slice::<Passkey>(&blob) {
         out.push(passkey);
      }
   }
   Ok(out)
}

pub async fn load_passkeys_for_email(
   pool: &deadpool_postgres::Pool,
   email: &str,
) -> Result<(i64, Vec<Passkey>)> {
   let conn = pool.get().await?;
   let Some(user_id) = users::by_email_id().bind(&conn, &email).opt().await? else {
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
   let conn = pool.get().await?;
   webauthn::credential_insert()
      .bind(&conn, &user_id, &cred_id, &blob, &name)
      .await?;
   Ok(())
}

/// Persist the updated Passkey blob and sign counter together after a
/// successful authentication.
#[expect(
   clippy::cognitive_complexity,
   reason = "linear ceremony post-processing kept in one place"
)]
pub async fn update_credential_after_auth(
   pool: &deadpool_postgres::Pool,
   cred_id: &[u8],
   auth_result: &AuthenticationResult,
) -> Result<()> {
   let new_counter = auth_result.counter();
   let mut conn = pool.get().await?;
   let txn = conn.transaction().await?;

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
   if prev_count > 0 && new_counter > 0 && i64::from(new_counter) <= i64::from(prev_count) {
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
            Ok(bytes) => Some(bytes),
            Err(err) => {
               tracing::warn!(error = ?err, "re-serialize Passkey failed; counter-only update");
               None
            },
         }
      },
      Err(err) => {
         tracing::warn!(error = ?err, "deserialize Passkey failed; counter-only update");
         None
      },
   };

   #[expect(
      clippy::cast_possible_wrap,
      reason = "webauthn sign counters fit in i32; DB column is int4"
   )]
   let counter_i32 = new_counter as i32;
   if let Some(blob_bytes) = new_blob {
      webauthn::credential_update_blob_and_count()
         .bind(&txn, &counter_i32, &blob_bytes, &cred_id_vec)
         .await?;
   } else {
      webauthn::credential_update_count_only()
         .bind(&txn, &counter_i32, &cred_id_vec)
         .await?;
   }

   txn.commit().await?;
   Ok(())
}
