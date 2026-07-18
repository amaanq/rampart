//! `/api/v1/domains` and `/api/v1/domain/*`. Includes the post-CRUD
//! sieve render and stalwart sync.

use std::fs;

use axum::{
   Extension,
   Json,
   extract::{
      Path,
      State,
   },
   http::StatusCode,
};
use deadpool_postgres::Client;
use rampart_codegen::queries::{
   domains,
   users,
};
use serde::{
   Deserialize,
   Serialize,
};
use serde_json::{
   Value,
   json,
};
use time::OffsetDateTime;

use super::shared::{
   self,
   deserialize_opt_field,
};
use crate::{
   AppState,
   abuse,
   auth::Principal,
   bootstrap::{
      self,
      JmapClient,
      Stats,
   },
   config::Config,
   domain_setup::{
      self,
      DkimRecord,
      DomainSetup,
   },
   error::{
      ApiError,
      ApiResult,
   },
   quota::{
      self,
      DEFAULT_MAX_DOMAINS,
      LOCK_CLASS_DOMAIN_CAP,
   },
   sieve,
};

#[derive(Serialize)]
pub(super) struct DomainView {
   id:                 i64,
   domain:             String,
   owner_id:           Option<i64>,
   shared:             bool,
   catch_all:          bool,
   random_prefix:      String,
   reply_prefix:       String,
   default_mailbox_id: Option<i64>,
   nb_alias:           i64,
   can_manage:         bool,
}

impl DomainView {
   fn from_row(row: domains::DomainRow, principal: &Principal) -> Self {
      let can_manage =
         principal.is_admin || (!row.shared && row.owner_id == Some(principal.user_id));
      Self {
         id: row.id,
         domain: row.domain,
         owner_id: row.owner_id,
         shared: row.shared,
         catch_all: row.catch_all,
         random_prefix: row.random_prefix,
         reply_prefix: row.reply_prefix,
         default_mailbox_id: row.default_mailbox_id,
         nb_alias: row.nb_alias,
         can_manage,
      }
   }
}

pub(super) async fn domains_list(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Json<Value>> {
   let conn = state.pool.get().await?;
   let out = domains::list_for_user()
      .bind(&conn, &principal.user_id, &principal.is_admin)
      .all()
      .await?
      .into_iter()
      .map(|row| DomainView::from_row(row, &principal))
      .collect::<Vec<DomainView>>();
   Ok(Json(json!({"domains": out})))
}

#[derive(Deserialize)]
pub(super) struct DomainCreate {
   domain:        String,
   random_prefix: Option<String>,
   // reply_prefix is intentionally NOT exposed: the rendered Sieve
   // hardcodes `ra+*` and the LMTP worker hardcodes
   // `ra+<token>@<domain>`. Schema CHECK pins the column to 'ra+'.
}

pub(super) async fn domain_create(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Json(body): Json<DomainCreate>,
) -> ApiResult<(StatusCode, Json<DomainView>)> {
   let domain = body.domain.trim().to_ascii_lowercase();
   let random_prefix = shared::trimmed_nonempty(body.random_prefix);
   shared::validate_domain(&domain)?;
   if let Some(rp) = random_prefix.as_deref() {
      shared::validate_random_prefix(rp)?;
   }
   let mut conn = state.pool.get().await?;
   let id = insert_domain(&mut conn, principal.user_id, &domain, random_prefix).await?;
   let view = domains::by_id().bind(&conn, &id).one().await?;
   render_and_sync_sieve_if_configured(&mut conn, &state.config).await?;
   Ok((
      StatusCode::CREATED,
      Json(DomainView::from_row(view, &principal)),
   ))
}

#[derive(Deserialize)]
pub(super) struct DomainPatch {
   catch_all:          Option<bool>,
   /// Per-domain cap on auto-created aliases (catch-all defense).
   /// Required by schema when `catch_all=TRUE`; expose so operators
   /// can flip both fields in one request.
   #[serde(default, deserialize_with = "deserialize_opt_field")]
   #[expect(
      clippy::option_option,
      reason = "Some(None) sets max_auto_created to null; None leaves it unchanged"
   )]
   max_auto_created:   Option<Option<i32>>,
   random_prefix:      Option<String>,
   #[serde(default, deserialize_with = "deserialize_opt_field")]
   #[expect(
      clippy::option_option,
      reason = "Some(None) sets default_mailbox_id to null; None leaves it unchanged"
   )]
   default_mailbox_id: Option<Option<i64>>,
}

pub(super) async fn domain_patch(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
   Json(body): Json<DomainPatch>,
) -> ApiResult<Json<DomainView>> {
   let conn = state.pool.get().await?;
   let ok = domains::exists_managable()
      .bind(&conn, &id, &principal.user_id, &principal.is_admin)
      .opt()
      .await?;
   if ok.is_none() {
      return Err(ApiError::NotFound);
   }
   // Apply catch_all + max_auto_created together so the
   // `catch_all_requires_cap` CHECK doesn't fire when one flips
   // without the other in the same payload.
   if body.catch_all.is_some() || body.max_auto_created.is_some() {
      let cur = domains::catch_all_and_cap().bind(&conn, &id).one().await?;
      let new_catch_all = body.catch_all.unwrap_or(cur.catch_all);
      let new_cap: Option<i32> = body.max_auto_created.unwrap_or(cur.max_auto_created);
      domains::set_catch_all_and_cap()
         .bind(&conn, &new_catch_all, &new_cap, &id)
         .await
         .map_err(shared::raise_exception_as_bad_request)?;
   }
   if let Some(value) = body.random_prefix {
      shared::validate_random_prefix(&value)?;
      domains::set_random_prefix()
         .bind(&conn, &value, &id)
         .await?;
   }
   if let Some(value) = body.default_mailbox_id {
      domains::set_default_mailbox()
         .bind(&conn, &value, &id)
         .await
         .map_err(shared::raise_exception_as_bad_request)?;
   }
   let row = domains::by_id().bind(&conn, &id).one().await?;
   Ok(Json(DomainView::from_row(row, &principal)))
}

pub(super) async fn domain_delete(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
   let mut conn = state.pool.get().await?;
   match domains::delete()
      .bind(&conn, &id, &principal.user_id, &principal.is_admin)
      .await
   {
      Ok(0) => Err(ApiError::NotFound),
      Ok(_) => {
         render_and_sync_sieve_if_configured(&mut conn, &state.config).await?;
         Ok(StatusCode::NO_CONTENT)
      },
      Err(err) if shared::is_fk_violation(&err) => Err(ApiError::Conflict(
         "domain has aliases. Delete them first".into(),
      )),
      Err(err) => Err(ApiError::Db(err)),
   }
}

pub(super) async fn domain_check(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<Json<DomainSetup>> {
   let conn = state.pool.get().await?;
   let row = domains::by_id_for_user()
      .bind(&conn, &id, &principal.user_id, &principal.is_admin)
      .opt()
      .await?
      .ok_or(ApiError::NotFound)?;
   let allowed = abuse::check(
      &state.pool,
      &format!("domain_dns_check:{}:{id}", principal.user_id),
      abuse::DOMAIN_DNS_CHECK,
   )
   .await
   .map_err(ApiError::Internal)?;
   if !allowed {
      return Err(ApiError::BadRequest(
         "DNS was checked too often. Wait a moment and try again".into(),
      ));
   }

   let dkim_records =
      refresh_dkim_records(&conn, row.id, &row.domain, &row.dkim_records, &state.config).await;
   let previous_status = domain_setup::parse_dns_status(&row.dns_status);
   let expected = domain_setup::build(
      row.id,
      &row.domain,
      &state.config.public_mx_hostname,
      &dkim_records,
      &previous_status,
      row.dns_checked_at,
      row.dns_verified_at,
   );
   let status = domain_setup::check(&expected.records)
      .await
      .map_err(ApiError::Internal)?;
   let now = OffsetDateTime::now_utc();
   let checked = domain_setup::build(
      row.id,
      &row.domain,
      &state.config.public_mx_hostname,
      &dkim_records,
      &status,
      Some(now),
      row.dns_verified_at,
   );
   let status_json = serde_json::to_value(&status).map_err(anyhow::Error::from)?;
   domains::set_dns_check()
      .bind(&conn, &status_json, &now, &checked.all_verified(), &row.id)
      .await?;
   Ok(Json(checked))
}

#[expect(
   clippy::cognitive_complexity,
   reason = "linear DKIM fetch/cache/log flow reads clearer inline than split apart"
)]
async fn refresh_dkim_records(
   conn: &Client,
   domain_id: i64,
   domain: &str,
   cached: &Value,
   cfg: &Config,
) -> Vec<DkimRecord> {
   let current = domain_setup::parse_dkim_records(cached);
   let complete = current.iter().any(|record| record.algorithm == "rsa")
      && current.iter().any(|record| record.algorithm == "ed25519");
   if complete {
      return current;
   }
   let (Some(jmap_url), Some(password_path)) = (
      cfg.stalwart_jmap_base_url.as_deref(),
      cfg.stalwart_admin_password_file.as_deref(),
   ) else {
      return current;
   };
   let result = async {
      let password = fs::read_to_string(password_path)
         .map_err(anyhow::Error::from)?
         .trim()
         .to_owned();
      let client = JmapClient::new(jmap_url, &cfg.stalwart_admin_username, &password)?;
      client.dkim_dns_records_for_domain(domain).await
   }
   .await;
   match result {
      Ok(records) if !records.is_empty() => {
         match serde_json::to_value(&records) {
            Ok(value) => {
               if let Err(error) = domains::set_dkim_records()
                  .bind(conn, &value, &domain_id)
                  .await
               {
                  tracing::warn!(domain, ?error, "failed to cache DKIM setup records");
               }
            },
            Err(error) => {
               tracing::warn!(domain, ?error, "failed to serialize DKIM setup records");
            },
         }
         records
      },
      Ok(_) => current,
      Err(error) => {
         tracing::warn!(domain, ?error, "DKIM setup records are not available yet");
         current
      },
   }
}

async fn insert_domain(
   conn: &mut Client,
   user_id: i64,
   domain: &str,
   random_prefix: Option<String>,
) -> ApiResult<i64> {
   let txn = conn.transaction().await?;
   txn.execute("SELECT pg_advisory_xact_lock($1)", &[&quota::lock_id(
      LOCK_CLASS_DOMAIN_CAP,
      user_id,
   )])
   .await?;
   let cap_row = users::cap_and_count_domains()
      .bind(&txn, &DEFAULT_MAX_DOMAINS, &user_id)
      .one()
      .await?;
   let cap = cap_row.cap;
   let current = cap_row.current;
   if current >= cap {
      return Err(ApiError::Conflict(format!(
         "domain cap reached ({current}/{cap})"
      )));
   }
   let id = domains::create()
      .bind(&txn, &domain, &Some(user_id), &random_prefix)
      .one()
      .await
      .map_err(|err| {
         if shared::is_unique_violation(&err) {
            ApiError::Conflict(format!("domain {domain} already configured"))
         } else {
            shared::raise_exception_as_bad_request(err)
         }
      })?;

   txn.commit().await?;
   Ok(id)
}

/// Render the Sieve file and push the snapshot to stalwart under one
/// advisory lock so two concurrent CRUDs can't race file-content-B
/// against stalwart-stage-branches-A. The JMAP push is best-effort
/// (logs warn, returns 201 anyway) so the dashboard never reports a
/// committed mutation as rolled back; the next bootstrap-stalwart run
/// reconciles. Render-or-write failures still hard-fail because the
/// file IS what bootstrap reads later.
async fn render_and_sync_sieve_if_configured(conn: &mut Client, cfg: &Config) -> ApiResult<()> {
   let Some(out) = cfg.sieve_output_path.as_ref() else {
      return Ok(());
   };
   let cfg_clone = cfg.clone();
   let result = sieve::render_write_and_sync_locked(conn, out, move |domains, sieve_text| {
      let config = cfg_clone;
      async move {
         if let Err(err) = sync_stalwart_snapshot(domains, sieve_text, &config).await {
            tracing::warn!(
                error = ?err,
                "stalwart-sync failed (sieve file is current; \
                 next rampart-bootstrap-stalwart run will reconcile)"
            );
         }
         Ok(())
      }
   })
   .await;
   if let Err(err) = result {
      return Err(ApiError::Internal(err));
   }
   tracing::info!(path = %out.display(), "sieve rewritten");
   Ok(())
}

async fn sync_stalwart_snapshot(
   domains: Vec<String>,
   sieve_contents: String,
   cfg: &Config,
) -> anyhow::Result<()> {
   let (Some(jmap_url), Some(admin_pw_path)) = (
      cfg.stalwart_jmap_base_url.as_deref(),
      cfg.stalwart_admin_password_file.as_deref(),
   ) else {
      tracing::debug!("stalwart-sync skipped: JMAP creds not configured");
      return Ok(());
   };
   let admin_pw = fs::read_to_string(admin_pw_path)
      .map_err(|err| anyhow::anyhow!("read {}: {err}", admin_pw_path.display()))?
      .trim()
      .to_owned();
   let client = JmapClient::new(jmap_url, &cfg.stalwart_admin_username, &admin_pw)?;
   let mut stats = Stats::default();
   for domain in &domains {
      bootstrap::upsert_managed_alias_domain(&client, &mut stats, domain, false).await?;
   }
   let notifier_domain = cfg
      .smtp_user
      .rsplit_once('@')
      .map(|(_, domain)| domain.to_owned())
      .unwrap_or_default();
   bootstrap::reconcile_alias_domains(&client, &mut stats, &domains, &notifier_domain, false)
      .await?;
   bootstrap::upsert_sieve_script(&client, &mut stats, &sieve_contents, false).await?;
   bootstrap::patch_stage_rcpt_script(&client, &mut stats, &domains, false).await?;
   if stats.created > 0 || stats.patched > 0 {
      client.reload_settings().await?;
   }
   tracing::info!(
      created = stats.created,
      patched = stats.patched,
      skipped = stats.skipped,
      "stalwart-sync"
   );
   Ok(())
}
