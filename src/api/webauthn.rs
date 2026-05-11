//! `/api/v1/user/webauthn/*` — passkey registration + credential listing.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use rampart_codegen::queries::{users, webauthn};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::AppState;
use crate::auth::Principal;
use crate::error::{ApiError, ApiResult};

#[derive(Serialize)]
pub(super) struct WebauthnStartResponse {
    ceremony_id: String,
    challenge: Value,
}

pub(super) async fn webauthn_register_start(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Json<WebauthnStartResponse>> {
    let existing = crate::webauthn::load_passkeys_for_user(&state.pool, p.user_id)
        .await
        .map_err(ApiError::Internal)?;
    let exclude = existing
        .iter()
        .map(|p| p.cred_id().clone())
        .collect::<Vec<_>>();
    // Stable per-account UUID derived from user_id. Webauthn requires
    // <=64 bytes; use a 16-byte SHA256 prefix.
    let mut hasher = hmac_sha256::Hash::new();
    hasher.update(b"rampart-user-");
    hasher.update(&p.user_id.to_be_bytes());
    let digest = hasher.finalize();
    let handle_bytes: [u8; 16] = digest[..16].try_into().unwrap();
    let user_handle = webauthn_rs::prelude::Uuid::from_bytes(handle_bytes);
    let c = state.pool.get().await?;
    let r = users::display_for_webauthn()
        .bind(&c, &p.user_id)
        .one()
        .await?;
    let display = r.display_name.clone().unwrap_or_else(|| r.email.clone());
    let (challenge, reg_state) = state
        .webauthn
        .start_passkey_registration(user_handle, &r.email, &display, Some(exclude))
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("webauthn start: {e}")))?;
    let ceremony_id = crate::webauthn::save_registration_state(&state.pool, p.user_id, &reg_state)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(WebauthnStartResponse {
        ceremony_id: hex::encode(&ceremony_id),
        challenge: serde_json::to_value(&challenge)
            .map_err(|e| ApiError::Internal(anyhow::anyhow!("json: {e}")))?,
    }))
}

#[derive(Deserialize)]
pub(super) struct WebauthnRegisterFinish {
    ceremony_id: String,
    name: String,
    credential: webauthn_rs::prelude::RegisterPublicKeyCredential,
}

pub(super) async fn webauthn_register_finish(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Json(body): Json<WebauthnRegisterFinish>,
) -> ApiResult<StatusCode> {
    let id = hex::decode(&body.ceremony_id)
        .map_err(|_| ApiError::BadRequest("bad ceremony id".into()))?;
    let reg_state = crate::webauthn::load_registration_state(&state.pool, &id, p.user_id)
        .await
        .map_err(|e| ApiError::BadRequest(format!("{e}")))?;
    let passkey = state
        .webauthn
        .finish_passkey_registration(&body.credential, &reg_state)
        .map_err(|e| ApiError::BadRequest(format!("{e}")))?;
    crate::webauthn::insert_credential(&state.pool, p.user_id, &body.name, &passkey)
        .await
        .map_err(ApiError::Internal)?;
    Ok(StatusCode::CREATED)
}

pub(super) async fn webauthn_list(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Json<Value>> {
    let c = state.pool.get().await?;
    let creds = webauthn::list_for_user().bind(&c, &p.user_id).all().await?;
    Ok(Json(json!({"credentials": creds})))
}

pub(super) async fn webauthn_delete(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let c = state.pool.get().await?;
    let n = webauthn::delete_for_user()
        .bind(&c, &id, &p.user_id)
        .await?;
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
