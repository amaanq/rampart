//! Bounce-VERP path tests. Covers HMAC-verified DSNs flipping
//! `email_log.status` to 'bounced', and refusing forged or
//! phase-mismatched VERPs.
#![expect(clippy::tests_outside_test_module, reason = "integration test file")]

mod support;

use std::sync::Arc;

use rampart::{
   config::Config,
   mailer::MemoryMailer,
   worker::{
      WorkerState,
      loop_guard::{
         BouncePhase,
         Rcpt,
      },
      pipeline::{
         self,
         Delivery,
         Verdict,
      },
      resubmit::{
         MemorySubmit,
         Submit,
      },
      verp,
   },
};
use support::TestDb;

macro_rules! test_db {
   () => {
      match TestDb::or_skip().await {
         Some(db) => db,
         None => return,
      }
   };
}

fn make_state(pool: deadpool_postgres::Pool) -> (WorkerState, Arc<MemorySubmit>) {
   let cfg = Config {
      database_url:                 "irrelevant".into(),
      listen:                       "127.0.0.1:0".parse().unwrap(),
      public_origin:                "http://localhost".into(),
      static_dir:                   "static".into(),
      sieve_output_path:            None,
      smtp_host:                    "localhost".into(),
      smtp_port:                    465,
      smtp_user:                    "x@x".into(),
      smtp_password_file:           None,
      notifier_from:                "\"rampart\" <x@x>".into(),
      webauthn_rp_id:               "localhost".into(),
      lmtp_listen:                  "127.0.0.1:0".parse().unwrap(),
      stalwart_hostname:            "test.example".into(),
      public_mx_hostname:           "test.example".into(),
      lmtp_drain_secs:              20,
      stalwart_jmap_base_url:       None,
      stalwart_admin_username:      "admin".into(),
      stalwart_admin_password_file: None,
      verp_key:                     b"test-key-32-bytes-long-padding-padding".to_vec(),
   };
   let submit = Arc::new(MemorySubmit::new());
   #[expect(
      clippy::clone_on_ref_ptr,
      reason = "clone coerces Arc<MemorySubmit> into the Arc<dyn Submit> field"
   )]
   let submit_dyn: Arc<dyn Submit> = submit.clone();
   let state = WorkerState {
      pool,
      config: Arc::new(cfg),
      mailer: Arc::new(MemoryMailer::new()),
      submit: submit_dyn,
   };
   (state, submit)
}

async fn seed_alias(client: &deadpool_postgres::Client) {
   client
      .execute(
         "INSERT INTO \"user\" (email, password_hash) VALUES ('alice@test', 'x')",
         &[],
      )
      .await
      .unwrap();
   client
      .execute(
         "INSERT INTO mailbox (user_id, email, verified) VALUES (1, 'alice@gmail.com', TRUE)",
         &[],
      )
      .await
      .unwrap();
   client
      .execute(
         "INSERT INTO alias_domain (domain, owner_id, shared) VALUES ('addy.test', 1, FALSE)",
         &[],
      )
      .await
      .unwrap();
   client
      .execute(
         "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES (1, 'abc@addy.test', \
          1, 1)",
         &[],
      )
      .await
      .unwrap();
}

/// Minimal RFC 3464 multipart/report body with a Diagnostic-Code line.
fn dsn_msg(diagnostic: &str) -> Vec<u8> {
   format!(
      "Subject: Mail delivery failed\r\nContent-Type: multipart/report; \
       report-type=delivery-status; boundary=\"BND\"\r\n\r\n--BND\r\nContent-Type: \
       text/plain\r\n\r\nA message you sent could not be delivered.\r\n\r\n--BND\r\nContent-Type: \
       message/delivery-status\r\n\r\nReporting-MTA: dns; mta.example\r\n\r\nFinal-Recipient: \
       rfc822;target@elsewhere\r\nAction: failed\r\nStatus: 5.1.1\r\nDiagnostic-Code: smtp; \
       {diagnostic}\r\n\r\n--BND--\r\n"
   )
   .into_bytes()
}

async fn insert_log(
   client: &deadpool_postgres::Client,
   action: &str,
   status: &str,
   reason: Option<&str>,
) -> i64 {
   let row = client
      .query_one(
         "INSERT INTO email_log (alias_id, action, status, reason, from_address) VALUES (1, $1, \
          $2, $3, 'sender@x') RETURNING id",
         &[&action, &status, &reason],
      )
      .await
      .unwrap();
   row.get("id")
}

#[tokio::test]
async fn bounce_valid_hmac_marks_row_bounced() {
   let db = test_db!();
   let (state, _submit) = make_state(db.pool.clone());

   let log_id = {
      let client = db.pool.get().await.unwrap();
      seed_alias(&client).await;
      insert_log(&client, "forward", "submitted", None).await
   };

   let payload = verp::make_local_payload(&state.config.verp_key, BouncePhase::Forward, log_id);
   let verdict = pipeline::process(&state, Delivery {
      rcpt:      Rcpt::Bounce { payload },
      mail_from: "<>".into(),
      raw:       dsn_msg("550 5.1.1 user unknown"),
   })
   .await;
   assert!(matches!(verdict, Verdict::Delivered), "got {verdict:?}");

   let client = db.pool.get().await.unwrap();
   let row = client
      .query_one("SELECT status, reason FROM email_log WHERE id = $1", &[
         &log_id,
      ])
      .await
      .unwrap();
   assert_eq!(row.get::<_, String>("status"), "bounced");
   assert!(
      row.get::<_, String>("reason").contains("user unknown"),
      "reason should pull diagnostic-code"
   );
   db.teardown().await;
}

#[tokio::test]
async fn bounce_forged_hmac_does_not_mutate() {
   let db = test_db!();
   let (state, _submit) = make_state(db.pool.clone());

   let log_id = {
      let client = db.pool.get().await.unwrap();
      seed_alias(&client).await;
      insert_log(&client, "forward", "submitted", None).await
   };

   // Right shape, wrong HMAC.
   let forged = format!("f+{log_id}+AAAAAAAAAAAAAAAA");
   let verdict = pipeline::process(&state, Delivery {
      rcpt:      Rcpt::Bounce { payload: forged },
      mail_from: "<>".into(),
      raw:       dsn_msg("550 anything"),
   })
   .await;
   assert!(matches!(verdict, Verdict::Delivered), "got {verdict:?}");

   let client = db.pool.get().await.unwrap();
   let status: String = client
      .query_one("SELECT status FROM email_log WHERE id = $1", &[&log_id])
      .await
      .unwrap()
      .get("status");
   assert_eq!(status, "submitted", "forged VERP must not mutate");
   db.teardown().await;
}

#[tokio::test]
async fn bounce_phase_mismatch_no_mutation() {
   let db = test_db!();
   let (state, _submit) = make_state(db.pool.clone());

   let log_id = {
      let client = db.pool.get().await.unwrap();
      seed_alias(&client).await;
      // Insert as 'reply' but bounce as 'forward'.
      insert_log(&client, "reply", "submitted", None).await
   };

   let payload = verp::make_local_payload(&state.config.verp_key, BouncePhase::Forward, log_id);
   let verdict = pipeline::process(&state, Delivery {
      rcpt:      Rcpt::Bounce { payload },
      mail_from: "<>".into(),
      raw:       dsn_msg("550 anything"),
   })
   .await;
   assert!(matches!(verdict, Verdict::Delivered), "got {verdict:?}");

   let client = db.pool.get().await.unwrap();
   let status: String = client
      .query_one("SELECT status FROM email_log WHERE id = $1", &[&log_id])
      .await
      .unwrap()
      .get("status");
   assert_eq!(status, "submitted", "phase mismatch must not flip");
   db.teardown().await;
}

#[tokio::test]
async fn bounce_already_bounced_idempotent() {
   let db = test_db!();
   let (state, _submit) = make_state(db.pool.clone());

   let log_id = {
      let client = db.pool.get().await.unwrap();
      seed_alias(&client).await;
      insert_log(&client, "forward", "bounced", Some("first reason")).await
   };

   let payload = verp::make_local_payload(&state.config.verp_key, BouncePhase::Forward, log_id);
   let verdict = pipeline::process(&state, Delivery {
      rcpt:      Rcpt::Bounce { payload },
      mail_from: "<>".into(),
      raw:       dsn_msg("550 second reason"),
   })
   .await;
   assert!(matches!(verdict, Verdict::Delivered), "got {verdict:?}");

   // Already-bounced row: reason must not be overwritten.
   let client = db.pool.get().await.unwrap();
   let reason: String = client
      .query_one("SELECT reason FROM email_log WHERE id = $1", &[&log_id])
      .await
      .unwrap()
      .get("reason");
   assert_eq!(reason, "first reason");
   db.teardown().await;
}
