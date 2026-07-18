//! Shared helpers used across the api submodules: validators, error
//! mapping, common constants, the password hasher.

use serde::Deserialize;
use tokio_postgres::error::SqlState;

use crate::{
   auth,
   error::{
      ApiError,
      ApiResult,
   },
};

pub const PAGE_SIZE: i64 = 50;

pub fn is_unique_violation(err: &tokio_postgres::Error) -> bool {
   err.as_db_error()
      .is_some_and(|db| db.code() == &SqlState::UNIQUE_VIOLATION)
}

pub fn is_fk_violation(err: &tokio_postgres::Error) -> bool {
   err.as_db_error()
      .is_some_and(|db| db.code() == &SqlState::FOREIGN_KEY_VIOLATION)
}

/// Translate a Postgres trigger / CHECK violation into a 400 with the
/// trigger's message — gives operators a usable error instead of an
/// opaque generic `CHECK_VIOLATION`.
pub fn raise_exception_as_bad_request(err: tokio_postgres::Error) -> ApiError {
   if let Some(db) = err.as_db_error() {
      let code = db.code();
      if code == &SqlState::RAISE_EXCEPTION || code == &SqlState::CHECK_VIOLATION {
         return ApiError::BadRequest(db.message().to_owned());
      }
   }
   ApiError::Db(err)
}

/// Mirrors the schema's `domain_shape` CHECK so the API returns a 400
/// instead of a generic `CHECK_VIOLATION` on bad input.
pub fn validate_domain(domain: &str) -> Result<(), ApiError> {
   let len = domain.len();
   if !(3..=253).contains(&len) {
      return Err(ApiError::BadRequest(
         "domain length must be 3..=253 chars".into(),
      ));
   }
   let mut labels = domain.split('.').peekable();
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
         .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
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
   if domain.eq_ignore_ascii_case("internal.rampart.lmtp") {
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

fn lowercase_starts_with_reserved(local: &str) -> Option<&'static str> {
   let lower = local.to_ascii_lowercase();
   RESERVED_LOCAL_PART_PREFIXES
      .iter()
      .copied()
      .find(|prefix| lower.starts_with(prefix))
}

/// Validate a local-part component pre-concatenation (alias prefix,
/// alias suffix, `alias_domain.random_prefix`). The DB-side
/// `alias_validate` trigger mirrors the post-concat shape for direct-
/// SQL inserts.
pub fn validate_local_part_fragment(fragment: &str, name: &str) -> Result<(), ApiError> {
   if fragment.is_empty() {
      return Err(ApiError::BadRequest(format!("{name} must be non-empty")));
   }
   if fragment.len() > 64 {
      return Err(ApiError::BadRequest(format!("{name} exceeds 64 bytes")));
   }
   if !fragment
      .chars()
      .all(|ch| ch.is_ascii_alphanumeric() || ".-_+".contains(ch))
   {
      return Err(ApiError::BadRequest(format!(
         "{name} must be ASCII alphanumeric or [.-_+]"
      )));
   }
   if fragment.starts_with('.') || fragment.ends_with('.') {
      return Err(ApiError::BadRequest(format!(
         "{name} must not start or end with '.'"
      )));
   }
   if fragment.contains("..") {
      return Err(ApiError::BadRequest(format!(
         "{name} must not contain consecutive dots"
      )));
   }
   if let Some(reserved) = lowercase_starts_with_reserved(fragment) {
      return Err(ApiError::BadRequest(format!(
         "{name} must not start with reserved prefix '{reserved}'"
      )));
   }
   Ok(())
}

/// `random_prefix` has a tighter 54-byte cap because `alias_random`
/// appends 10 hex chars; the final local-part stays within 64 bytes.
/// Empty disables random alias minting entirely.
pub fn validate_random_prefix(prefix: &str) -> Result<(), ApiError> {
   if prefix.is_empty() {
      return Ok(());
   }
   validate_local_part_fragment(prefix, "random_prefix")?;
   if prefix.len() > 54 {
      return Err(ApiError::BadRequest(
         "random_prefix exceeds 54 bytes (10 hex chars are appended on creation, total cap 64)"
            .into(),
      ));
   }
   Ok(())
}

pub fn trimmed_nonempty(value: Option<String>) -> Option<String> {
   value
      .map(|value| value.trim().to_owned())
      .filter(|value| !value.is_empty())
}

/// Distinguishes "field absent in JSON" from "field present and null"
/// for PATCH semantics: `Some(None)` is "set to null", `None` is
/// "leave unchanged".
#[expect(
   clippy::option_option,
   reason = "Some(None) means set-to-null; None means field absent in PATCH body"
)]
pub fn deserialize_opt_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
   D: serde::Deserializer<'de>,
   T: Deserialize<'de>,
{
   Option::<T>::deserialize(deserializer).map(Some)
}

pub fn hash_password(password: &str) -> ApiResult<String> {
   auth::hash_password(password).map_err(ApiError::Internal)
}

#[cfg(test)]
#[expect(
   clippy::inline_modules,
   reason = "small cohesive submodule kept inline"
)]
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
