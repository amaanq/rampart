//! `/api/v1/admin/*` — user CRUD + flipping a domain to shared.

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use rampart_codegen::queries::{aliases, api_keys, domains, sessions, users};
use serde::Deserialize;

use crate::AppState;
use crate::auth::AdminPrincipal;
use crate::error::{ApiError, ApiResult};

use super::shared::{hash_password, is_unique_violation};

pub(super) async fn admin_users_list(
    State(state): State<AppState>,
    _: AdminPrincipal,
) -> ApiResult<Json<serde_json::Value>> {
    let c = state.pool.get().await?;
    let users = users::list_admin().bind(&c).all().await?;
    Ok(Json(serde_json::json!({"users": users})))
}

#[derive(Deserialize)]
pub(super) struct AdminUserCreate {
    email: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    is_admin: bool,
    password: String,
}

pub(super) async fn admin_user_create(
    State(state): State<AppState>,
    _: AdminPrincipal,
    Json(body): Json<AdminUserCreate>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
    if body.password.len() < 10 {
        return Err(ApiError::BadRequest(
            "password must be at least 10 characters".into(),
        ));
    }
    let hash = hash_password(&body.password)?;
    let c = state.pool.get().await?;
    let id = users::create()
        .bind(
            &c,
            &body.email,
            &Some(hash),
            &body.display_name,
            &body.is_admin,
        )
        .one()
        .await
        .map_err(|e| {
            if is_unique_violation(&e) {
                ApiError::Conflict(format!("user {} already exists", body.email))
            } else {
                ApiError::Db(e)
            }
        })?;
    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

pub(super) async fn admin_user_enable(
    State(state): State<AppState>,
    _: AdminPrincipal,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    set_admin_user_enabled(&state, None, id, true).await
}

pub(super) async fn admin_user_disable(
    State(state): State<AppState>,
    AdminPrincipal(principal): AdminPrincipal,
    Path(id): Path<i64>,
) -> ApiResult<StatusCode> {
    set_admin_user_enabled(&state, Some(principal.user_id), id, false).await
}

#[derive(Deserialize)]
pub(super) struct AdminUserPatch {
    enabled: bool,
}

pub(super) async fn admin_user_patch(
    State(state): State<AppState>,
    AdminPrincipal(principal): AdminPrincipal,
    Path(id): Path<i64>,
    Json(body): Json<AdminUserPatch>,
) -> ApiResult<StatusCode> {
    set_admin_user_enabled(&state, Some(principal.user_id), id, body.enabled).await
}

fn reject_self_disable(actor_id: Option<i64>, target_id: i64) -> ApiResult<()> {
    if actor_id == Some(target_id) {
        return Err(ApiError::BadRequest(
            "you cannot disable your own account".into(),
        ));
    }
    Ok(())
}

async fn set_admin_user_enabled(
    state: &AppState,
    actor_id: Option<i64>,
    target_id: i64,
    enabled: bool,
) -> ApiResult<StatusCode> {
    if enabled {
        let c = state.pool.get().await?;
        let updated = users::enable().bind(&c, &target_id).await?;
        return if updated == 0 {
            Err(ApiError::NotFound)
        } else {
            Ok(StatusCode::NO_CONTENT)
        };
    }

    reject_self_disable(actor_id, target_id)?;
    let mut c = state.pool.get().await?;
    let txn = c.transaction().await?;
    let updated = users::disable().bind(&txn, &target_id).await?;
    if updated == 0 {
        return Err(ApiError::NotFound);
    }
    sessions::delete_by_user().bind(&txn, &target_id).await?;
    api_keys::revoke_all_for_user()
        .bind(&txn, &target_id)
        .await?;
    aliases::disable_all_for_user()
        .bind(&txn, &target_id)
        .await?;
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub(super) struct AdminDomainShared {
    shared: bool,
}

pub(super) async fn admin_domain_set_shared(
    State(state): State<AppState>,
    _: AdminPrincipal,
    Path(id): Path<i64>,
    Json(body): Json<AdminDomainShared>,
) -> ApiResult<StatusCode> {
    let mut c = state.pool.get().await?;
    let txn = c.transaction().await?;
    // Lock the alias_domain row FIRST, then COUNT — without this, a
    // concurrent alias INSERT can validate against shared=TRUE
    // (the alias_validate trigger now takes the same row lock) while
    // our COUNT runs on a snapshot that doesn't yet see the new alias,
    // then commits the new alias right before we flip shared=FALSE.
    // Holding the row lock through COUNT means the trigger waits for
    // us, and any newly-committed alias is visible to our COUNT.
    let row = txn
        .query_opt(
            "SELECT 1 FROM alias_domain WHERE id = $1 FOR UPDATE",
            &[&id],
        )
        .await?;
    if row.is_none() {
        return Err(ApiError::NotFound);
    }
    // Unsharing a domain with live non-owner aliases strands those rows:
    // the Sieve `rampart_sieve_lookup` view doesn't filter by
    // alias_domain.shared, so they keep forwarding, but alias_validate
    // rejects any UPDATE because the domain is no longer accessible
    // to the alias owner. Refuse the flip and make the admin clean up
    // first.
    if !body.shared {
        let row = txn
            .query_one(
                "SELECT COUNT(*)::bigint FROM alias a, alias_domain d \
                 WHERE a.domain_id = d.id AND d.id = $1 \
                   AND (d.owner_id IS NULL OR a.user_id <> d.owner_id)",
                &[&id],
            )
            .await?;
        let non_owner: i64 = row.get(0);
        if non_owner > 0 {
            return Err(ApiError::BadRequest(format!(
                "cannot unshare: {non_owner} alias(es) belong to non-owner users; \
                 delete or reassign them first"
            )));
        }
    }
    let n = domains::set_shared().bind(&txn, &body.shared, &id).await?;
    if n == 0 {
        return Err(ApiError::NotFound);
    }
    txn.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_disable_is_rejected() {
        assert!(matches!(
            reject_self_disable(Some(7), 7),
            Err(ApiError::BadRequest(message)) if message == "you cannot disable your own account"
        ));
    }

    #[test]
    fn another_user_can_be_disabled() {
        assert!(reject_self_disable(Some(7), 8).is_ok());
    }
}
