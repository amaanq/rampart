//! `rampart migrate` — apply pending refinery migrations, then exit.
//! Uses a dedicated tokio-postgres client so concurrent web-pool
//! starts can't race the migration. Idempotent.

use anyhow::{
   Context as _,
   Result,
};

use crate::db;

refinery::embed_migrations!("../../migrations");

// Re-export the runner so integration tests can reuse the embedded
// migrations without invoking `embed_migrations!` in their own
// compilation unit.
pub use self::migrations::runner;

/// Apply pending refinery migrations, then return.
///
/// # Errors
///
/// Returns an error if the initial database connection fails or the
/// refinery runner aborts (e.g. on a divergent or missing migration).
pub async fn run(url: &str) -> Result<()> {
   let mut client = db::connect_once(url).await?;
   // Abort on divergent/missing migrations so V001 edits fail loudly.
   let report = migrations::runner()
      .set_abort_divergent(true)
      .set_abort_missing(true)
      .run_async(&mut client)
      .await
      .context("refinery migration runner failed")?;

   let applied = report.applied_migrations();
   if applied.is_empty() {
      tracing::info!("no migrations to apply");
   } else {
      for migration in applied {
         tracing::info!(
            version = migration.version(),
            name = migration.name(),
            "applied migration"
         );
      }
   }
   Ok(())
}
