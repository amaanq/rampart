//! Outbound transactional mail via lettre SMTP AUTH to stalwart
//! (default localhost:465). Test builds can swap in an in-memory
//! capturer via the Mailer trait.

use anyhow::{Context, Result};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::sync::{Arc, Mutex};

#[async_trait::async_trait]
pub trait Mailer: Send + Sync {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()>;
}

pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl SmtpMailer {
    pub fn from_config(cfg: &crate::config::Config) -> Result<Self> {
        let password = match &cfg.smtp_password_file {
            Some(path) => std::fs::read_to_string(path)
                .with_context(|| format!("reading RAMPART_SMTP_PASSWORD_FILE {}", path.display()))?
                .trim()
                .to_owned(),
            None => String::new(),
        };
        let creds = Credentials::new(cfg.smtp_user.clone(), password);
        // Port 465 is implicit TLS (SMTPS); port 587 is STARTTLS. `.relay()`
        // defaults to STARTTLS which is wrong for 465. Override based on port.
        let is_implicit_tls = cfg.smtp_port == 465;
        // For localhost submission we need to accept the self-signed stalwart cert.
        let mut tls_params = TlsParameters::builder(cfg.smtp_host.clone());
        if cfg.smtp_host == "localhost" || cfg.smtp_host == "127.0.0.1" {
            tls_params = tls_params
                .dangerous_accept_invalid_certs(true)
                .dangerous_accept_invalid_hostnames(true);
        }
        let tls_params = tls_params.build().context("build TLS parameters")?;
        let builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)?
            .port(cfg.smtp_port)
            .credentials(creds);
        let transport = if is_implicit_tls {
            builder.tls(Tls::Wrapper(tls_params)).build()
        } else {
            builder.tls(Tls::Required(tls_params)).build()
        };
        Ok(Self {
            transport,
            from: cfg.notifier_from.clone(),
        })
    }
}

#[async_trait::async_trait]
impl Mailer for SmtpMailer {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        let msg = Message::builder()
            .from(self.from.parse()?)
            .to(to.parse()?)
            .subject(subject)
            .header(lettre::message::header::ContentType::TEXT_PLAIN)
            .body(body.to_owned())?;
        self.transport.send(msg).await.context("smtp send")?;
        tracing::info!(to, subject, "sent transactional mail");
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SentEmail {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Default)]
pub struct MemoryMailer {
    pub sent: Arc<Mutex<Vec<SentEmail>>>,
}

impl MemoryMailer {
    pub fn new() -> Self {
        Self::default()
    }
    #[allow(dead_code)]
    pub fn drain(&self) -> Vec<SentEmail> {
        let mut v = self.sent.lock().unwrap();
        std::mem::take(&mut *v)
    }
}

#[async_trait::async_trait]
impl Mailer for MemoryMailer {
    async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()> {
        self.sent.lock().unwrap().push(SentEmail {
            to: to.to_owned(),
            subject: subject.to_owned(),
            body: body.to_owned(),
        });
        Ok(())
    }
}
