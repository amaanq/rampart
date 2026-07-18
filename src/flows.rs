//! Token-based flows: password reset, email change, mailbox verify.
//!
//! Shape is identical across flows:
//!   1. `start_X`: generate token, insert row, send email.
//!   2. `apply_X`: on POST from confirmation page, validate token,
//!      apply side effect, mark token used.
//!
//! Tokens are 24 random bytes, base64url-encoded for the URL; stored as
//! sha256 so a DB leak doesn't yield click-through credentials.

use anyhow::{Context, Result};
use data_encoding::BASE64URL_NOPAD;
use deadpool_postgres::Pool;
use hmac_sha256::Hash;
use rampart_codegen::queries::{mailboxes, sessions, tokens, users};
use rand::TryRngCore;
use time::{Duration, OffsetDateTime};

use crate::auth::hash_password;
use crate::mailer::Mailer;

pub const DEFAULT_TOKEN_TTL_HOURS: i64 = 1;

#[derive(Debug, thiserror::Error)]
pub enum InviteSignupError {
    #[error("password must be at least 10 characters")]
    PasswordTooShort,
    #[error("invite is invalid")]
    Invalid,
    #[error("invite has expired")]
    Expired,
    #[error("invite has already been used")]
    AlreadyUsed,
    #[error("invite is tied to a different email")]
    EmailMismatch,
    #[error("email already registered")]
    AlreadyRegistered,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PasswordResetError {
    #[error("password must be at least 10 characters")]
    PasswordTooShort,
    #[error("password reset token is invalid")]
    Invalid,
    #[error("password reset token has expired")]
    Expired,
    #[error("password reset token has already been used")]
    AlreadyUsed,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EmailChangeError {
    #[error("email change token is invalid")]
    Invalid,
    #[error("email change token has expired")]
    Expired,
    #[error("email change token has already been used")]
    AlreadyUsed,
    #[error("email already registered")]
    AlreadyRegistered,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

fn generate_token() -> (String, Vec<u8>) {
    let mut bytes = [0u8; 24];
    rand::rngs::OsRng
        .try_fill_bytes(&mut bytes)
        .expect("OsRng must not fail");
    let token = BASE64URL_NOPAD.encode(&bytes);
    let hash = Hash::hash(token.as_bytes()).to_vec();
    (token, hash)
}

fn hash_token(token: &str) -> Vec<u8> {
    Hash::hash(token.as_bytes()).to_vec()
}

/// Advisory-lock key serializing concurrent /setup POSTs. INSERT ... WHERE
/// NOT EXISTS under READ COMMITTED only guarantees per-statement atomicity;
/// two concurrent statements take independent snapshots, both see an empty
/// user table, both insert. We funnel them through `pg_advisory_xact_lock`.
pub const FIRST_ADMIN_LOCK_KEY: i64 = 0x52414D505F464131; // "RAMP_FA1"

/// First-run admin bootstrap. Creates a user with `is_admin = true` only
/// if no user exists yet. Race-safe via `FIRST_ADMIN_LOCK_KEY` —
/// concurrent setup POSTs serialize against the lock, then the inner
/// `INSERT ... WHERE NOT EXISTS` guard returns `Ok(None)` for the loser.
pub async fn bootstrap_first_admin(
    pool: &Pool,
    email: &str,
    password: &str,
    display_name: Option<&str>,
) -> anyhow::Result<Option<i64>> {
    if password.len() < 10 {
        anyhow::bail!("password must be at least 10 characters");
    }
    let password_hash = hash_password(password)?;
    let mut c = pool.get().await?;
    let txn = c.transaction().await?;
    txn.execute("SET LOCAL idle_in_transaction_session_timeout = 0", &[])
        .await
        .context("disable idle-in-txn timeout for first-admin lock")?;
    txn.execute(
        "SELECT pg_advisory_xact_lock($1::bigint)",
        &[&FIRST_ADMIN_LOCK_KEY],
    )
    .await
    .context("pg_advisory_xact_lock for first-admin")?;
    let id = users::create_first_admin()
        .bind(&txn, &email, &password_hash, &display_name)
        .opt()
        .await
        .context("inserting first admin")?;
    txn.commit().await?;
    Ok(id)
}

pub async fn claim_invite_and_create_user(
    pool: &Pool,
    token: &str,
    email: &str,
    password: &str,
    display_name: Option<&str>,
) -> std::result::Result<(i64, bool), InviteSignupError> {
    if password.len() < 10 {
        return Err(InviteSignupError::PasswordTooShort);
    }
    let token_hash = hash_token(token);
    let mut c = pool.get().await.context("opening invite transaction")?;
    let txn = c
        .transaction()
        .await
        .context("starting invite transaction")?;

    let claimed = tokens::invite_claim()
        .bind(&txn, &token_hash, &email)
        .opt()
        .await
        .context("claiming invite")?;
    if claimed.is_none() {
        let failure = tokens::invite_failure()
            .bind(&txn, &token_hash, &email)
            .opt()
            .await
            .context("inspecting invite failure")?;
        txn.rollback().await.ok();
        return Err(match failure {
            None => InviteSignupError::Invalid,
            Some(failure) if failure.used => InviteSignupError::AlreadyUsed,
            Some(failure) if failure.expired => InviteSignupError::Expired,
            Some(failure) if failure.email_mismatch => InviteSignupError::EmailMismatch,
            Some(_) => InviteSignupError::Invalid,
        });
    }

    let password_hash = hash_password(password).context("hashing invite password")?;

    let row = users::create_via_invite()
        .bind(&txn, &email, &password_hash, &display_name)
        .one()
        .await
        .map_err(|e| {
            if let Some(db) = e.as_db_error() {
                if db.code() == &tokio_postgres::error::SqlState::UNIQUE_VIOLATION {
                    return InviteSignupError::AlreadyRegistered;
                }
            }
            InviteSignupError::Internal(e.into())
        })?;
    let user_id = row.id;
    let is_admin = row.is_admin;

    tokens::invite_set_used_by()
        .bind(&txn, &user_id, &token_hash)
        .await
        .context("recording invite user")?;
    txn.commit().await.context("committing invite signup")?;

    Ok((user_id, is_admin))
}

pub async fn start_password_reset(
    pool: &Pool,
    mailer: &dyn Mailer,
    public_origin: &str,
    email: &str,
) -> Result<()> {
    let c = pool.get().await?;
    let Some(user_id) = users::by_email_id().bind(&c, &email).opt().await? else {
        tracing::info!(email, "forgot-password: no matching user (silent)");
        return Ok(());
    };
    let (token, hash) = generate_token();
    let expires = OffsetDateTime::now_utc() + Duration::hours(DEFAULT_TOKEN_TTL_HOURS);
    tokens::password_reset_create()
        .bind(&c, &hash, &user_id, &expires)
        .await?;

    let link = format!("{public_origin}/auth/reset/{token}");
    let body = format!(
        "Hi,\n\nTo reset your rampart password, visit:\n\n    {link}\n\n\
         This link is valid for {DEFAULT_TOKEN_TTL_HOURS} hour(s) and can be used once.\n\
         If you didn't request this, ignore this email.\n"
    );
    mailer
        .send(email, "rampart password reset", &body)
        .await
        .context("sending password-reset email")?;
    tracing::info!(email, "sent password reset link");
    Ok(())
}

pub async fn apply_password_reset(
    pool: &Pool,
    verify_cache: &crate::auth::VerifyCache,
    token: &str,
    new_password: &str,
) -> std::result::Result<(), PasswordResetError> {
    if new_password.len() < 10 {
        return Err(PasswordResetError::PasswordTooShort);
    }
    let mut c = pool.get().await.context("opening password reset")?;
    let hash = hash_token(token);

    let txn = c
        .transaction()
        .await
        .context("starting password reset transaction")?;
    let Some(user_id) = tokens::password_reset_claim()
        .bind(&txn, &hash)
        .opt()
        .await
        .context("claiming password reset token")?
    else {
        let failure = tokens::password_reset_failure()
            .bind(&txn, &hash)
            .opt()
            .await
            .context("inspecting password reset failure")?;
        txn.rollback().await.ok();
        return Err(match failure {
            None => PasswordResetError::Invalid,
            Some(failure) if failure.used => PasswordResetError::AlreadyUsed,
            Some(failure) if failure.expired => PasswordResetError::Expired,
            Some(_) => PasswordResetError::Invalid,
        });
    };

    let pw_hash = hash_password(new_password).context("hashing reset password")?;

    users::set_password()
        .bind(&txn, &Some(pw_hash), &user_id)
        .await
        .context("storing reset password")?;
    sessions::delete_by_user()
        .bind(&txn, &user_id)
        .await
        .context("ending sessions after password reset")?;
    txn.commit().await.context("committing password reset")?;

    verify_cache.invalidate_user(user_id);
    tracing::info!(user_id, "password reset applied");
    Ok(())
}

pub async fn start_email_change(
    pool: &Pool,
    mailer: &dyn Mailer,
    public_origin: &str,
    user_id: i64,
    new_email: &str,
) -> Result<()> {
    let c = pool.get().await?;
    let existing = users::email_exists_for_other()
        .bind(&c, &new_email, &user_id)
        .opt()
        .await?;
    if existing.is_some() {
        anyhow::bail!("email already registered to another user");
    }
    let (token, hash) = generate_token();
    let expires = OffsetDateTime::now_utc() + Duration::hours(DEFAULT_TOKEN_TTL_HOURS);
    tokens::email_change_create()
        .bind(&c, &hash, &user_id, &new_email, &expires)
        .await?;
    let link = format!("{public_origin}/auth/change-email/{token}");
    let body = format!(
        "Hi,\n\nSomeone (probably you) requested to change your rampart account email to {new_email}.\n\
         To confirm, visit:\n\n    {link}\n\n\
         If you didn't request this, ignore this email.\n"
    );
    mailer
        .send(new_email, "rampart email change — please confirm", &body)
        .await?;
    tracing::info!(user_id, new_email, "sent email-change confirmation");
    Ok(())
}

pub async fn apply_email_change(
    pool: &Pool,
    token: &str,
) -> std::result::Result<String, EmailChangeError> {
    let mut c = pool.get().await.context("opening email change")?;
    let hash = hash_token(token);
    let txn = c
        .transaction()
        .await
        .context("starting email change transaction")?;
    let Some(claimed) = tokens::email_change_claim()
        .bind(&txn, &hash)
        .opt()
        .await
        .context("claiming email change token")?
    else {
        let failure = tokens::email_change_failure()
            .bind(&txn, &hash)
            .opt()
            .await
            .context("inspecting email change failure")?;
        txn.rollback().await.ok();
        return Err(match failure {
            None => EmailChangeError::Invalid,
            Some(failure) if failure.used => EmailChangeError::AlreadyUsed,
            Some(failure) if failure.expired => EmailChangeError::Expired,
            Some(_) => EmailChangeError::Invalid,
        });
    };
    let user_id = claimed.user_id;
    let new_email = claimed.new_email;
    users::set_email()
        .bind(&txn, &new_email, &user_id)
        .await
        .map_err(|e| {
            if let Some(db) = e.as_db_error() {
                if db.code() == &tokio_postgres::error::SqlState::UNIQUE_VIOLATION {
                    return EmailChangeError::AlreadyRegistered;
                }
            }
            EmailChangeError::Internal(e.into())
        })?;
    txn.commit().await.context("committing email change")?;
    tracing::info!(user_id, new_email, "email changed");
    Ok(new_email)
}

pub async fn start_mailbox_verify(
    pool: &Pool,
    mailer: &dyn Mailer,
    public_origin: &str,
    mailbox_id: i64,
) -> Result<()> {
    let c = pool.get().await?;
    let Some(mb) = mailboxes::email_and_verified()
        .bind(&c, &mailbox_id)
        .opt()
        .await?
    else {
        anyhow::bail!("mailbox not found");
    };
    if mb.verified {
        return Ok(());
    }
    let email = mb.email;
    let (token, hash) = generate_token();
    let expires = OffsetDateTime::now_utc() + Duration::hours(24);
    tokens::mailbox_verify_create()
        .bind(&c, &hash, &mailbox_id, &expires)
        .await?;
    let link = format!("{public_origin}/mailbox/verify/{token}");
    let body = format!(
        "Hi,\n\nTo prove ownership of this mailbox ({email}) for your rampart account, visit:\n\n    {link}\n\n\
         Link expires in 24 hours.\n"
    );
    mailer
        .send(&email, "rampart mailbox verification", &body)
        .await?;
    tracing::info!(mailbox_id, email, "sent mailbox verification link");
    Ok(())
}

pub async fn apply_mailbox_verify(pool: &Pool, token: &str) -> Result<i64> {
    let mut c = pool.get().await?;
    let hash = hash_token(token);
    let txn = c.transaction().await?;
    let Some(mailbox_id) = tokens::mailbox_verify_claim()
        .bind(&txn, &hash)
        .opt()
        .await?
    else {
        anyhow::bail!("invalid, expired, or already-used token");
    };
    mailboxes::set_verified().bind(&txn, &mailbox_id).await?;
    txn.commit().await?;
    tracing::info!(mailbox_id, "mailbox verified");
    Ok(mailbox_id)
}
