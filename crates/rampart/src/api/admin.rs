//! `/api/v1/admin/*` — user CRUD + flipping a domain to shared.

use axum::{
   Json,
   extract::{
      Path,
      State,
   },
   http::StatusCode,
};
use rampart_codegen::queries::{
   aliases,
   api_keys,
   domains,
   sessions,
   tokens,
   users,
};
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;

use super::shared;
use crate::{
   AppState,
   auth::AdminPrincipal,
   error::{
      ApiError,
      ApiResult,
   },
   flows,
};

pub(super) async fn admin_users_list(
   State(state): State<AppState>,
   _: AdminPrincipal,
) -> ApiResult<Json<serde_json::Value>> {
   let conn = state.pool.get().await?;
   let users = users::list_admin().bind(&conn).all().await?;
   Ok(Json(serde_json::json!({"users": users})))
}

#[derive(Deserialize)]
pub(super) struct AdminUserCreate {
   email:        String,
   #[serde(default)]
   display_name: Option<String>,
   #[serde(default)]
   is_admin:     bool,
   password:     String,
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
   let hash = shared::hash_password(&body.password)?;
   let conn = state.pool.get().await?;
   let id = users::create()
      .bind(
         &conn,
         &body.email,
         &Some(hash),
         &body.display_name,
         &body.is_admin,
      )
      .one()
      .await
      .map_err(|err| {
         if shared::is_unique_violation(&err) {
            ApiError::Conflict(format!("user {} already exists", body.email))
         } else {
            ApiError::Db(err)
         }
      })?;
   Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
}

#[derive(Deserialize)]
pub(super) struct AdminInviteCreate {
   #[serde(default)]
   email: Option<String>,
}

pub(super) async fn admin_invite_create(
   State(state): State<AppState>,
   AdminPrincipal(principal): AdminPrincipal,
   Json(body): Json<AdminInviteCreate>,
) -> ApiResult<(StatusCode, Json<serde_json::Value>)> {
   use std::str::FromStr as _;

   let email = body.email.and_then(|email| {
      let email = email.trim().to_owned();
      (!email.is_empty()).then_some(email)
   });
   if let Some(email) = email.as_deref() {
      lettre::Address::from_str(email)
         .map_err(|_| ApiError::BadRequest("Enter a valid email address.".into()))?;
   }

   let conn = state.pool.get().await?;
   let invite = flows::create_invite(&conn, Some(principal.user_id), email.as_deref())
      .await
      .map_err(ApiError::Internal)?;
   let url = format!(
      "{}/signup/{}",
      state.config.public_origin.trim_end_matches('/'),
      invite.token
   );
   let expires_at = invite
      .expires_at
      .format(&Rfc3339)
      .map_err(|error| ApiError::Internal(error.into()))?;
   let delivered = if let Some(email) = email.as_deref() {
      let message = format!(
         "You have been invited to Rampart.\n\nOpen this link to create your \
          account\n{url}\n\nThis link expires in seven days.\n"
      );
      match state
         .mailer
         .send(email, "rampart invitation", &message)
         .await
      {
         Ok(()) => true,
         Err(error) => {
            tracing::error!(%error, %email, "invite email delivery failed");
            false
         },
      }
   } else {
      false
   };

   Ok((
      StatusCode::CREATED,
      Json(serde_json::json!({
         "id": invite.id,
         "email": email,
         "expires_at": expires_at,
         "url": url,
         "delivered": delivered,
      })),
   ))
}

pub(super) async fn admin_invite_revoke(
   State(state): State<AppState>,
   _: AdminPrincipal,
   Path(id): Path<String>,
) -> ApiResult<StatusCode> {
   let token_hash =
      hex::decode(id).map_err(|_| ApiError::BadRequest("Invalid invitation identifier.".into()))?;
   if token_hash.len() != 32 {
      return Err(ApiError::BadRequest(
         "Invalid invitation identifier.".into(),
      ));
   }
   let conn = state.pool.get().await?;
   let deleted = tokens::invite_revoke().bind(&conn, &token_hash).await?;
   if deleted == 0 {
      Err(ApiError::NotFound)
   } else {
      Ok(StatusCode::NO_CONTENT)
   }
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

#[derive(Deserialize)]
pub(super) struct AdminUserRolePatch {
   is_admin: bool,
}

pub(super) async fn admin_user_role_patch(
   State(state): State<AppState>,
   AdminPrincipal(principal): AdminPrincipal,
   Path(id): Path<i64>,
   Json(body): Json<AdminUserRolePatch>,
) -> ApiResult<StatusCode> {
   reject_self_demotion(principal.user_id, id, body.is_admin)?;
   let conn = state.pool.get().await?;
   let updated = users::set_admin().bind(&conn, &body.is_admin, &id).await?;
   if updated == 0 {
      Err(ApiError::NotFound)
   } else {
      Ok(StatusCode::NO_CONTENT)
   }
}

fn reject_self_demotion(actor_id: i64, target_id: i64, is_admin: bool) -> ApiResult<()> {
   if actor_id == target_id && !is_admin {
      return Err(ApiError::BadRequest(
         "you cannot demote your own account".into(),
      ));
   }
   Ok(())
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
      let conn = state.pool.get().await?;
      let updated = users::enable().bind(&conn, &target_id).await?;
      return if updated == 0 {
         Err(ApiError::NotFound)
      } else {
         Ok(StatusCode::NO_CONTENT)
      };
   }

   reject_self_disable(actor_id, target_id)?;
   let mut conn = state.pool.get().await?;
   let txn = conn.transaction().await?;
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
   let mut conn = state.pool.get().await?;
   let txn = conn.transaction().await?;
   // Lock the alias_domain row FIRST, then COUNT — without this, a
   // concurrent alias INSERT can validate against shared=TRUE
   // (the alias_validate trigger now takes the same row lock) while
   // our COUNT runs on a snapshot that doesn't yet see the new alias,
   // then commits the new alias right before we flip shared=FALSE.
   // Holding the row lock through COUNT means the trigger waits for
   // us, and any newly-committed alias is visible to our COUNT.
   let row = txn
      .query_opt("SELECT 1 FROM alias_domain WHERE id = $1 FOR UPDATE", &[
         &id,
      ])
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
      let count_row = txn
         .query_one(
            "SELECT COUNT(*)::bigint FROM alias a, alias_domain d WHERE a.domain_id = d.id AND \
             d.id = $1 AND (d.owner_id IS NULL OR a.user_id <> d.owner_id)",
            &[&id],
         )
         .await?;
      let non_owner: i64 = count_row.get(0);
      if non_owner > 0 {
         return Err(ApiError::BadRequest(format!(
            "cannot unshare: {non_owner} alias(es) belong to non-owner users; delete or reassign \
             them first"
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
#[expect(
   clippy::inline_modules,
   reason = "small cohesive submodule kept inline"
)]
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
      reject_self_disable(Some(7), 8).unwrap();
   }

   #[test]
   fn self_demotion_is_rejected() {
      assert!(matches!(
         reject_self_demotion(7, 7, false),
         Err(ApiError::BadRequest(message)) if message == "you cannot demote your own account"
      ));
   }

   #[test]
   fn another_user_role_can_change() {
      reject_self_demotion(7, 8, false).unwrap();
      reject_self_demotion(7, 8, true).unwrap();
   }
}
