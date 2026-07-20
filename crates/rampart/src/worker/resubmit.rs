//! Resubmit outbound via SMTP AUTH on localhost:465 as rampart-notifier.

use std::{
   fs,
   mem,
   str,
   sync::{
      Arc,
      Mutex,
   },
};

use anyhow::{
   Context as _,
   Result,
};
use async_trait::async_trait;
use lettre::{
   AsyncSmtpTransport,
   AsyncTransport as _,
   Tokio1Executor,
   transport::smtp::{
      authentication::Credentials,
      client::{
         Tls,
         TlsParameters,
      },
   },
};

use crate::config::Config;

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
   /// Build the SMTP submit transport from the worker's configuration.
   ///
   /// # Errors
   /// Returns an error if the SMTP password file can't be read or the TLS
   /// parameters / relay transport can't be constructed.
   pub fn from_config(cfg: &Config) -> Result<Self> {
      let password = match cfg.smtp_password_file.as_ref() {
         Some(path) => fs::read_to_string(path)?.trim().to_owned(),
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
   pub rcpt_to:   String,
   pub body:      Vec<u8>,
}

/// In-memory Submit. Tests `drain()` in send order.
#[derive(Default)]
pub struct MemorySubmit {
   pub sent: Arc<Mutex<Vec<SubmittedMessage>>>,
}

impl MemorySubmit {
   #[must_use]
   pub fn new() -> Self {
      Self::default()
   }

   /// Take and return all messages recorded so far, in send order.
   ///
   /// # Panics
   /// Panics if the internal mutex is poisoned.
   #[must_use]
   pub fn drain(&self) -> Vec<SubmittedMessage> {
      let mut guard = self.sent.lock().unwrap();
      mem::take(&mut *guard)
   }
}

#[async_trait]
impl Submit for MemorySubmit {
   async fn submit(&self, mail_from: &str, rcpt_to: &str, body: &[u8]) -> Result<()> {
      self.sent.lock().unwrap().push(SubmittedMessage {
         mail_from: mail_from.to_owned(),
         rcpt_to:   rcpt_to.to_owned(),
         body:      body.to_vec(),
      });
      Ok(())
   }
}

/// Strip CR/LF/NUL/control chars — defeats header injection via a hostile
/// display name or address.
fn sanitize_header_value(value: &str) -> String {
   value
      .chars()
      .filter(|ch| *ch != '\r' && *ch != '\n' && *ch != '\0' && !ch.is_control())
      .collect()
}

/// RFC 5322 §3.2.4 quoted-string for display-name. Without this a display
/// like `Foo\` close-escapes the quote and smuggles the rest of the header.
#[must_use]
pub fn rfc5322_quoted_display(value: &str) -> String {
   let mut out = String::with_capacity(value.len() + 2);
   out.push('"');
   for ch in value.chars() {
      match ch {
         '\r' | '\n' | '\0' => {},
         _ if ch.is_control() => {},
         '"' | '\\' => {
            out.push('\\');
            out.push(ch);
         },
         _ => out.push(ch),
      }
   }
   out.push('"');
   out
}

#[must_use]
pub fn rewrite_headers(
   raw: &[u8],
   new_from: &str,
   new_to: &str,
   strip_ar_authserv_id: &str,
) -> Vec<u8> {
   let new_from = sanitize_header_value(new_from);
   let new_to = sanitize_header_value(new_to);
   let (headers, body) = find_header_end(raw).map_or((raw, &[][..]), |bounds| {
      (&raw[..bounds.0], &raw[bounds.1..])
   });
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
         if let Some((authserv, _)) = value.split_once(';')
            && authserv.trim().eq_ignore_ascii_case(strip_ar_authserv_id)
         {
            continue;
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
   if let Some(idx) = find_bytes(raw, b"\r\n\r\n") {
      return Some((idx, idx + 4));
   }
   if let Some(idx) = find_bytes(raw, b"\n\n") {
      return Some((idx, idx + 2));
   }
   None
}

fn find_bytes(hay: &[u8], needle: &[u8]) -> Option<usize> {
   hay.windows(needle.len())
      .position(|window| window == needle)
}

/// Folded header lines, each including the terminator.
fn iter_folded_headers(headers: &[u8]) -> Vec<&[u8]> {
   let mut lines = Vec::new();
   let mut start = 0;
   let mut idx = 0;
   while idx < headers.len() {
      while idx < headers.len() {
         if headers[idx] == b'\n' {
            let next = idx + 1;
            if next < headers.len() && (headers[next] == b' ' || headers[next] == b'\t') {
               idx = next;
               continue;
            }
            idx = next;
            break;
         }
         idx += 1;
      }
      lines.push(&headers[start..idx]);
      start = idx;
   }
   lines
}

fn line_name_lower(line: &[u8]) -> String {
   let end = line
      .iter()
      .position(|byte| *byte == b':')
      .unwrap_or(line.len());
   str::from_utf8(&line[..end])
      .unwrap_or("")
      .trim()
      .to_ascii_lowercase()
}

fn line_value(line: &[u8]) -> &str {
   let colon = line
      .iter()
      .position(|byte| *byte == b':')
      .unwrap_or(line.len());
   str::from_utf8(&line[colon + 1..]).unwrap_or("").trim()
}

#[cfg(test)]
#[expect(clippy::inline_modules, reason = "unit tests kept beside impl")]
mod tests {
   use super::*;

   #[test]
   fn replaces_from_preserves_body() {
      let raw = b"From: old@x.com\r\nSubject: hi\r\nDate: now\r\n\r\nbody bytes\r\nmore";
      let out = rewrite_headers(raw, "new@y.com", "alias@addy.test", "mail.test");
      let out_str = str::from_utf8(&out).unwrap();
      assert!(out_str.contains("From: new@y.com\r\n"));
      assert!(!out_str.contains("From: old@x.com"));
      assert!(out_str.ends_with("body bytes\r\nmore"));
   }

   #[test]
   fn strips_only_own_authresults() {
      let raw = b"Authentication-Results: mail.test; dmarc=pass\r\n\
                    Authentication-Results: upstream.example; dmarc=fail\r\n\
                    From: x@x\r\n\
                    \r\n\
                    body";
      let out = rewrite_headers(raw, "new@y.com", "alias@addy.test", "mail.test");
      let out_str = str::from_utf8(&out).unwrap();
      assert!(!out_str.contains("mail.test; dmarc=pass"));
      assert!(out_str.contains("upstream.example; dmarc=fail"));
   }

   #[test]
   fn sanitizes_header_injection() {
      let raw = b"From: evil@x\r\nSubject: original\r\n\r\nbody";
      let evil = "inject\r\nBcc: attacker@evil.test <x@y>";
      let out = rewrite_headers(raw, evil, "alias@addy.test", "mail.test");
      let out_str = str::from_utf8(&out).unwrap();
      assert!(
         !out_str.contains("\r\nBcc:"),
         "CRLF-prefixed injection leaked: {out_str:?}"
      );
      let from_count = out_str
         .split("\r\n")
         .filter(|line| line.starts_with("From:"))
         .count();
      assert_eq!(from_count, 1, "exactly one From: header expected");
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
      let raw = b"DKIM-Signature: v=1; a=rsa-sha256; d=gmail.com; ...\r\nFrom: a@gmail\r\n\r\nbody";
      let out = rewrite_headers(raw, "ra+tok@addy.test", "alias@addy.test", "mail.test");
      let out_str = str::from_utf8(&out).unwrap();
      assert!(out_str.contains("DKIM-Signature: v=1"));
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
      let out_str = str::from_utf8(&out).unwrap();
      assert!(!out_str.contains("Reply-To:"), "Reply-To must be stripped");
      assert!(!out_str.contains("Cc:"), "Cc must be stripped");
      assert!(
         !out_str.contains("leaked@third.test"),
         "extra To recipient leaked"
      );
      assert!(!out_str.contains("cc1@elsewhere"));
      assert!(!out_str.contains("cc2@elsewhere"));
      assert!(out_str.contains("To: alias@addy.test\r\n"));
      assert!(out_str.contains("From: ra+tok@addy.test\r\n"));
      assert!(!out_str.contains("From: sender@external"));
   }

   #[test]
   fn case_insensitive_header_matching() {
      let raw =
         b"From: x@x\r\nREPLY-TO: bypass@evil\r\nCC: cc@elsewhere\r\nto: orig@addy\r\n\r\nbody";
      let out = rewrite_headers(raw, "new@y", "alias@addy.test", "mail.test");
      let out_str = str::from_utf8(&out).unwrap();
      assert!(!out_str.to_lowercase().contains("reply-to:"));
      assert!(!out_str.to_lowercase().contains("cc:"));
      assert!(!out_str.contains("orig@addy"));
      assert!(out_str.contains("To: alias@addy.test"));
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
      let out_str = str::from_utf8(&out).unwrap();
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
         assert!(
            !out_str.contains(needle),
            "{needle:?} survived rewrite: {out_str:?}"
         );
      }
   }

   #[test]
   fn missing_to_synthesizes_one() {
      let raw = b"From: x@x\r\nSubject: hi\r\n\r\nbody";
      let out = rewrite_headers(raw, "new@y", "alias@addy.test", "mail.test");
      let out_str = str::from_utf8(&out).unwrap();
      let to_lines: Vec<&str> = out_str
         .split("\r\n")
         .filter(|line| line.starts_with("To:"))
         .collect();
      assert_eq!(to_lines.len(), 1);
      assert_eq!(to_lines[0], "To: alias@addy.test");
   }
}
