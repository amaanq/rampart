use askama::Template;
use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{delete, get, patch, post, put},
};
use std::net::SocketAddr;
use time::Duration;
use time::OffsetDateTime;
use tower_http::services::ServeDir;

use crate::template_filters as filters;
use crate::web::{AliasRowView, DomainRowView, MailboxSummaryView};
use rampart_codegen::queries::{
    contacts as cq, domains as dq, email_log as elq, mailboxes as mbq, users as uq, webauthn as wq,
};

fn ts() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

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

fn user_email() -> String {
    "dev@dev.local (preview)".into()
}

fn is_admin() -> bool {
    true
}

async fn login_page() -> Response {
    #[derive(Template)]
    #[template(path = "login.html")]
    struct LoginPage<'a> {
        error: Option<&'a str>,
    }
    render(&LoginPage { error: None })
}

async fn signup_page(Path(token): Path<String>) -> Response {
    #[derive(Template)]
    #[template(path = "signup.html")]
    struct SignupPage<'a> {
        token: &'a str,
        error: Option<&'a str>,
    }
    let t = SignupPage {
        token: &token,
        error: None,
    };
    render(&t)
}

async fn forgot_page() -> Response {
    #[derive(Template)]
    #[template(path = "forgot.html")]
    struct ForgotPage<'a> {
        sent: bool,
        error: Option<&'a str>,
    }
    render(&ForgotPage {
        sent: false,
        error: None,
    })
}

async fn reset_page(Path(token): Path<String>) -> Response {
    #[derive(Template)]
    #[template(path = "reset.html")]
    struct ResetPage<'a> {
        token: &'a str,
        error: Option<&'a str>,
    }
    let r = ResetPage {
        token: &token,
        error: None,
    };
    render(&r)
}

async fn confirm_page(token: String, which: &str) -> Response {
    #[derive(Template)]
    #[template(path = "confirm.html")]
    struct ConfirmPage<'a> {
        title: &'a str,
        body: &'a str,
        action: &'a str,
    }
    let (title, body) = match which {
        "change-email" => (
            "confirm email change",
            "Click confirm to change your rampart account email to the address this message was sent to.",
        ),
        "verify" => (
            "verify mailbox",
            "Click confirm to prove ownership of this mailbox.",
        ),
        _ => ("confirm", "Confirm this action."),
    };
    let action = format!("/auth/{which}/{token}");
    render(&ConfirmPage {
        title,
        body,
        action: &action,
    })
}

async fn change_email_page(Path(token): Path<String>) -> Response {
    confirm_page(token, "change-email").await
}

async fn mailbox_verify_page(Path(token): Path<String>) -> Response {
    confirm_page(token, "verify").await
}

async fn setup_page() -> Response {
    #[derive(Template)]
    #[template(path = "setup.html")]
    struct SetupPage<'a> {
        error: Option<&'a str>,
        csrf_token: &'a str,
    }
    render(&SetupPage {
        error: None,
        csrf_token: "preview-mode",
    })
}

async fn aliases_page() -> Response {
    #[derive(Template)]
    #[template(path = "aliases.html")]
    struct Page {
        aliases: Vec<AliasRowView>,
        domains: Vec<DomainRowView>,
        total: i64,
        user_email: String,
        is_admin: bool,
    }
    let aliases = mock_aliases();
    let total = aliases.len() as i64;
    render(&Page {
        aliases,
        domains: mock_domains(),
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

async fn domains_page() -> Response {
    #[derive(Template)]
    #[template(path = "domains.html")]
    struct Page {
        domains: Vec<DomainRowView>,
        user_email: String,
        is_admin: bool,
    }
    render(&Page {
        domains: mock_domains(),
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

async fn password_change() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn email_change() -> StatusCode {
    StatusCode::ACCEPTED
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

pub async fn serve(listen: SocketAddr, static_dir: String) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/login", get(login_page))
        .route("/signup/{token}", get(signup_page))
        .route("/auth/forgot", get(forgot_page))
        .route("/auth/reset/{token}", get(reset_page))
        .route("/auth/change-email/{token}", get(change_email_page))
        .route("/mailbox/verify/{token}", get(mailbox_verify_page))
        .route("/setup", get(setup_page))
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
        .route("/healthz", get(healthz))
        .nest_service("/static", ServeDir::new(static_dir));

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
