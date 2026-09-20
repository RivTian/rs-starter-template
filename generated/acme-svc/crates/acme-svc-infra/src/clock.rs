//! The real clock.

use time::OffsetDateTime;

use acme_svc_domain::ports::Clock;

/// Reads the system clock in UTC.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
