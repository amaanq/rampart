//! `rampart admin gc` cleanup tests. Seed each cleanable table with fresh AND
//! expired/used rows, run `admin::gc`, assert the right rows survive.

mod support;

use hmac_sha256::Hash;
use rampart::admin::gc;
use time::{Duration, OffsetDateTime};

use support::TestDb;
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

async fn seed_user(c: &deadpool_postgres::Client, email: &str) -> i64 {
    c.query_one(
        "INSERT INTO \"user\" (email, password_hash) VALUES ($1, 'x') RETURNING id",
        &[&email],
    )
    .await
    .unwrap()
    .get("id")
}

async fn seed_mailbox(c: &deadpool_postgres::Client, user_id: i64, email: &str) -> i64 {
    c.query_one(
        "INSERT INTO mailbox (user_id, email, verified) VALUES ($1, $2, TRUE) RETURNING id",
        &[&user_id, &email],
    )
    .await
    .unwrap()
    .get("id")
}

async fn count(c: &deadpool_postgres::Client, table: &str) -> i64 {
    let sql = format!("SELECT count(*)::bigint AS n FROM {table}");
    let row = c.query_one(&sql, &[]).await.unwrap();
    row.get("n")
}

fn h(s: &str) -> Vec<u8> {
    Hash::hash(s.as_bytes()).to_vec()
}

#[tokio::test]
async fn gc_clears_expired_and_used_tokens() {
    let db = test_db!();
    let uid = {
        let c = db.pool.get().await.unwrap();
        seed_user(&c, "alice@test").await
    };
    let now = OffsetDateTime::now_utc();
    let past = now - Duration::hours(2);
    let future = now + Duration::hours(2);

    {
        let c = db.pool.get().await.unwrap();
        // invite_token: fresh + expired + used
        c.execute(
            "INSERT INTO invite_token (token_hash, expires_at) VALUES ($1, $2)",
            &[&h("fresh") as &(dyn ToSql + Sync), &future],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO invite_token (token_hash, expires_at) VALUES ($1, $2)",
            &[&h("expired") as &(dyn ToSql + Sync), &past],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO invite_token (token_hash, expires_at, used_at) VALUES ($1, $2, now())",
            &[&h("used") as &(dyn ToSql + Sync), &future],
        )
        .await
        .unwrap();

        // password_reset_token
        c.execute(
            "INSERT INTO password_reset_token (token_hash, user_id, expires_at) VALUES ($1, $2, $3)",
            &[&h("fresh-pw") as &(dyn ToSql + Sync), &uid, &future],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO password_reset_token (token_hash, user_id, expires_at) VALUES ($1, $2, $3)",
            &[&h("expired-pw") as &(dyn ToSql + Sync), &uid, &past],
        )
        .await
        .unwrap();

        // email_change_token
        c.execute(
            "INSERT INTO email_change_token (token_hash, user_id, new_email, expires_at) \
             VALUES ($1, $2, 'a2@test', $3)",
            &[&h("fresh-em") as &(dyn ToSql + Sync), &uid, &future],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO email_change_token (token_hash, user_id, new_email, expires_at) \
             VALUES ($1, $2, 'a3@test', $3)",
            &[&h("used-em") as &(dyn ToSql + Sync), &uid, &future],
        )
        .await
        .unwrap();
        c.execute(
            "UPDATE email_change_token SET used_at = now() WHERE token_hash = $1",
            &[&h("used-em") as &(dyn ToSql + Sync)],
        )
        .await
        .unwrap();

        // mailbox_verify_token
        let mid = seed_mailbox(&c, uid, "alice@gmail.com").await;
        c.execute(
            "INSERT INTO mailbox_verify_token (token_hash, mailbox_id, expires_at) VALUES ($1, $2, $3)",
            &[&h("fresh-mv") as &(dyn ToSql + Sync), &mid, &future],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO mailbox_verify_token (token_hash, mailbox_id, expires_at) VALUES ($1, $2, $3)",
            &[&h("expired-mv") as &(dyn ToSql + Sync), &mid, &past],
        )
        .await
        .unwrap();
    }

    let stats = gc(&db.url, 90, false).await.unwrap();
    assert_eq!(stats.invite_token, 2, "expired + used invite removed");
    assert_eq!(stats.password_reset_token, 1, "expired pw token removed");
    assert_eq!(stats.email_change_token, 1, "used email_change removed");
    assert_eq!(
        stats.mailbox_verify_token, 1,
        "expired mailbox_verify removed"
    );

    // Verify only fresh rows survive.
    let c = db.pool.get().await.unwrap();
    assert_eq!(count(&c, "invite_token").await, 1);
    assert_eq!(count(&c, "password_reset_token").await, 1);
    assert_eq!(count(&c, "email_change_token").await, 1);
    assert_eq!(count(&c, "mailbox_verify_token").await, 1);

    db.teardown().await;
}

#[tokio::test]
async fn gc_clears_expired_webauthn_ceremony_and_session() {
    let db = test_db!();
    let uid = {
        let c = db.pool.get().await.unwrap();
        seed_user(&c, "alice@test").await
    };
    let now = OffsetDateTime::now_utc();
    let past = now - Duration::hours(1);
    let future = now + Duration::hours(1);

    {
        let c = db.pool.get().await.unwrap();
        // webauthn_ceremony — expiry only
        c.execute(
            "INSERT INTO webauthn_ceremony (id, user_id, kind, state_blob, expires_at) \
             VALUES ($1, $2, 'register', $3, $4)",
            &[
                &b"ceremony-fresh".as_slice() as &(dyn ToSql + Sync),
                &uid,
                &b"state".as_slice() as &(dyn ToSql + Sync),
                &future,
            ],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO webauthn_ceremony (id, user_id, kind, state_blob, expires_at) \
             VALUES ($1, $2, 'register', $3, $4)",
            &[
                &b"ceremony-expired".as_slice() as &(dyn ToSql + Sync),
                &uid,
                &b"state".as_slice() as &(dyn ToSql + Sync),
                &past,
            ],
        )
        .await
        .unwrap();

        // session
        c.execute(
            "INSERT INTO session (id, user_id, expires_at) VALUES ($1, $2, $3)",
            &[
                &b"sess-fresh".as_slice() as &(dyn ToSql + Sync),
                &uid,
                &future,
            ],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO session (id, user_id, expires_at) VALUES ($1, $2, $3)",
            &[
                &b"sess-expired".as_slice() as &(dyn ToSql + Sync),
                &uid,
                &past,
            ],
        )
        .await
        .unwrap();
    }

    let stats = gc(&db.url, 90, false).await.unwrap();
    assert_eq!(stats.webauthn_ceremony, 1);
    assert_eq!(stats.session, 1);

    let c = db.pool.get().await.unwrap();
    assert_eq!(count(&c, "webauthn_ceremony").await, 1);
    assert_eq!(count(&c, "session").await, 1);

    db.teardown().await;
}

#[tokio::test]
async fn gc_clears_old_rate_limit_buckets() {
    let db = test_db!();
    let two_days_ago = OffsetDateTime::now_utc() - Duration::hours(24 * 2);
    let recent = OffsetDateTime::now_utc() - Duration::hours(2);
    {
        let c = db.pool.get().await.unwrap();
        c.execute(
            "INSERT INTO rate_limit_bucket (key, count, window_start) VALUES ($1, 1, $2)",
            &[&"old", &two_days_ago],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO rate_limit_bucket (key, count, window_start) VALUES ($1, 1, $2)",
            &[&"recent", &recent],
        )
        .await
        .unwrap();
    }

    let stats = gc(&db.url, 90, false).await.unwrap();
    assert_eq!(stats.rate_limit_bucket, 1);

    let c = db.pool.get().await.unwrap();
    assert_eq!(count(&c, "rate_limit_bucket").await, 1);

    db.teardown().await;
}

#[tokio::test]
async fn gc_clears_old_email_log() {
    let db = test_db!();
    let (uid, mid, did, alias_id) = {
        let c = db.pool.get().await.unwrap();
        let uid = seed_user(&c, "alice@test").await;
        let mid = seed_mailbox(&c, uid, "alice@gmail.com").await;
        let did: i64 = c
            .query_one(
                "INSERT INTO alias_domain (domain, owner_id, shared) VALUES ('addy.test', $1, FALSE) RETURNING id",
                &[&uid],
            )
            .await
            .unwrap()
            .get("id");
        let aid: i64 = c
            .query_one(
                "INSERT INTO alias (user_id, address, domain_id, mailbox_id) \
                 VALUES ($1, 'a@addy.test', $2, $3) RETURNING id",
                &[&uid, &did, &mid],
            )
            .await
            .unwrap()
            .get("id");
        (uid, mid, did, aid)
    };
    let _ = (uid, mid, did);

    let old = OffsetDateTime::now_utc() - Duration::hours(24 * 100);
    let recent = OffsetDateTime::now_utc() - Duration::hours(24 * 10);
    {
        let c = db.pool.get().await.unwrap();
        c.execute(
            "INSERT INTO email_log (alias_id, action, created_at) VALUES ($1, 'forward', $2)",
            &[&alias_id, &old],
        )
        .await
        .unwrap();
        c.execute(
            "INSERT INTO email_log (alias_id, action, created_at) VALUES ($1, 'forward', $2)",
            &[&alias_id, &recent],
        )
        .await
        .unwrap();
    }

    // Use 90-day retention — old (100 days) cleared, recent (10 days) survives.
    let stats = gc(&db.url, 90, false).await.unwrap();
    assert_eq!(stats.email_log, 1);

    let c = db.pool.get().await.unwrap();
    assert_eq!(count(&c, "email_log").await, 1);

    db.teardown().await;
}

#[tokio::test]
async fn gc_dry_run_changes_nothing() {
    let db = test_db!();
    let uid = {
        let c = db.pool.get().await.unwrap();
        seed_user(&c, "alice@test").await
    };
    let past = OffsetDateTime::now_utc() - Duration::hours(2);
    {
        let c = db.pool.get().await.unwrap();
        c.execute(
            "INSERT INTO password_reset_token (token_hash, user_id, expires_at) VALUES ($1, $2, $3)",
            &[&h("expired") as &(dyn ToSql + Sync), &uid, &past],
        )
        .await
        .unwrap();
    }
    let before = {
        let c = db.pool.get().await.unwrap();
        count(&c, "password_reset_token").await
    };

    let stats = gc(&db.url, 90, true).await.unwrap();
    assert_eq!(
        stats.password_reset_token, 1,
        "dry-run should still report counts"
    );

    let after = {
        let c = db.pool.get().await.unwrap();
        count(&c, "password_reset_token").await
    };
    assert_eq!(before, after, "dry-run must not delete");

    db.teardown().await;
}

#[tokio::test]
async fn gc_idempotent_second_run_is_zero() {
    let db = test_db!();
    let uid = {
        let c = db.pool.get().await.unwrap();
        seed_user(&c, "alice@test").await
    };
    let past = OffsetDateTime::now_utc() - Duration::hours(2);
    {
        let c = db.pool.get().await.unwrap();
        for k in ["a", "b", "c"] {
            c.execute(
                "INSERT INTO password_reset_token (token_hash, user_id, expires_at) VALUES ($1, $2, $3)",
                &[&h(k) as &(dyn ToSql + Sync), &uid, &past],
            )
            .await
            .unwrap();
        }
    }
    let first = gc(&db.url, 90, false).await.unwrap();
    assert_eq!(first.password_reset_token, 3);
    let second = gc(&db.url, 90, false).await.unwrap();
    assert_eq!(second.password_reset_token, 0);

    db.teardown().await;
}
