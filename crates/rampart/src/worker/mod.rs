//! LMTP resubmit worker.
//!
//! Receives mail forwarded by stalwart via LMTP
//! on an internal synthetic domain, rewrites headers (From:),
//! upserts `reverse_contact` rows, and resubmits outbound via SMTP AUTH
//! to stalwart.
//!
//! Entry: `rampart worker` subcommand.

pub mod auth_results;
pub mod lmtp;
pub mod loop_guard;
pub mod pipeline;
pub mod resubmit;
pub mod verp;

use std::sync::Arc;

use anyhow::{
   Context as _,
   Result,
};
use deadpool_postgres::Pool;

use crate::{
   config::Config,
   db,
   mailer::{
      Mailer,
      SmtpMailer,
   },
   worker::resubmit::{
      Submit,
      SubmitClient,
   },
};

#[derive(Clone)]
#[expect(
   clippy::module_name_repetitions,
   reason = "WorkerState is the established public name"
)]
pub struct WorkerState {
   pub pool:   Pool,
   pub config: Arc<Config>,
   pub mailer: Arc<dyn Mailer>,
   pub submit: Arc<dyn Submit>,
}

/// Dispatch an assembled LMTP delivery into the processing pipeline.
///
/// Abstracted so tests/worker.rs can drive `handle_session_io` with a
/// mock handler that returns preset verdicts without touching Postgres.
#[async_trait::async_trait]
pub trait DeliveryHandler: Send + Sync {
   async fn handle(&self, state: &WorkerState, delivery: pipeline::Delivery) -> pipeline::Verdict;
}

/// Production handler: runs the real DB-backed pipeline.
pub struct PipelineHandler;

#[async_trait::async_trait]
impl DeliveryHandler for PipelineHandler {
   async fn handle(&self, state: &WorkerState, delivery: pipeline::Delivery) -> pipeline::Verdict {
      pipeline::process(state, delivery).await
   }
}

/// Run the LMTP resubmit worker until shutdown.
///
/// # Errors
/// Returns an error if the DB pool can't be built or probed, the SMTP and
/// submit clients can't be configured, or the LMTP listener fails.
pub async fn run(cfg: Config) -> Result<()> {
   let pool = db::build_pool(&cfg.database_url)?;
   let _probe = pool.get().await.context("db probe")?;
   let smtp = SmtpMailer::from_config(&cfg).context("configure SmtpMailer for worker")?;
   let mailer: Arc<dyn Mailer> = Arc::new(smtp);
   let submit_client =
      SubmitClient::from_config(&cfg).context("configure SubmitClient for worker")?;
   let submit: Arc<dyn Submit> = Arc::new(submit_client);
   let state = WorkerState {
      pool,
      config: Arc::new(cfg),
      mailer,
      submit,
   };
   lmtp::serve(state).await
}
