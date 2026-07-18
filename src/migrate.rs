//! `rampart migrate` — apply pending refinery migrations, then exit.
//! Uses a dedicated tokio-postgres client so concurrent web-pool
//! starts can't race the migration. Idempotent.

use anyhow::{
   Context,
   Result,
};

refinery::embed_migrations!("migrations");

// Re-export the runner so integration tests can reuse the embedded
// migrations without invoking `embed_migrations!` in their own
// compilation unit.
pub use self::migrations::runner;

pub async fn run(url: &str) -> Result<()> {
   let mut client = crate::db::connect_once(url).await?;
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
      for m in applied {
         tracing::info!(version = m.version(), name = m.name(), "applied migration");
      }
   }
   Ok(())
}
