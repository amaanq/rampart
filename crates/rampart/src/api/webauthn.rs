//! `/api/v1/user/webauthn/*` — passkey registration + credential listing.

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
   users,
   webauthn,
};
use serde::{
   Deserialize,
   Serialize,
};
use serde_json::{
   Value,
   json,
};
use webauthn_rs::prelude::RegisterPublicKeyCredential;

use crate::{
   AppState,
   auth::Principal,
   error::{
      ApiError,
      ApiResult,
   },
   webauthn as passkey_flow,
};

#[derive(Serialize)]
pub(super) struct WebauthnStartResponse {
   ceremony_id: String,
   challenge:   Value,
}

pub(super) async fn webauthn_register_start(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Json<WebauthnStartResponse>> {
   let existing = passkey_flow::load_passkeys_for_user(&state.pool, principal.user_id)
      .await
      .map_err(ApiError::Internal)?;
   let exclude = existing
      .iter()
      .map(|passkey| passkey.cred_id().clone())
      .collect::<Vec<_>>();
   let user_handle = passkey_flow::user_handle(principal.user_id);
   let conn = state.pool.get().await?;
   let row = users::display_for_webauthn()
      .bind(&conn, &principal.user_id)
      .one()
      .await?;
   let display = row
      .display_name
      .clone()
      .unwrap_or_else(|| row.email.clone());
   let (challenge, reg_state) = state
      .webauthn
      .start_passkey_registration(user_handle, &row.email, &display, Some(exclude))
      .map_err(|err| ApiError::Internal(anyhow::anyhow!("webauthn start: {err}")))?;
   let ceremony_id =
      passkey_flow::save_registration_state(&state.pool, principal.user_id, &reg_state)
         .await
         .map_err(ApiError::Internal)?;
   Ok(Json(WebauthnStartResponse {
      ceremony_id: hex::encode(&ceremony_id),
      challenge:   serde_json::to_value(&challenge)
         .map_err(|err| ApiError::Internal(anyhow::anyhow!("json: {err}")))?,
   }))
}

#[derive(Deserialize)]
pub(super) struct WebauthnRegisterFinish {
   ceremony_id: String,
   name:        String,
   credential:  RegisterPublicKeyCredential,
}

pub(super) async fn webauthn_register_finish(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Json(body): Json<WebauthnRegisterFinish>,
) -> ApiResult<StatusCode> {
   let id =
      hex::decode(&body.ceremony_id).map_err(|_| ApiError::BadRequest("bad ceremony id".into()))?;
   let reg_state = passkey_flow::load_registration_state(&state.pool, &id, principal.user_id)
      .await
      .map_err(|err| ApiError::BadRequest(format!("{err}")))?;
   let passkey = state
      .webauthn
      .finish_passkey_registration(&body.credential, &reg_state)
      .map_err(|err| ApiError::BadRequest(format!("{err}")))?;
   passkey_flow::insert_credential(&state.pool, principal.user_id, &body.name, &passkey)
      .await
      .map_err(ApiError::Internal)?;
   Ok(StatusCode::CREATED)
}

pub(super) async fn webauthn_list(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Json<Value>> {
   let conn = state.pool.get().await?;
   let creds = webauthn::list_for_user()
      .bind(&conn, &principal.user_id)
      .all()
      .await?;
   Ok(Json(json!({"credentials": creds})))
}

pub(super) async fn webauthn_delete(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
   let conn = state.pool.get().await?;
   let n = webauthn::delete_for_user()
      .bind(&conn, &id, &principal.user_id)
      .await?;
   if n == 0 {
      return Err(ApiError::NotFound);
   }
   Ok(StatusCode::NO_CONTENT)
}
