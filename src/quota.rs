//! Per-user resource caps and the advisory-lock keys that protect
//! cap-check-then-insert from races. Lock classes are encoded into the
//! top 8 bits of the lock key so different resource types don't block
//! each other.

/// Default per-user alias cap when `user.max_aliases` is NULL.
///
/// Duplicated as the literal `200` in `rampart_resolve_or_create`'s plpgsql
/// body in V001 (the catch-all path enforces the same cap from the Sieve
/// side, where this Rust constant isn't reachable). Keep the two in sync —
/// search V001 for `COALESCE(max_aliases::bigint, 200)`.
pub const DEFAULT_MAX_ALIASES: i64 = 200;
/// Default per-user alias-domain cap when `user.max_domains` is NULL.
pub const DEFAULT_MAX_DOMAINS: i64 = 5;

/// Advisory-lock class IDs. Different classes don't block each other.
pub const LOCK_CLASS_ALIAS_CAP: u8 = 1;
pub const LOCK_CLASS_DOMAIN_CAP: u8 = 2;

/// Compose `pg_advisory_xact_lock(bigint)` arg as (class << 56) | `user_id`.
/// Top 8 bits = lock class, bottom 56 bits = `user_id`.
///
/// User IDs >= 2^56 would alias against another user's lock; BIGSERIAL
/// goes to 2^63-1 so this is ~13 orders of magnitude past
/// friend-pool scale. If rampart ever runs as a public service, switch to
/// a hash-derived lock key.
#[expect(
   clippy::cast_sign_loss,
   clippy::cast_possible_wrap,
   reason = "deliberate bit reinterpretation to pack lock class and user id into one i64"
)]
pub const fn lock_id(class: u8, user_id: i64) -> i64 {
   let user_low = (user_id as u64) & 0x00FF_FFFF_FFFF_FFFF;
   (((class as u64) << 56) | user_low) as i64
}
