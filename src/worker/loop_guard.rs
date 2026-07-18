//! Validate incoming LMTP recipients.
//!
//! Accept only
//!   rampart-<digits>@internal.rampart.lmtp                  (inbound forward)
//!   rampart-reply-<digits>@internal.rampart.lmtp            (inbound reply)
//!   rampart-bnc-<payload>@internal.rampart.lmtp             (signed DSN bounce
//! VERP) Anything else → 550. Prevents loops and mistaken direct delivery.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BouncePhase {
   Forward,
   Reply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rcpt {
   Forward(i64),
   Reply(i64),
   /// DSN delivered to a `bnc+{payload}@<alias_domain>` VERP we used
   /// as MAIL FROM. `payload` is the raw (untrusted) string
   /// `{phase}+{id}+{tag}`; the worker HMAC-verifies it against
   /// `verp_key` before mutating any `email_log` row.
   Bounce {
      payload: String,
   },
}

// BIGSERIAL ids are strictly positive and fit i64. Reject leading
// sign, zero, or values that would wrap on i64 cast.
fn parse_bigserial(digits: &str) -> Option<i64> {
   let value = i64::try_from(digits.parse::<u64>().ok()?).ok()?;
   (value > 0).then_some(value)
}

pub fn parse_rcpt(addr: &str, expected_domain: &str) -> Option<Rcpt> {
   let (local, domain) = addr.split_once('@')?;
   if !domain.eq_ignore_ascii_case(expected_domain) {
      return None;
   }
   if let Some(rest) = local.strip_prefix("rampart-bnc-") {
      if rest.is_empty() {
         return None;
      }
      Some(Rcpt::Bounce {
         payload: rest.to_owned(),
      })
   } else if let Some(rest) = local.strip_prefix("rampart-reply-") {
      parse_bigserial(rest).map(Rcpt::Reply)
   } else if let Some(rest) = local.strip_prefix("rampart-") {
      parse_bigserial(rest).map(Rcpt::Forward)
   } else {
      None
   }
}

#[cfg(test)]
#[expect(clippy::inline_modules, reason = "unit tests kept beside impl")]
mod tests {
   use super::*;

   #[test]
   fn parses_forward() {
      assert_eq!(
         parse_rcpt("rampart-42@internal.rampart.lmtp", "internal.rampart.lmtp"),
         Some(Rcpt::Forward(42))
      );
   }

   #[test]
   fn parses_reply() {
      assert_eq!(
         parse_rcpt(
            "rampart-reply-77@internal.rampart.lmtp",
            "internal.rampart.lmtp"
         ),
         Some(Rcpt::Reply(77))
      );
   }

   #[test]
   fn parses_bounce_payload_opaque() {
      // The recipient parser doesn't validate the HMAC — verp.rs does
      // that. We just check the wrapper extracted the rest.
      assert_eq!(
         parse_rcpt(
            "rampart-bnc-f+12+abcDEF@internal.rampart.lmtp",
            "internal.rampart.lmtp"
         ),
         Some(Rcpt::Bounce {
            payload: "f+12+abcDEF".into(),
         })
      );
   }

   #[test]
   fn rejects_bnc_with_no_payload() {
      assert_eq!(
         parse_rcpt(
            "rampart-bnc-@internal.rampart.lmtp",
            "internal.rampart.lmtp"
         ),
         None
      );
   }

   #[test]
   fn rejects_unknown_local() {
      assert_eq!(
         parse_rcpt("foo@internal.rampart.lmtp", "internal.rampart.lmtp"),
         None
      );
   }

   #[test]
   fn rejects_wrong_domain() {
      assert_eq!(
         parse_rcpt("rampart-1@elsewhere.test", "internal.rampart.lmtp"),
         None
      );
   }

   #[test]
   fn rejects_non_numeric() {
      assert_eq!(
         parse_rcpt("rampart-abc@internal.rampart.lmtp", "internal.rampart.lmtp"),
         None
      );
   }

   #[test]
   fn rejects_non_positive() {
      assert_eq!(
         parse_rcpt("rampart-0@internal.rampart.lmtp", "internal.rampart.lmtp"),
         None
      );
      assert_eq!(
         parse_rcpt("rampart--1@internal.rampart.lmtp", "internal.rampart.lmtp"),
         None
      );
      assert_eq!(
         parse_rcpt(
            "rampart-reply-0@internal.rampart.lmtp",
            "internal.rampart.lmtp"
         ),
         None
      );
   }

   #[test]
   fn rejects_u64_over_i64_max() {
      // u64::MAX — would wrap to -1 on a naive cast to i64
      let over = format!("rampart-{}@internal.rampart.lmtp", u64::MAX);
      assert_eq!(parse_rcpt(&over, "internal.rampart.lmtp"), None);
   }
}
