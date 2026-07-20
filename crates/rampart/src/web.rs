//! Dashboard routes. Askama templates, user-scoped like the API.

use askama::Template;
use axum::{
   Extension,
   Router,
   extract::{
      Path,
      Query,
      State,
   },
   http::StatusCode,
   middleware,
   response::{
      Html,
      IntoResponse as _,
      Response,
   },
   routing,
};
use rampart_codegen::queries::{
   aliases,
   api_keys,
   contacts,
   domains,
   email_log,
   mailboxes,
   users,
   webauthn,
};
use serde::{
   Deserialize,
   Serialize,
};
use time::OffsetDateTime;

use crate::{
   AppState,
   auth::{
      self,
      AdminPrincipal,
      Principal,
   },
   domain_setup::{
      self,
      DomainSetup,
   },
   error::{
      ApiError,
      ApiResult,
   },
   template_filters as filters,
};

pub fn router() -> Router<AppState> {
   let admin_routes = Router::new()
      .route("/admin/users", routing::get(admin_users_page))
      .route("/admin/domains", routing::get(admin_domains_page))
      .layer(middleware::from_fn(auth::admin_layer));

   Router::new()
      .route("/", routing::get(aliases_page))
      .route("/mailboxes", routing::get(mailboxes_page))
      .route("/domains", routing::get(domains_page))
      .route("/domains/{id}", routing::get(domain_setup_page))
      .route("/settings", routing::get(settings_page))
      .route("/aliases/{id}/contacts", routing::get(contacts_page))
      .route("/aliases/{id}/activity", routing::get(activity_page))
      .merge(admin_routes)
}

#[derive(Serialize, Clone)]
pub struct AliasRowView {
   pub id:            i64,
   pub address:       String,
   pub enabled:       bool,
   pub note:          Option<String>,
   pub pinned:        bool,
   pub nb_forward:    i64,
   pub nb_block:      i64,
   pub nb_reply:      i64,
   pub mailbox:       MailboxSummaryView,
   pub domain:        String,
   pub last_email_at: Option<OffsetDateTime>,
}

#[derive(Serialize, Clone)]
pub struct MailboxSummaryView {
   pub id:    i64,
   pub email: String,
}

#[derive(Serialize, Clone)]
pub struct DomainRowView {
   pub id:            i64,
   pub domain:        String,
   pub shared:        bool,
   pub mine:          bool,
   pub random_prefix: String,
   pub reply_prefix:  String,
   pub nb_alias:      i64,
   pub setup_state:   String,
}

pub type MailboxRowView = mailboxes::MailboxRow;
pub type PasskeyRowView = webauthn::ListForUser;
pub type ApiKeyRowView = api_keys::ApiKeyRow;
pub type AdminUserRowView = users::ListAdminCompact;
pub type AdminDomainRowView = domains::ListAdmin;
pub type ContactRowView = contacts::ListForAlias;
pub type ActivityRowView = email_log::ActivityForAlias;

#[derive(Template)]
#[template(path = "aliases.html")]
struct AliasesPage {
   aliases:              Vec<AliasRowView>,
   domains:              Vec<DomainRowView>,
   has_verified_mailbox: bool,
   total:                i64,
   user_email:           String,
   is_admin:             bool,
}

async fn aliases_page(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let aliases: Vec<AliasRowView> = aliases::list_for_dashboard()
      .bind(&conn, &principal.user_id)
      .all()
      .await?
      .into_iter()
      .map(|row| AliasRowView {
         id:            row.id,
         address:       row.address,
         enabled:       row.enabled,
         note:          row.note,
         pinned:        row.pinned,
         nb_forward:    row.nb_forward,
         nb_block:      row.nb_block,
         nb_reply:      row.nb_reply,
         mailbox:       MailboxSummaryView {
            id:    row.mailbox_id,
            email: row.mailbox_email,
         },
         domain:        row.domain,
         last_email_at: row.last_email_at,
      })
      .collect();
   let total = i64::try_from(aliases.len()).expect("alias count fits in i64");

   let domains: Vec<DomainRowView> = domains::list_for_dashboard()
      .bind(&conn, &principal.user_id, &principal.is_admin)
      .all()
      .await?
      .into_iter()
      .map(|row| DomainRowView {
         id:            row.id,
         domain:        row.domain.clone(),
         shared:        row.shared,
         mine:          row.owner_id == Some(principal.user_id),
         random_prefix: row.random_prefix,
         reply_prefix:  row.reply_prefix,
         nb_alias:      row.nb_alias,
         setup_state:   domain_setup::build(
            row.id,
            &row.domain,
            &state.config.public_mx_hostname,
            &domain_setup::parse_dkim_records(&row.dkim_records),
            &domain_setup::parse_dns_status(&row.dns_status),
            row.dns_checked_at,
            row.dns_verified_at,
         )
         .state_label
         .to_owned(),
      })
      .collect();
   let has_verified_mailbox = mailboxes::first_verified_for_user()
      .bind(&conn, &principal.user_id)
      .opt()
      .await?
      .is_some();

   let user_email = lookup_user_email(&conn, principal.user_id).await?;

   render(&AliasesPage {
      aliases,
      domains,
      has_verified_mailbox,
      total,
      user_email,
      is_admin: principal.is_admin,
   })
}

#[derive(Template)]
#[template(path = "mailboxes.html")]
struct MailboxesPage {
   mailboxes:  Vec<MailboxRowView>,
   user_email: String,
   is_admin:   bool,
}

async fn mailboxes_page(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let mailboxes = mailboxes::list_for_user()
      .bind(&conn, &principal.user_id)
      .all()
      .await?;
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   render(&MailboxesPage {
      mailboxes,
      user_email,
      is_admin: principal.is_admin,
   })
}

#[derive(Template)]
#[template(path = "domains.html")]
struct DomainsPage {
   domains:    Vec<DomainRowView>,
   user_email: String,
   is_admin:   bool,
}

async fn domains_page(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let domains: Vec<DomainRowView> = domains::list_for_dashboard()
      .bind(&conn, &principal.user_id, &principal.is_admin)
      .all()
      .await?
      .into_iter()
      .map(|row| DomainRowView {
         id:            row.id,
         domain:        row.domain.clone(),
         shared:        row.shared,
         mine:          row.owner_id == Some(principal.user_id),
         random_prefix: row.random_prefix,
         reply_prefix:  row.reply_prefix,
         nb_alias:      row.nb_alias,
         setup_state:   domain_setup::build(
            row.id,
            &row.domain,
            &state.config.public_mx_hostname,
            &domain_setup::parse_dkim_records(&row.dkim_records),
            &domain_setup::parse_dns_status(&row.dns_status),
            row.dns_checked_at,
            row.dns_verified_at,
         )
         .state_label
         .to_owned(),
      })
      .collect();
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   render(&DomainsPage {
      domains,
      user_email,
      is_admin: principal.is_admin,
   })
}

#[derive(Template)]
#[template(path = "domain_setup.html")]
struct DomainSetupPage {
   setup:      DomainSetup,
   user_email: String,
   is_admin:   bool,
}

async fn domain_setup_page(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(domain_id): Path<i64>,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let row = domains::by_id_for_user()
      .bind(&conn, &domain_id, &principal.user_id, &principal.is_admin)
      .opt()
      .await?
      .ok_or(ApiError::NotFound)?;
   let setup = domain_setup::build(
      row.id,
      &row.domain,
      &state.config.public_mx_hostname,
      &domain_setup::parse_dkim_records(&row.dkim_records),
      &domain_setup::parse_dns_status(&row.dns_status),
      row.dns_checked_at,
      row.dns_verified_at,
   );
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   render(&DomainSetupPage {
      setup,
      user_email,
      is_admin: principal.is_admin,
   })
}

fn render<T>(template: &T) -> Result<Response, ApiError>
where
   T: Template,
{
   let body = template.render()?;
   Ok((StatusCode::OK, Html(body)).into_response())
}

async fn lookup_user_email(conn: &deadpool_postgres::Client, user_id: i64) -> ApiResult<String> {
   Ok(users::email_by_id().bind(conn, &user_id).one().await?)
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsPage {
   user_email: String,
   is_admin:   bool,
   passkeys:   Vec<PasskeyRowView>,
   api_keys:   Vec<ApiKeyRowView>,
}

async fn settings_page(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   let passkeys = webauthn::list_for_user()
      .bind(&conn, &principal.user_id)
      .all()
      .await?;
   let api_keys = api_keys::list_for_user()
      .bind(&conn, &principal.user_id)
      .all()
      .await?;
   render(&SettingsPage {
      user_email,
      is_admin: principal.is_admin,
      passkeys,
      api_keys,
   })
}

#[derive(Template)]
#[template(path = "admin_users.html")]
struct AdminUsersPage {
   user_email:      String,
   /// Read only through the inherited `layout.html` `{% if is_admin %}` block.
   is_admin:        bool,
   current_user_id: i64,
   users:           Vec<AdminUserRowView>,
}

async fn admin_users_page(
   State(state): State<AppState>,
   AdminPrincipal(principal): AdminPrincipal,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   let users = users::list_admin_compact().bind(&conn).all().await?;
   render(&AdminUsersPage {
      user_email,
      is_admin: principal.is_admin,
      current_user_id: principal.user_id,
      users,
   })
}

#[derive(Template)]
#[template(path = "admin_domains.html")]
struct AdminDomainsPage {
   user_email: String,
   is_admin:   bool,
   domains:    Vec<AdminDomainRowView>,
}

#[derive(Template)]
#[template(path = "contacts.html")]
struct ContactsPage {
   alias_address: String,
   contacts:      Vec<ContactRowView>,
   user_email:    String,
   is_admin:      bool,
}

async fn contacts_page(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(alias_id): Path<i64>,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let alias_address = aliases::address_for_user()
      .bind(&conn, &alias_id, &principal.user_id)
      .opt()
      .await?
      .ok_or(ApiError::NotFound)?;
   let contacts = contacts::list_for_alias()
      .bind(&conn, &alias_id)
      .all()
      .await?;
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   render(&ContactsPage {
      alias_address,
      contacts,
      user_email,
      is_admin: principal.is_admin,
   })
}

#[derive(Deserialize)]
struct ActivityQuery {
   page: Option<i64>,
}

#[derive(Template)]
#[template(path = "activity.html")]
struct ActivityPage {
   alias_address: String,
   activities:    Vec<ActivityRowView>,
   page:          i64,
   has_next:      bool,
   user_email:    String,
   is_admin:      bool,
}

const ACTIVITY_PAGE_SIZE: i64 = 50;

async fn activity_page(
   State(state): State<AppState>,
   Extension(principal): Extension<Principal>,
   Path(alias_id): Path<i64>,
   Query(query): Query<ActivityQuery>,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let alias_address = aliases::address_for_user()
      .bind(&conn, &alias_id, &principal.user_id)
      .opt()
      .await?
      .ok_or(ApiError::NotFound)?;
   let page = query.page.unwrap_or(0).max(0);
   let activities = email_log::activity_for_alias()
      .bind(
         &conn,
         &alias_id,
         &(ACTIVITY_PAGE_SIZE + 1),
         &(page * ACTIVITY_PAGE_SIZE),
      )
      .all()
      .await?;
   let has_next =
      i64::try_from(activities.len()).expect("activity count fits in i64") > ACTIVITY_PAGE_SIZE;
   let activities: Vec<ActivityRowView> = activities
      .into_iter()
      .take(usize::try_from(ACTIVITY_PAGE_SIZE).expect("page size fits in usize"))
      .collect();
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   render(&ActivityPage {
      alias_address,
      activities,
      page,
      has_next,
      user_email,
      is_admin: principal.is_admin,
   })
}

async fn admin_domains_page(
   State(state): State<AppState>,
   AdminPrincipal(principal): AdminPrincipal,
) -> ApiResult<Response> {
   let conn = state.pool.get().await?;
   let user_email = lookup_user_email(&conn, principal.user_id).await?;
   let domains = domains::list_admin().bind(&conn).all().await?;
   render(&AdminDomainsPage {
      user_email,
      is_admin: principal.is_admin,
      domains,
   })
}
