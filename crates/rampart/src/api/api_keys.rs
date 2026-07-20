use axum::{
   Extension,
   Json,
   extract::{
      Path,
      State,
   },
   http::StatusCode,
};
use rampart_codegen::queries::api_keys;
use serde::{
   Deserialize,
   Serialize,
};
use serde_json::{
   Value,
   json,
};
use time::OffsetDateTime;

use crate::{
   AppState,
   auth::{
      self,
      Principal,
   },
   error::{
      ApiError,
      ApiResult,
   },
};

const EXTENSION_SCOPES: [&str; 2] = ["alias:create", "extension:read"];

#[derive(Deserialize)]
pub(super) struct ApiKeyCreate {
   name:       String,
   #[serde(default, with = "time::serde::rfc3339::option")]
   expires_at: Option<OffsetDateTime>,
}

#[derive(Serialize)]
pub(super) struct ApiKeyCreated {
   token: String,
   key:   api_keys::ApiKeyRow,
}

pub(super) async fn api_keys_list(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Json<Value>> {
   let conn = state.pool.get().await?;
   let keys = api_keys::list_for_user()
      .bind(&conn, &principal.user_id)
      .all()
      .await?;
   Ok(Json(json!({"api_keys": keys})))
}

pub(super) async fn api_key_create(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Json(body): Json<ApiKeyCreate>,
) -> ApiResult<(StatusCode, Json<ApiKeyCreated>)> {
   let name = body.name.trim();
   if name.is_empty() || name.len() > 80 {
      return Err(ApiError::BadRequest(
         "API key name must be 1..=80 characters".into(),
      ));
   }
   if body
      .expires_at
      .is_some_and(|expiry| expiry <= OffsetDateTime::now_utc())
   {
      return Err(ApiError::BadRequest(
         "API key expiration must be in the future".into(),
      ));
   }

   let token = auth::generate_api_key_token();
   let key_hash = auth::hash_api_key(&token);
   let token_prefix = Some(token.chars().take(12).collect::<String>());
   let scopes = EXTENSION_SCOPES.map(str::to_owned).to_vec();
   let conn = state.pool.get().await?;
   let key = api_keys::create_extension()
      .bind(
         &conn,
         &principal.user_id,
         &name,
         &key_hash,
         &scopes,
         &token_prefix,
         &body.expires_at,
      )
      .one()
      .await?;
   Ok((StatusCode::CREATED, Json(ApiKeyCreated { token, key })))
}

pub(super) async fn api_key_revoke(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
   let conn = state.pool.get().await?;
   let changed = api_keys::revoke_for_user()
      .bind(&conn, &id, &principal.user_id)
      .await?;
   if changed == 0 {
      return Err(ApiError::NotFound);
   }
   Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn api_key_revoke_self(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<StatusCode> {
   let id = principal
      .api_key_id
      .ok_or_else(|| ApiError::BadRequest("request did not use an API key".into()))?;
   let conn = state.pool.get().await?;
   let changed = api_keys::revoke_self()
      .bind(&conn, &id, &principal.user_id)
      .await?;
   if changed == 0 {
      return Err(ApiError::NotFound);
   }
   Ok(StatusCode::NO_CONTENT)
}
