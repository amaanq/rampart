//! `/api/v1/mailboxes` and `/api/v1/mailbox/*`.

use std::str::FromStr as _;

use axum::{
   Extension,
   Json,
   extract::{
      Path,
      State,
   },
   http::StatusCode,
};
use rampart_codegen::queries::mailboxes;
use serde::Deserialize;
use serde_json::{
   Value,
   json,
};

use super::shared::{
   self,
   deserialize_opt_field,
};
use crate::{
   AppState,
   abuse,
   auth::Principal,
   error::{
      ApiError,
      ApiResult,
   },
   flows,
};

pub(super) type MailboxView = mailboxes::MailboxRow;

pub(super) async fn mailboxes_list(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Json<Value>> {
   let conn = state.pool.get().await?;
   let out = mailboxes::list_for_user()
      .bind(&conn, &principal.user_id)
      .all()
      .await?;
   Ok(Json(json!({"mailboxes": out})))
}

#[derive(Deserialize)]
pub(super) struct MailboxCreate {
   email:        String,
   #[serde(default)]
   display_name: Option<String>,
}

pub(super) async fn mailbox_create(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Json(body): Json<MailboxCreate>,
) -> ApiResult<(StatusCode, Json<MailboxView>)> {
   let conn = state.pool.get().await?;
   let email = body.email.trim();
   let display_name = shared::trimmed_nonempty(body.display_name);
   // RFC 5321-ish parse via lettre — the bare `contains('@')` check let
   // typos like `alice@@example` through, which later wedged submit() on
   // first forward attempt. Same parser used in src/admin.rs add-mailbox.
   lettre::Address::from_str(email)
      .map_err(|err| ApiError::BadRequest(format!("invalid email '{email}': {err}")))?;
   let id = mailboxes::create()
      .bind(&conn, &principal.user_id, &email, &display_name)
      .one()
      .await
      .map_err(|err| {
         if shared::is_unique_violation(&err) {
            ApiError::Conflict(format!("mailbox {email} already exists"))
         } else {
            ApiError::Db(err)
         }
      })?;
   if let Err(err) = flows::start_mailbox_verify(
      &state.pool,
      state.mailer.as_ref(),
      &state.config.public_origin,
      id,
   )
   .await
   {
      tracing::warn!(error = ?err, "failed to send verification email");
   }
   let view = mailboxes::by_id().bind(&conn, &id).one().await?;
   Ok((StatusCode::CREATED, Json(view)))
}

#[derive(Deserialize)]
pub(super) struct MailboxPatch {
   #[serde(default, deserialize_with = "deserialize_opt_field")]
   #[expect(
      clippy::option_option,
      reason = "Some(None) sets display_name to null; None leaves it unchanged"
   )]
   display_name: Option<Option<String>>,
   enabled:      Option<bool>,
}

pub(super) async fn mailbox_patch(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
   Json(body): Json<MailboxPatch>,
) -> ApiResult<Json<MailboxView>> {
   let conn = state.pool.get().await?;
   if let Some(dn) = body.display_name {
      mailboxes::set_display_name()
         .bind(&conn, &dn, &id, &principal.user_id)
         .await?;
   }
   if let Some(en) = body.enabled {
      mailboxes::set_enabled()
         .bind(&conn, &en, &id, &principal.user_id)
         .await?;
   }
   let view = mailboxes::by_id_user()
      .bind(&conn, &id, &principal.user_id)
      .opt()
      .await?
      .ok_or(ApiError::NotFound)?;
   Ok(Json(view))
}

pub(super) async fn mailbox_delete(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
   let conn = state.pool.get().await?;
   match mailboxes::delete()
      .bind(&conn, &id, &principal.user_id)
      .await
   {
      Ok(0) => Err(ApiError::NotFound),
      Ok(_) => Ok(StatusCode::NO_CONTENT),
      Err(err) if shared::is_fk_violation(&err) => Err(ApiError::Conflict(
         "mailbox has aliases pointing at it. Reassign or delete them first".into(),
      )),
      Err(err) => Err(ApiError::Db(err)),
   }
}

pub(super) async fn mailbox_resend_verify(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
   let conn = state.pool.get().await?;
   let Some(verified) = mailboxes::verified_for_user()
      .bind(&conn, &id, &principal.user_id)
      .opt()
      .await?
   else {
      return Err(ApiError::NotFound);
   };
   if verified {
      return Ok(StatusCode::NO_CONTENT);
   }
   let ok = abuse::check(
      &state.pool,
      &format!("mailbox_verify:{id}"),
      abuse::MAILBOX_VERIFY_RESEND,
   )
   .await
   .map_err(ApiError::Internal)?;
   if !ok {
      return Err(ApiError::BadRequest("too many requests".into()));
   }
   flows::start_mailbox_verify(
      &state.pool,
      state.mailer.as_ref(),
      &state.config.public_origin,
      id,
   )
   .await
   .map_err(ApiError::Internal)?;
   Ok(StatusCode::ACCEPTED)
}
