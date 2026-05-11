//! Rate limiting for abuse-prone endpoints — login, forgot-password,
//! email-change, verify-resend, invite, alias/domain creates.
//!
//! Sliding-window-ish implementation via the `rate_limit_bucket` table.
//! Each key has a `window_start` and a `count`. On each hit we check
//! whether `window_start` is older than the window; if yes, reset.
//! Otherwise increment and reject on cap.

use anyhow::Result;
use deadpool_postgres::Pool;
use rampart_codegen::queries::rate_limit;
use std::time::Duration;
use time::OffsetDateTime;

#[derive(Clone, Copy)]
pub struct Limit {
    pub window: Duration,
    pub max: i32,
}

pub const FORGOT_PASSWORD: Limit = Limit {
    window: Duration::from_secs(3600),
    max: 5,
};
pub const MAILBOX_VERIFY_RESEND: Limit = Limit {
    window: Duration::from_secs(3600),
    max: 3,
};
pub const LOGIN_FAIL: Limit = Limit {
    window: Duration::from_secs(600),
    max: 10,
};
pub const EMAIL_CHANGE: Limit = Limit {
    window: Duration::from_secs(3600),
    max: 3,
};
pub const RESET_APPLY: Limit = Limit {
    window: Duration::from_secs(3600),
    max: 20,
};

/// Returns `Ok(true)` if the hit is allowed (within cap), `Ok(false)` if throttled.
pub async fn check(pool: &Pool, key: &str, limit: Limit) -> Result<bool> {
    let c = pool.get().await?;
    let now = OffsetDateTime::now_utc();
    let window_start_min = now - time::Duration::seconds(limit.window.as_secs() as i64);
    let count = rate_limit::check()
        .bind(&c, &key, &now, &window_start_min)
        .one()
        .await?;
    Ok(count <= limit.max)
}

/// Clear a bucket — useful after a successful login to discard
/// accumulated failure counts.
pub async fn clear(pool: &Pool, key: &str) -> Result<()> {
    let c = pool.get().await?;
    rate_limit::clear().bind(&c, &key).await?;
    Ok(())
}
