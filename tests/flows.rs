//! Token-flow tests: password reset, email change, mailbox verify, +
//! signup atomic-claim race. Drives `flows::*` directly against an
//! ephemeral DB and a `MemoryMailer` capture.
#![expect(clippy::tests_outside_test_module, reason = "integration test file")]

mod support;

use hmac_sha256::Hash;
use rampart::{
   auth::VerifyCache,
   flows::{
      self,
      EmailChangeError,
      InviteSignupError,
      MailboxVerifyError,
      PasswordResetError,
      StartEmailChangeError,
   },
   mailer::MemoryMailer,
};
use support::TestDb;
use time::{
   Duration,
   OffsetDateTime,
};
use tokio_postgres::types::ToSql;

// Shared with all DB-backed suites — see TestDb::or_skip docs.
macro_rules! test_db {
   () => {
      match TestDb::or_skip().await {
         Some(db) => db,
         None => return,
      }
   };
}

/// Insert a user with a known argon2 hash for "passwordpassword" so
/// `password_reset` round-trips can verify the new hash differs from the
/// original.
async fn seed_user(client: &deadpool_postgres::Client, email: &str) -> i64 {
   client
      .query_one(
         "INSERT INTO \"user\" (email, password_hash) VALUES ($1, 'placeholder-hash') RETURNING id",
         &[&email],
      )
      .await
      .unwrap()
      .get("id")
}

async fn seed_mailbox(client: &deadpool_postgres::Client, user_id: i64, email: &str) -> i64 {
   client
      .query_one(
         "INSERT INTO mailbox (user_id, email, verified) VALUES ($1, $2, FALSE) RETURNING id",
         &[&user_id, &email],
      )
      .await
      .unwrap()
      .get("id")
}

/// Pull the URL-safe base64 token out of an email body. The flows write
/// links of the form `<origin>/<path>/<token>`; we just regex out the
/// last path segment of the first URL.
fn extract_token(body: &str) -> String {
   let url_marker = "http";
   let start = body.find(url_marker).expect("url in body");
   let rest = &body[start..];
   let end = rest
      .find(|ch: char| ch.is_whitespace())
      .unwrap_or(rest.len());
   let url = &rest[..end];
   url.rsplit('/').next().unwrap().to_owned()
}

#[tokio::test]
async fn forgot_password_round_trip() {
   let db = test_db!();
   {
      let client = db.pool.get().await.unwrap();
      let _uid = seed_user(&client, "alice@test").await;
   }
   let mailer = MemoryMailer::new();
   flows::start_password_reset(&db.pool, &mailer, "http://localhost", "alice@test")
      .await
      .unwrap();
   let sent = mailer.drain();
   assert_eq!(sent.len(), 1);
   let token = extract_token(&sent[0].body);

   // Apply the reset.
   let cache = VerifyCache::new();
   flows::apply_password_reset(&db.pool, &cache, &token, "newpassword12345")
      .await
      .unwrap();

   // password_hash should have changed; sessions table empty.
   let client = db.pool.get().await.unwrap();
   let row = client
      .query_one(
         "SELECT password_hash FROM \"user\" WHERE email = 'alice@test'",
         &[],
      )
      .await
      .unwrap();
   let hash: String = row.get("password_hash");
   assert_ne!(hash, "placeholder-hash");
   let n: i64 = client
      .query_one("SELECT count(*)::bigint AS n FROM session", &[])
      .await
      .unwrap()
      .get("n");
   assert_eq!(n, 0);

   db.teardown().await;
}

#[tokio::test]
async fn email_change_round_trip() {
   let db = test_db!();
   let uid = {
      let client = db.pool.get().await.unwrap();
      seed_user(&client, "alice@test").await
   };
   let mailer = MemoryMailer::new();
   flows::start_email_change(&db.pool, &mailer, "http://localhost", uid, "alice2@test")
      .await
      .unwrap();
   let token = extract_token(&mailer.drain()[0].body);
   let new_email = flows::apply_email_change(&db.pool, &token).await.unwrap();
   assert_eq!(new_email, "alice2@test");

   let client = db.pool.get().await.unwrap();
   let row = client
      .query_one(
         "SELECT email::text AS email FROM \"user\" WHERE id = $1",
         &[&uid],
      )
      .await
      .unwrap();
   let email: String = row.get("email");
   assert_eq!(email, "alice2@test");

   db.teardown().await;
}

#[tokio::test]
async fn email_change_rejects_registered_address() {
   let db = test_db!();
   let uid = {
      let client = db.pool.get().await.unwrap();
      let uid = seed_user(&client, "alice@test").await;
      seed_user(&client, "taken@test").await;
      uid
   };
   let error = flows::start_email_change(
      &db.pool,
      &MemoryMailer::new(),
      "http://localhost",
      uid,
      "taken@test",
   )
   .await
   .unwrap_err();
   assert!(matches!(error, StartEmailChangeError::AlreadyRegistered));

   db.teardown().await;
}

#[tokio::test]
async fn email_change_reports_distinct_failures() {
   let db = test_db!();
   let (uid, _taken_uid) = {
      let client = db.pool.get().await.unwrap();
      (
         seed_user(&client, "alice@test").await,
         seed_user(&client, "taken@test").await,
      )
   };
   let now = OffsetDateTime::now_utc();
   let future = now + Duration::hours(1);
   let past = now - Duration::hours(1);
   let expired_hash = Hash::hash(b"expired-email-change").to_vec();
   let used_hash = Hash::hash(b"used-email-change").to_vec();
   let duplicate_hash = Hash::hash(b"duplicate-email-change").to_vec();
   {
      let client = db.pool.get().await.unwrap();
      client
         .execute(
            "INSERT INTO email_change_token (token_hash, user_id, new_email, expires_at) VALUES \
             ($1, $2, $3, $4)",
            &[&expired_hash, &uid, &"new@test", &past],
         )
         .await
         .unwrap();
      client
         .execute(
            "INSERT INTO email_change_token (token_hash, user_id, new_email, expires_at, used_at) \
             VALUES ($1, $2, $3, $4, $5)",
            &[&used_hash, &uid, &"new@test", &future, &now],
         )
         .await
         .unwrap();
      client
         .execute(
            "INSERT INTO email_change_token (token_hash, user_id, new_email, expires_at) VALUES \
             ($1, $2, $3, $4)",
            &[&duplicate_hash, &uid, &"taken@test", &future],
         )
         .await
         .unwrap();
   }

   let invalid = flows::apply_email_change(&db.pool, "missing-email-change")
      .await
      .unwrap_err();
   assert!(matches!(invalid, EmailChangeError::Invalid));

   let expired = flows::apply_email_change(&db.pool, "expired-email-change")
      .await
      .unwrap_err();
   assert!(matches!(expired, EmailChangeError::Expired));

   let used = flows::apply_email_change(&db.pool, "used-email-change")
      .await
      .unwrap_err();
   assert!(matches!(used, EmailChangeError::AlreadyUsed));

   let duplicate = flows::apply_email_change(&db.pool, "duplicate-email-change")
      .await
      .unwrap_err();
   assert!(matches!(duplicate, EmailChangeError::AlreadyRegistered));

   db.teardown().await;
}

#[tokio::test]
async fn mailbox_verify_round_trip() {
   let db = test_db!();
   let mid = {
      let client = db.pool.get().await.unwrap();
      let uid = seed_user(&client, "alice@test").await;
      seed_mailbox(&client, uid, "alice@gmail.com").await
   };
   let mailer = MemoryMailer::new();
   flows::start_mailbox_verify(&db.pool, &mailer, "http://localhost", mid)
      .await
      .unwrap();
   let token = extract_token(&mailer.drain()[0].body);
   let returned_id = flows::apply_mailbox_verify(&db.pool, &token).await.unwrap();
   assert_eq!(returned_id, mid);

   let client = db.pool.get().await.unwrap();
   let verified: bool = client
      .query_one("SELECT verified FROM mailbox WHERE id = $1", &[&mid])
      .await
      .unwrap()
      .get("verified");
   assert!(verified);
   db.teardown().await;
}

#[tokio::test]
async fn mailbox_verify_reports_distinct_failures() {
   let db = test_db!();
   let (mid, token) = {
      let client = db.pool.get().await.unwrap();
      let uid = seed_user(&client, "alice@test").await;
      let mid = seed_mailbox(&client, uid, "alice@gmail.com").await;
      drop(client);
      let mailer = MemoryMailer::new();
      flows::start_mailbox_verify(&db.pool, &mailer, "http://localhost", mid)
         .await
         .unwrap();
      (mid, extract_token(&mailer.drain()[0].body))
   };
   flows::apply_mailbox_verify(&db.pool, &token).await.unwrap();
   let used = flows::apply_mailbox_verify(&db.pool, &token)
      .await
      .unwrap_err();
   assert!(matches!(used, MailboxVerifyError::AlreadyUsed));

   let expired_token = "expired-mailbox-verification";
   let expired_hash = Hash::hash(expired_token.as_bytes()).to_vec();
   let past = OffsetDateTime::now_utc() - Duration::hours(1);
   {
      let client = db.pool.get().await.unwrap();
      client
         .execute(
            "INSERT INTO mailbox_verify_token (token_hash, mailbox_id, expires_at) VALUES ($1, \
             $2, $3)",
            &[&expired_hash, &mid, &past],
         )
         .await
         .unwrap();
   }
   let expired = flows::apply_mailbox_verify(&db.pool, expired_token)
      .await
      .unwrap_err();
   assert!(matches!(expired, MailboxVerifyError::Expired));

   let invalid = flows::apply_mailbox_verify(&db.pool, "missing-mailbox-verification")
      .await
      .unwrap_err();
   assert!(matches!(invalid, MailboxVerifyError::Invalid));

   db.teardown().await;
}

#[tokio::test]
async fn reset_rejects_used_token() {
   let db = test_db!();
   {
      let client = db.pool.get().await.unwrap();
      seed_user(&client, "alice@test").await;
   }
   let mailer = MemoryMailer::new();
   flows::start_password_reset(&db.pool, &mailer, "http://localhost", "alice@test")
      .await
      .unwrap();
   let token = extract_token(&mailer.drain()[0].body);
   let cache = VerifyCache::new();
   flows::apply_password_reset(&db.pool, &cache, &token, "validlongpw")
      .await
      .unwrap();
   let err = flows::apply_password_reset(&db.pool, &cache, &token, "anotherone")
      .await
      .expect_err("second use must fail");
   assert!(matches!(err, PasswordResetError::AlreadyUsed));

   db.teardown().await;
}

#[tokio::test]
async fn reset_rejects_expired_token() {
   let db = test_db!();
   let uid = {
      let client = db.pool.get().await.unwrap();
      seed_user(&client, "alice@test").await
   };
   // Insert an already-expired token by hand (start_password_reset uses
   // a 1-hour TTL; we want one in the past).
   let token = "manually-crafted-token-string";
   let token_hash = Hash::hash(token.as_bytes()).to_vec();
   let past = OffsetDateTime::now_utc() - Duration::hours(1);
   {
      let client = db.pool.get().await.unwrap();
      client
         .execute(
            "INSERT INTO password_reset_token (token_hash, user_id, expires_at) VALUES ($1, $2, \
             $3)",
            &[&token_hash as &(dyn ToSql + Sync), &uid, &past],
         )
         .await
         .unwrap();
   }
   let cache = VerifyCache::new();
   let err = flows::apply_password_reset(&db.pool, &cache, token, "validlongpw")
      .await
      .expect_err("expired must fail");
   assert!(matches!(err, PasswordResetError::Expired));

   let invalid =
      flows::apply_password_reset(&db.pool, &cache, "not-a-real-reset-token", "validlongpw")
         .await
         .expect_err("unknown token must fail");
   assert!(matches!(invalid, PasswordResetError::Invalid));

   db.teardown().await;
}

#[tokio::test]
async fn signup_claim_atomic() {
   let db = test_db!();
   // Seed an invite token.
   let raw_token = "race-token-12345";
   let token_hash = Hash::hash(raw_token.as_bytes()).to_vec();
   let exp = OffsetDateTime::now_utc() + Duration::hours(24);
   {
      let client = db.pool.get().await.unwrap();
      client
         .execute(
            "INSERT INTO invite_token (token_hash, preset_email, expires_at) VALUES ($1, NULL, $2)",
            &[&token_hash as &(dyn ToSql + Sync), &exp],
         )
         .await
         .unwrap();
   }

   // Two concurrent signups against the SAME token: exactly one must win.
   let pool1 = db.pool.clone();
   let pool2 = db.pool.clone();
   let h1 = tokio::spawn(async move {
      flows::claim_invite_and_create_user(&pool1, raw_token, "user1@test", "pw-1234567890", None)
         .await
   });
   let h2 = tokio::spawn(async move {
      flows::claim_invite_and_create_user(&pool2, raw_token, "user2@test", "pw-1234567890", None)
         .await
   });
   let r1 = h1.await.unwrap();
   let r2 = h2.await.unwrap();
   let oks = [r1.is_ok(), r2.is_ok()].iter().filter(|ok| **ok).count();
   assert_eq!(oks, 1, "exactly one signup must win the race");

   let err = match r1 {
      Err(error) => error,
      Ok(_) => r2.unwrap_err(),
   };
   assert!(matches!(err, InviteSignupError::AlreadyUsed));

   db.teardown().await;
}

#[tokio::test]
async fn signup_reports_distinct_invite_failures() {
   let db = test_db!();
   let client = db.pool.get().await.unwrap();
   let now = OffsetDateTime::now_utc();
   let future = now + Duration::hours(24);
   let past = now - Duration::hours(1);
   let expired_hash = Hash::hash(b"expired-invite").to_vec();
   let used_hash = Hash::hash(b"used-invite").to_vec();
   let mismatch_hash = Hash::hash(b"mismatch-invite").to_vec();
   client
      .execute(
         "INSERT INTO invite_token (token_hash, expires_at) VALUES ($1, $2)",
         &[&expired_hash, &past],
      )
      .await
      .unwrap();
   client
      .execute(
         "INSERT INTO invite_token (token_hash, expires_at, used_at) VALUES ($1, $2, $3)",
         &[&used_hash, &future, &now],
      )
      .await
      .unwrap();
   client
      .execute(
         "INSERT INTO invite_token (token_hash, preset_email, expires_at) VALUES ($1, $2, $3)",
         &[&mismatch_hash, &"invited@test", &future],
      )
      .await
      .unwrap();
   drop(client);

   let invalid = flows::claim_invite_and_create_user(
      &db.pool,
      "missing-invite",
      "user@test",
      "pw-1234567890",
      None,
   )
   .await
   .unwrap_err();
   assert!(matches!(invalid, InviteSignupError::Invalid));

   let expired = flows::claim_invite_and_create_user(
      &db.pool,
      "expired-invite",
      "user@test",
      "pw-1234567890",
      None,
   )
   .await
   .unwrap_err();
   assert!(matches!(expired, InviteSignupError::Expired));

   let used = flows::claim_invite_and_create_user(
      &db.pool,
      "used-invite",
      "user@test",
      "pw-1234567890",
      None,
   )
   .await
   .unwrap_err();
   assert!(matches!(used, InviteSignupError::AlreadyUsed));

   let mismatch = flows::claim_invite_and_create_user(
      &db.pool,
      "mismatch-invite",
      "user@test",
      "pw-1234567890",
      None,
   )
   .await
   .unwrap_err();
   assert!(matches!(mismatch, InviteSignupError::EmailMismatch));

   db.teardown().await;
}
