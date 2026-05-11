//! `/api/v1/aliases/{id}/contacts` and `/api/v1/contacts/{id}` —
//! reverse_contact rows.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use rampart_codegen::queries::{aliases, contacts};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;
use crate::auth::Principal;
use crate::error::{ApiError, ApiResult};

use super::shared::deserialize_opt_field;

pub(super) async fn contacts_list(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(alias_id): Path<i64>,
) -> ApiResult<Json<Value>> {
    let c = state.pool.get().await?;
    let owns = aliases::exists_for_user()
        .bind(&c, &alias_id, &p.user_id)
        .opt()
        .await?;
    if owns.is_none() {
        return Err(ApiError::NotFound);
    }
    let out = contacts::list_for_alias().bind(&c, &alias_id).all().await?;
    Ok(Json(json!({"contacts": out})))
}

#[derive(Deserialize)]
pub(super) struct ContactPatch {
    enabled: Option<bool>,
    block_reply: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_opt_field")]
    display_name: Option<Option<String>>,
}

pub(super) async fn contact_patch(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
    Json(body): Json<ContactPatch>,
) -> ApiResult<StatusCode> {
    let c = state.pool.get().await?;
    let owns = contacts::exists_for_user()
        .bind(&c, &id, &p.user_id)
        .opt()
        .await?;
    if owns.is_none() {
        return Err(ApiError::NotFound);
    }
    if let Some(v) = body.enabled {
        contacts::set_enabled().bind(&c, &v, &id).await?;
    }
    if let Some(v) = body.block_reply {
        contacts::set_block_reply().bind(&c, &v, &id).await?;
    }
    if let Some(v) = body.display_name {
        contacts::set_display_name().bind(&c, &v, &id).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn contact_delete(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let c = state.pool.get().await?;
    let n = contacts::delete_for_user()
        .bind(&c, &id, &p.user_id)
        .await?;
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}
