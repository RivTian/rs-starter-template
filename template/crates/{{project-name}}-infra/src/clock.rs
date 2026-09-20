//! The real clock.

use {{crate_name}}_domain::ports::Clock;
use time::OffsetDateTime;

/// Reads the system clock in UTC.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
