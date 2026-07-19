use axum::{
   Extension,
   Json,
   extract::State,
};
use rampart_codegen::queries::{
   domains,
   mailboxes,
   users,
};
use serde::Serialize;

use crate::{
   AppState,
   auth::Principal,
   domain_setup::{
      self,
      SetupState,
   },
   error::ApiResult,
   quota::DEFAULT_MAX_ALIASES,
};

#[derive(Serialize)]
struct ExtensionDomain {
   id:                 i64,
   domain:             String,
   default_mailbox_id: Option<i64>,
   ready:              bool,
}

#[derive(Serialize)]
struct ExtensionMailbox {
   id:           i64,
   email:        String,
   display_name: Option<String>,
}

#[derive(Serialize)]
pub(super) struct ExtensionBootstrap {
   domains:        Vec<ExtensionDomain>,
   mailboxes:      Vec<ExtensionMailbox>,
   alias_count:    i64,
   alias_limit:    i64,
   api_key_id:     Option<i64>,
   api_key_scopes: Vec<String>,
}

pub(super) async fn extension_bootstrap(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Json<ExtensionBootstrap>> {
   let conn = state.pool.get().await?;
   let domains = domains::list_for_user()
      .bind(&conn, &principal.user_id, &principal.is_admin)
      .all()
      .await?
      .into_iter()
      .map(|row| {
         let ready = domain_setup::build(
            row.id,
            &row.domain,
            &state.config.public_mx_hostname,
            &domain_setup::parse_dkim_records(&row.dkim_records),
            &domain_setup::parse_dns_status(&row.dns_status),
            row.dns_checked_at,
            row.dns_verified_at,
         )
         .state
            == SetupState::Ready;
         ExtensionDomain {
            id: row.id,
            domain: row.domain,
            default_mailbox_id: row.default_mailbox_id,
            ready,
         }
      })
      .collect();
   let mailboxes = mailboxes::list_for_user()
      .bind(&conn, &principal.user_id)
      .all()
      .await?
      .into_iter()
      .filter(|row| row.enabled && row.verified)
      .map(|row| ExtensionMailbox {
         id:           row.id,
         email:        row.email,
         display_name: row.display_name,
      })
      .collect();
   let cap = users::cap_and_count_aliases()
      .bind(&conn, &DEFAULT_MAX_ALIASES, &principal.user_id)
      .one()
      .await?;
   Ok(Json(ExtensionBootstrap {
      domains,
      mailboxes,
      alias_count: cap.current,
      alias_limit: cap.cap,
      api_key_id: principal.api_key_id,
      api_key_scopes: principal.scopes,
   }))
}
