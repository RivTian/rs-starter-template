//! A clock tests can move by hand.

use std::sync::Mutex;
use std::time::Duration;

use time::OffsetDateTime;

use acme_svc_domain::ports::Clock;

/// Starts at a fixed instant; advances only when told to.
#[derive(Debug)]
pub struct FakeClock {
    now: Mutex<OffsetDateTime>,
}

impl FakeClock {
    /// A clock frozen at `start`.
    #[must_use]
    pub const fn new(start: OffsetDateTime) -> Self {
        Self {
            now: Mutex::new(start),
        }
    }

    /// A clock frozen at the Unix epoch.
    #[must_use]
    pub const fn at_epoch() -> Self {
        Self::new(OffsetDateTime::UNIX_EPOCH)
    }

    /// Moves time forward.
    pub fn advance(&self, by: Duration) {
        let mut now = self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *now += by;
    }

    /// Sets the time.
    pub fn set(&self, to: OffsetDateTime) {
        *self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = to;
    }
}

impl Clock for FakeClock {
    fn now(&self) -> OffsetDateTime {
        *self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}
