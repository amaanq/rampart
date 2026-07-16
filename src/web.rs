//! Dashboard routes. Askama templates, user-scoped like the API.

use askama::Template;
use axum::{
    Extension, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rampart_codegen::queries::{aliases, contacts, domains, email_log, mailboxes, users, webauthn};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::AppState;
use crate::auth::{self, AdminPrincipal, Principal};
use crate::error::{ApiError, ApiResult};
use crate::template_filters as filters;

pub fn router() -> Router<AppState> {
    let admin_routes = Router::new()
        .route("/admin/users", get(admin_users_page))
        .route("/admin/domains", get(admin_domains_page))
        .layer(middleware::from_fn(auth::admin_layer));

    Router::new()
        .route("/", get(aliases_page))
        .route("/mailboxes", get(mailboxes_page))
        .route("/domains", get(domains_page))
        .route("/settings", get(settings_page))
        .route("/aliases/{id}/contacts", get(contacts_page))
        .route("/aliases/{id}/activity", get(activity_page))
        .merge(admin_routes)
}

#[derive(Serialize, Clone)]
pub struct AliasRowView {
    pub id: i64,
    pub address: String,
    pub enabled: bool,
    pub note: Option<String>,
    pub pinned: bool,
    pub nb_forward: i64,
    pub nb_block: i64,
    pub nb_reply: i64,
    pub mailbox: MailboxSummaryView,
    pub domain: String,
    pub last_email_at: Option<OffsetDateTime>,
}

#[derive(Serialize, Clone)]
pub struct MailboxSummaryView {
    pub id: i64,
    pub email: String,
}

#[derive(Serialize, Clone)]
pub struct DomainRowView {
    pub id: i64,
    pub domain: String,
    pub shared: bool,
    pub mine: bool,
    pub random_prefix: String,
    pub reply_prefix: String,
    pub nb_alias: i64,
}

pub type MailboxRowView = mailboxes::MailboxRow;
pub type PasskeyRowView = webauthn::ListForUser;
pub type AdminUserRowView = users::ListAdminCompact;
pub type AdminDomainRowView = domains::ListAdmin;
pub type ContactRowView = contacts::ListForAlias;
pub type ActivityRowView = email_log::ActivityForAlias;

#[derive(Template)]
#[template(path = "aliases.html")]
struct AliasesPage {
    aliases: Vec<AliasRowView>,
    domains: Vec<DomainRowView>,
    total: i64,
    user_email: String,
    is_admin: bool,
}

async fn aliases_page(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let aliases: Vec<AliasRowView> = aliases::list_for_dashboard()
        .bind(&c, &p.user_id)
        .all()
        .await?
        .into_iter()
        .map(|row| AliasRowView {
            id: row.id,
            address: row.address,
            enabled: row.enabled,
            note: row.note,
            pinned: row.pinned,
            nb_forward: row.nb_forward,
            nb_block: row.nb_block,
            nb_reply: row.nb_reply,
            mailbox: MailboxSummaryView {
                id: row.mailbox_id,
                email: row.mailbox_email,
            },
            domain: row.domain,
            last_email_at: row.last_email_at,
        })
        .collect();
    let total = aliases.len() as i64;

    let domains: Vec<DomainRowView> = domains::list_for_dashboard()
        .bind(&c, &p.user_id, &p.is_admin)
        .all()
        .await?
        .into_iter()
        .map(|row| DomainRowView {
            id: row.id,
            domain: row.domain,
            shared: row.shared,
            mine: row.owner_id == Some(p.user_id),
            random_prefix: row.random_prefix,
            reply_prefix: row.reply_prefix,
            nb_alias: row.nb_alias,
        })
        .collect();

    let user_email = lookup_user_email(&c, p.user_id).await?;

    Ok(render(&AliasesPage {
        aliases,
        domains,
        total,
        user_email,
        is_admin: p.is_admin,
    })?)
}

#[derive(Template)]
#[template(path = "mailboxes.html")]
struct MailboxesPage {
    mailboxes: Vec<MailboxRowView>,
    user_email: String,
    is_admin: bool,
}

async fn mailboxes_page(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let mailboxes = mailboxes::list_for_user()
        .bind(&c, &p.user_id)
        .all()
        .await?;
    let user_email = lookup_user_email(&c, p.user_id).await?;
    Ok(render(&MailboxesPage {
        mailboxes,
        user_email,
        is_admin: p.is_admin,
    })?)
}

#[derive(Template)]
#[template(path = "domains.html")]
struct DomainsPage {
    domains: Vec<DomainRowView>,
    user_email: String,
    is_admin: bool,
}

async fn domains_page(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let domains: Vec<DomainRowView> = domains::list_for_dashboard()
        .bind(&c, &p.user_id, &p.is_admin)
        .all()
        .await?
        .into_iter()
        .map(|row| DomainRowView {
            id: row.id,
            domain: row.domain,
            shared: row.shared,
            mine: row.owner_id == Some(p.user_id),
            random_prefix: row.random_prefix,
            reply_prefix: row.reply_prefix,
            nb_alias: row.nb_alias,
        })
        .collect();
    let user_email = lookup_user_email(&c, p.user_id).await?;
    Ok(render(&DomainsPage {
        domains,
        user_email,
        is_admin: p.is_admin,
    })?)
}

fn render<T: Template>(t: &T) -> Result<Response, crate::error::ApiError> {
    let body = t.render()?;
    Ok((StatusCode::OK, Html(body)).into_response())
}

async fn lookup_user_email(c: &deadpool_postgres::Client, user_id: i64) -> ApiResult<String> {
    Ok(users::email_by_id().bind(c, &user_id).one().await?)
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsPage {
    user_email: String,
    is_admin: bool,
    passkeys: Vec<PasskeyRowView>,
}

async fn settings_page(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let user_email = lookup_user_email(&c, p.user_id).await?;
    let passkeys = webauthn::list_for_user().bind(&c, &p.user_id).all().await?;
    Ok(render(&SettingsPage {
        user_email,
        is_admin: p.is_admin,
        passkeys,
    })?)
}

#[derive(Template)]
#[template(path = "admin_users.html")]
struct AdminUsersPage {
    user_email: String,
    /// Read by askama in `{% if is_admin %}` template blocks; rustc
    /// can't see through the macro so the field looks dead-coded.
    #[allow(dead_code)]
    is_admin: bool,
    users: Vec<AdminUserRowView>,
}

async fn admin_users_page(
    State(state): State<AppState>,
    AdminPrincipal(p): AdminPrincipal,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let user_email = lookup_user_email(&c, p.user_id).await?;
    let users = users::list_admin_compact().bind(&c).all().await?;
    Ok(render(&AdminUsersPage {
        user_email,
        is_admin: p.is_admin,
        users,
    })?)
}

#[derive(Template)]
#[template(path = "admin_domains.html")]
struct AdminDomainsPage {
    user_email: String,
    /// Read by askama (see AdminUsersPage::is_admin).
    #[allow(dead_code)]
    is_admin: bool,
    domains: Vec<AdminDomainRowView>,
}

#[derive(Template)]
#[template(path = "contacts.html")]
struct ContactsPage {
    alias_address: String,
    contacts: Vec<ContactRowView>,
    user_email: String,
    is_admin: bool,
}

async fn contacts_page(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(alias_id): Path<i64>,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let alias_address = aliases::address_for_user()
        .bind(&c, &alias_id, &p.user_id)
        .opt()
        .await?
        .ok_or(ApiError::NotFound)?;
    let contacts = contacts::list_for_alias().bind(&c, &alias_id).all().await?;
    let user_email = lookup_user_email(&c, p.user_id).await?;
    Ok(render(&ContactsPage {
        alias_address,
        contacts,
        user_email,
        is_admin: p.is_admin,
    })?)
}

#[derive(Deserialize)]
struct ActivityQuery {
    page: Option<i64>,
}

#[derive(Template)]
#[template(path = "activity.html")]
struct ActivityPage {
    alias_address: String,
    activities: Vec<ActivityRowView>,
    page: i64,
    has_next: bool,
    user_email: String,
    is_admin: bool,
}

const ACTIVITY_PAGE_SIZE: i64 = 50;

async fn activity_page(
    State(state): State<AppState>,
    Extension(p): Extension<Principal>,
    Path(alias_id): Path<i64>,
    Query(q): Query<ActivityQuery>,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let alias_address = aliases::address_for_user()
        .bind(&c, &alias_id, &p.user_id)
        .opt()
        .await?
        .ok_or(ApiError::NotFound)?;
    let page = q.page.unwrap_or(0).max(0);
    let activities = email_log::activity_for_alias()
        .bind(
            &c,
            &alias_id,
            &(ACTIVITY_PAGE_SIZE + 1),
            &(page * ACTIVITY_PAGE_SIZE),
        )
        .all()
        .await?;
    let has_next = activities.len() as i64 > ACTIVITY_PAGE_SIZE;
    let activities: Vec<ActivityRowView> = activities
        .into_iter()
        .take(ACTIVITY_PAGE_SIZE as usize)
        .collect();
    let user_email = lookup_user_email(&c, p.user_id).await?;
    Ok(render(&ActivityPage {
        alias_address,
        activities,
        page,
        has_next,
        user_email,
        is_admin: p.is_admin,
    })?)
}

async fn admin_domains_page(
    State(state): State<AppState>,
    AdminPrincipal(p): AdminPrincipal,
) -> ApiResult<Response> {
    let c = state.pool.get().await?;
    let user_email = lookup_user_email(&c, p.user_id).await?;
    let domains = domains::list_admin().bind(&c).all().await?;
    Ok(render(&AdminDomainsPage {
        user_email,
        is_admin: p.is_admin,
        domains,
    })?)
}
