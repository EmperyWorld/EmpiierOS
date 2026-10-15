//! Wall-clock helpers.

use std::time::{SystemTime, UNIX_EPOCH};

#[must_use]
pub fn now_epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}
