//! Forward + reply pipeline tests with ephemeral DB + MemorySubmit.
//! Bounce-VERP tests live in `tests/bounce.rs`.
//!
//! `tests/fixtures/ar_real.txt` is the AR header we parse against.
//! Replace it with a real captured header from your stalwart if the
//! parser ever drifts.

mod support;

use std::sync::Arc;

use rampart::config::Config;
use rampart::mailer::MemoryMailer;
use rampart::worker::{
    WorkerState,
    loop_guard::Rcpt,
    pipeline::{Delivery, Verdict, process},
    resubmit::MemorySubmit,
};
use support::TestDb;
use tokio_postgres::types::ToSql;

const STALWART_HOSTNAME: &str = "test.example";

// Shared with all DB-backed suites — see TestDb::or_skip docs.
macro_rules! test_db {
    () => {
        match TestDb::or_skip().await {
            Some(db) => db,
            None => return,
        }
    };
}

fn real_ar_header() -> String {
    // First non-empty, non-comment (`##`-prefixed) line. Lets the
    // fixture file carry maintainer notes alongside the actual header
    // value without those notes leaking into the test message.
    let raw = include_str!("fixtures/ar_real.txt");
    raw.lines()
        .find(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("##")
        })
        .expect("ar_real.txt must contain at least one AR-header line")
        .trim_end()
        .to_owned()
}

/// Build the worker state used by all reply-path tests. Returns the
/// state plus the MemorySubmit so the test can drain captured outbound
/// submissions.
fn make_state(pool: deadpool_postgres::Pool) -> (WorkerState, Arc<MemorySubmit>) {
    let cfg = Config {
        database_url: "irrelevant".into(),
        listen: "127.0.0.1:0".parse().unwrap(),
        public_origin: "http://localhost".into(),
        static_dir: "static".into(),
        sieve_output_path: None,
        smtp_host: "localhost".into(),
        smtp_port: 465,
        smtp_user: "x@x".into(),
        smtp_password_file: None,
        notifier_from: "\"rampart\" <x@x>".into(),
        webauthn_rp_id: "localhost".into(),
        lmtp_listen: "127.0.0.1:0".parse().unwrap(),
        stalwart_hostname: STALWART_HOSTNAME.into(),
        public_mx_hostname: STALWART_HOSTNAME.into(),
        lmtp_drain_secs: 20,
        stalwart_jmap_base_url: None,
        stalwart_admin_username: "admin".into(),
        stalwart_admin_password_file: None,
        verp_key: b"test-key-32-bytes-long-padding-padding".to_vec(),
    };
    let submit = Arc::new(MemorySubmit::new());
    let state = WorkerState {
        pool,
        config: Arc::new(cfg),
        mailer: Arc::new(MemoryMailer::new()),
        submit: submit.clone(),
    };
    (state, submit)
}

/// Insert user / mailbox / domain / alias. Returns ids.
async fn seed(
    c: &deadpool_postgres::Client,
    user_email: &str,
    mailbox_email: &str,
    alias_domain: &str,
    alias_local: &str,
) -> (i64, i64, i64, i64) {
    let u: i64 = c
        .query_one(
            "INSERT INTO \"user\" (email, password_hash) VALUES ($1, 'x') RETURNING id",
            &[&user_email],
        )
        .await
        .unwrap()
        .get("id");
    let m: i64 = c
        .query_one(
            "INSERT INTO mailbox (user_id, email, verified) VALUES ($1, $2, TRUE) RETURNING id",
            &[&u, &mailbox_email],
        )
        .await
        .unwrap()
        .get("id");
    let d: i64 = c
        .query_one(
            "INSERT INTO alias_domain (domain, owner_id, shared) VALUES ($1, $2, FALSE) RETURNING id",
            &[&alias_domain, &u],
        )
        .await
        .unwrap()
        .get("id");
    let alias_addr = format!("{alias_local}@{alias_domain}");
    let a: i64 = c
        .query_one(
            "INSERT INTO alias (user_id, address, domain_id, mailbox_id) \
             VALUES ($1, $2, $3, $4) RETURNING id",
            &[&u, &alias_addr, &d, &m],
        )
        .await
        .unwrap()
        .get("id");
    (u, m, d, a)
}

async fn insert_reverse_contact(
    c: &deadpool_postgres::Client,
    alias_id: i64,
    real_email: &str,
    alias_domain: &str,
) -> i64 {
    let token = format!("tok-{alias_id}-{}", real_email.replace(['@', '.'], ""));
    let reply = format!("ra+{token}@{alias_domain}");
    c.query_one(
        "INSERT INTO reverse_contact (alias_id, real_email, token, reply_address) \
         VALUES ($1, $2, $3, $4) RETURNING id",
        &[&alias_id, &real_email, &token, &reply],
    )
    .await
    .unwrap()
    .get("id")
}

/// Build a forward-direction inbound message body: external sender with
/// a regular From header, no Authentication-Results header.
fn forward_msg(from: &str, subject: &str, body: &str) -> Vec<u8> {
    format!(
        "From: <{from}>\r\nSubject: {subject}\r\nDate: Thu, 24 Apr 2026 12:00:00 +0000\r\n\r\n{body}"
    )
    .into_bytes()
}

/// Reply-direction inbound: visible From is the mailbox owner; AR
/// header is what stalwart wrote during DATA. `ar_authserv_id` controls
/// what authserv id to use ("test.example" = ours; anything else =
/// upstream-only).
fn reply_msg(visible_from: &str, ar_line: &str, body: &str) -> Vec<u8> {
    format!(
        "Authentication-Results: {ar_line}\r\nFrom: <{visible_from}>\r\nSubject: re\r\nDate: now\r\n\r\n{body}"
    )
    .into_bytes()
}

#[tokio::test]
async fn forward_happy_path() {
    let db = test_db!();
    {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
    }
    let (state, submit) = make_state(db.pool.clone());
    let alias_id = 1;
    let raw = forward_msg("ext@sender.test", "hi", "body bytes");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Forward(alias_id),
            mail_from: "ext@sender.test".into(),
            raw: raw.clone(),
        },
    )
    .await;
    assert!(matches!(v, Verdict::Delivered), "got {v:?}");
    let captured = submit.drain();
    assert_eq!(captured.len(), 1);
    // Codex P1.1: outbound forward MAIL FROM is the HMAC-signed bounce
    // VERP, not the alias address. Shape: bnc+f+<id>+<tag>@<domain>.
    assert!(
        captured[0].mail_from.starts_with("bnc+f+"),
        "MAIL FROM should be a bounce VERP, got {:?}",
        captured[0].mail_from
    );
    assert!(
        captured[0].mail_from.ends_with("@addy.test"),
        "VERP must use the alias domain, got {:?}",
        captured[0].mail_from
    );
    assert_eq!(captured[0].rcpt_to, "alice@gmail.com");
    let body_str = String::from_utf8_lossy(&captured[0].body);
    // From rewritten to a reverse-contact reply address on the alias domain.
    assert!(body_str.contains("From: "), "no From: in body: {body_str}");
    assert!(
        body_str.contains("@addy.test>"),
        "no addy.test From: {body_str}"
    );
    assert!(body_str.contains("body bytes"), "body lost: {body_str}");

    db.teardown().await;
}

#[tokio::test]
async fn forward_creates_reverse_contact() {
    let db = test_db!();
    {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
    }
    let (state, _submit) = make_state(db.pool.clone());
    let raw = forward_msg("friend@example.org", "hi", "msg");
    let _ = process(
        &state,
        Delivery {
            rcpt: Rcpt::Forward(1),
            mail_from: "friend@example.org".into(),
            raw,
        },
    )
    .await;

    let c = db.pool.get().await.unwrap();
    let row = c
        .query_one(
            "SELECT alias_id, real_email::text AS email FROM reverse_contact ORDER BY id DESC LIMIT 1",
            &[],
        )
        .await
        .unwrap();
    let real: String = row.get("email");
    let aid: i64 = row.get("alias_id");
    assert_eq!(real, "friend@example.org");
    assert_eq!(aid, 1);

    db.teardown().await;
}

#[tokio::test]
async fn forward_disabled_alias_5xx() {
    let db = test_db!();
    {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
        c.execute("UPDATE alias SET enabled = FALSE WHERE id = 1", &[])
            .await
            .unwrap();
    }
    let (state, _submit) = make_state(db.pool.clone());
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Forward(1),
            mail_from: "ext@sender.test".into(),
            raw: forward_msg("ext@sender.test", "hi", "x"),
        },
    )
    .await;
    assert!(matches!(v, Verdict::Perm { .. }), "got {v:?}");
    db.teardown().await;
}

#[tokio::test]
async fn reply_dmarc_pass_aligned_using_real_ar_header() {
    let db = test_db!();
    let (rc_id, _alias_addr) = {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
        let rc = insert_reverse_contact(&c, 1, "friend@example.org", "addy.test").await;
        (rc, "abc@addy.test".to_owned())
    };
    let (state, submit) = make_state(db.pool.clone());
    let raw = reply_msg("alice@gmail.com", &real_ar_header(), "reply body");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Reply(rc_id),
            mail_from: "alice@gmail.com".into(),
            raw,
        },
    )
    .await;
    assert!(matches!(v, Verdict::Delivered), "got {v:?}");
    let captured = submit.drain();
    assert_eq!(captured.len(), 1);
    // Codex P1.1: reply MAIL FROM is also a signed bounce VERP, on the
    // alias_domain so DSN bounces flow back to us authenticated.
    assert!(
        captured[0].mail_from.starts_with("bnc+r+"),
        "reply MAIL FROM should be a bounce VERP, got {:?}",
        captured[0].mail_from
    );
    assert!(
        captured[0].mail_from.ends_with("@addy.test"),
        "reply VERP must use alias domain, got {:?}",
        captured[0].mail_from
    );
    assert_eq!(captured[0].rcpt_to, "friend@example.org");
    db.teardown().await;
}

#[tokio::test]
async fn reply_dmarc_fail_5xx() {
    let db = test_db!();
    let rc_id = {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
        insert_reverse_contact(&c, 1, "friend@example.org", "addy.test").await
    };
    let (state, submit) = make_state(db.pool.clone());
    let ar = format!("{STALWART_HOSTNAME}; dmarc=fail header.from=gmail.com");
    let raw = reply_msg("alice@gmail.com", &ar, "x");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Reply(rc_id),
            mail_from: "alice@gmail.com".into(),
            raw,
        },
    )
    .await;
    assert!(matches!(v, Verdict::Perm { .. }), "got {v:?}");
    assert!(submit.drain().is_empty());
    db.teardown().await;
}

#[tokio::test]
async fn reply_unaligned_5xx() {
    let db = test_db!();
    let rc_id = {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
        insert_reverse_contact(&c, 1, "friend@example.org", "addy.test").await
    };
    let (state, _submit) = make_state(db.pool.clone());
    // AR pass for spoof.com but mailbox is at gmail.com — alignment fails.
    let ar = format!("{STALWART_HOSTNAME}; dmarc=pass header.from=spoof.com");
    let raw = reply_msg("imposter@spoof.com", &ar, "x");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Reply(rc_id),
            mail_from: "imposter@spoof.com".into(),
            raw,
        },
    )
    .await;
    assert!(matches!(v, Verdict::Perm { .. }), "got {v:?}");
    db.teardown().await;
}

#[tokio::test]
async fn reply_cross_user_same_domain_rejected_then_exact_match_passes() {
    let db = test_db!();
    let rc_id = {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
        insert_reverse_contact(&c, 1, "friend@example.org", "addy.test").await
    };
    let (state, submit) = make_state(db.pool.clone());
    let ar = format!("{STALWART_HOSTNAME}; dmarc=pass header.from=gmail.com");
    // Same gmail tenant, different user — exact-mailbox check rejects.
    let raw_other = reply_msg("bob@gmail.com", &ar, "x");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Reply(rc_id),
            mail_from: "bob@gmail.com".into(),
            raw: raw_other,
        },
    )
    .await;
    assert!(
        matches!(v, Verdict::Perm { .. }),
        "cross-user must reject: {v:?}"
    );
    assert!(submit.drain().is_empty());

    // Exact match passes and produces one outbound submit.
    let raw_self = reply_msg("alice@gmail.com", &ar, "x");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Reply(rc_id),
            mail_from: "alice@gmail.com".into(),
            raw: raw_self,
        },
    )
    .await;
    assert!(matches!(v, Verdict::Delivered), "exact must pass: {v:?}");
    assert_eq!(submit.drain().len(), 1);

    db.teardown().await;
}

#[tokio::test]
async fn reply_block_reply_5xx() {
    let db = test_db!();
    let rc_id = {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
        let rc = insert_reverse_contact(&c, 1, "friend@example.org", "addy.test").await;
        c.execute(
            "UPDATE reverse_contact SET block_reply = TRUE WHERE id = $1",
            &[&rc as &(dyn ToSql + Sync)],
        )
        .await
        .unwrap();
        rc
    };
    let (state, submit) = make_state(db.pool.clone());
    let ar = format!("{STALWART_HOSTNAME}; dmarc=pass header.from=gmail.com");
    let raw = reply_msg("alice@gmail.com", &ar, "x");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Reply(rc_id),
            mail_from: "alice@gmail.com".into(),
            raw,
        },
    )
    .await;
    assert!(matches!(v, Verdict::Perm { .. }), "got {v:?}");
    assert!(submit.drain().is_empty());
    db.teardown().await;
}

#[tokio::test]
async fn reply_parser_divergence_rejected() {
    let db = test_db!();
    let rc_id = {
        let c = db.pool.get().await.unwrap();
        seed(&c, "alice@test", "alice@gmail.com", "addy.test", "abc").await;
        insert_reverse_contact(&c, 1, "friend@example.org", "addy.test").await
    };
    let (state, _submit) = make_state(db.pool.clone());
    // Stalwart vouched for gmail.com; we parse impostor.example in the
    // visible From — binding check rejects so an attacker can't exploit
    // a parser split between us and stalwart. Discriminate from the
    // alignment-failure case (which would also Perm-reject) by asserting
    // the binding error message specifically.
    let ar = format!("{STALWART_HOSTNAME}; dmarc=pass header.from=gmail.com");
    let raw = reply_msg("alice@impostor.example", &ar, "x");
    let v = process(
        &state,
        Delivery {
            rcpt: Rcpt::Reply(rc_id),
            mail_from: "alice@impostor.example".into(),
            raw,
        },
    )
    .await;
    match &v {
        Verdict::Perm { internal, smtp } => {
            assert!(
                internal.contains("does not match parsed visible From"),
                "expected binding-check error, got internal: {internal}"
            );
            assert_eq!(
                *smtp, "reply rejected",
                "SMTP text should be generic, leaks otherwise"
            );
        }
        _ => panic!("expected Perm verdict, got {v:?}"),
    }
    db.teardown().await;
}
