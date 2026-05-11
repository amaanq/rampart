//! Resubmit outbound via SMTP AUTH on localhost:465 as rampart-notifier.

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};
use std::sync::{Arc, Mutex};

/// Abstract MAIL-FROM/RCPT-TO submit sink. Production: `SubmitClient`
/// (SMTP AUTH to stalwart). Tests: `MemorySubmit`.
#[async_trait]
pub trait Submit: Send + Sync {
    async fn submit(&self, mail_from: &str, rcpt_to: &str, body: &[u8]) -> Result<()>;
}

pub struct SubmitClient {
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

impl SubmitClient {
    pub fn from_config(cfg: &crate::config::Config) -> Result<Self> {
        let password = match &cfg.smtp_password_file {
            Some(path) => std::fs::read_to_string(path)?.trim().to_owned(),
            None => String::new(),
        };
        let is_implicit_tls = cfg.smtp_port == 465;
        let mut tls_params = TlsParameters::builder(cfg.smtp_host.clone());
        if cfg.smtp_host == "localhost" || cfg.smtp_host == "127.0.0.1" {
            tls_params = tls_params
                .dangerous_accept_invalid_certs(true)
                .dangerous_accept_invalid_hostnames(true);
        }
        let tls_params = tls_params.build().context("build TLS parameters")?;
        let builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)?
            .port(cfg.smtp_port)
            .credentials(Credentials::new(cfg.smtp_user.clone(), password));
        let transport = if is_implicit_tls {
            builder.tls(Tls::Wrapper(tls_params)).build()
        } else {
            builder.tls(Tls::Required(tls_params)).build()
        };
        Ok(Self { transport })
    }
}

#[async_trait]
impl Submit for SubmitClient {
    async fn submit(&self, mail_from: &str, rcpt_to: &str, body: &[u8]) -> Result<()> {
        use lettre::address::Envelope;
        let env = Envelope::new(Some(mail_from.parse()?), vec![rcpt_to.parse()?])?;
        self.transport.send_raw(&env, body).await?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct SubmittedMessage {
    pub mail_from: String,
    pub rcpt_to: String,
    pub body: Vec<u8>,
}

/// In-memory Submit. Tests `drain()` in send order.
#[derive(Default)]
pub struct MemorySubmit {
    pub sent: Arc<Mutex<Vec<SubmittedMessage>>>,
}

impl MemorySubmit {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn drain(&self) -> Vec<SubmittedMessage> {
        let mut v = self.sent.lock().unwrap();
        std::mem::take(&mut *v)
    }
}

#[async_trait]
impl Submit for MemorySubmit {
    async fn submit(&self, mail_from: &str, rcpt_to: &str, body: &[u8]) -> Result<()> {
        self.sent.lock().unwrap().push(SubmittedMessage {
            mail_from: mail_from.to_owned(),
            rcpt_to: rcpt_to.to_owned(),
            body: body.to_vec(),
        });
        Ok(())
    }
}

/// Strip CR/LF/NUL/control chars — defeats header injection via a hostile
/// display name or address.
fn sanitize_header_value(s: &str) -> String {
    s.chars()
        .filter(|c| *c != '\r' && *c != '\n' && *c != '\0' && !c.is_control())
        .collect()
}

/// RFC 5322 §3.2.4 quoted-string for display-name. Without this a display
/// like `Foo\` close-escapes the quote and smuggles the rest of the header.
pub fn rfc5322_quoted_display(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '\r' | '\n' | '\0' => continue,
            c if c.is_control() => continue,
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

pub fn rewrite_headers(
    raw: &[u8],
    new_from: &str,
    new_to: &str,
    strip_ar_authserv_id: &str,
) -> Vec<u8> {
    let new_from = sanitize_header_value(new_from);
    let new_to = sanitize_header_value(new_to);
    let split = find_header_end(raw);
    let (headers, body) = match split {
        Some(i) => (&raw[..i.0], &raw[i.1..]),
        None => (raw, &[][..]),
    };
    let mut out_headers: Vec<u8> = Vec::with_capacity(headers.len());
    let mut from_replaced = false;
    let mut to_replaced = false;

    for line in iter_folded_headers(headers) {
        let name_lower = line_name_lower(line);
        if name_lower == "from" {
            // First From only — drop the rest if a hostile upstream sent multiple.
            if !from_replaced {
                out_headers.extend_from_slice(format!("From: {new_from}\r\n").as_bytes());
                from_replaced = true;
            }
            continue;
        }
        // Drop anything that would let MUA Reply/ReplyAll route around the
        // alias, leak mailbox-side recipients, or expose alias tokens via
        // mailto: links. To: is rewritten below to the canonical alias.
        if name_lower == "reply-to"
            || name_lower == "cc"
            || name_lower == "bcc"
            || name_lower == "sender"
            || name_lower.starts_with("resent-")
            || name_lower.starts_with("list-")
        {
            continue;
        }
        if name_lower == "to" {
            if !to_replaced {
                out_headers.extend_from_slice(format!("To: {new_to}\r\n").as_bytes());
                to_replaced = true;
            }
            continue;
        }
        if name_lower == "authentication-results" {
            let value = line_value(line);
            if let Some((authserv, _)) = value.split_once(';') {
                if authserv.trim().eq_ignore_ascii_case(strip_ar_authserv_id) {
                    continue;
                }
            }
        }
        out_headers.extend_from_slice(line);
        if !line.ends_with(b"\r\n") && !line.ends_with(b"\n") {
            out_headers.extend_from_slice(b"\r\n");
        }
    }

    if !from_replaced {
        out_headers.extend_from_slice(format!("From: {new_from}\r\n").as_bytes());
    }
    if !to_replaced {
        out_headers.extend_from_slice(format!("To: {new_to}\r\n").as_bytes());
    }

    let mut full = out_headers;
    full.extend_from_slice(b"\r\n");
    full.extend_from_slice(body);
    full
}

/// Returns (end-of-headers-exclusive, start-of-body).
fn find_header_end(raw: &[u8]) -> Option<(usize, usize)> {
    if let Some(i) = find_bytes(raw, b"\r\n\r\n") {
        return Some((i, i + 4));
    }
    if let Some(i) = find_bytes(raw, b"\n\n") {
        return Some((i, i + 2));
    }
    None
}

fn find_bytes(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Folded header lines, each including the terminator.
fn iter_folded_headers(headers: &[u8]) -> Vec<&[u8]> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < headers.len() {
        while i < headers.len() {
            if headers[i] == b'\n' {
                let next = i + 1;
                if next < headers.len() && (headers[next] == b' ' || headers[next] == b'\t') {
                    i = next;
                    continue;
                }
                i = next;
                break;
            }
            i += 1;
        }
        lines.push(&headers[start..i]);
        start = i;
    }
    lines
}

fn line_name_lower(line: &[u8]) -> String {
    let end = line.iter().position(|b| *b == b':').unwrap_or(line.len());
    std::str::from_utf8(&line[..end])
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
}

fn line_value(line: &[u8]) -> &str {
    let colon = line.iter().position(|b| *b == b':').unwrap_or(line.len());
    std::str::from_utf8(&line[colon + 1..]).unwrap_or("").trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_from_preserves_body() {
        let raw = b"From: old@x.com\r\nSubject: hi\r\nDate: now\r\n\r\nbody bytes\r\nmore";
        let out = rewrite_headers(raw, "new@y.com", "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        assert!(s.contains("From: new@y.com\r\n"));
        assert!(!s.contains("From: old@x.com"));
        assert!(s.ends_with("body bytes\r\nmore"));
    }

    #[test]
    fn strips_only_own_authresults() {
        let raw = b"Authentication-Results: mail.test; dmarc=pass\r\n\
                    Authentication-Results: upstream.example; dmarc=fail\r\n\
                    From: x@x\r\n\
                    \r\n\
                    body";
        let out = rewrite_headers(raw, "new@y.com", "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        assert!(!s.contains("mail.test; dmarc=pass"));
        assert!(s.contains("upstream.example; dmarc=fail"));
    }

    #[test]
    fn sanitizes_header_injection() {
        let raw = b"From: evil@x\r\nSubject: original\r\n\r\nbody";
        let evil = "inject\r\nBcc: attacker@evil.test <x@y>";
        let out = rewrite_headers(raw, evil, "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        assert!(
            !s.contains("\r\nBcc:"),
            "CRLF-prefixed injection leaked: {s:?}"
        );
        let from_lines: Vec<&str> = s.split("\r\n").filter(|l| l.starts_with("From:")).collect();
        assert_eq!(from_lines.len(), 1, "exactly one From: header expected");
    }

    #[test]
    fn quoted_display_escapes_backslash_and_quote() {
        assert_eq!(rfc5322_quoted_display("Foo\\"), "\"Foo\\\\\"");
        assert_eq!(
            rfc5322_quoted_display("Alice \"Bob\""),
            "\"Alice \\\"Bob\\\"\""
        );
        assert_eq!(rfc5322_quoted_display("a\r\nb"), "\"ab\"");
        assert_eq!(rfc5322_quoted_display("a\x00b"), "\"ab\"");
        assert_eq!(rfc5322_quoted_display("Plain Name"), "\"Plain Name\"");
    }

    #[test]
    fn preserves_inbound_dkim() {
        let raw =
            b"DKIM-Signature: v=1; a=rsa-sha256; d=gmail.com; ...\r\nFrom: a@gmail\r\n\r\nbody";
        let out = rewrite_headers(raw, "ra+tok@addy.test", "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        assert!(s.contains("DKIM-Signature: v=1"));
    }

    #[test]
    fn strips_reply_to_and_cc_replaces_to() {
        let raw = b"From: sender@external\r\n\
                    Reply-To: bypass@evil.test\r\n\
                    To: alias@addy.test, leaked@third.test\r\n\
                    Cc: cc1@elsewhere, cc2@elsewhere\r\n\
                    Subject: hi\r\n\
                    \r\n\
                    body";
        let out = rewrite_headers(raw, "ra+tok@addy.test", "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        assert!(!s.contains("Reply-To:"), "Reply-To must be stripped");
        assert!(!s.contains("Cc:"), "Cc must be stripped");
        assert!(
            !s.contains("leaked@third.test"),
            "extra To recipient leaked"
        );
        assert!(!s.contains("cc1@elsewhere"));
        assert!(!s.contains("cc2@elsewhere"));
        assert!(s.contains("To: alias@addy.test\r\n"));
        assert!(s.contains("From: ra+tok@addy.test\r\n"));
        assert!(!s.contains("From: sender@external"));
    }

    #[test]
    fn case_insensitive_header_matching() {
        let raw =
            b"From: x@x\r\nREPLY-TO: bypass@evil\r\nCC: cc@elsewhere\r\nto: orig@addy\r\n\r\nbody";
        let out = rewrite_headers(raw, "new@y", "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        assert!(!s.to_lowercase().contains("reply-to:"));
        assert!(!s.to_lowercase().contains("cc:"));
        assert!(!s.contains("orig@addy"));
        assert!(s.contains("To: alias@addy.test"));
    }

    #[test]
    fn strips_bcc_sender_resent_and_list_headers() {
        // Each survives LMTP from stalwart and either leaks recipients (Bcc),
        // provides a MUA-honored alternate identity (Sender), exposes alias
        // tokens via mailto: (List-Unsubscribe), or re-opens the Reply bypass
        // for resent mail (Resent-*).
        let raw = b"From: sender@external\r\n\
                    Sender: real-sender@elsewhere\r\n\
                    Bcc: hidden@target.test\r\n\
                    Resent-From: resent@elsewhere\r\n\
                    Resent-To: resent-to@elsewhere\r\n\
                    Resent-Cc: resent-cc@elsewhere\r\n\
                    List-Unsubscribe: <mailto:list-unsub@list.test?subject=unsub>\r\n\
                    List-Id: <list.list.test>\r\n\
                    Subject: hi\r\n\
                    \r\n\
                    body";
        let out = rewrite_headers(raw, "ra+tok@addy.test", "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        for needle in [
            "Sender:",
            "Bcc:",
            "Resent-From:",
            "Resent-To:",
            "Resent-Cc:",
            "List-Unsubscribe:",
            "List-Id:",
            "real-sender@elsewhere",
            "hidden@target.test",
            "resent@elsewhere",
            "list-unsub@list.test",
        ] {
            assert!(!s.contains(needle), "{needle:?} survived rewrite: {s:?}");
        }
    }

    #[test]
    fn missing_to_synthesizes_one() {
        let raw = b"From: x@x\r\nSubject: hi\r\n\r\nbody";
        let out = rewrite_headers(raw, "new@y", "alias@addy.test", "mail.test");
        let s = std::str::from_utf8(&out).unwrap();
        let to_lines: Vec<&str> = s.split("\r\n").filter(|l| l.starts_with("To:")).collect();
        assert_eq!(to_lines.len(), 1);
        assert_eq!(to_lines[0], "To: alias@addy.test");
    }
}
