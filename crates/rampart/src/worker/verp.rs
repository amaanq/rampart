//! Bounce VERP signing — `MAIL FROM =
//! bnc+{phase}+{email_log_id}+{tag}@<alias_domain>`, where `tag` is
//! URL-safe-base64 of `HMAC-SHA256(verp_key, "{phase}|{id}")` truncated
//! to 12 bytes.
//!
//! The Sieve template re-emits the recipient as
//! `rampart-bnc-{phase}+{id}+{tag}@internal.rampart.lmtp`; the worker verifies
//! the tag before mutating the named `email_log` row.
//!
//! 96 bits suffices against online forgery; the key only lives in the worker
//! via `LoadCredential`. No expiry baked in — `handle_bounce` only mutates rows
//! with `status IN ('pending','submitted')`.

use data_encoding::BASE64URL_NOPAD;
use hmac_sha256::HMAC;

use crate::worker::loop_guard::BouncePhase;

const TAG_LEN: usize = 12;

const fn phase_byte(phase: BouncePhase) -> &'static str {
   match phase {
      BouncePhase::Forward => "f",
      BouncePhase::Reply => "r",
   }
}

fn mac(key: &[u8], phase: BouncePhase, email_log_id: i64) -> [u8; TAG_LEN] {
   let mut hmac = HMAC::new(key);
   hmac.update(phase_byte(phase).as_bytes());
   hmac.update(b"|");
   hmac.update(email_log_id.to_string().as_bytes());
   let out = hmac.finalize();
   let mut tag = [0_u8; TAG_LEN];
   tag.copy_from_slice(&out[..TAG_LEN]);
   tag
}

fn b64_encode(bytes: &[u8]) -> String {
   BASE64URL_NOPAD.encode(bytes)
}

fn b64_decode(encoded: &str) -> Option<Vec<u8>> {
   BASE64URL_NOPAD.decode(encoded.as_bytes()).ok()
}

/// Inner local-part — pipeline prepends `bnc+` and appends `@<alias_domain>`.
#[must_use]
pub fn make_local_payload(key: &[u8], phase: BouncePhase, email_log_id: i64) -> String {
   let tag = mac(key, phase, email_log_id);
   format!(
      "{}+{}+{}",
      phase_byte(phase),
      email_log_id,
      b64_encode(&tag)
   )
}

/// `(phase, email_log_id)` only if the tag matches — caller MUST treat
/// `None` as "do not mutate state".
#[must_use]
pub fn verify_payload(key: &[u8], payload: &str) -> Option<(BouncePhase, i64)> {
   // b64url uses `[A-Za-z0-9_-]`, so the tag contains no `+` and splitn(3) is
   // unambiguous.
   let mut parts = payload.splitn(3, '+');
   let phase_s = parts.next()?;
   let id_s = parts.next()?;
   let tag_s = parts.next()?;
   if tag_s.contains('+') {
      return None;
   }
   let phase = match phase_s {
      "f" => BouncePhase::Forward,
      "r" => BouncePhase::Reply,
      _ => return None,
   };
   let email_log_id: i64 = id_s.parse().ok().filter(|value: &i64| *value > 0)?;
   let expected = mac(key, phase, email_log_id);
   let got = b64_decode(tag_s)?;
   if got.len() != TAG_LEN {
      return None;
   }
   constant_time_eq::constant_time_eq(&got, &expected).then_some((phase, email_log_id))
}

#[cfg(test)]
#[expect(clippy::inline_modules, reason = "unit tests kept beside impl")]
mod tests {
   use super::*;

   fn key() -> &'static [u8] {
      b"test-key-32-bytes-long-padding-padding"
   }

   #[test]
   fn roundtrip_forward() {
      let payload = make_local_payload(key(), BouncePhase::Forward, 42);
      assert!(payload.starts_with("f+42+"));
      assert_eq!(
         verify_payload(key(), &payload),
         Some((BouncePhase::Forward, 42))
      );
   }

   #[test]
   fn roundtrip_reply() {
      let payload = make_local_payload(key(), BouncePhase::Reply, 9_999_999);
      assert!(payload.starts_with("r+9999999+"));
      assert_eq!(
         verify_payload(key(), &payload),
         Some((BouncePhase::Reply, 9_999_999))
      );
   }

   #[test]
   fn rejects_tampered_id() {
      let payload = make_local_payload(key(), BouncePhase::Forward, 42);
      let bad = payload.replacen("+42+", "+43+", 1);
      assert!(verify_payload(key(), &bad).is_none());
   }

   #[test]
   fn rejects_tampered_phase() {
      let payload = make_local_payload(key(), BouncePhase::Forward, 42);
      let bad = payload.replacen("f+", "r+", 1);
      assert!(verify_payload(key(), &bad).is_none());
   }

   #[test]
   fn rejects_wrong_key() {
      let payload = make_local_payload(key(), BouncePhase::Forward, 42);
      let bad_key: &[u8] = b"different-key-32-bytes-aaaaaaaaaaaaaa";
      assert!(verify_payload(bad_key, &payload).is_none());
   }

   #[test]
   fn rejects_malformed() {
      assert!(verify_payload(key(), "").is_none());
      assert!(verify_payload(key(), "f+42").is_none());
      assert!(verify_payload(key(), "f++").is_none());
      assert!(verify_payload(key(), "x+42+aaaa").is_none());
      assert!(verify_payload(key(), "f+0+aaaa").is_none());
   }
}
