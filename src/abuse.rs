//! Rate limiting for abuse-prone endpoints — login, forgot-password,
//! email-change, verify-resend, invite, alias/domain creates.
//!
//! Sliding-window-ish implementation via the `rate_limit_bucket` table.
//! Each key has a `window_start` and a `count`. On each hit we check
//! whether `window_start` is older than the window; if yes, reset.
//! Otherwise increment and reject on cap.

use std::time::Duration;

use anyhow::Result;
use deadpool_postgres::Pool;
use rampart_codegen::queries::rate_limit;
use time::OffsetDateTime;

#[derive(Clone, Copy)]
pub struct Limit {
   pub window: Duration,
   pub max:    i32,
}

pub const FORGOT_PASSWORD: Limit = Limit {
   window: Duration::from_hours(1),
   max:    5,
};
pub const MAILBOX_VERIFY_RESEND: Limit = Limit {
   window: Duration::from_hours(1),
   max:    3,
};
pub const LOGIN_FAIL: Limit = Limit {
   window: Duration::from_mins(10),
   max:    10,
};
pub const EMAIL_CHANGE: Limit = Limit {
   window: Duration::from_hours(1),
   max:    3,
};
pub const RESET_APPLY: Limit = Limit {
   window: Duration::from_hours(1),
   max:    20,
};
pub const DOMAIN_DNS_CHECK: Limit = Limit {
   window: Duration::from_mins(1),
   max:    20,
};

/// Returns `Ok(true)` if the hit is allowed (within cap), `Ok(false)` if
/// throttled.
pub async fn check(pool: &Pool, key: &str, limit: Limit) -> Result<bool> {
   let client = pool.get().await?;
   let now = OffsetDateTime::now_utc();
   let window_secs = i64::try_from(limit.window.as_secs()).expect("rate-limit window fits in i64");
   let window_start_min = now - time::Duration::seconds(window_secs);
   let count = rate_limit::check()
      .bind(&client, &key, &now, &window_start_min)
      .one()
      .await?;
   Ok(count <= limit.max)
}

/// Clear a bucket — useful after a successful login to discard
/// accumulated failure counts.
pub async fn clear(pool: &Pool, key: &str) -> Result<()> {
   let client = pool.get().await?;
   rate_limit::clear().bind(&client, &key).await?;
   Ok(())
}
