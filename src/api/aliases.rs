//! `/api/v1/aliases/*` and `/api/v1/alias/{random,custom}/new`.

use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use deadpool_postgres::Client;
use rampart_codegen::queries::{aliases, domains, email_log, mailboxes, users};
use rand::{TryRngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AppState;
use crate::auth::Principal;
use crate::error::{ApiError, ApiResult};
use crate::quota::{DEFAULT_MAX_ALIASES, LOCK_CLASS_ALIAS_CAP, lock_id};

use super::shared::{
    PAGE_SIZE, deserialize_opt_field, is_unique_violation, raise_exception_as_bad_request,
    validate_local_part_fragment,
};

#[derive(Serialize)]
pub(super) struct AliasView {
    id: i64,
    address: String,
    enabled: bool,
    note: Option<String>,
    pinned: bool,
    nb_forward: i64,
    nb_block: i64,
    nb_reply: i64,
    mailbox: MailboxSummary,
    domain: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: time::OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    last_email_at: Option<time::OffsetDateTime>,
}

#[derive(Serialize)]
struct MailboxSummary {
    id: i64,
    email: String,
}

impl From<aliases::AliasJoinedRow> for AliasView {
    fn from(r: aliases::AliasJoinedRow) -> Self {
        Self {
            id: r.id,
            address: r.address,
            enabled: r.enabled,
            note: r.note,
            pinned: r.pinned,
            nb_forward: r.nb_forward,
            nb_block: r.nb_block,
            nb_reply: r.nb_reply,
            mailbox: MailboxSummary {
                id: r.mailbox_id,
                email: r.mailbox_email,
            },
            domain: r.domain,
            created_at: r.created_at,
            last_email_at: r.last_email_at,
        }
    }
}

#[derive(Deserialize)]
pub(super) struct AliasesQuery {
    #[serde(default)]
    page: i64,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    pinned: Option<bool>,
}

pub(super) async fn aliases_list(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Query(q): Query<AliasesQuery>,
) -> ApiResult<Json<Value>> {
    let c = state.pool.get().await?;
    let page = q.page.max(0);
    let query_str = q
        .query
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));
    let aliases: Vec<AliasView> = aliases::list_for_user_filtered()
        .bind(
            &c,
            &p.user_id,
            &query_str,
            &q.pinned,
            &PAGE_SIZE,
            &(page * PAGE_SIZE),
        )
        .all()
        .await?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Json(json!({"aliases": aliases, "page": page})))
}

pub(super) async fn alias_get(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<Json<AliasView>> {
    let c = state.pool.get().await?;
    let row = aliases::by_id_user()
        .bind(&c, &id, &p.user_id)
        .opt()
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(row.into()))
}

#[derive(Deserialize)]
pub(super) struct AliasPatch {
    #[serde(default, deserialize_with = "deserialize_opt_field")]
    note: Option<Option<String>>,
    pinned: Option<bool>,
    mailbox_id: Option<i64>,
}

pub(super) async fn alias_patch(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
    Json(body): Json<AliasPatch>,
) -> ApiResult<Json<AliasView>> {
    let c = state.pool.get().await?;
    if let Some(note) = body.note {
        aliases::set_note().bind(&c, &note, &id, &p.user_id).await?;
    }
    if let Some(pinned) = body.pinned {
        aliases::set_pinned()
            .bind(&c, &pinned, &id, &p.user_id)
            .await?;
    }
    if let Some(mailbox_id) = body.mailbox_id {
        let ok = mailboxes::exists_verified()
            .bind(&c, &mailbox_id, &p.user_id)
            .opt()
            .await?;
        if ok.is_none() {
            return Err(ApiError::BadRequest(
                "mailbox not found or not verified".into(),
            ));
        }
        aliases::set_mailbox()
            .bind(&c, &mailbox_id, &id, &p.user_id)
            .await
            .map_err(raise_exception_as_bad_request)?;
    }
    alias_get(State(state), Extension(p), Path(id)).await
}

pub(super) async fn alias_toggle(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<Json<AliasView>> {
    let c = state.pool.get().await?;
    let n = aliases::toggle_enabled().bind(&c, &id, &p.user_id).await?;
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    alias_get(State(state), Extension(p), Path(id)).await
}

pub(super) async fn alias_delete(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let c = state.pool.get().await?;
    let n = aliases::delete().bind(&c, &id, &p.user_id).await?;
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub(super) struct ActivityQuery {
    page: Option<i64>,
}

pub(super) async fn alias_activities(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
    Query(q): Query<ActivityQuery>,
) -> ApiResult<Json<Value>> {
    let c = state.pool.get().await?;
    let owns = aliases::exists_for_user()
        .bind(&c, &id, &p.user_id)
        .opt()
        .await?;
    if owns.is_none() {
        return Err(ApiError::NotFound);
    }
    let page = q.page.unwrap_or(0).max(0);
    let activities = email_log::activity_for_alias_api()
        .bind(&c, &id, &PAGE_SIZE, &(page * PAGE_SIZE))
        .all()
        .await?
        .into_iter()
        .map(|l| {
            json!({
                "id": l.id,
                "action": l.action,
                "status": l.status,
                "from": l.from_address,
                "created_at": l.created_at.format(&time::format_description::well_known::Rfc3339).unwrap_or_default(),
                "reason": l.reason,
            })
        })
        .collect::<Vec<Value>>();
    Ok(Json(json!({"activities": activities, "page": page})))
}

#[derive(Deserialize)]
pub(super) struct AliasRandom {
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    mailbox_id: Option<i64>,
}

pub(super) async fn alias_random(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Json(body): Json<AliasRandom>,
) -> ApiResult<(StatusCode, Json<AliasView>)> {
    let mut c = state.pool.get().await?;
    let (dom, mb_id) = resolve_domain_and_mailbox(&c, &p, body.domain, body.mailbox_id).await?;
    let local = random_local_part(&dom.random_prefix);
    let addr = format!("{local}@{}", dom.domain);

    let id = insert_alias(&mut c, p.user_id, &addr, dom.id, mb_id, &body.note, false).await?;
    let row = aliases::by_id().bind(&c, &id).one().await?;
    Ok((StatusCode::CREATED, Json(row.into())))
}

#[derive(Deserialize)]
pub(super) struct AliasCustomNew {
    alias_prefix: String,
    #[serde(default)]
    alias_suffix: Option<String>,
    domain: String,
    mailbox_id: Option<i64>,
    #[serde(default)]
    note: Option<String>,
}

pub(super) async fn alias_custom_new(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Json(body): Json<AliasCustomNew>,
) -> ApiResult<(StatusCode, Json<AliasView>)> {
    let mut c = state.pool.get().await?;
    let (dom, mb_id) =
        resolve_domain_and_mailbox(&c, &p, Some(body.domain), body.mailbox_id).await?;
    let prefix = body.alias_prefix.trim();
    validate_local_part_fragment(prefix, "alias_prefix")?;
    let suffix = body.alias_suffix.as_deref().unwrap_or("").trim();
    let local = if suffix.is_empty() {
        prefix.to_owned()
    } else {
        validate_local_part_fragment(suffix, "alias_suffix")?;
        format!("{prefix}-{suffix}")
    };
    if local.len() > 64 {
        return Err(ApiError::BadRequest(
            "combined alias local-part exceeds 64 bytes".into(),
        ));
    }
    let addr = format!("{local}@{}", dom.domain);

    let id = insert_alias(&mut c, p.user_id, &addr, dom.id, mb_id, &body.note, false).await?;
    let row = aliases::by_id().bind(&c, &id).one().await?;
    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn insert_alias(
    c: &mut Client,
    user_id: i64,
    addr: &str,
    domain_id: i64,
    mailbox_id: i64,
    note: &Option<String>,
    auto_created: bool,
) -> ApiResult<i64> {
    let txn = c.transaction().await?;
    txn.execute(
        "SELECT pg_advisory_xact_lock($1)",
        &[&lock_id(LOCK_CLASS_ALIAS_CAP, user_id)],
    )
    .await?;

    let cap_row = users::cap_and_count_aliases()
        .bind(&txn, &DEFAULT_MAX_ALIASES, &user_id)
        .one()
        .await?;
    let cap = cap_row.cap;
    let current = cap_row.current;
    if current >= cap {
        return Err(ApiError::Conflict(format!(
            "alias cap reached ({current}/{cap})"
        )));
    }

    let id = aliases::create()
        .bind(
            &txn,
            &user_id,
            &addr,
            &domain_id,
            &mailbox_id,
            note,
            &auto_created,
        )
        .one()
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                ApiError::Conflict(format!("alias {addr} already exists"))
            } else {
                raise_exception_as_bad_request(e)
            }
        })?;

    txn.commit().await?;
    Ok(id)
}

async fn resolve_domain_and_mailbox(
    c: &Client,
    p: &Principal,
    domain: Option<String>,
    mailbox_id: Option<i64>,
) -> ApiResult<(domains::AliasDomainRow, i64)> {
    let dom = match domain {
        Some(d) => domains::by_domain_for_user()
            .bind(c, &d, &p.user_id, &p.is_admin)
            .opt()
            .await?
            .ok_or_else(|| ApiError::BadRequest(format!("domain not accessible: {d}")))?,
        None => domains::first_accessible_for_user()
            .bind(c, &p.user_id, &p.is_admin)
            .opt()
            .await?
            .ok_or_else(|| ApiError::BadRequest("no alias_domain rows accessible".into()))?,
    };
    let mb_id = if let Some(id) = mailbox_id {
        let ok = mailboxes::exists_verified()
            .bind(c, &id, &p.user_id)
            .opt()
            .await?;
        if ok.is_none() {
            return Err(ApiError::BadRequest(
                "mailbox not found or not verified".into(),
            ));
        }
        id
    } else {
        let default_for_user = if let Some(did) = dom.default_mailbox_id {
            mailboxes::id_if_verified()
                .bind(c, &did, &p.user_id)
                .opt()
                .await?
        } else {
            None
        };
        if let Some(id) = default_for_user {
            id
        } else {
            mailboxes::first_verified_for_user()
                .bind(c, &p.user_id)
                .opt()
                .await?
                .ok_or_else(|| {
                    ApiError::BadRequest("no verified enabled mailbox for user".into())
                })?
        }
    };
    Ok((dom, mb_id))
}

fn random_local_part(prefix: &str) -> String {
    let mut bytes = [0u8; 5];
    OsRng
        .try_fill_bytes(&mut bytes)
        .expect("OsRng must not fail");
    format!("{prefix}{}", hex::encode(bytes))
}
