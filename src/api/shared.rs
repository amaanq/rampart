//! Shared helpers used across the api submodules: validators, error
//! mapping, common constants, the password hasher.

use serde::Deserialize;
use tokio_postgres::error::SqlState;

use crate::error::{ApiError, ApiResult};

pub(crate) const PAGE_SIZE: i64 = 50;

pub(crate) fn is_unique_violation(e: &tokio_postgres::Error) -> bool {
    e.as_db_error()
        .map(|d| d.code() == &SqlState::UNIQUE_VIOLATION)
        .unwrap_or(false)
}

pub(crate) fn is_fk_violation(e: &tokio_postgres::Error) -> bool {
    e.as_db_error()
        .map(|d| d.code() == &SqlState::FOREIGN_KEY_VIOLATION)
        .unwrap_or(false)
}

/// Translate a Postgres trigger / CHECK violation into a 400 with the
/// trigger's message — gives operators a usable error instead of an
/// opaque generic CHECK_VIOLATION.
pub(crate) fn raise_exception_as_bad_request(e: tokio_postgres::Error) -> ApiError {
    if let Some(db) = e.as_db_error() {
        let code = db.code();
        if code == &SqlState::RAISE_EXCEPTION || code == &SqlState::CHECK_VIOLATION {
            return ApiError::BadRequest(db.message().to_owned());
        }
    }
    ApiError::Db(e)
}

/// Mirrors the schema's `domain_shape` CHECK so the API returns a 400
/// instead of a generic CHECK_VIOLATION on bad input.
pub(crate) fn validate_domain(s: &str) -> Result<(), ApiError> {
    let len = s.len();
    if !(3..=253).contains(&len) {
        return Err(ApiError::BadRequest(
            "domain length must be 3..=253 chars".into(),
        ));
    }
    let mut labels = s.split('.').peekable();
    if labels.peek().is_none() {
        return Err(ApiError::BadRequest("domain must contain a dot".into()));
    }
    let mut label_count = 0;
    for label in labels {
        label_count += 1;
        if label.is_empty() || label.len() > 63 {
            return Err(ApiError::BadRequest(
                "domain label must be 1..=63 chars".into(),
            ));
        }
        let bytes = label.as_bytes();
        if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
            return Err(ApiError::BadRequest(
                "domain label must not start or end with hyphen".into(),
            ));
        }
        if !label
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(ApiError::BadRequest(
                "domain must be lowercase ASCII letters, digits, hyphens (CITEXT-folded)".into(),
            ));
        }
    }
    if label_count < 2 {
        return Err(ApiError::BadRequest(
            "domain must have at least one dot".into(),
        ));
    }
    // Reserved: the synthetic LMTP-routing domain used by the Sieve to
    // hand inbound off to the worker. A user-created alias_domain with
    // this name would let routes collide.
    if s.eq_ignore_ascii_case("internal.rampart.lmtp") {
        return Err(ApiError::BadRequest(
            "domain 'internal.rampart.lmtp' is reserved".into(),
        ));
    }
    Ok(())
}

/// `ra+` (reverse-alias) and `bnc+` (bounce VERP) are Sieve-routed
/// namespaces. User local-parts must not start with them or they
/// hijack the routing path.
const RESERVED_LOCAL_PART_PREFIXES: &[&str] = &["ra+", "bnc+"];

fn lowercase_starts_with_reserved(s: &str) -> Option<&'static str> {
    let lower = s.to_ascii_lowercase();
    RESERVED_LOCAL_PART_PREFIXES
        .iter()
        .copied()
        .find(|p| lower.starts_with(p))
}

/// Validate a local-part component pre-concatenation (alias prefix,
/// alias suffix, alias_domain.random_prefix). The DB-side
/// `alias_validate` trigger mirrors the post-concat shape for direct-
/// SQL inserts.
pub(crate) fn validate_local_part_fragment(s: &str, name: &str) -> Result<(), ApiError> {
    if s.is_empty() {
        return Err(ApiError::BadRequest(format!("{name} must be non-empty")));
    }
    if s.len() > 64 {
        return Err(ApiError::BadRequest(format!("{name} exceeds 64 bytes")));
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-_+".contains(c))
    {
        return Err(ApiError::BadRequest(format!(
            "{name} must be ASCII alphanumeric or [.-_+]"
        )));
    }
    if s.starts_with('.') || s.ends_with('.') {
        return Err(ApiError::BadRequest(format!(
            "{name} must not start or end with '.'"
        )));
    }
    if s.contains("..") {
        return Err(ApiError::BadRequest(format!(
            "{name} must not contain consecutive dots"
        )));
    }
    if let Some(reserved) = lowercase_starts_with_reserved(s) {
        return Err(ApiError::BadRequest(format!(
            "{name} must not start with reserved prefix '{reserved}'"
        )));
    }
    Ok(())
}

/// random_prefix has a tighter 54-byte cap because alias_random
/// appends 10 hex chars; the final local-part stays within 64 bytes.
/// Empty disables random alias minting entirely.
pub(crate) fn validate_random_prefix(s: &str) -> Result<(), ApiError> {
    if s.is_empty() {
        return Ok(());
    }
    validate_local_part_fragment(s, "random_prefix")?;
    if s.len() > 54 {
        return Err(ApiError::BadRequest(
            "random_prefix exceeds 54 bytes (10 hex chars are appended on creation, total cap 64)"
                .into(),
        ));
    }
    Ok(())
}

pub(crate) fn trimmed_nonempty(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Distinguishes "field absent in JSON" from "field present and null"
/// for PATCH semantics: `Some(None)` is "set to null", `None` is
/// "leave unchanged".
pub(crate) fn deserialize_opt_field<'de, D, T>(d: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(d).map(Some)
}

pub(crate) fn hash_password(password: &str) -> ApiResult<String> {
    crate::auth::hash_password(password).map_err(ApiError::Internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_optional_user_text() {
        assert_eq!(
            trimmed_nonempty(Some("  useful  ".into())).as_deref(),
            Some("useful")
        );
        assert_eq!(trimmed_nonempty(Some("   ".into())), None);
        assert_eq!(trimmed_nonempty(None), None);
    }
}
