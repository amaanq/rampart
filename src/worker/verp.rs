//! Bounce VERP signing — `MAIL FROM = bnc+{phase}+{email_log_id}+{tag}@<alias_domain>`,
//! where `tag` is URL-safe-base64 of `HMAC-SHA256(verp_key, "{phase}|{id}")` truncated
//! to 12 bytes. The Sieve template re-emits the recipient as
//! `rampart-bnc-{phase}+{id}+{tag}@internal.rampart.lmtp`; the worker verifies the tag
//! before mutating the named email_log row.
//!
//! 96 bits suffices against online forgery; the key only lives in the worker
//! via LoadCredential. No expiry baked in — `handle_bounce` only mutates rows
//! with `status IN ('pending','submitted')`.

use data_encoding::BASE64URL_NOPAD;
use hmac_sha256::HMAC;

use crate::worker::loop_guard::BouncePhase;

const TAG_LEN: usize = 12;

fn phase_byte(phase: BouncePhase) -> &'static str {
    match phase {
        BouncePhase::Forward => "f",
        BouncePhase::Reply => "r",
    }
}

fn mac(key: &[u8], phase: BouncePhase, email_log_id: i64) -> [u8; TAG_LEN] {
    let mut m = HMAC::new(key);
    m.update(phase_byte(phase).as_bytes());
    m.update(b"|");
    m.update(email_log_id.to_string().as_bytes());
    let out = m.finalize();
    let mut tag = [0u8; TAG_LEN];
    tag.copy_from_slice(&out[..TAG_LEN]);
    tag
}

fn b64_encode(bytes: &[u8]) -> String {
    BASE64URL_NOPAD.encode(bytes)
}

fn b64_decode(s: &str) -> Option<Vec<u8>> {
    BASE64URL_NOPAD.decode(s.as_bytes()).ok()
}

/// Inner local-part — pipeline prepends `bnc+` and appends `@<alias_domain>`.
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
pub fn verify_payload(key: &[u8], payload: &str) -> Option<(BouncePhase, i64)> {
    // b64url uses `[A-Za-z0-9_-]`, so the tag contains no `+` and splitn(3) is unambiguous.
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
    let email_log_id: i64 = id_s.parse().ok().filter(|v: &i64| *v > 0)?;
    let expected = mac(key, phase, email_log_id);
    let got = b64_decode(tag_s)?;
    if got.len() != TAG_LEN {
        return None;
    }
    if constant_time_eq::constant_time_eq(&got, &expected) {
        Some((phase, email_log_id))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k() -> &'static [u8] {
        b"test-key-32-bytes-long-padding-padding"
    }

    #[test]
    fn roundtrip_forward() {
        let p = make_local_payload(k(), BouncePhase::Forward, 42);
        assert!(p.starts_with("f+42+"));
        assert_eq!(verify_payload(k(), &p), Some((BouncePhase::Forward, 42)));
    }

    #[test]
    fn roundtrip_reply() {
        let p = make_local_payload(k(), BouncePhase::Reply, 9_999_999);
        assert!(p.starts_with("r+9999999+"));
        assert_eq!(
            verify_payload(k(), &p),
            Some((BouncePhase::Reply, 9_999_999))
        );
    }

    #[test]
    fn rejects_tampered_id() {
        let p = make_local_payload(k(), BouncePhase::Forward, 42);
        let bad = p.replacen("+42+", "+43+", 1);
        assert!(verify_payload(k(), &bad).is_none());
    }

    #[test]
    fn rejects_tampered_phase() {
        let p = make_local_payload(k(), BouncePhase::Forward, 42);
        let bad = p.replacen("f+", "r+", 1);
        assert!(verify_payload(k(), &bad).is_none());
    }

    #[test]
    fn rejects_wrong_key() {
        let p = make_local_payload(k(), BouncePhase::Forward, 42);
        let bad_key: &[u8] = b"different-key-32-bytes-aaaaaaaaaaaaaa";
        assert!(verify_payload(bad_key, &p).is_none());
    }

    #[test]
    fn rejects_malformed() {
        assert!(verify_payload(k(), "").is_none());
        assert!(verify_payload(k(), "f+42").is_none());
        assert!(verify_payload(k(), "f++").is_none());
        assert!(verify_payload(k(), "x+42+aaaa").is_none());
        assert!(verify_payload(k(), "f+0+aaaa").is_none());
    }
}
