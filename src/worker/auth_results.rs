//! Parse the topmost `Authentication-Results:` header written by our
//! stalwart (matches authserv-id against configured hostname).
//!
//! Ignore deeper AR headers — they were written by earlier hops and are not
//! authoritative for our trust decision.

#[derive(Debug, Default, Clone)]
pub struct AuthResults {
   pub dmarc:             Option<String>,
   pub dkim:              Option<String>,
   pub spf:               Option<String>,
   /// For dmarc: the d=... value of whatever DKIM pass backed it
   /// (convenience — not a replacement for proper alignment logic).
   pub dmarc_header_from: Option<String>,
}

/// Parse the raw AR header line (without the "Authentication-Results:" prefix).
#[must_use]
pub fn parse_single(line: &str) -> Option<(String, AuthResults)> {
   // format: authserv-id; method=result ...; method=result ...
   let (authserv, rest) = line.split_once(';')?;
   let authserv = authserv.trim().to_owned();
   let mut out = AuthResults::default();
   for part in rest.split(';') {
      let part = part.trim();
      if part.is_empty() {
         continue;
      }
      let (kv, _props) = part.split_once(' ').unwrap_or((part, ""));
      let (method, result) = kv.split_once('=')?;
      let method = method.trim().to_ascii_lowercase();
      let result = result.trim().to_ascii_lowercase();
      match method.as_str() {
         "dmarc" => out.dmarc = Some(result),
         "dkim" => out.dkim = Some(result),
         "spf" => out.spf = Some(result),
         _ => {},
      }
   }
   // Very shallow parse of header.from=<domain>. Real implementations
   // would honor the full ABNF; we only need the domain for alignment.
   if let Some(idx) = rest.find("header.from=") {
      let after = &rest[idx + "header.from=".len()..];
      let end = after.find([' ', ';', '\r', '\n']).unwrap_or(after.len());
      out.dmarc_header_from = Some(after[..end].trim().to_owned());
   }
   Some((authserv, out))
}

/// Given the full list of AR header values (multi-line), return the results
/// written by `expected_authserv_id` (usually our stalwart's hostname).
///
/// If multiple, take the topmost — sender-supplied AR headers may be present
/// deeper in the mail.
pub fn extract_for<I, S>(headers: I, expected_authserv_id: &str) -> Option<AuthResults>
where
   I: IntoIterator<Item = S>,
   S: AsRef<str>,
{
   for header in headers {
      if let Some((authserv, results)) = parse_single(header.as_ref())
         && authserv.eq_ignore_ascii_case(expected_authserv_id)
      {
         return Some(results);
      }
   }
   None
}

/// Reply-path policy: AR.dmarc=pass, AR header.from binds to the
/// visible From we parsed, and the visible From equals the mailbox
/// address exactly.
///
/// Same-domain alignment is too loose — any gmail
/// user could reply to alice@gmail.com's reverse alias and impersonate
/// her. Tighten only when an authorized-sender table exists.
///
/// # Errors
/// Returns an error describing the first failed check: DMARC not `pass`,
/// a missing or misaligned `header.from` binding, or a visible From that
/// doesn't equal the mailbox address.
pub fn reply_policy_ok(
   ar: &AuthResults,
   mailbox_email: &str,
   visible_from: &str,
) -> Result<(), String> {
   if ar.dmarc.as_deref() != Some("pass") {
      return Err(format!(
         "DMARC not pass (got {:?})",
         ar.dmarc.as_deref().unwrap_or("<absent>")
      ));
   }
   // Binding: what stalwart vouched for must equal what we're about to
   // trust. Guards against parser divergence between stalwart and us.
   let Some(visible_from_domain) = visible_from.rsplit_once('@').map(|(_, domain)| domain) else {
      return Err(format!("visible From '{visible_from}' has no @"));
   };
   match ar.dmarc_header_from.as_deref() {
      None => {
         return Err("DMARC result lacks header.from binding; cannot trust alignment".into());
      },
      Some(header_from) => {
         // header.from can be a bare domain or an address; compare domains.
         let ar_domain = header_from
            .rsplit_once('@')
            .map_or(header_from, |(_, domain)| domain);
         if !ar_domain.eq_ignore_ascii_case(visible_from_domain) {
            return Err(format!(
               "AR header.from domain {ar_domain} does not match parsed visible From domain \
                {visible_from_domain}"
            ));
         }
      },
   }
   if !visible_from.eq_ignore_ascii_case(mailbox_email) {
      return Err(format!(
         "visible From {visible_from} != mailbox {mailbox_email} (no authorized-sender list yet)"
      ));
   }
   Ok(())
}

#[cfg(test)]
#[expect(clippy::inline_modules, reason = "unit tests kept beside impl")]
mod tests {
   use super::*;

   #[test]
   fn parses_dmarc_pass() {
      let (authserv, results) =
         parse_single("mail.example.com; dmarc=pass header.from=gmail.com; spf=pass").unwrap();
      assert_eq!(authserv, "mail.example.com");
      assert_eq!(results.dmarc.as_deref(), Some("pass"));
      assert_eq!(results.dmarc_header_from.as_deref(), Some("gmail.com"));
   }

   #[test]
   fn rejects_other_authserv() {
      let ar = extract_for(
         vec!["other.example; dmarc=pass header.from=gmail.com"],
         "mail.example.com",
      );
      assert!(ar.is_none());
   }

   fn ar_pass(hfrom: &str) -> AuthResults {
      AuthResults {
         dmarc: Some("pass".into()),
         dmarc_header_from: Some(hfrom.to_owned()),
         ..Default::default()
      }
   }

   #[test]
   fn reply_policy_rejects_same_domain_other_user() {
      let ar = ar_pass("gmail.com");
      let err = reply_policy_ok(&ar, "alice@gmail.com", "someone@gmail.com")
         .expect_err("same-domain-but-different-mailbox must be rejected");
      assert!(err.contains("!= mailbox"));
   }

   #[test]
   fn reply_policy_accepts_exact_mailbox() {
      let ar = ar_pass("gmail.com");
      reply_policy_ok(&ar, "alice@gmail.com", "alice@gmail.com").unwrap();
   }

   #[test]
   fn reply_policy_unaligned() {
      let ar = ar_pass("spoof.com");
      assert!(reply_policy_ok(&ar, "alice@gmail.com", "someone@spoof.com").is_err());
   }

   #[test]
   fn reply_policy_dmarc_fail() {
      let ar = AuthResults {
         dmarc: Some("fail".into()),
         dmarc_header_from: Some("gmail.com".into()),
         ..Default::default()
      };
      assert!(reply_policy_ok(&ar, "alice@gmail.com", "alice@gmail.com").is_err());
   }

   #[test]
   fn reply_policy_rejects_missing_header_from() {
      // Stalwart returned dmarc=pass but no header.from — unbindable.
      let ar = AuthResults {
         dmarc: Some("pass".into()),
         dmarc_header_from: None,
         ..Default::default()
      };
      let err = reply_policy_ok(&ar, "alice@gmail.com", "alice@gmail.com")
         .expect_err("must reject when AR lacks header.from");
      assert!(err.contains("binding"));
   }

   #[test]
   fn reply_policy_rejects_parser_divergence() {
      // Stalwart vouched for gmail.com but we parsed alice@impostor.example.
      let ar = ar_pass("gmail.com");
      let err = reply_policy_ok(&ar, "alice@gmail.com", "alice@impostor.example")
         .expect_err("must reject when our From disagrees with AR header.from");
      assert!(err.contains("does not match parsed visible From"));
   }
}
