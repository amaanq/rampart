use askama::Template;
use axum::{
    Form, Json, Router,
    extract::{Path, Query},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{delete, get, patch, post, put},
};
use serde::Deserialize;
use std::net::SocketAddr;
use time::Duration;
use time::OffsetDateTime;
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};

use crate::template_filters as filters;
use crate::web::{AliasRowView, DomainRowView, MailboxSummaryView};
use rampart_codegen::queries::{
    contacts as cq, domains as dq, email_log as elq, mailboxes as mbq, users as uq, webauthn as wq,
};

fn ts_ago(hours: i64) -> OffsetDateTime {
    OffsetDateTime::now_utc() - Duration::hours(hours)
}

fn mock_aliases() -> Vec<AliasRowView> {
    let mbox = MailboxSummaryView {
        id: 1,
        email: "dev@dev.local".into(),
    };
    let entries = [
        (
            "github@dev.local",
            true,
            false,
            Some("GitHub notifications"),
            142,
            3,
            5,
            Some(ts_ago(1)),
        ),
        (
            "shopping@dev.local",
            true,
            false,
            Some("Online shopping"),
            27,
            12,
            0,
            Some(ts_ago(24)),
        ),
        (
            "newsletter@dev.local",
            true,
            true,
            Some("Monthly newsletter"),
            8,
            0,
            0,
            Some(ts_ago(72)),
        ),
        (
            "social@dev.local",
            true,
            false,
            None,
            53,
            2,
            1,
            Some(ts_ago(5)),
        ),
        (
            "work@dev.local",
            true,
            false,
            Some("Work-related"),
            210,
            0,
            3,
            Some(ts_ago(2)),
        ),
        (
            "spamcatcher@dev.local",
            false,
            false,
            Some("Caught too much spam"),
            0,
            99,
            0,
            Some(ts_ago(168)),
        ),
        (
            "testing@dev.local",
            true,
            false,
            Some("Alias for testing"),
            4,
            0,
            0,
            None,
        ),
    ];
    entries
        .iter()
        .enumerate()
        .map(
            |(i, (addr, ena, pin, note, fwd, blk, rep, last))| AliasRowView {
                id: (i + 1) as i64,
                address: addr.to_string(),
                enabled: *ena,
                note: note.map(|s| s.to_string()),
                pinned: *pin,
                nb_forward: *fwd,
                nb_block: *blk,
                nb_reply: *rep,
                mailbox: mbox.clone(),
                domain: "dev.local".into(),
                last_email_at: *last,
            },
        )
        .collect()
}

fn mock_domains() -> Vec<DomainRowView> {
    let entries = [
        ("dev.local", false, true, "rnd", "ra+", 7),
        ("shared.example", true, false, "shared", "ra+", 3),
        ("admin.example", false, false, "adm", "ra+", 0),
    ];
    entries
        .iter()
        .enumerate()
        .map(|(i, (dom, shared, mine, rp, reply, n))| DomainRowView {
            id: (i + 1) as i64,
            domain: dom.to_string(),
            shared: *shared,
            mine: *mine,
            random_prefix: rp.to_string(),
            reply_prefix: reply.to_string(),
            nb_alias: *n,
        })
        .collect()
}

fn mock_mailboxes() -> Vec<mbq::MailboxRow> {
    let entries = [
        ("dev@dev.local", Some("Developer"), true, true, 7),
        ("dev+alt@dev.local", None, false, true, 0),
        (
            "dev+unverified@dev.local",
            Some("Unverified"),
            false,
            false,
            0,
        ),
    ];
    entries
        .iter()
        .enumerate()
        .map(
            |(i, (email, name, verified, enabled, nb))| mbq::MailboxRow {
                id: (i + 1) as i64,
                email: email.to_string(),
                display_name: name.map(|s| s.to_string()),
                verified: *verified,
                enabled: *enabled,
                created_at: ts_ago(24 * (i as i64 + 1)),
                nb_alias: *nb,
            },
        )
        .collect()
}

fn mock_passkeys() -> Vec<wq::ListForUser> {
    vec![
        wq::ListForUser {
            id: 1,
            name: "work yubikey".into(),
            created_at: ts_ago(720),
            last_used_at: Some(ts_ago(2)),
        },
        wq::ListForUser {
            id: 2,
            name: "personal phone".into(),
            created_at: ts_ago(168),
            last_used_at: None,
        },
    ]
}

fn mock_admin_users() -> Vec<uq::ListAdminCompact> {
    vec![
        uq::ListAdminCompact {
            id: 1,
            email: "dev@dev.local".into(),
            enabled: true,
            is_admin: true,
            created_at: ts_ago(720),
            nb_aliases: 7,
            nb_domains: 1,
        },
        uq::ListAdminCompact {
            id: 2,
            email: "friend@shared.example".into(),
            enabled: true,
            is_admin: false,
            created_at: ts_ago(480),
            nb_aliases: 3,
            nb_domains: 0,
        },
        uq::ListAdminCompact {
            id: 3,
            email: "disabled@example.com".into(),
            enabled: false,
            is_admin: false,
            created_at: ts_ago(240),
            nb_aliases: 0,
            nb_domains: 0,
        },
    ]
}

fn mock_admin_domains() -> Vec<dq::ListAdmin> {
    vec![
        dq::ListAdmin {
            id: 1,
            domain: "dev.local".into(),
            shared: false,
            owner_email: Some("dev@dev.local".into()),
            nb_alias: 7,
        },
        dq::ListAdmin {
            id: 2,
            domain: "shared.example".into(),
            shared: true,
            owner_email: Some("friend@shared.example".into()),
            nb_alias: 3,
        },
        dq::ListAdmin {
            id: 3,
            domain: "admin.example".into(),
            shared: false,
            owner_email: None,
            nb_alias: 0,
        },
    ]
}

fn mock_contacts() -> Vec<cq::ListForAlias> {
    vec![
        cq::ListForAlias {
            id: 1,
            real_email: "alice@example.com".into(),
            reply_address: "ra+tok@dev.local".into(),
            display_name: Some("Alice".into()),
            enabled: true,
            block_reply: false,
            last_seen_at: Some(ts_ago(2)),
            created_at: ts_ago(720),
        },
        cq::ListForAlias {
            id: 2,
            real_email: "bob@spam.net".into(),
            reply_address: "ra+tok@dev.local".into(),
            display_name: None,
            enabled: false,
            block_reply: true,
            last_seen_at: Some(ts_ago(48)),
            created_at: ts_ago(600),
        },
        cq::ListForAlias {
            id: 3,
            real_email: "charlie@company.org".into(),
            reply_address: "ra+tok@dev.local".into(),
            display_name: Some("Charlie".into()),
            enabled: true,
            block_reply: false,
            last_seen_at: Some(ts_ago(1)),
            created_at: ts_ago(480),
        },
    ]
}

fn mock_activities() -> Vec<elq::ActivityForAlias> {
    vec![
        elq::ActivityForAlias {
            action: "forward".into(),
            from_address: Some("alice@example.com".into()),
            reason: None,
            created_at: ts_ago(1),
        },
        elq::ActivityForAlias {
            action: "block".into(),
            from_address: Some("spammer@bad.net".into()),
            reason: Some("SPF fail".into()),
            created_at: ts_ago(2),
        },
        elq::ActivityForAlias {
            action: "forward".into(),
            from_address: Some("charlie@company.org".into()),
            reason: None,
            created_at: ts_ago(4),
        },
        elq::ActivityForAlias {
            action: "reply".into(),
            from_address: None,
            reason: Some("auto-reply sent".into()),
            created_at: ts_ago(6),
        },
    ]
}

fn render<T: Template>(t: &T) -> Response {
    match t.render() {
        Ok(body) => (StatusCode::OK, Html(body)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("template error: {e}"),
        )
            .into_response(),
    }
}

fn render_simple_message_page(
    heading: &str,
    message: &str,
    link_href: &str,
    link_label: &str,
) -> Response {
    #[derive(Template)]
    #[template(path = "simple_message.html")]
    struct SimpleMessage<'a> {
        heading: &'a str,
        message: &'a str,
        show_link: bool,
        link_href: &'a str,
        link_label: &'a str,
    }
    render(&SimpleMessage {
        heading,
        message,
        show_link: true,
        link_href,
        link_label,
    })
}

fn user_email() -> String {
    "dev@dev.local (preview)".into()
}

fn is_admin() -> bool {
    true
}

#[derive(Default, Deserialize)]
struct PreviewLoginQuery {
    #[serde(default)]
    next: String,
    #[serde(default)]
    reset: bool,
    #[serde(default)]
    email: String,
}

fn preview_login_destination(next: &str) -> &str {
    if next.starts_with('/')
        && !next.starts_with("//")
        && !next.contains('\\')
        && !next.chars().any(char::is_control)
    {
        next
    } else {
        "/"
    }
}

fn render_login_page(
    error: Option<&str>,
    next: &str,
    password_reset: bool,
    email: &str,
    focus_password: bool,
) -> Response {
    #[derive(Template)]
    #[template(path = "login.html")]
    struct LoginPage<'a> {
        error: Option<&'a str>,
        next: &'a str,
        password_reset: bool,
        email: &'a str,
        focus_password: bool,
    }
    render(&LoginPage {
        error,
        next,
        password_reset,
        email,
        focus_password,
    })
}

async fn login_page(Query(query): Query<PreviewLoginQuery>) -> Response {
    render_login_page(
        None,
        preview_login_destination(&query.next),
        query.reset,
        "",
        false,
    )
}

async fn login_post(Form(form): Form<PreviewLoginQuery>) -> Response {
    if form.email == "invalid@preview.test" {
        return render_login_page(
            Some("Email or password is incorrect."),
            preview_login_destination(&form.next),
            false,
            &form.email,
            true,
        );
    }
    Redirect::to(preview_login_destination(&form.next)).into_response()
}

fn render_signup_page(
    token: &str,
    error: Option<&str>,
    email: &str,
    display_name: &str,
) -> Response {
    #[derive(Template)]
    #[template(path = "signup.html")]
    struct SignupPage<'a> {
        token: &'a str,
        error: Option<&'a str>,
        email: &'a str,
        display_name: &'a str,
    }
    render(&SignupPage {
        token,
        error,
        email,
        display_name,
    })
}

async fn signup_page(Path(token): Path<String>) -> Response {
    render_signup_page(&token, None, "", "")
}

#[derive(Deserialize)]
struct PreviewSignupForm {
    email: String,
    #[serde(default)]
    display_name: Option<String>,
}

async fn signup_post(Path(token): Path<String>, Form(form): Form<PreviewSignupForm>) -> Response {
    let error = match token.as_str() {
        "expired" => "This invitation has expired. Ask an administrator for a new one.",
        "used" => "This invitation has already been used.",
        "email-mismatch" => "This invitation is tied to a different email address.",
        _ => "This invitation isn’t valid.",
    };
    render_signup_page(
        &token,
        Some(error),
        &form.email,
        form.display_name.as_deref().unwrap_or(""),
    )
}

fn render_forgot_page(sent: bool, error: Option<&str>, email: &str) -> Response {
    #[derive(Template)]
    #[template(path = "forgot.html")]
    struct ForgotPage<'a> {
        sent: bool,
        error: Option<&'a str>,
        email: &'a str,
    }
    render(&ForgotPage { sent, error, email })
}

async fn forgot_page() -> Response {
    render_forgot_page(false, None, "")
}

#[derive(Deserialize)]
struct PreviewForgotForm {
    email: String,
}

async fn forgot_post(Form(form): Form<PreviewForgotForm>) -> Response {
    if form.email == "rate@limited.test" {
        return render_forgot_page(
            false,
            Some("Too many reset requests. Try again later."),
            &form.email,
        );
    }
    if form.email == "slow@preview.test" {
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    }
    render_forgot_page(true, None, "")
}

fn render_reset_page(token: &str, error: Option<&str>) -> Response {
    #[derive(Template)]
    #[template(path = "reset.html")]
    struct ResetPage<'a> {
        token: &'a str,
        error: Option<&'a str>,
    }
    render(&ResetPage { token, error })
}

async fn reset_page(Path(token): Path<String>) -> Response {
    render_reset_page(&token, None)
}

async fn reset_post(Path(token): Path<String>) -> Response {
    if token == "success" {
        return Redirect::to("/login?reset=true").into_response();
    }
    let (heading, message) = match token.as_str() {
        "expired" => (
            "Reset link expired",
            "This password reset link has expired. Request a new one to continue.",
        ),
        "used" => (
            "Reset link already used",
            "This password reset link has already been used. Request a new one if you still need to change your password.",
        ),
        _ => (
            "Reset link isn’t valid",
            "Check that you opened the complete link from your password reset email.",
        ),
    };
    render_simple_message_page(heading, message, "/auth/forgot", "Request a new reset link")
}

async fn confirm_page(token: String, which: &str) -> Response {
    #[derive(Template)]
    #[template(path = "confirm.html")]
    struct ConfirmPage<'a> {
        title: &'a str,
        body: &'a str,
        action: &'a str,
        button_label: &'a str,
        pending_label: &'a str,
    }
    let (title, body, action, button_label, pending_label) = match which {
        "change-email" => (
            "Confirm email change",
            "Change your rampart account email to the address this message was sent to.",
            format!("/auth/change-email/{token}"),
            "Change email",
            "Changing email…",
        ),
        "verify" => (
            "Verify mailbox",
            "Confirm that you own the mailbox this message was sent to.",
            format!("/mailbox/verify/{token}"),
            "Verify mailbox",
            "Verifying mailbox…",
        ),
        _ => (
            "Confirm action",
            "Confirm this action.",
            format!("/auth/{which}/{token}"),
            "Confirm",
            "Confirming…",
        ),
    };
    render(&ConfirmPage {
        title,
        body,
        action: &action,
        button_label,
        pending_label,
    })
}

async fn change_email_page(Path(token): Path<String>) -> Response {
    confirm_page(token, "change-email").await
}

async fn change_email_post(Path(token): Path<String>) -> Response {
    let (heading, message) = match token.as_str() {
        "invalid" => (
            "Email change link isn’t valid",
            "Check that you opened the complete link from your email.",
        ),
        "expired" => (
            "Email change link expired",
            "This email change link has expired. Start the change again from settings.",
        ),
        "used" => (
            "Email change link already used",
            "This email change link has already been used. Check your current address in settings.",
        ),
        "email-in-use" => (
            "Email already in use",
            "Another account already uses this email address. Choose a different address in settings.",
        ),
        _ => (
            "Email changed",
            "Your rampart sign-in email has been updated.",
        ),
    };
    render_simple_message_page(heading, message, "/settings", "Go to settings")
}

async fn mailbox_verify_page(Path(token): Path<String>) -> Response {
    confirm_page(token, "verify").await
}

async fn mailbox_verify_post(Path(token): Path<String>) -> Response {
    let (heading, message) = match token.as_str() {
        "invalid" => (
            "Verification link isn’t valid",
            "Check that you opened the complete link from your email.",
        ),
        "expired" => (
            "Verification link expired",
            "This mailbox verification link has expired. Send a new one from mailboxes.",
        ),
        "used" => (
            "Verification link already used",
            "This link has already been used. Check the mailbox status in rampart.",
        ),
        _ => (
            "Mailbox verified",
            "This mailbox is ready to use with rampart.",
        ),
    };
    render_simple_message_page(heading, message, "/mailboxes", "Go to mailboxes")
}

fn render_setup_page(
    error: Option<&str>,
    email: &str,
    display_name: &str,
    focus_password: bool,
) -> Response {
    #[derive(Template)]
    #[template(path = "setup.html")]
    struct SetupPage<'a> {
        error: Option<&'a str>,
        csrf_token: &'a str,
        email: &'a str,
        display_name: &'a str,
        focus_password: bool,
    }
    render(&SetupPage {
        error,
        csrf_token: "preview-mode",
        email,
        display_name,
        focus_password,
    })
}

async fn setup_page() -> Response {
    render_setup_page(None, "", "", false)
}

#[derive(Deserialize)]
struct PreviewSetupForm {
    email: String,
    #[serde(default)]
    display_name: Option<String>,
}

async fn setup_post(Form(form): Form<PreviewSetupForm>) -> Response {
    render_setup_page(
        Some("This setup page expired. Review the details and try again."),
        &form.email,
        form.display_name.as_deref().unwrap_or(""),
        true,
    )
}

#[derive(Default, Deserialize)]
struct PreviewAliasesQuery {
    #[serde(default)]
    empty: bool,
}

async fn aliases_page(Query(query): Query<PreviewAliasesQuery>) -> Response {
    #[derive(Template)]
    #[template(path = "aliases.html")]
    struct Page {
        aliases: Vec<AliasRowView>,
        domains: Vec<DomainRowView>,
        total: i64,
        user_email: String,
        is_admin: bool,
    }
    let aliases = if query.empty { vec![] } else { mock_aliases() };
    let total = aliases.len() as i64;
    render(&Page {
        aliases,
        domains: if query.empty { vec![] } else { mock_domains() },
        total,
        user_email: user_email(),
        is_admin: is_admin(),
    })
}

async fn mailboxes_page() -> Response {
    #[derive(Template)]
    #[template(path = "mailboxes.html")]
    struct Page {
        mailboxes: Vec<mbq::MailboxRow>,
        user_email: String,
        is_admin: bool,
    }
    render(&Page {
        mailboxes: mock_mailboxes(),
        user_email: user_email(),
        is_admin: is_admin(),
    })
}

#[derive(Default, Deserialize)]
struct PreviewDomainsQuery {
    #[serde(default)]
    empty: bool,
}

async fn domains_page(Query(query): Query<PreviewDomainsQuery>) -> Response {
    #[derive(Template)]
    #[template(path = "domains.html")]
    struct Page {
        domains: Vec<DomainRowView>,
        user_email: String,
        is_admin: bool,
    }
    render(&Page {
        domains: if query.empty { vec![] } else { mock_domains() },
        user_email: user_email(),
        is_admin: is_admin(),
    })
}

async fn settings_page() -> Response {
    #[derive(Template)]
    #[template(path = "settings.html")]
    struct Page {
        user_email: String,
        is_admin: bool,
        passkeys: Vec<wq::ListForUser>,
    }
    render(&Page {
        user_email: user_email(),
        is_admin: is_admin(),
        passkeys: mock_passkeys(),
    })
}

async fn contacts_page(Path(_alias_id): Path<i64>) -> Response {
    #[derive(Template)]
    #[template(path = "contacts.html")]
    struct Page {
        alias_address: String,
        contacts: Vec<cq::ListForAlias>,
        user_email: String,
        is_admin: bool,
    }
    render(&Page {
        alias_address: "github@dev.local".into(),
        contacts: mock_contacts(),
        user_email: user_email(),
        is_admin: is_admin(),
    })
}

async fn activity_page(Path(_alias_id): Path<i64>) -> Response {
    #[derive(Template)]
    #[template(path = "activity.html")]
    struct Page {
        alias_address: String,
        activities: Vec<elq::ActivityForAlias>,
        page: i64,
        has_next: bool,
        user_email: String,
        is_admin: bool,
    }
    render(&Page {
        alias_address: "github@dev.local".into(),
        activities: mock_activities(),
        page: 0,
        has_next: false,
        user_email: user_email(),
        is_admin: is_admin(),
    })
}

async fn admin_users_page() -> Response {
    #[derive(Template)]
    #[template(path = "admin_users.html")]
    struct Page {
        user_email: String,
        #[allow(dead_code)]
        is_admin: bool,
        users: Vec<uq::ListAdminCompact>,
    }
    render(&Page {
        user_email: user_email(),
        is_admin: true,
        users: mock_admin_users(),
    })
}

async fn admin_domains_page() -> Response {
    #[derive(Template)]
    #[template(path = "admin_domains.html")]
    struct Page {
        user_email: String,
        #[allow(dead_code)]
        is_admin: bool,
        domains: Vec<dq::ListAdmin>,
    }
    render(&Page {
        user_email: user_email(),
        is_admin: true,
        domains: mock_admin_domains(),
    })
}

async fn healthz() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct PreviewPasswordChange {
    current_password: String,
}

async fn password_change(Json(body): Json<PreviewPasswordChange>) -> Response {
    if body.current_password == "wrong" {
        return (StatusCode::BAD_REQUEST, "Current password is incorrect.").into_response();
    }
    StatusCode::NO_CONTENT.into_response()
}

#[derive(Deserialize)]
struct PreviewEmailChange {
    new_email: String,
}

async fn email_change(Json(body): Json<PreviewEmailChange>) -> Response {
    if body.new_email == "taken@example.com" {
        return (
            StatusCode::CONFLICT,
            "An account already uses this email address.",
        )
            .into_response();
    }
    use std::str::FromStr;
    if lettre::Address::from_str(&body.new_email).is_err() {
        return (StatusCode::BAD_REQUEST, "Enter a valid email address.").into_response();
    }
    StatusCode::ACCEPTED.into_response()
}

async fn verification_sent() -> StatusCode {
    StatusCode::ACCEPTED
}

async fn deleted() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn updated() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn unauthorized() -> (StatusCode, &'static str) {
    (StatusCode::UNAUTHORIZED, "401 unauthorized")
}

pub async fn serve(listen: SocketAddr, static_dir: String) -> anyhow::Result<()> {
    let static_files = ServiceBuilder::new()
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache"),
        ))
        .service(ServeDir::new(static_dir));
    let app = Router::new()
        .route("/login", get(login_page).post(login_post))
        .route("/signup/{token}", get(signup_page).post(signup_post))
        .route("/auth/forgot", get(forgot_page).post(forgot_post))
        .route("/auth/reset/{token}", get(reset_page).post(reset_post))
        .route(
            "/auth/change-email/{token}",
            get(change_email_page).post(change_email_post),
        )
        .route(
            "/mailbox/verify/{token}",
            get(mailbox_verify_page).post(mailbox_verify_post),
        )
        .route("/setup", get(setup_page).post(setup_post))
        .route("/", get(aliases_page))
        .route("/mailboxes", get(mailboxes_page))
        .route("/domains", get(domains_page))
        .route("/settings", get(settings_page))
        .route("/aliases/{id}/contacts", get(contacts_page))
        .route("/aliases/{id}/activity", get(activity_page))
        .route("/admin/users", get(admin_users_page))
        .route("/admin/domains", get(admin_domains_page))
        .route("/api/v1/user/password", post(password_change))
        .route("/api/v1/user/email", post(email_change))
        .route("/api/v1/aliases/{id}", delete(deleted))
        .route("/api/v1/aliases/{id}/toggle", put(updated))
        .route("/api/v1/mailbox/{id}", patch(updated))
        .route("/api/v1/mailbox/{id}", delete(deleted))
        .route(
            "/api/v1/mailbox/{id}/resend-verify",
            post(verification_sent),
        )
        .route("/api/v1/contacts/{id}", patch(updated))
        .route("/api/v1/contacts/{id}", delete(deleted))
        .route("/api/v1/user/webauthn/credentials/{id}", delete(deleted))
        .route("/api/v1/admin/domains/{id}/shared", put(updated))
        .route("/api/v1/admin/users/{id}/enable", put(unauthorized))
        .route("/healthz", get(healthz))
        .nest_service("/static", static_files);

    let listener = tokio::net::TcpListener::bind(listen).await?;
    tracing::info!(addr = %listen, "preview server listening");
    tracing::warn!("PREVIEW MODE -- no auth, no DB, all data is fake");

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;

    Ok(())
}
