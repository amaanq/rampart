//! `rampart admin bootstrap-stalwart` — seed stalwart's JMAP registry for
//! LMTP-routed reply-via-alias delivery.
//!
//! Stalwart 0.16+ stores outbound queue config and the `session.rcpt`
//! allow-relaying expression in JMAP registry objects, not TOML. We
//! idempotently create/patch those objects so a fresh stalwart routes
//! to our LMTP worker after one invocation.
//!
//! JMAP method names: `x:<T>/get`, `x:<T>/set`, `x:<T>/query`.
//! Singletons use id `"singleton"`. `List<T>` fields serialize as
//! INDEXED OBJECTS `{"0":...}`, NOT JSON arrays — arrays silently
//! invalidPatch.
#![expect(clippy::print_stdout, reason = "CLI command output")]

use std::{
   fs,
   path::PathBuf,
};

use anyhow::{
   Context as _,
   Result,
};

use crate::{
   db,
   sieve,
};

mod domains;
mod jmap;
mod objects;

pub(crate) use domains::{
   reconcile_alias_domains,
   upsert_managed_alias_domain,
};
pub(crate) use jmap::JmapClient;
pub(crate) use objects::{
   patch_stage_rcpt_script,
   upsert_sieve_script,
};

pub(crate) const SINGLETON_ID: &str = "singleton";

const ROUTE_NAME: &str = "rampart-lmtp";
const VQ_NAME: &str = "ramplmtp"; // 8 chars; stalwart caps name at 8.
const SCHEDULE_NAME: &str = "rampart_lmtp"; // 7 chars.
const CONN_NAME: &str = "rampart_lmtp";
const RELAY_GUARD: &str = "rcpt_domain == 'internal.rampart.lmtp'";

struct BootstrapArgs {
   jmap_base_url:                  String,
   admin_username:                 String,
   admin_password_file:            PathBuf,
   rampart_notifier_password_file: PathBuf,
   rampart_notifier_address:       String,
   lmtp_address:                   String,
   lmtp_port:                      u16,
   alias_domains:                  Vec<String>,
   database_url:                   String,
   sieve_path:                     PathBuf,
   dry_run:                        bool,
}

#[derive(Default)]
pub(crate) struct Stats {
   pub created: u32,
   pub patched: u32,
   pub skipped: u32,
}

#[expect(clippy::cognitive_complexity, reason = "linear bootstrap sequence")]
async fn run(args: BootstrapArgs) -> Result<()> {
   let admin_password = fs::read_to_string(&args.admin_password_file)
      .with_context(|| format!("reading {}", args.admin_password_file.display()))?
      .trim()
      .to_owned();
   let notifier_password = fs::read_to_string(&args.rampart_notifier_password_file)
      .with_context(|| format!("reading {}", args.rampart_notifier_password_file.display()))?
      .trim()
      .to_owned();

   // alias_domain rows are the source of truth; --alias-domain CLI args
   // only seed a fresh deploy. Read table + rendered sieve under the
   // same advisory lock the API uses so a parallel CRUD can't interleave.
   let mut pg_for_lock = db::connect_once(&args.database_url).await?;
   let lock_txn = pg_for_lock.transaction().await?;
   lock_txn
      .execute("SET LOCAL idle_in_transaction_session_timeout = 0", &[])
      .await
      .context("disable idle-in-txn timeout for bootstrap lock")?;
   lock_txn
      .execute("SELECT pg_advisory_xact_lock($1::bigint)", &[
         &sieve::SIEVE_RENDER_LOCK_KEY,
      ])
      .await
      .context("pg_advisory_xact_lock for bootstrap")?;
   let alias_domains_rows = lock_txn
      .query(
         "SELECT domain::text AS domain FROM alias_domain ORDER BY domain",
         &[],
      )
      .await?;
   let alias_domains: Vec<String> = alias_domains_rows
      .into_iter()
      .map(|row| row.get::<_, String>("domain"))
      .collect();
   // Render from the in-memory snapshot, not from disk. Reading sieve_path
   // here would push whatever the last render-sieve unit wrote, which can
   // lag the DB on a fresh deploy (alias_domain rows committed before the
   // render unit ran) or skew after a manual edit. Rendering inside the
   // lock guarantees stage-branches and Sieve text reflect the same DB
   // snapshot, then we heal the on-disk file to match.
   let sieve_contents_locked = sieve::render_for_domains(&alias_domains)?;
   if !args.dry_run {
      sieve::atomic_write_file(&args.sieve_path, sieve_contents_locked.as_bytes())
         .with_context(|| format!("writing {}", args.sieve_path.display()))?;
   }

   // Lock held through the JMAP push so API CRUDs serialize against
   // bootstrap. CLI --alias-domain entries not in the DB are a no-op;
   // the rendered Sieve only branches on DB rows.
   if !args.alias_domains.is_empty() {
      let unhandled: Vec<&str> = args
         .alias_domains
         .iter()
         .filter(|domain| {
            !alias_domains
               .iter()
               .any(|existing| existing.eq_ignore_ascii_case(domain))
         })
         .map(String::as_str)
         .collect();
      if !unhandled.is_empty() {
         tracing::warn!(
             cli_seed_unhandled = ?unhandled,
             "--alias-domain CLI args reference domains absent from alias_domain DB; \
              add them via the dashboard for them to take effect"
         );
      }
   }

   let client = JmapClient::new(&args.jmap_base_url, &args.admin_username, &admin_password)?;

   // systemd `After=` doesn't prove JMAP readiness; bound retries to ~60s.
   client.wait_until_ready().await?;

   let mut stats = Stats::default();

   objects::patch_stage_rcpt(&client, &mut stats, args.dry_run).await?;
   objects::upsert_route(
      &client,
      &mut stats,
      &args.lmtp_address,
      args.lmtp_port,
      args.dry_run,
   )
   .await?;

   // schedule.queueId is the VQ's id, not its name.
   let vq_id = objects::upsert_virtual_queue(&client, &mut stats, args.dry_run).await?;
   objects::upsert_delivery_schedule(&client, &mut stats, &vq_id, args.dry_run).await?;
   objects::upsert_connection_strategy(&client, &mut stats, args.dry_run).await?;
   objects::patch_outbound_strategy(&client, &mut stats, args.dry_run).await?;

   // Marker-stamped Domains must exist before upsert_notifier — if the
   // notifier shares a domain with an alias_domain, an unmarked Domain
   // would block later marker adoption.
   for domain in &alias_domains {
      domains::upsert_managed_alias_domain(&client, &mut stats, domain, args.dry_run).await?;
   }
   let notifier_domain = args
      .rampart_notifier_address
      .rsplit_once('@')
      .map(|(_, domain)| domain.to_owned())
      .unwrap_or_default();
   domains::reconcile_alias_domains(
      &client,
      &mut stats,
      &alias_domains,
      &notifier_domain,
      args.dry_run,
   )
   .await?;

   objects::upsert_notifier(
      &client,
      &mut stats,
      &args.rampart_notifier_address,
      &notifier_password,
      args.dry_run,
   )
   .await?;

   // Without this, mustMatchSender rejects worker outbound `5.5.4` — the
   // alias-rewritten From doesn't match the notifier's authed identity.
   objects::patch_must_match_sender(
      &client,
      &mut stats,
      &args.rampart_notifier_address,
      args.dry_run,
   )
   .await?;

   objects::upsert_store_lookup(&client, &mut stats, &args.database_url, args.dry_run).await?;
   objects::upsert_sieve_script(&client, &mut stats, &sieve_contents_locked, args.dry_run).await?;
   objects::patch_stage_rcpt_script(&client, &mut stats, &alias_domains, args.dry_run).await?;

   // Settings-shaped singletons (queue config especially) need a reload
   // to take effect at the SMTP layer.
   if !args.dry_run && (stats.created > 0 || stats.patched > 0) {
      client.reload_settings().await?;
      tracing::info!("stalwart: settings reloaded");
   }

   lock_txn.commit().await?;
   drop(pg_for_lock);

   println!(
      "bootstrap-stalwart: created={} patched={} skipped={}",
      stats.created, stats.patched, stats.skipped
   );
   Ok(())
}

/// Run `rampart admin bootstrap-stalwart` from parsed CLI arguments.
///
/// # Errors
///
/// Returns an error if reading the password files, connecting to Postgres,
/// rendering the Sieve script, or any JMAP registry push fails.
#[expect(clippy::too_many_arguments, reason = "one-to-one mapping of CLI flags")]
pub async fn cli(
   jmap_base_url: String,
   admin_username: String,
   admin_password_file: PathBuf,
   rampart_notifier_password_file: PathBuf,
   rampart_notifier_address: String,
   lmtp_address: String,
   lmtp_port: u16,
   alias_domains: Vec<String>,
   database_url: String,
   sieve_path: PathBuf,
   dry_run: bool,
) -> Result<()> {
   run(BootstrapArgs {
      jmap_base_url,
      admin_username,
      admin_password_file,
      rampart_notifier_password_file,
      rampart_notifier_address,
      lmtp_address,
      lmtp_port,
      alias_domains,
      database_url,
      sieve_path,
      dry_run,
   })
   .await
}
