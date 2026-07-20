//! Token-based flows: password reset, email change, mailbox verify.
//!
//! Shape is identical across flows:
//!   1. `start_X`: generate token, insert row, send email.
//!   2. `apply_X`: on POST from confirmation page, validate token, apply side
//!      effect, mark token used.
//!
//! Tokens are 24 random bytes, base64url-encoded for the URL; stored as
//! sha256 so a DB leak doesn't yield click-through credentials.

use anyhow::{
   Context as _,
   Result,
};
use data_encoding::BASE64URL_NOPAD;
use deadpool_postgres::Pool;
use hmac_sha256::Hash;
use rampart_codegen::queries::{
   mailboxes,
   sessions,
   tokens,
   users,
};
use rand::{
   TryRng as _,
   rngs::SysRng,
};
use time::{
   Duration,
   OffsetDateTime,
};
use tokio_postgres::error::SqlState;

use crate::{
   auth::{
      self,
      VerifyCache,
   },
   mailer::Mailer,
};

pub const DEFAULT_TOKEN_TTL_HOURS: i64 = 1;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
pub enum StartEmailChangeError {
   #[error("email already registered to another user")]
   AlreadyRegistered,
   #[error(transparent)]
   Internal(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum MailboxVerifyError {
   #[error("mailbox verification token is invalid")]
   Invalid,
   #[error("mailbox verification token has expired")]
   Expired,
   #[error("mailbox verification token has already been used")]
   AlreadyUsed,
   #[error(transparent)]
   Internal(#[from] anyhow::Error),
}

fn generate_token() -> (String, Vec<u8>) {
   let mut bytes = [0_u8; 24];
   SysRng
      .try_fill_bytes(&mut bytes)
      .expect("SysRng must not fail");
   let token = BASE64URL_NOPAD.encode(&bytes);
   let hash = Hash::hash(token.as_bytes()).to_vec();
   (token, hash)
}

fn hash_token(token: &str) -> Vec<u8> {
   Hash::hash(token.as_bytes()).to_vec()
}

/// Advisory-lock key serializing concurrent /setup POSTs.
///
/// INSERT ... WHERE NOT EXISTS under READ COMMITTED only guarantees
/// per-statement atomicity; two concurrent statements take independent
/// snapshots, both see an empty user table, both insert. We funnel them
/// through `pg_advisory_xact_lock`.
pub const FIRST_ADMIN_LOCK_KEY: i64 = 0x5241_4D50_5F46_4131; // "RAMP_FA1"

/// First-run admin bootstrap.
///
/// Creates a user with `is_admin = true` only if no user exists yet.
/// Race-safe via `FIRST_ADMIN_LOCK_KEY` — concurrent setup POSTs serialize
/// against the lock, then the inner `INSERT ... WHERE NOT EXISTS` guard
/// returns `Ok(None)` for the loser.
///
/// # Errors
///
/// Returns an error if the password is shorter than 10 characters or if a
/// database operation fails.
pub async fn bootstrap_first_admin(
   pool: &Pool,
   email: &str,
   password: &str,
   display_name: Option<&str>,
) -> anyhow::Result<Option<i64>> {
   if password.len() < 10 {
      anyhow::bail!("password must be at least 10 characters");
   }
   let password_hash = auth::hash_password(password)?;
   let mut conn = pool.get().await?;
   let txn = conn.transaction().await?;
   txn.execute("SET LOCAL idle_in_transaction_session_timeout = 0", &[])
      .await
      .context("disable idle-in-txn timeout for first-admin lock")?;
   txn.execute("SELECT pg_advisory_xact_lock($1::bigint)", &[
      &FIRST_ADMIN_LOCK_KEY,
   ])
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

/// Claim an invite token and create the associated user.
///
/// # Errors
///
/// Returns [`InviteSignupError`] if the password is too short, the invite is
/// invalid/expired/already-used, the email doesn't match the invite, the email
/// is already registered, or a database operation fails.
pub async fn claim_invite_and_create_user(
   pool: &Pool,
   token: &str,
   email: &str,
   password: &str,
   display_name: Option<&str>,
) -> Result<(i64, bool), InviteSignupError> {
   if password.len() < 10 {
      return Err(InviteSignupError::PasswordTooShort);
   }
   let token_hash = hash_token(token);
   let mut conn = pool.get().await.context("opening invite transaction")?;
   let txn = conn
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
      let _ = txn.rollback().await;
      return Err(match failure {
         Some(failure) if failure.used => InviteSignupError::AlreadyUsed,
         Some(failure) if failure.expired => InviteSignupError::Expired,
         Some(failure) if failure.email_mismatch => InviteSignupError::EmailMismatch,
         _ => InviteSignupError::Invalid,
      });
   }

   let password_hash = auth::hash_password(password).context("hashing invite password")?;

   let row = users::create_via_invite()
      .bind(&txn, &email, &password_hash, &display_name)
      .one()
      .await
      .map_err(|err| {
         if let Some(db) = err.as_db_error()
            && db.code() == &SqlState::UNIQUE_VIOLATION
         {
            return InviteSignupError::AlreadyRegistered;
         }
         InviteSignupError::Internal(err.into())
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

/// Start a password-reset flow by emailing a single-use token link.
///
/// # Errors
///
/// Returns an error if a database operation or sending the email fails.
pub async fn start_password_reset(
   pool: &Pool,
   mailer: &dyn Mailer,
   public_origin: &str,
   email: &str,
) -> Result<()> {
   let conn = pool.get().await?;
   let Some(user_id) = users::by_email_id().bind(&conn, &email).opt().await? else {
      tracing::info!(email, "forgot-password: no matching user (silent)");
      return Ok(());
   };
   let (token, hash) = generate_token();
   let expires = OffsetDateTime::now_utc() + Duration::hours(DEFAULT_TOKEN_TTL_HOURS);
   tokens::password_reset_create()
      .bind(&conn, &hash, &user_id, &expires)
      .await?;

   let link = format!("{public_origin}/auth/reset/{token}");
   let body = format!(
      "Hi,\n\nTo reset your rampart password, visit:\n\n    {link}\n\nThis link is valid for \
       {DEFAULT_TOKEN_TTL_HOURS} hour(s) and can be used once.\nIf you didn't request this, \
       ignore this email.\n"
   );
   mailer
      .send(email, "rampart password reset", &body)
      .await
      .context("sending password-reset email")?;
   tracing::info!(email, "sent password reset link");
   Ok(())
}

/// Apply a password reset given a valid token and new password.
///
/// # Errors
///
/// Returns [`PasswordResetError`] if the new password is too short, the token
/// is invalid/expired/already-used, or a database operation fails.
pub async fn apply_password_reset(
   pool: &Pool,
   verify_cache: &VerifyCache,
   token: &str,
   new_password: &str,
) -> Result<(), PasswordResetError> {
   if new_password.len() < 10 {
      return Err(PasswordResetError::PasswordTooShort);
   }
   let mut conn = pool.get().await.context("opening password reset")?;
   let hash = hash_token(token);

   let txn = conn
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
      let _ = txn.rollback().await;
      return Err(match failure {
         Some(failure) if failure.used => PasswordResetError::AlreadyUsed,
         Some(failure) if failure.expired => PasswordResetError::Expired,
         _ => PasswordResetError::Invalid,
      });
   };

   let pw_hash = auth::hash_password(new_password).context("hashing reset password")?;

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

/// Start an email-change flow by emailing a confirmation link to the new
/// address.
///
/// # Errors
///
/// Returns [`StartEmailChangeError`] if the new email is already registered to
/// another user, or a database/email operation fails.
pub async fn start_email_change(
   pool: &Pool,
   mailer: &dyn Mailer,
   public_origin: &str,
   user_id: i64,
   new_email: &str,
) -> Result<(), StartEmailChangeError> {
   let conn = pool.get().await.context("opening email change request")?;
   let existing = users::email_exists_for_other()
      .bind(&conn, &new_email, &user_id)
      .opt()
      .await
      .context("checking email availability")?;
   if existing.is_some() {
      return Err(StartEmailChangeError::AlreadyRegistered);
   }
   let (token, hash) = generate_token();
   let expires = OffsetDateTime::now_utc() + Duration::hours(DEFAULT_TOKEN_TTL_HOURS);
   tokens::email_change_create()
      .bind(&conn, &hash, &user_id, &new_email, &expires)
      .await
      .context("creating email change token")?;
   let link = format!("{public_origin}/auth/change-email/{token}");
   let body = format!(
      "Hi,\n\nSomeone (probably you) requested to change your rampart account email to \
       {new_email}.\nTo confirm, visit:\n\n    {link}\n\nIf you didn't request this, ignore this \
       email.\n"
   );
   mailer
      .send(new_email, "rampart email change — please confirm", &body)
      .await
      .context("sending email change confirmation")?;
   tracing::info!(user_id, new_email, "sent email-change confirmation");
   Ok(())
}

/// Apply an email change given a valid token, returning the new email.
///
/// # Errors
///
/// Returns [`EmailChangeError`] if the token is invalid/expired/already-used,
/// the email is already registered, or a database operation fails.
pub async fn apply_email_change(pool: &Pool, token: &str) -> Result<String, EmailChangeError> {
   let mut conn = pool.get().await.context("opening email change")?;
   let hash = hash_token(token);
   let txn = conn
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
      let _ = txn.rollback().await;
      return Err(match failure {
         Some(failure) if failure.used => EmailChangeError::AlreadyUsed,
         Some(failure) if failure.expired => EmailChangeError::Expired,
         _ => EmailChangeError::Invalid,
      });
   };
   let user_id = claimed.user_id;
   let new_email = claimed.new_email;
   users::set_email()
      .bind(&txn, &new_email, &user_id)
      .await
      .map_err(|err| {
         if let Some(db) = err.as_db_error()
            && db.code() == &SqlState::UNIQUE_VIOLATION
         {
            return EmailChangeError::AlreadyRegistered;
         }
         EmailChangeError::Internal(err.into())
      })?;
   txn.commit().await.context("committing email change")?;
   tracing::info!(user_id, new_email, "email changed");
   Ok(new_email)
}

/// Start a mailbox-verification flow by emailing a confirmation link.
///
/// # Errors
///
/// Returns an error if the mailbox is not found, or a database/email operation
/// fails.
pub async fn start_mailbox_verify(
   pool: &Pool,
   mailer: &dyn Mailer,
   public_origin: &str,
   mailbox_id: i64,
) -> Result<()> {
   let conn = pool.get().await?;
   let Some(mb) = mailboxes::email_and_verified()
      .bind(&conn, &mailbox_id)
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
      .bind(&conn, &hash, &mailbox_id, &expires)
      .await?;
   let link = format!("{public_origin}/mailbox/verify/{token}");
   let body = format!(
         "Hi,\n\nTo prove ownership of this mailbox ({email}) for your rampart account, \
          visit:\n\n    {link}\n\nLink expires in 24 hours.\n"
      );
   mailer
      .send(&email, "rampart mailbox verification", &body)
      .await?;
   tracing::info!(mailbox_id, email, "sent mailbox verification link");
   Ok(())
}

/// Apply a mailbox verification given a valid token, returning the mailbox id.
///
/// # Errors
///
/// Returns [`MailboxVerifyError`] if the token is invalid/expired/already-used,
/// or a database operation fails.
pub async fn apply_mailbox_verify(pool: &Pool, token: &str) -> Result<i64, MailboxVerifyError> {
   let mut conn = pool.get().await.context("opening mailbox verification")?;
   let hash = hash_token(token);
   let txn = conn
      .transaction()
      .await
      .context("starting mailbox verification transaction")?;
   let Some(mailbox_id) = tokens::mailbox_verify_claim()
      .bind(&txn, &hash)
      .opt()
      .await
      .context("claiming mailbox verification token")?
   else {
      let failure = tokens::mailbox_verify_failure()
         .bind(&txn, &hash)
         .opt()
         .await
         .context("inspecting mailbox verification failure")?;
      let _ = txn.rollback().await;
      return Err(match failure {
         Some(failure) if failure.used => MailboxVerifyError::AlreadyUsed,
         Some(failure) if failure.expired => MailboxVerifyError::Expired,
         _ => MailboxVerifyError::Invalid,
      });
   };
   mailboxes::set_verified()
      .bind(&txn, &mailbox_id)
      .await
      .context("marking mailbox verified")?;
   txn.commit()
      .await
      .context("committing mailbox verification")?;
   tracing::info!(mailbox_id, "mailbox verified");
   Ok(mailbox_id)
}
