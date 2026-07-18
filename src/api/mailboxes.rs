//! `/api/v1/mailboxes` and `/api/v1/mailbox/*`.

use axum::{
    Extension, Json,
    extract::{Path, State},
    http::StatusCode,
};
use rampart_codegen::queries::mailboxes;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;
use crate::auth::Principal;
use crate::error::{ApiError, ApiResult};

use super::shared::{
    deserialize_opt_field, is_fk_violation, is_unique_violation, trimmed_nonempty,
};

pub(super) type MailboxView = mailboxes::MailboxRow;

pub(super) async fn mailboxes_list(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Json<Value>> {
    let c = state.pool.get().await?;
    let out = mailboxes::list_for_user()
        .bind(&c, &p.user_id)
        .all()
        .await?;
    Ok(Json(json!({"mailboxes": out})))
}

#[derive(Deserialize)]
pub(super) struct MailboxCreate {
    email: String,
    #[serde(default)]
    display_name: Option<String>,
}

pub(super) async fn mailbox_create(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Json(body): Json<MailboxCreate>,
) -> ApiResult<(StatusCode, Json<MailboxView>)> {
    let c = state.pool.get().await?;
    let email = body.email.trim();
    let display_name = trimmed_nonempty(body.display_name);
    // RFC 5321-ish parse via lettre — the bare `contains('@')` check let
    // typos like `alice@@example` through, which later wedged submit() on
    // first forward attempt. Same parser used in src/admin.rs add-mailbox.
    use std::str::FromStr;
    lettre::Address::from_str(email)
        .map_err(|e| ApiError::BadRequest(format!("invalid email '{email}': {e}")))?;
    let id = mailboxes::create()
        .bind(&c, &p.user_id, &email, &display_name)
        .one()
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                ApiError::Conflict(format!("mailbox {email} already exists"))
            } else {
                ApiError::Db(e)
            }
        })?;
    if let Err(e) = crate::flows::start_mailbox_verify(
        &state.pool,
        state.mailer.as_ref(),
        &state.config.public_origin,
        id,
    )
    .await
    {
        tracing::warn!(error = ?e, "failed to send verification email");
    }
    let view = mailboxes::by_id().bind(&c, &id).one().await?;
    Ok((StatusCode::CREATED, Json(view)))
}

#[derive(Deserialize)]
pub(super) struct MailboxPatch {
    #[serde(default, deserialize_with = "deserialize_opt_field")]
    display_name: Option<Option<String>>,
    enabled: Option<bool>,
}

pub(super) async fn mailbox_patch(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
    Json(body): Json<MailboxPatch>,
) -> ApiResult<Json<MailboxView>> {
    let c = state.pool.get().await?;
    if let Some(dn) = body.display_name {
        mailboxes::set_display_name()
            .bind(&c, &dn, &id, &p.user_id)
            .await?;
    }
    if let Some(en) = body.enabled {
        mailboxes::set_enabled()
            .bind(&c, &en, &id, &p.user_id)
            .await?;
    }
    let view = mailboxes::by_id_user()
        .bind(&c, &id, &p.user_id)
        .opt()
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(view))
}

pub(super) async fn mailbox_delete(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let c = state.pool.get().await?;
    match mailboxes::delete().bind(&c, &id, &p.user_id).await {
        Ok(0) => Err(ApiError::NotFound),
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) if is_fk_violation(&e) => Err(ApiError::Conflict(
            "mailbox has aliases pointing at it; reassign or delete them first".into(),
        )),
        Err(e) => Err(ApiError::Db(e)),
    }
}

pub(super) async fn mailbox_resend_verify(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    let c = state.pool.get().await?;
    let Some(verified) = mailboxes::verified_for_user()
        .bind(&c, &id, &p.user_id)
        .opt()
        .await?
    else {
        return Err(ApiError::NotFound);
    };
    if verified {
        return Ok(StatusCode::NO_CONTENT);
    }
    let ok = crate::abuse::check(
        &state.pool,
        &format!("mailbox_verify:{id}"),
        crate::abuse::MAILBOX_VERIFY_RESEND,
    )
    .await
    .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::BadRequest("too many requests".into()));
    }
    crate::flows::start_mailbox_verify(
        &state.pool,
        state.mailer.as_ref(),
        &state.config.public_origin,
        id,
    )
    .await
    .map_err(ApiError::Internal)?;
    Ok(StatusCode::ACCEPTED)
}
