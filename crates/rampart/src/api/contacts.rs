//! `/api/v1/aliases/{id}/contacts` and `/api/v1/contacts/{id}` —
//! `reverse_contact` rows.

use axum::{
   Extension,
   Json,
   extract::{
      Path,
      State,
   },
   http::StatusCode,
};
use rampart_codegen::queries::{
   aliases,
   contacts,
};
use serde::Deserialize;
use serde_json::{
   Value,
   json,
};

use super::shared::deserialize_opt_field;
use crate::{
   AppState,
   auth::Principal,
   error::{
      ApiError,
      ApiResult,
   },
};

pub(super) async fn contacts_list(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(alias_id): Path<i64>,
) -> ApiResult<Json<Value>> {
   let conn = state.pool.get().await?;
   let owns = aliases::exists_for_user()
      .bind(&conn, &alias_id, &principal.user_id)
      .opt()
      .await?;
   if owns.is_none() {
      return Err(ApiError::NotFound);
   }
   let out = contacts::list_for_alias()
      .bind(&conn, &alias_id)
      .all()
      .await?;
   Ok(Json(json!({"contacts": out})))
}

#[derive(Deserialize)]
pub(super) struct ContactPatch {
   enabled:      Option<bool>,
   block_reply:  Option<bool>,
   #[serde(default, deserialize_with = "deserialize_opt_field")]
   #[expect(
      clippy::option_option,
      reason = "Some(None) sets display_name to null; None leaves it unchanged"
   )]
   display_name: Option<Option<String>>,
}

pub(super) async fn contact_patch(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
   Json(body): Json<ContactPatch>,
) -> ApiResult<StatusCode> {
   let conn = state.pool.get().await?;
   let owns = contacts::exists_for_user()
      .bind(&conn, &id, &principal.user_id)
      .opt()
      .await?;
   if owns.is_none() {
      return Err(ApiError::NotFound);
   }
   if let Some(value) = body.enabled {
      contacts::set_enabled().bind(&conn, &value, &id).await?;
   }
   if let Some(value) = body.block_reply {
      contacts::set_block_reply().bind(&conn, &value, &id).await?;
   }
   if let Some(value) = body.display_name {
      contacts::set_display_name()
         .bind(&conn, &value, &id)
         .await?;
   }
   Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn contact_delete(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
   let conn = state.pool.get().await?;
   let n = contacts::delete_for_user()
      .bind(&conn, &id, &principal.user_id)
      .await?;
   if n == 0 {
      return Err(ApiError::NotFound);
   }
   Ok(StatusCode::NO_CONTENT)
}
