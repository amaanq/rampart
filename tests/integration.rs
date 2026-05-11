//! Integration tests.
//!
//! Require `RAMPART_TEST_DB_URL` pointing at a postgres with CREATEDB. Skip
//! silently if absent so `cargo test` on a dev box without a DB works.

mod support;

use support::TestDb;

// Shared with all DB-backed suites — see TestDb::or_skip docs.
macro_rules! test_db {
    () => {
        match TestDb::or_skip().await {
            Some(db) => db,
            None => return,
        }
    };
}

/// Seed a user + mailbox + domain. Returns (user_id, mailbox_id, domain_id).
async fn seed_basic(
    client: &deadpool_postgres::Client,
    user_email: &str,
    mailbox: &str,
    domain: &str,
) -> (i64, i64, i64) {
    let u: i64 = client
        .query_one(
            "INSERT INTO \"user\" (email, password_hash) VALUES ($1, 'placeholder') RETURNING id",
            &[&user_email],
        )
        .await
        .unwrap()
        .get("id");
    let m: i64 = client
        .query_one(
            "INSERT INTO mailbox (user_id, email, verified) VALUES ($1, $2, TRUE) RETURNING id",
            &[&u, &mailbox],
        )
        .await
        .unwrap()
        .get("id");
    let d: i64 = client
        .query_one(
            "INSERT INTO alias_domain (domain, owner_id, shared) VALUES ($1, $2, FALSE) RETURNING id",
            &[&domain, &u],
        )
        .await
        .unwrap()
        .get("id");
    (u, m, d)
}

#[tokio::test]
async fn migrations_apply_cleanly() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let row = c
        .query_one(
            "SELECT COUNT(*)::int AS n FROM refinery_schema_history",
            &[],
        )
        .await
        .unwrap();
    let n: i32 = row.get("n");
    assert!(n >= 1, "at least one migration applied, got {n}");
    db.teardown().await;
}

#[tokio::test]
async fn insert_domain_mailbox_and_alias_round_trip() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;

    c.execute(
        "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
        &[&u, &"abc123@addy.test", &d, &m],
    )
    .await
    .unwrap();

    let row = c
        .query_one(
            "SELECT forward_to, enabled, user_id FROM rampart_sieve_lookup WHERE address = $1",
            &[&"abc123@addy.test"],
        )
        .await
        .unwrap();
    let forward: String = row.get("forward_to");
    let enabled: bool = row.get("enabled");
    let row_user_id: i64 = row.get("user_id");
    assert_eq!(forward, "alice@mail.test");
    assert!(enabled);
    assert_eq!(row_user_id, u);
    db.teardown().await;
}

#[tokio::test]
async fn alias_validate_rejects_mismatched_domain() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;

    let err = c
        .execute(
            "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
            &[&u, &"abc@wrong.test", &d, &m],
        )
        .await
        .expect_err("mismatched domain must be rejected");
    let msg = err
        .as_db_error()
        .map(|e| e.message().to_string())
        .unwrap_or_default();
    assert!(msg.contains("does not match"), "got: {msg}");
    db.teardown().await;
}

#[tokio::test]
async fn alias_validate_rejects_reply_prefix() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;

    let err = c
        .execute(
            "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
            &[&u, &"ra+abc@addy.test", &d, &m],
        )
        .await
        .expect_err("reply-prefix local-part must be rejected");
    let msg = err
        .as_db_error()
        .map(|e| e.message().to_string())
        .unwrap_or_default();
    assert!(msg.contains("reserved"), "got: {msg}");
    db.teardown().await;
}

#[tokio::test]
async fn alias_validate_rejects_other_users_mailbox() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (alice, _am, adom) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    let (_bob, bm, _bdom) = seed_basic(&c, "bob@test", "bob@mail.test", "bob-private.test").await;

    let err = c
        .execute(
            "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
            &[&alice, &"a1@addy.test", &adom, &bm],
        )
        .await
        .expect_err("alice using bob's mailbox must be rejected");
    let msg = err
        .as_db_error()
        .map(|e| e.message().to_string())
        .unwrap_or_default();
    assert!(msg.contains("does not belong to user"), "got: {msg}");
    db.teardown().await;
}

#[tokio::test]
async fn alias_validate_rejects_other_users_private_domain() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (alice, am, _adom) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    let (_bob, _bm, bdom) = seed_basic(&c, "bob@test", "bob@mail.test", "bob-private.test").await;

    let err = c
        .execute(
            "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
            &[&alice, &"sneak@bob-private.test", &bdom, &am],
        )
        .await
        .expect_err("alice using bob's private domain must be rejected");
    let msg = err
        .as_db_error()
        .map(|e| e.message().to_string())
        .unwrap_or_default();
    assert!(msg.contains("not accessible"), "got: {msg}");
    db.teardown().await;
}

#[tokio::test]
async fn shared_domain_accepted_for_any_user() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();

    // Admin sets up a shared domain
    let admin: i64 = c
        .query_one(
            "INSERT INTO \"user\" (email, password_hash, is_admin) VALUES ($1, 'x', TRUE) RETURNING id",
            &[&"admin@test"],
        )
        .await
        .unwrap()
        .get("id");
    let _admin_mb: i64 = c
        .query_one(
            "INSERT INTO mailbox (user_id, email, verified) VALUES ($1, $2, TRUE) RETURNING id",
            &[&admin, &"admin@mail.test"],
        )
        .await
        .unwrap()
        .get("id");
    let shared_dom: i64 = c
        .query_one(
            "INSERT INTO alias_domain (domain, owner_id, shared) VALUES ($1, $2, TRUE) RETURNING id",
            &[&"shared.test", &admin],
        )
        .await
        .unwrap()
        .get("id");

    // Regular user alice can create aliases on the shared domain
    let (alice, am, _adom) =
        seed_basic(&c, "alice@test", "alice@mail.test", "alice-private.test").await;
    c.execute(
        "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
        &[&alice, &"pooled@shared.test", &shared_dom, &am],
    )
    .await
    .expect("shared domain should accept non-owner users");

    db.teardown().await;
}

#[tokio::test]
async fn reverse_contact_requires_reply_prefix() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    c.execute(
        "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
        &[&u, &"orig@addy.test", &d, &m],
    )
    .await
    .unwrap();

    // Missing reply prefix → reject
    let err = c
        .execute(
            "INSERT INTO reverse_contact (alias_id, real_email, token, reply_address) \
             VALUES (1, $1, $2, $3)",
            &[&"party@example.com", &"tok1", &"party123@addy.test"],
        )
        .await
        .expect_err("must require reply_prefix");
    let msg = err
        .as_db_error()
        .map(|e| e.message().to_string())
        .unwrap_or_default();
    assert!(msg.contains("reply_prefix"), "got: {msg}");

    // With reply prefix → accepted
    c.execute(
        "INSERT INTO reverse_contact (alias_id, real_email, token, reply_address) \
         VALUES (1, $1, $2, $3)",
        &[&"party@example.com", &"tok2", &"ra+party123@addy.test"],
    )
    .await
    .expect("reply_prefix match should be accepted");

    db.teardown().await;
}

#[tokio::test]
async fn sieve_view_respects_user_enabled() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    c.execute(
        "INSERT INTO alias (user_id, address, domain_id, mailbox_id) VALUES ($1, $2, $3, $4)",
        &[&u, &"on@addy.test", &d, &m],
    )
    .await
    .unwrap();

    // Alias enabled
    let row = c
        .query_one(
            "SELECT enabled FROM rampart_sieve_lookup WHERE address = $1",
            &[&"on@addy.test"],
        )
        .await
        .unwrap();
    assert!(row.get::<_, bool>("enabled"));

    // Disable the user → alias becomes enabled=false in the view
    c.execute("UPDATE \"user\" SET enabled = FALSE WHERE id = $1", &[&u])
        .await
        .unwrap();
    let row = c
        .query_one(
            "SELECT enabled FROM rampart_sieve_lookup WHERE address = $1",
            &[&"on@addy.test"],
        )
        .await
        .unwrap();
    assert!(!row.get::<_, bool>("enabled"));

    db.teardown().await;
}

/// Enable catch-all on a private domain — set default_mailbox_id to the
/// owner's verified mailbox and flip catch_all=TRUE. The schema has a
/// `catch_all_requires_cap` CHECK constraint forcing max_auto_created
/// non-NULL whenever catch_all=TRUE; passing `None` here means "set a
/// permissive cap suitable for tests that don't care about cap behavior".
async fn seed_catchall_domain(
    client: &deadpool_postgres::Client,
    user_id: i64,
    mailbox_id: i64,
    domain_id: i64,
    max_auto: Option<i32>,
) {
    let effective_cap = max_auto.unwrap_or(1_000_000);
    client
        .execute(
            "UPDATE alias_domain SET default_mailbox_id = $2, catch_all = TRUE, max_auto_created = $3 \
             WHERE id = $1",
            &[&domain_id, &mailbox_id, &effective_cap],
        )
        .await
        .unwrap();
    let _ = user_id;
}

#[tokio::test]
async fn catch_all_enabled_creates_on_demand() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    seed_catchall_domain(&c, u, m, d, None).await;

    let row = c
        .query_one(
            "SELECT rampart_resolve_or_create('fresh@addy.test'::CITEXT) AS id",
            &[],
        )
        .await
        .unwrap();
    let id: Option<i64> = row.get("id");
    let id = id.expect("catch-all should auto-create");
    let row = c
        .query_one(
            "SELECT address::text AS address, auto_created FROM alias WHERE id = $1",
            &[&id],
        )
        .await
        .unwrap();
    let address: String = row.get("address");
    let auto_created: bool = row.get("auto_created");
    assert_eq!(address, "fresh@addy.test");
    assert!(auto_created);

    db.teardown().await;
}

#[tokio::test]
async fn catch_all_disabled_returns_null() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    // set default mailbox but leave catch_all = FALSE
    c.execute(
        "UPDATE alias_domain SET default_mailbox_id = $2 WHERE id = $1",
        &[&d, &m],
    )
    .await
    .unwrap();
    let _ = u;

    let row = c
        .query_one(
            "SELECT rampart_resolve_or_create('fresh@addy.test'::CITEXT) AS id",
            &[],
        )
        .await
        .unwrap();
    assert!(row.get::<_, Option<i64>>("id").is_none());

    db.teardown().await;
}

#[tokio::test]
async fn catch_all_idempotent() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    seed_catchall_domain(&c, u, m, d, None).await;

    let id1: i64 = c
        .query_one(
            "SELECT rampart_resolve_or_create('x@addy.test'::CITEXT) AS id",
            &[],
        )
        .await
        .unwrap()
        .get::<_, Option<i64>>("id")
        .unwrap();
    let id2: i64 = c
        .query_one(
            "SELECT rampart_resolve_or_create('x@addy.test'::CITEXT) AS id",
            &[],
        )
        .await
        .unwrap()
        .get::<_, Option<i64>>("id")
        .unwrap();
    assert_eq!(id1, id2, "same address must resolve to same id");

    db.teardown().await;
}

#[tokio::test]
async fn catch_all_respects_default_mailbox_disabled() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    seed_catchall_domain(&c, u, m, d, None).await;
    c.execute("UPDATE mailbox SET enabled = FALSE WHERE id = $1", &[&m])
        .await
        .unwrap();

    let row = c
        .query_one(
            "SELECT rampart_resolve_or_create('blah@addy.test'::CITEXT) AS id",
            &[],
        )
        .await
        .unwrap();
    assert!(row.get::<_, Option<i64>>("id").is_none());

    db.teardown().await;
}

#[tokio::test]
async fn catch_all_respects_owner_disabled() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    seed_catchall_domain(&c, u, m, d, None).await;
    c.execute("UPDATE \"user\" SET enabled = FALSE WHERE id = $1", &[&u])
        .await
        .unwrap();

    let row = c
        .query_one(
            "SELECT rampart_resolve_or_create('blah@addy.test'::CITEXT) AS id",
            &[],
        )
        .await
        .unwrap();
    assert!(row.get::<_, Option<i64>>("id").is_none());

    db.teardown().await;
}

#[tokio::test]
async fn catch_all_respects_max_auto_created() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    seed_catchall_domain(&c, u, m, d, Some(3)).await;

    for local in ["one", "two", "three"] {
        let row = c
            .query_one(
                &format!("SELECT rampart_resolve_or_create('{local}@addy.test'::CITEXT) AS id"),
                &[],
            )
            .await
            .unwrap();
        assert!(
            row.get::<_, Option<i64>>("id").is_some(),
            "'{local}' should succeed"
        );
    }
    // Fourth distinct local-part should be rejected.
    let row = c
        .query_one(
            "SELECT rampart_resolve_or_create('four@addy.test'::CITEXT) AS id",
            &[],
        )
        .await
        .unwrap();
    assert!(row.get::<_, Option<i64>>("id").is_none(), "cap must hold");

    db.teardown().await;
}

#[tokio::test]
async fn catch_all_concurrent_cap_not_exceeded() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (u, m, d) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    seed_catchall_domain(&c, u, m, d, Some(3)).await;
    drop(c);

    // Fire 10 concurrent resolves against 10 distinct local-parts. Exactly
    // 3 must succeed (matching max_auto_created); the rest return NULL.
    // Proves the advisory lock serializes the cap check + insert.
    let pool = db.pool.clone();
    let mut handles = Vec::new();
    for i in 0..10 {
        let pool = pool.clone();
        handles.push(tokio::spawn(async move {
            let c = pool.get().await.unwrap();
            let row = c
                .query_one(
                    &format!("SELECT rampart_resolve_or_create('race{i}@addy.test'::CITEXT) AS id"),
                    &[],
                )
                .await
                .unwrap();
            row.get::<_, Option<i64>>("id")
        }));
    }
    let mut successes = 0;
    for h in handles {
        if h.await.unwrap().is_some() {
            successes += 1;
        }
    }
    assert_eq!(
        successes, 3,
        "max_auto_created=3 must be a hard cap under concurrency"
    );

    // Verify final row count in the table matches.
    let c = db.pool.get().await.unwrap();
    let row = c
        .query_one(
            "SELECT count(*)::bigint AS n FROM alias WHERE domain_id = $1 AND auto_created",
            &[&d],
        )
        .await
        .unwrap();
    assert_eq!(row.get::<_, i64>("n"), 3);

    db.teardown().await;
}

#[tokio::test]
async fn catch_all_default_mailbox_must_belong_to_owner() {
    let db = test_db!();
    let c = db.pool.get().await.unwrap();
    let (_alice, am, _adom) = seed_basic(&c, "alice@test", "alice@mail.test", "addy.test").await;
    let (bob, _bm, _bdom) = seed_basic(&c, "bob@test", "bob@mail.test", "bob-private.test").await;

    // bob owns a domain but tries to set alice's mailbox as default — should fail
    c.execute(
        "INSERT INTO alias_domain (domain, owner_id, shared) VALUES ($1, $2, FALSE)",
        &[&"bob-other.test", &bob],
    )
    .await
    .unwrap();
    let err = c
        .execute(
            "UPDATE alias_domain SET default_mailbox_id = $2 WHERE domain = $1",
            &[&"bob-other.test", &am],
        )
        .await
        .expect_err("can't set default_mailbox to another user's mailbox");
    let msg = err
        .as_db_error()
        .map(|e| e.message().to_string())
        .unwrap_or_default();
    assert!(
        msg.contains("does not belong to domain owner"),
        "got: {msg}"
    );

    db.teardown().await;
}
