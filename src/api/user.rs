//! `/api/v1/user/*` — info, password change, email change.

use axum::{Extension, Json, extract::State, http::StatusCode};
use rampart_codegen::queries::{sessions, users};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::Principal;
use crate::error::{ApiError, ApiResult};

use super::shared::hash_password;

#[derive(Serialize)]
pub(super) struct UserInfo {
    user_id: i64,
    email: String,
    is_admin: bool,
    alias_count: i64,
    mailbox_count: i64,
    domain_count: i64,
}

pub(super) async fn user_info(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Json<UserInfo>> {
    let c = state.pool.get().await?;
    let r = users::info().bind(&c, &p.user_id).one().await?;
    Ok(Json(UserInfo {
        user_id: p.user_id,
        email: r.email,
        is_admin: r.is_admin,
        alias_count: r.alias_count,
        mailbox_count: r.mailbox_count,
        domain_count: r.domain_count,
    }))
}

#[derive(Deserialize)]
pub(super) struct ChangePassword {
    current_password: String,
    new_password: String,
}

pub(super) async fn user_change_password(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Json(body): Json<ChangePassword>,
) -> ApiResult<StatusCode> {
    if body.new_password.len() < 10 {
        return Err(ApiError::BadRequest(
            "New password must be at least 10 characters.".into(),
        ));
    }
    let mut c = state.pool.get().await?;
    let txn = c.transaction().await?;
    // Read + verify + write all under a row-level lock. Without
    // FOR UPDATE here, a concurrent admin reset / password-reset-token
    // flow can commit a recovery password BETWEEN our verify and our
    // UPDATE — our in-flight request (authenticated with the now-stale
    // old password) then clobbers the recovery. SELECT FOR UPDATE
    // serializes against `apply_password_reset` and any future admin
    // reset path that takes the row lock via UPDATE.
    let row = txn
        .query_opt(
            "SELECT password_hash::text FROM \"user\" WHERE id = $1 FOR UPDATE",
            &[&p.user_id],
        )
        .await?;
    let stored: Option<String> = row.and_then(|r| r.get::<_, Option<String>>(0));
    let Some(stored) = stored else {
        return Err(ApiError::BadRequest(
            "No password is set for this account.".into(),
        ));
    };
    if !argon2::verify_encoded(&stored, body.current_password.as_bytes()).unwrap_or(false) {
        return Err(ApiError::BadRequest(
            "Current password is incorrect.".into(),
        ));
    }
    let new_hash = hash_password(&body.new_password)?;
    users::set_password()
        .bind(&txn, &Some(new_hash), &p.user_id)
        .await?;
    sessions::delete_by_user().bind(&txn, &p.user_id).await?;
    txn.commit().await?;
    state.verify_cache.invalidate_user(p.user_id);
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub(super) struct ChangeEmailRequest {
    new_email: String,
}

pub(super) async fn user_start_email_change(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Json(body): Json<ChangeEmailRequest>,
) -> ApiResult<StatusCode> {
    use std::str::FromStr;
    let new_email = body.new_email.trim();
    lettre::Address::from_str(new_email)
        .map_err(|_| ApiError::BadRequest("Enter a valid email address.".into()))?;
    let ok = crate::abuse::check(
        &state.pool,
        &format!("email_change:{}", p.user_id),
        crate::abuse::EMAIL_CHANGE,
    )
    .await
    .map_err(ApiError::Internal)?;
    if !ok {
        return Err(ApiError::BadRequest(
            "too many requests. Try again later".into(),
        ));
    }
    crate::flows::start_email_change(
        &state.pool,
        state.mailer.as_ref(),
        &state.config.public_origin,
        p.user_id,
        new_email,
    )
    .await
    .map_err(|error| match error {
        crate::flows::StartEmailChangeError::AlreadyRegistered => {
            ApiError::Conflict("An account already uses this email address.".into())
        }
        crate::flows::StartEmailChangeError::Internal(error) => ApiError::Internal(error),
    })?;
    Ok(StatusCode::ACCEPTED)
}
