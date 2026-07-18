//! `/api/v1/domains` and `/api/v1/domain/*`. Includes the post-CRUD
//! sieve render and stalwart sync.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use deadpool_postgres::Client;
use rampart_codegen::queries::{domains, users};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;

use crate::config::Config;
use crate::domain_setup::{self, DkimRecord, DomainSetup};
use crate::error::{ApiError, ApiResult};
use crate::quota::{DEFAULT_MAX_DOMAINS, LOCK_CLASS_DOMAIN_CAP, lock_id};
use crate::{AppState, bootstrap::JmapClient};
use crate::{auth::Principal, bootstrap::Stats};

use super::shared::{
    deserialize_opt_field, is_fk_violation, is_unique_violation, raise_exception_as_bad_request,
    trimmed_nonempty, validate_domain, validate_random_prefix,
};

#[derive(Serialize)]
pub(super) struct DomainView {
    id: i64,
    domain: String,
    owner_id: Option<i64>,
    shared: bool,
    catch_all: bool,
    random_prefix: String,
    reply_prefix: String,
    default_mailbox_id: Option<i64>,
    nb_alias: i64,
    can_manage: bool,
}

impl DomainView {
    fn from_row(r: domains::DomainRow, p: &Principal) -> Self {
        let can_manage = p.is_admin || (!r.shared && r.owner_id == Some(p.user_id));
        Self {
            id: r.id,
            domain: r.domain,
            owner_id: r.owner_id,
            shared: r.shared,
            catch_all: r.catch_all,
            random_prefix: r.random_prefix,
            reply_prefix: r.reply_prefix,
            default_mailbox_id: r.default_mailbox_id,
            nb_alias: r.nb_alias,
            can_manage,
        }
    }
}

pub(super) async fn domains_list(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Json<Value>> {
    let c = state.pool.get().await?;
    let out = domains::list_for_user()
        .bind(&c, &p.user_id, &p.is_admin)
        .all()
        .await?
        .into_iter()
        .map(|r| DomainView::from_row(r, &p))
        .collect::<Vec<DomainView>>();
    Ok(Json(json!({"domains": out})))
}

#[derive(Deserialize)]
pub(super) struct DomainCreate {
    domain: String,
    random_prefix: Option<String>,
    // reply_prefix is intentionally NOT exposed: the rendered Sieve
    // hardcodes `ra+*` and the LMTP worker hardcodes
    // `ra+<token>@<domain>`. Schema CHECK pins the column to 'ra+'.
}

pub(super) async fn domain_create(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Json(body): Json<DomainCreate>,
) -> ApiResult<(StatusCode, Json<DomainView>)> {
    let domain = body.domain.trim().to_ascii_lowercase();
    let random_prefix = trimmed_nonempty(body.random_prefix);
    validate_domain(&domain)?;
    if let Some(rp) = random_prefix.as_deref() {
        validate_random_prefix(rp)?;
    }
    let mut c = state.pool.get().await?;
    let id = insert_domain(&mut c, p.user_id, &domain, random_prefix).await?;
    let view = domains::by_id().bind(&c, &id).one().await?;
    render_and_sync_sieve_if_configured(&mut c, &state.config).await?;
    Ok((StatusCode::CREATED, Json(DomainView::from_row(view, &p))))
}

#[derive(Deserialize)]
pub(super) struct DomainPatch {
    catch_all: Option<bool>,
    /// Per-domain cap on auto-created aliases (catch-all defense).
    /// Required by schema when `catch_all=TRUE`; expose so operators
    /// can flip both fields in one request.
    #[serde(default, deserialize_with = "deserialize_opt_field")]
    max_auto_created: Option<Option<i32>>,
    random_prefix: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_field")]
    default_mailbox_id: Option<Option<i64>>,
}

pub(super) async fn domain_patch(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
    Json(body): Json<DomainPatch>,
) -> ApiResult<Json<DomainView>> {
    let c = state.pool.get().await?;
    let ok = domains::exists_managable()
        .bind(&c, &id, &p.user_id, &p.is_admin)
        .opt()
        .await?;
    if ok.is_none() {
        return Err(ApiError::NotFound);
    }
    // Apply catch_all + max_auto_created together so the
    // `catch_all_requires_cap` CHECK doesn't fire when one flips
    // without the other in the same payload.
    if body.catch_all.is_some() || body.max_auto_created.is_some() {
        let cur = domains::catch_all_and_cap().bind(&c, &id).one().await?;
        let new_catch_all = body.catch_all.unwrap_or(cur.catch_all);
        let new_cap: Option<i32> = match body.max_auto_created {
            Some(v) => v,
            None => cur.max_auto_created,
        };
        domains::set_catch_all_and_cap()
            .bind(&c, &new_catch_all, &new_cap, &id)
            .await
            .map_err(raise_exception_as_bad_request)?;
    }
    if let Some(v) = body.random_prefix {
        validate_random_prefix(&v)?;
        domains::set_random_prefix().bind(&c, &v, &id).await?;
    }
    if let Some(v) = body.default_mailbox_id {
        domains::set_default_mailbox()
            .bind(&c, &v, &id)
            .await
            .map_err(raise_exception_as_bad_request)?;
    }
    let row = domains::by_id().bind(&c, &id).one().await?;
    Ok(Json(DomainView::from_row(row, &p)))
}

pub(super) async fn domain_delete(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let mut c = state.pool.get().await?;
    match domains::delete()
        .bind(&c, &id, &p.user_id, &p.is_admin)
        .await
    {
        Ok(0) => Err(ApiError::NotFound),
        Ok(_) => {
            render_and_sync_sieve_if_configured(&mut c, &state.config).await?;
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) if is_fk_violation(&e) => Err(ApiError::Conflict(
            "domain has aliases. Delete them first".into(),
        )),
        Err(e) => Err(ApiError::Db(e)),
    }
}

pub(super) async fn domain_check(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<Json<DomainSetup>> {
    let c = state.pool.get().await?;
    let row = domains::by_id_for_user()
        .bind(&c, &id, &p.user_id, &p.is_admin)
        .opt()
        .await?
        .ok_or(ApiError::NotFound)?;
    let allowed = crate::abuse::check(
        &state.pool,
        &format!("domain_dns_check:{}:{id}", p.user_id),
        crate::abuse::DOMAIN_DNS_CHECK,
    )
    .await
    .map_err(ApiError::Internal)?;
    if !allowed {
        return Err(ApiError::BadRequest(
            "DNS was checked too often. Wait a moment and try again".into(),
        ));
    }

    let dkim_records =
        refresh_dkim_records(&c, row.id, &row.domain, &row.dkim_records, &state.config).await;
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
        .bind(&c, &status_json, &now, &checked.all_verified(), &row.id)
        .await?;
    Ok(Json(checked))
}

async fn refresh_dkim_records(
    c: &Client,
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
        let password = std::fs::read_to_string(password_path)
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
                        .bind(c, &value, &domain_id)
                        .await
                    {
                        tracing::warn!(domain, ?error, "failed to cache DKIM setup records");
                    }
                }
                Err(error) => {
                    tracing::warn!(domain, ?error, "failed to serialize DKIM setup records");
                }
            }
            records
        }
        Ok(_) => current,
        Err(error) => {
            tracing::warn!(domain, ?error, "DKIM setup records are not available yet");
            current
        }
    }
}

async fn insert_domain(
    c: &mut Client,
    user_id: i64,
    domain: &str,
    random_prefix: Option<String>,
) -> ApiResult<i64> {
    let txn = c.transaction().await?;
    txn.execute(
        "SELECT pg_advisory_xact_lock($1)",
        &[&lock_id(LOCK_CLASS_DOMAIN_CAP, user_id)],
    )
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
        .map_err(|e| {
            if is_unique_violation(&e) {
                ApiError::Conflict(format!("domain {domain} already configured"))
            } else {
                raise_exception_as_bad_request(e)
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
async fn render_and_sync_sieve_if_configured(c: &mut Client, cfg: &Config) -> ApiResult<()> {
    let Some(out) = &cfg.sieve_output_path else {
        return Ok(());
    };
    let cfg_clone = cfg.clone();
    let result = crate::sieve::render_write_and_sync_locked(c, out, move |domains, sieve_text| {
        let cfg = cfg_clone;
        async move {
            if let Err(e) = sync_stalwart_snapshot(domains, sieve_text, &cfg).await {
                tracing::warn!(
                    error = ?e,
                    "stalwart-sync failed (sieve file is current; \
                     next rampart-bootstrap-stalwart run will reconcile)"
                );
            }
            Ok(())
        }
    })
    .await;
    if let Err(e) = result {
        return Err(ApiError::Internal(e));
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
    let admin_pw = std::fs::read_to_string(admin_pw_path)
        .map_err(|e| anyhow::anyhow!("read {}: {e}", admin_pw_path.display()))?
        .trim()
        .to_owned();
    let client = JmapClient::new(jmap_url, &cfg.stalwart_admin_username, &admin_pw)?;
    let mut stats = Stats::default();
    for d in &domains {
        crate::bootstrap::upsert_managed_alias_domain(&client, &mut stats, d, false).await?;
    }
    let notifier_domain = cfg
        .smtp_user
        .rsplit_once('@')
        .map(|(_, d)| d.to_string())
        .unwrap_or_default();
    crate::bootstrap::reconcile_alias_domains(
        &client,
        &mut stats,
        &domains,
        &notifier_domain,
        false,
    )
    .await?;
    crate::bootstrap::upsert_sieve_script(&client, &mut stats, &sieve_contents, false).await?;
    crate::bootstrap::patch_stage_rcpt_script(&client, &mut stats, &domains, false).await?;
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
