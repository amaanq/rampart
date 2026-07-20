//! Shared test helpers. Spins up a unique throwaway database per test
//! so the tests run in parallel without interfering. `RAMPART_TEST_DB_URL`
//! must be set to a libpq connection string with CREATEDB privilege.
#![expect(
   clippy::print_stderr,
   reason = "test helper reports skip reason to stderr"
)]

use std::{
   env,
   iter,
};

use anyhow::{
   Context as _,
   Result,
};
use deadpool_postgres::Pool;
use rampart::migrate;
use rand::RngExt as _;
use tokio_postgres::NoTls;

#[expect(
   clippy::partial_pub_fields,
   reason = "test helper exposes handles, hides admin URL"
)]
pub struct TestDb {
   #[expect(
      clippy::allow_attributes,
      reason = "dead_code varies per test binary (url is read only by gc.rs), so allow — not \
                expect — is required"
   )]
   #[allow(
      dead_code,
      reason = "read only by gc.rs; dead in the other integration test binaries"
   )]
   pub url:   String,
   pub name:  String,
   pub pool:  Pool,
   admin_url: String,
}

impl TestDb {
   /// Build a fresh per-test database. Errors are bubbled up so a
   /// broken setup is legible. `or_skip` is what tests actually call
   /// — it interprets "missing env var" as a skip but bubbles real
   /// failures.
   ///
   /// # Errors
   ///
   /// Returns an error if `RAMPART_TEST_DB_URL` is unset, the admin or
   /// per-test connection fails, `CREATE DATABASE` fails, or migrations
   /// do not apply.
   pub async fn try_new() -> Result<Self> {
      let admin_url = env::var("RAMPART_TEST_DB_URL").context("RAMPART_TEST_DB_URL not set")?;
      let name: String = format!(
         "rampart_test_{}",
         iter::repeat_with(|| {
            let ch: u8 = rand::rng().random_range(b'a'..=b'z');
            ch as char
         })
         .take(8)
         .collect::<String>()
      );

      let (client, conn) = tokio_postgres::connect(&admin_url, NoTls)
         .await
         .with_context(|| format!("connect admin URL {admin_url}"))?;
      tokio::spawn(async move {
         let _ = conn.await;
      });
      client
         .execute(&format!("CREATE DATABASE {name}"), &[])
         .await
         .with_context(|| format!("CREATE DATABASE {name}"))?;

      let url = rewrite_dbname(&admin_url, &name);

      let mut m_client = {
         let (mig_client, mig_conn) = tokio_postgres::connect(&url, NoTls)
            .await
            .with_context(|| format!("connect new DB {name}"))?;
         tokio::spawn(async move {
            let _ = mig_conn.await;
         });
         mig_client
      };
      migrate::runner()
         .run_async(&mut m_client)
         .await
         .context("apply migrations to test DB")?;

      let mut cfg = deadpool_postgres::Config::new();
      cfg.url = Some(url.clone());
      cfg.manager = Some(deadpool_postgres::ManagerConfig {
         recycling_method: deadpool_postgres::RecyclingMethod::Fast,
      });
      let pool = cfg
         .create_pool(Some(deadpool_postgres::Runtime::Tokio1), NoTls)
         .context("build test pool")?;

      Ok(Self {
         url,
         name,
         pool,
         admin_url,
      })
   }

   /// Open a `TestDb`, or `None` if the test should skip. With
   /// `RAMPART_REQUIRE_DB_TESTS=1` (predeploy gate) any setup failure —
   /// missing env, unreachable DB, migration error — panics with the
   /// underlying error. Otherwise the test prints a WARN and skips.
   ///
   /// # Panics
   ///
   /// Panics when `RAMPART_REQUIRE_DB_TESTS` is set and setup fails, so
   /// the predeploy gate hard-fails instead of silently skipping.
   pub async fn or_skip() -> Option<Self> {
      match Self::try_new().await {
         Ok(db) => Some(db),
         Err(err) => {
            assert!(
               env::var("RAMPART_REQUIRE_DB_TESTS").is_err(),
               "TestDb setup failed: {err:#}"
            );
            eprintln!(
               "WARN: TestDb unavailable ({err}); skipping. Predeploy: set \
                RAMPART_REQUIRE_DB_TESTS=1 to hard-fail."
            );
            None
         },
      }
   }

   pub async fn teardown(self) {
      // Close all pool connections so Postgres allows DROP DATABASE.
      drop(self.pool);
      let Ok((client, conn)) = tokio_postgres::connect(&self.admin_url, NoTls).await else {
         return;
      };
      tokio::spawn(async move {
         let _ = conn.await;
      });
      let _ = client
         .execute(&format!("DROP DATABASE IF EXISTS {}", self.name), &[])
         .await;
   }
}

/// Replace the `dbname=<...>` token in a libpq connection string.
/// Handles both keyword form and URL form.
fn rewrite_dbname(url: &str, name: &str) -> String {
   if url.starts_with("postgres://") || url.starts_with("postgresql://") {
      // URL form: find last path segment after the host
      // Simplified — works for `postgres://user@host/db?...`
      let (base, query) = url.split_once('?').unwrap_or((url, ""));
      let trimmed = base.trim_end_matches('/');
      let mut parts: Vec<&str> = trimmed.rsplitn(2, '/').collect();
      parts.reverse();
      let rebuilt = if parts.len() == 2 {
         format!("{}/{}", parts[0], name)
      } else {
         format!("{trimmed}/{name}")
      };
      if query.is_empty() {
         rebuilt
      } else {
         format!("{rebuilt}?{query}")
      }
   } else {
      // keyword form: strip any existing dbname=... and append
      let cleaned: String = url
         .split_whitespace()
         .filter(|tok| !tok.starts_with("dbname="))
         .collect::<Vec<_>>()
         .join(" ");
      if cleaned.is_empty() {
         format!("dbname={name}")
      } else {
         format!("{cleaned} dbname={name}")
      }
   }
}
