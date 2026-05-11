//! Runtime configuration, loaded from env. See `Config` fields for the
//! `RAMPART_*` variable each one reads.

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub listen: SocketAddr,
    pub public_origin: String,
    pub static_dir: PathBuf,
    pub sieve_output_path: Option<PathBuf>,

    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_password_file: Option<PathBuf>,
    pub notifier_from: String,

    pub webauthn_rp_id: String,

    pub lmtp_listen: SocketAddr,
    pub stalwart_hostname: String,
    /// SIGTERM grace window for in-flight LMTP sessions. Default 20.
    pub lmtp_drain_secs: u64,

    /// Optional: when unset, post-CRUD JMAP sync is skipped and the operator
    /// must run `rampart admin bootstrap-stalwart` to push.
    pub stalwart_jmap_base_url: Option<String>,
    pub stalwart_admin_username: String,
    pub stalwart_admin_password_file: Option<PathBuf>,

    /// HMAC-SHA256 key for bounce VERP signing — without it, any internet
    /// host could forge VERPs that mutate email_log rows.
    pub verp_key: Vec<u8>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = env("RAMPART_DATABASE_URL").or_else(|_| env("DATABASE_URL"))?;
        let listen: SocketAddr = env_or("RAMPART_LISTEN", "[::1]:8090")
            .parse()
            .context("parsing RAMPART_LISTEN")?;
        let public_origin = normalize_origin(&env("RAMPART_PUBLIC_ORIGIN")?)?;
        let host_from_origin = host_from_origin(&public_origin);
        let static_dir = PathBuf::from(env_or("RAMPART_STATIC_DIR", "static"));
        let sieve_output_path = std::env::var("RAMPART_SIEVE_OUTPUT_PATH")
            .ok()
            .map(PathBuf::from);

        let smtp_host = env_or("RAMPART_SMTP_HOST", "localhost");
        let smtp_port: u16 = env_or("RAMPART_SMTP_PORT", "465")
            .parse()
            .context("parsing RAMPART_SMTP_PORT")?;
        let smtp_user = env_or(
            "RAMPART_SMTP_USER",
            &format!("rampart-notifier@{}", &host_from_origin),
        );
        let smtp_password_file = std::env::var("RAMPART_SMTP_PASSWORD_FILE")
            .ok()
            .map(PathBuf::from);
        let notifier_from = env_or(
            "RAMPART_NOTIFIER_FROM",
            &format!("\"rampart\" <{}>", &smtp_user),
        );

        let webauthn_rp_id = env_or("RAMPART_WEBAUTHN_RP_ID", &host_from_origin);

        let lmtp_listen: SocketAddr = env_or("RAMPART_LMTP_LISTEN", "127.0.0.1:8024")
            .parse()
            .context("parsing RAMPART_LMTP_LISTEN")?;
        let stalwart_hostname = env_or("RAMPART_STALWART_HOSTNAME", &host_from_origin);
        let lmtp_drain_secs: u64 = env_or("RAMPART_LMTP_DRAIN_SECS", "20")
            .parse()
            .context("parsing RAMPART_LMTP_DRAIN_SECS")?;

        let stalwart_jmap_base_url = std::env::var("RAMPART_STALWART_JMAP_BASE_URL").ok();
        let stalwart_admin_username = env_or("RAMPART_STALWART_ADMIN_USERNAME", "admin");
        let stalwart_admin_password_file = std::env::var("RAMPART_STALWART_ADMIN_PASSWORD_FILE")
            .ok()
            .map(PathBuf::from);

        // Trim trailing whitespace so `openssl rand -base64 32 > /tmp/k`
        // works without surprises; reject <32B (brute-able offline).
        let verp_key = {
            let path = std::env::var("RAMPART_VERP_KEY_FILE")
                .context("env RAMPART_VERP_KEY_FILE must be set (path to >=32-byte HMAC key)")?;
            let raw = std::fs::read(&path)
                .with_context(|| format!("reading RAMPART_VERP_KEY_FILE='{path}'"))?;
            let trimmed: Vec<u8> = raw
                .into_iter()
                .rev()
                .skip_while(|b| matches!(*b, b'\n' | b'\r' | b' ' | b'\t'))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if trimmed.len() < 32 {
                anyhow::bail!(
                    "RAMPART_VERP_KEY_FILE contents must be at least 32 bytes (got {})",
                    trimmed.len()
                );
            }
            trimmed
        };

        Ok(Self {
            database_url,
            listen,
            public_origin,
            static_dir,
            sieve_output_path,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password_file,
            notifier_from,
            webauthn_rp_id,
            lmtp_listen,
            stalwart_hostname,
            lmtp_drain_secs,
            stalwart_jmap_base_url,
            stalwart_admin_username,
            stalwart_admin_password_file,
            verp_key,
        })
    }
}

fn env(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("env {name} must be set"))
}

fn env_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_owned())
}

fn host_from_origin(origin: &str) -> String {
    url::Url::parse(origin)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_owned()))
        .unwrap_or_else(|| "localhost".to_owned())
}

/// Canonicalize so the browser-supplied Origin header matches by exact
/// string equality. Browsers emit `<scheme>://<host>` lowercased with the
/// default port (80/443) elided; we mirror that and reject path / query /
/// fragment / userinfo. Without this, a trailing `/`, explicit `:443`, or
/// uppercase host in the env var would silently false-reject every CSRF check.
fn normalize_origin(raw: &str) -> Result<String> {
    let url = url::Url::parse(raw.trim()).context("RAMPART_PUBLIC_ORIGIN must be a valid URL")?;
    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        anyhow::bail!("RAMPART_PUBLIC_ORIGIN scheme must be http or https (got {scheme})");
    }
    let host = url
        .host_str()
        .context("RAMPART_PUBLIC_ORIGIN must have a host")?
        .to_ascii_lowercase();
    if !url.username().is_empty() || url.password().is_some() {
        anyhow::bail!("RAMPART_PUBLIC_ORIGIN must not contain userinfo");
    }
    if url.path() != "" && url.path() != "/" {
        anyhow::bail!(
            "RAMPART_PUBLIC_ORIGIN must not contain a path (got {})",
            url.path()
        );
    }
    if url.query().is_some() || url.fragment().is_some() {
        anyhow::bail!("RAMPART_PUBLIC_ORIGIN must not contain a query or fragment");
    }
    let default_port = if scheme == "https" { 443 } else { 80 };
    let port = url.port().filter(|p| *p != default_port);
    Ok(match port {
        Some(p) => format!("{scheme}://{host}:{p}"),
        None => format!("{scheme}://{host}"),
    })
}

#[cfg(test)]
mod tests {
    use super::normalize_origin;

    #[test]
    fn drops_trailing_slash() {
        assert_eq!(
            normalize_origin("https://rampart.example.com/").unwrap(),
            "https://rampart.example.com"
        );
    }

    #[test]
    fn drops_default_port() {
        assert_eq!(
            normalize_origin("https://rampart.example.com:443").unwrap(),
            "https://rampart.example.com"
        );
        assert_eq!(
            normalize_origin("http://rampart.example.com:80").unwrap(),
            "http://rampart.example.com"
        );
    }

    #[test]
    fn keeps_non_default_port() {
        assert_eq!(
            normalize_origin("http://localhost:8090").unwrap(),
            "http://localhost:8090"
        );
    }

    #[test]
    fn lowercases_host_and_scheme() {
        assert_eq!(
            normalize_origin("HTTPS://RAMPART.EXAMPLE.COM").unwrap(),
            "https://rampart.example.com"
        );
    }

    #[test]
    fn rejects_path_query_userinfo() {
        assert!(normalize_origin("https://rampart.example.com/foo").is_err());
        assert!(normalize_origin("https://rampart.example.com/?x=y").is_err());
        assert!(normalize_origin("https://user@rampart.example.com").is_err());
    }

    #[test]
    fn rejects_non_http_scheme() {
        assert!(normalize_origin("ftp://rampart.example.com").is_err());
        assert!(normalize_origin("javascript:alert(1)").is_err());
    }
}
