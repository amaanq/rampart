//! `/api/v1/aliases/*` and `/api/v1/alias/{random,custom}/new`.

use axum::{
   Extension,
   Json,
   extract::{
      Path,
      Query,
      State,
   },
   http::StatusCode,
};
use deadpool_postgres::Client;
use rampart_codegen::queries::{
   aliases,
   domains,
   email_log,
   mailboxes,
   users,
};
use rand::{
   TryRng as _,
   rngs::SysRng,
};
use serde::{
   Deserialize,
   Serialize,
};
use serde_json::{
   Value,
   json,
};
use time::format_description::well_known::Rfc3339;

use super::shared::{
   self,
   PAGE_SIZE,
   deserialize_opt_field,
};
use crate::{
   AppState,
   auth::Principal,
   error::{
      ApiError,
      ApiResult,
   },
   quota::{
      self,
      DEFAULT_MAX_ALIASES,
      LOCK_CLASS_ALIAS_CAP,
   },
};

#[derive(Serialize)]
pub(super) struct AliasView {
   id:            i64,
   address:       String,
   enabled:       bool,
   note:          Option<String>,
   pinned:        bool,
   nb_forward:    i64,
   nb_block:      i64,
   nb_reply:      i64,
   mailbox:       MailboxSummary,
   domain:        String,
   #[serde(with = "time::serde::rfc3339")]
   created_at:    time::OffsetDateTime,
   #[serde(with = "time::serde::rfc3339::option")]
   last_email_at: Option<time::OffsetDateTime>,
}

#[derive(Serialize)]
struct MailboxSummary {
   id:    i64,
   email: String,
}

impl From<aliases::AliasJoinedRow> for AliasView {
   fn from(row: aliases::AliasJoinedRow) -> Self {
      Self {
         id:            row.id,
         address:       row.address,
         enabled:       row.enabled,
         note:          row.note,
         pinned:        row.pinned,
         nb_forward:    row.nb_forward,
         nb_block:      row.nb_block,
         nb_reply:      row.nb_reply,
         mailbox:       MailboxSummary {
            id:    row.mailbox_id,
            email: row.mailbox_email,
         },
         domain:        row.domain,
         created_at:    row.created_at,
         last_email_at: row.last_email_at,
      }
   }
}

#[derive(Deserialize)]
pub(super) struct AliasesQuery {
   #[serde(default)]
   page:   i64,
   #[serde(default)]
   query:  Option<String>,
   #[serde(default)]
   pinned: Option<bool>,
}

pub(super) async fn aliases_list(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Query(query): Query<AliasesQuery>,
) -> ApiResult<Json<Value>> {
   let conn = state.pool.get().await?;
   let page = query.page.max(0);
   let query_str = query
      .query
      .as_deref()
      .filter(|text| !text.is_empty())
      .map(|text| format!("%{text}%"));
   let aliases: Vec<AliasView> = aliases::list_for_user_filtered()
      .bind(
         &conn,
         &principal.user_id,
         &query_str,
         &query.pinned,
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
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<Json<AliasView>> {
   let conn = state.pool.get().await?;
   let row = aliases::by_id_user()
      .bind(&conn, &id, &principal.user_id)
      .opt()
      .await?
      .ok_or(ApiError::NotFound)?;
   Ok(Json(row.into()))
}

#[derive(Deserialize)]
pub(super) struct AliasPatch {
   #[serde(default, deserialize_with = "deserialize_opt_field")]
   #[expect(
      clippy::option_option,
      reason = "Some(None) sets note to null; None leaves it unchanged"
   )]
   note:       Option<Option<String>>,
   pinned:     Option<bool>,
   mailbox_id: Option<i64>,
}

pub(super) async fn alias_patch(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
   Json(body): Json<AliasPatch>,
) -> ApiResult<Json<AliasView>> {
   let conn = state.pool.get().await?;
   if let Some(note) = body.note {
      aliases::set_note()
         .bind(&conn, &note, &id, &principal.user_id)
         .await?;
   }
   if let Some(pinned) = body.pinned {
      aliases::set_pinned()
         .bind(&conn, &pinned, &id, &principal.user_id)
         .await?;
   }
   if let Some(mailbox_id) = body.mailbox_id {
      let ok = mailboxes::exists_verified()
         .bind(&conn, &mailbox_id, &principal.user_id)
         .opt()
         .await?;
      if ok.is_none() {
         return Err(ApiError::BadRequest(
            "mailbox not found or not verified".into(),
         ));
      }
      aliases::set_mailbox()
         .bind(&conn, &mailbox_id, &id, &principal.user_id)
         .await
         .map_err(shared::raise_exception_as_bad_request)?;
   }
   alias_get(State(state), Extension(principal), Path(id)).await
}

pub(super) async fn alias_toggle(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<Json<AliasView>> {
   let conn = state.pool.get().await?;
   let n = aliases::toggle_enabled()
      .bind(&conn, &id, &principal.user_id)
      .await?;
   if n == 0 {
      return Err(ApiError::NotFound);
   }
   alias_get(State(state), Extension(principal), Path(id)).await
}

pub(super) async fn alias_delete(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
   let conn = state.pool.get().await?;
   let n = aliases::delete()
      .bind(&conn, &id, &principal.user_id)
      .await?;
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
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
   Query(query): Query<ActivityQuery>,
) -> ApiResult<Json<Value>> {
   let conn = state.pool.get().await?;
   let owns = aliases::exists_for_user()
      .bind(&conn, &id, &principal.user_id)
      .opt()
      .await?;
   if owns.is_none() {
      return Err(ApiError::NotFound);
   }
   let page = query.page.unwrap_or(0).max(0);
   let activities = email_log::activity_for_alias_api()
      .bind(&conn, &id, &PAGE_SIZE, &(page * PAGE_SIZE))
      .all()
      .await?
      .into_iter()
      .map(|entry| {
         json!({
             "id": entry.id,
             "action": entry.action,
             "status": entry.status,
             "from": entry.from_address,
             "created_at": entry.created_at.format(&Rfc3339).unwrap_or_default(),
             "reason": entry.reason,
         })
      })
      .collect::<Vec<Value>>();
   Ok(Json(json!({"activities": activities, "page": page})))
}

#[derive(Deserialize)]
pub(super) struct AliasRandom {
   #[serde(default)]
   domain:     Option<String>,
   #[serde(default)]
   note:       Option<String>,
   #[serde(default)]
   mailbox_id: Option<i64>,
}

pub(super) async fn alias_random(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Json(body): Json<AliasRandom>,
) -> ApiResult<(StatusCode, Json<AliasView>)> {
   let mut conn = state.pool.get().await?;
   let (dom, mb_id) =
      resolve_domain_and_mailbox(&conn, &principal, body.domain, body.mailbox_id).await?;
   let local = random_local_part(&dom.random_prefix);
   let addr = format!("{local}@{}", dom.domain);
   let note = shared::trimmed_nonempty(body.note);

   let id = insert_alias(
      &mut conn,
      principal.user_id,
      &addr,
      dom.id,
      mb_id,
      &note,
      false,
   )
   .await?;
   let row = aliases::by_id().bind(&conn, &id).one().await?;
   Ok((StatusCode::CREATED, Json(row.into())))
}

#[derive(Deserialize)]
pub(super) struct AliasCustomNew {
   alias_prefix: String,
   #[serde(default)]
   alias_suffix: Option<String>,
   domain:       String,
   mailbox_id:   Option<i64>,
   #[serde(default)]
   note:         Option<String>,
}

pub(super) async fn alias_custom_new(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Json(body): Json<AliasCustomNew>,
) -> ApiResult<(StatusCode, Json<AliasView>)> {
   let mut conn = state.pool.get().await?;
   let (dom, mb_id) =
      resolve_domain_and_mailbox(&conn, &principal, Some(body.domain), body.mailbox_id).await?;
   let prefix = body.alias_prefix.trim();
   shared::validate_local_part_fragment(prefix, "alias_prefix")?;
   let suffix = body.alias_suffix.as_deref().unwrap_or("").trim();
   let local = if suffix.is_empty() {
      prefix.to_owned()
   } else {
      shared::validate_local_part_fragment(suffix, "alias_suffix")?;
      format!("{prefix}-{suffix}")
   };
   if local.len() > 64 {
      return Err(ApiError::BadRequest(
         "combined alias local-part exceeds 64 bytes".into(),
      ));
   }
   let addr = format!("{local}@{}", dom.domain);
   let note = shared::trimmed_nonempty(body.note);

   let id = insert_alias(
      &mut conn,
      principal.user_id,
      &addr,
      dom.id,
      mb_id,
      &note,
      false,
   )
   .await?;
   let row = aliases::by_id().bind(&conn, &id).one().await?;
   Ok((StatusCode::CREATED, Json(row.into())))
}

#[expect(
   clippy::ref_option,
   reason = "note is forwarded directly as a bind parameter to the generated query"
)]
async fn insert_alias(
   conn: &mut Client,
   user_id: i64,
   addr: &str,
   domain_id: i64,
   mailbox_id: i64,
   note: &Option<String>,
   auto_created: bool,
) -> ApiResult<i64> {
   let txn = conn.transaction().await?;
   txn.execute("SELECT pg_advisory_xact_lock($1)", &[&quota::lock_id(
      LOCK_CLASS_ALIAS_CAP,
      user_id,
   )])
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
      .map_err(|err| {
         if shared::is_unique_violation(&err) {
            ApiError::Conflict(format!("alias {addr} already exists"))
         } else {
            shared::raise_exception_as_bad_request(err)
         }
      })?;

   txn.commit().await?;
   Ok(id)
}

async fn resolve_domain_and_mailbox(
   conn: &Client,
   principal: &Principal,
   domain: Option<String>,
   mailbox_id: Option<i64>,
) -> ApiResult<(domains::AliasDomainRow, i64)> {
   let dom = match domain {
      Some(value) => {
         let value = value.trim();
         domains::by_domain_for_user()
            .bind(conn, &value, &principal.user_id, &principal.is_admin)
            .opt()
            .await?
            .ok_or_else(|| ApiError::BadRequest(format!("domain not accessible: {value}")))?
      },
      None => domains::first_accessible_for_user()
         .bind(conn, &principal.user_id, &principal.is_admin)
         .opt()
         .await?
         .ok_or_else(|| ApiError::BadRequest("no alias_domain rows accessible".into()))?,
   };
   let mb_id = if let Some(id) = mailbox_id {
      let ok = mailboxes::exists_verified()
         .bind(conn, &id, &principal.user_id)
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
            .bind(conn, &did, &principal.user_id)
            .opt()
            .await?
      } else {
         None
      };
      if let Some(id) = default_for_user {
         id
      } else {
         mailboxes::first_verified_for_user()
            .bind(conn, &principal.user_id)
            .opt()
            .await?
            .ok_or_else(|| ApiError::BadRequest("no verified enabled mailbox for user".into()))?
      }
   };
   Ok((dom, mb_id))
}

fn random_local_part(prefix: &str) -> String {
   let mut bytes = [0_u8; 5];
   SysRng
      .try_fill_bytes(&mut bytes)
      .expect("SysRng must not fail");
   format!("{prefix}{}", hex::encode(bytes))
}
