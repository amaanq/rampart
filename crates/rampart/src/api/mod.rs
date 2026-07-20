//! JSON API under /api/v1. All responses are JSON. All mutations are
//! user-scoped via the `Principal` extension. Admin-only routes live
//! on a sub-router that's `.layer(admin_layer)`-gated so handlers
//! can't be added without the check.

use axum::{
   Router,
   middleware,
   routing::{
      delete,
      get,
      patch,
      post,
      put,
   },
};

use crate::{
   AppState,
   auth,
};

mod admin;
mod aliases;
mod api_keys;
mod contacts;
mod domains;
mod extension;
mod mailboxes;
mod shared;
mod user;
mod webauthn;

pub fn router() -> Router<AppState> {
   let admin_routes = Router::new()
      .route("/api/v1/admin/users", get(admin::admin_users_list))
      .route("/api/v1/admin/users", post(admin::admin_user_create))
      .route("/api/v1/admin/users/{id}", patch(admin::admin_user_patch))
      .route(
         "/api/v1/admin/users/{id}/enable",
         put(admin::admin_user_enable),
      )
      .route(
         "/api/v1/admin/users/{id}/disable",
         put(admin::admin_user_disable),
      )
      .route(
         "/api/v1/admin/domains/{id}/shared",
         put(admin::admin_domain_set_shared),
      )
      .layer(middleware::from_fn(auth::admin_layer));

   Router::new()
        .route("/api/v1/user/info", get(user::user_info))
        .route("/api/v1/user/api-keys", get(api_keys::api_keys_list))
        .route("/api/v1/user/api-keys", post(api_keys::api_key_create))
        .route(
            "/api/v1/user/api-keys/{id}",
            delete(api_keys::api_key_revoke),
        )
        .route(
            "/api/v1/api-key/self",
            delete(api_keys::api_key_revoke_self),
        )
        .route(
            "/api/v1/extension/bootstrap",
            get(extension::extension_bootstrap),
        )
        // aliases (user-scoped)
        .route("/api/v1/aliases", get(aliases::aliases_list))
        .route("/api/v1/aliases/{id}", get(aliases::alias_get))
        .route("/api/v1/aliases/{id}", patch(aliases::alias_patch))
        .route("/api/v1/aliases/{id}", delete(aliases::alias_delete))
        .route("/api/v1/aliases/{id}/toggle", put(aliases::alias_toggle))
        .route(
            "/api/v1/aliases/{id}/activities",
            get(aliases::alias_activities),
        )
        .route("/api/v1/alias/random", post(aliases::alias_random))
        .route("/api/v1/alias/prefix", post(aliases::alias_random))
        .route("/api/v1/alias/custom/new", post(aliases::alias_custom_new))
        // mailboxes (user-scoped)
        .route("/api/v1/mailboxes", get(mailboxes::mailboxes_list))
        .route("/api/v1/mailbox", post(mailboxes::mailbox_create))
        .route("/api/v1/mailbox/{id}", patch(mailboxes::mailbox_patch))
        .route("/api/v1/mailbox/{id}", delete(mailboxes::mailbox_delete))
        // domains (user sees own + shared; admin can promote via /admin)
        .route("/api/v1/domains", get(domains::domains_list))
        .route("/api/v1/domain", post(domains::domain_create))
        .route("/api/v1/domain/{id}", patch(domains::domain_patch))
        .route("/api/v1/domain/{id}", delete(domains::domain_delete))
        .route("/api/v1/domain/{id}/check", post(domains::domain_check))
        // user self-service
        .route("/api/v1/user/password", post(user::user_change_password))
        .route("/api/v1/user/email", post(user::user_start_email_change))
        .route(
            "/api/v1/mailbox/{id}/resend-verify",
            post(mailboxes::mailbox_resend_verify),
        )
        // webauthn (registration — authed)
        .route(
            "/api/v1/user/webauthn/register/start",
            post(webauthn::webauthn_register_start),
        )
        .route(
            "/api/v1/user/webauthn/register/finish",
            post(webauthn::webauthn_register_finish),
        )
        .route(
            "/api/v1/user/webauthn/credentials",
            get(webauthn::webauthn_list),
        )
        .route(
            "/api/v1/user/webauthn/credentials/{id}",
            delete(webauthn::webauthn_delete),
        )
        // contacts (reverse_contact rows under aliases)
        .route(
            "/api/v1/aliases/{id}/contacts",
            get(contacts::contacts_list),
        )
        .route("/api/v1/contacts/{id}", patch(contacts::contact_patch))
        .route("/api/v1/contacts/{id}", delete(contacts::contact_delete))
        .merge(admin_routes)
}
