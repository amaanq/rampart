use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use rampart::{admin, bootstrap, config, migrate, preview, serve, worker};

#[derive(Parser)]
#[command(name = "rampart", version, about = "Forward-only email alias manager.")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the HTTP server.
    Serve,

    /// Start a zero-dependency preview server with mock data.
    Preview,

    /// Run the LMTP resubmit worker.
    Worker,

    /// Apply pending database migrations, then exit.
    Migrate,

    /// Operator commands.
    Admin {
        #[command(subcommand)]
        cmd: AdminCmd,
    },
}

#[derive(Subcommand)]
enum AdminCmd {
    /// Render the session.rcpt Sieve from current DB state.
    RenderSieve {
        #[arg(long)]
        output: Option<std::path::PathBuf>,
    },

    /// Create a user account.
    UserAdd {
        email: String,
        #[arg(long)]
        password: String,
        #[arg(long)]
        display_name: Option<String>,
        #[arg(long)]
        admin: bool,
    },

    /// List all users.
    UserList,

    /// Disable a user (clear sessions + revoke api keys + disable aliases).
    UserDisable { email: String },

    /// Reset a user's password. New password is printed.
    ResetPassword { email: String },

    /// Generate a one-time invite URL.
    Invite {
        /// Optional — if set, the invite only works for this exact email.
        #[arg(long)]
        email: Option<String>,
    },

    /// Add a verified mailbox for a user.
    AddMailbox {
        /// Owner user's email
        #[arg(long = "user")]
        user_email: String,
        /// Mailbox address
        email: String,
        #[arg(long)]
        display_name: Option<String>,
    },

    /// List mailboxes (optionally filtered by owning user).
    ListMailboxes {
        #[arg(long = "user")]
        user_email: Option<String>,
    },

    /// Set the default mailbox for an alias domain.
    SetDefaultMailbox {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        mailbox: String,
    },

    /// Export aliases to CSV (optionally filtered by user).
    ExportAliases {
        #[arg(long = "user")]
        user_email: Option<String>,
    },

    /// Import aliases from CSV produced by export-aliases.
    ImportAliases { file: std::path::PathBuf },

    /// Idempotently seed the database with demo data for UI development.
    DevSeed,

    /// Prune expired/used tokens, stale rate-limit buckets, expired sessions,
    /// expired webauthn ceremonies, old email_log rows. Idempotent.
    Gc {
        /// Delete email_log rows older than this many days.
        #[arg(long, default_value_t = rampart::admin::DEFAULT_EMAIL_LOG_DAYS)]
        email_log_days: i32,
        /// Print would-delete counts without changing state.
        #[arg(long)]
        dry_run: bool,
    },

    /// Idempotently seed stalwart's JMAP registry for reply-via-alias.
    BootstrapStalwart {
        /// e.g. http://127.0.0.1:8080 or https://stalwart.example.com
        #[arg(long)]
        jmap_base_url: String,
        #[arg(long, default_value = "admin")]
        admin_username: String,
        #[arg(long)]
        admin_password_file: std::path::PathBuf,
        /// Same password rampart uses for SMTP AUTH.
        #[arg(long)]
        rampart_notifier_password_file: std::path::PathBuf,
        /// e.g. rampart-notifier@rampart.email
        #[arg(long)]
        rampart_notifier_address: String,
        #[arg(long, default_value = "127.0.0.1")]
        lmtp_address: String,
        #[arg(long, default_value_t = 8024)]
        lmtp_port: u16,
        /// Repeatable. Each becomes a stalwart `Domain` with auto-DKIM and
        /// a sieve rcpt-domain match → 'rampart_rcpt'.
        #[arg(long = "alias-domain", value_name = "DOMAIN", num_args = 0..)]
        alias_domains: Vec<String>,
        /// libpq connection string. Drives stalwart's StoreLookup
        /// (namespace `sql`) so `query('sql', ...)` in the sieve resolves.
        #[arg(long)]
        database_url: String,
        /// Rendered Sieve path — pushed into `SieveSystemScript rampart_rcpt`
        /// so stalwart doesn't depend on `%{file:...}%`.
        #[arg(long)]
        sieve_path: std::path::PathBuf,
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("build tokio runtime")?;
    rt.block_on(run(cli))
}

async fn run(cli: Cli) -> Result<()> {
    match cli.cmd {
        Cmd::Serve => {
            let cfg = config::Config::from_env()?;
            serve::serve(cfg).await
        }
        Cmd::Preview => {
            use std::net::SocketAddr;
            let listen: SocketAddr = std::env::var("RAMPART_LISTEN")
                .unwrap_or_else(|_| "127.0.0.1:8090".into())
                .parse()
                .context("parsing RAMPART_LISTEN")?;
            let static_dir =
                std::env::var("RAMPART_STATIC_DIR").unwrap_or_else(|_| "static".into());
            preview::serve(listen, static_dir).await
        }
        Cmd::Worker => {
            let cfg = config::Config::from_env()?;
            worker::run(cfg).await
        }
        Cmd::Migrate => {
            let url = database_url()?;
            migrate::run(&url).await
        }
        Cmd::Admin { cmd } => match cmd {
            // Bootstrap takes DB params from CLI args, not env.
            AdminCmd::BootstrapStalwart {
                jmap_base_url,
                admin_username,
                admin_password_file,
                rampart_notifier_password_file,
                rampart_notifier_address,
                lmtp_address,
                lmtp_port,
                alias_domains,
                database_url,
                sieve_path,
                dry_run,
            } => {
                bootstrap::cli(
                    jmap_base_url,
                    admin_username,
                    admin_password_file,
                    rampart_notifier_password_file,
                    rampart_notifier_address,
                    lmtp_address,
                    lmtp_port,
                    alias_domains,
                    database_url,
                    sieve_path,
                    dry_run,
                )
                .await
            }
            AdminCmd::RenderSieve { output } => admin::render_sieve(&database_url()?, output).await,
            AdminCmd::UserAdd {
                email,
                password,
                display_name,
                admin: is_admin,
            } => admin::user_add(&database_url()?, email, display_name, is_admin, password).await,
            AdminCmd::UserList => admin::user_list(&database_url()?).await,
            AdminCmd::UserDisable { email } => admin::user_disable(&database_url()?, email).await,
            AdminCmd::ResetPassword { email } => {
                admin::reset_password(&database_url()?, email).await
            }
            AdminCmd::Invite { email } => admin::invite(&database_url()?, email).await,
            AdminCmd::AddMailbox {
                user_email,
                email,
                display_name,
            } => admin::add_mailbox(&database_url()?, user_email, email, display_name).await,
            AdminCmd::ListMailboxes { user_email } => {
                admin::list_mailboxes(&database_url()?, user_email).await
            }
            AdminCmd::SetDefaultMailbox { domain, mailbox } => {
                admin::set_default_mailbox(&database_url()?, domain, mailbox).await
            }
            AdminCmd::ExportAliases { user_email } => {
                admin::export_aliases(&database_url()?, user_email).await
            }
            AdminCmd::ImportAliases { file } => admin::import_aliases(&database_url()?, file).await,
            AdminCmd::DevSeed => admin::dev_seed(&database_url()?).await,
            AdminCmd::Gc {
                email_log_days,
                dry_run,
            } => {
                let stats = admin::gc(&database_url()?, email_log_days, dry_run).await?;
                stats.print(dry_run);
                Ok(())
            }
        },
    }
}

fn database_url() -> Result<String> {
    std::env::var("RAMPART_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .context("RAMPART_DATABASE_URL or DATABASE_URL must be set")
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,rampart=debug"));
    fmt().with_env_filter(filter).with_target(false).init();
}
