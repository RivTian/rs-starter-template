//! Adapters: concrete implementations of the domain's ports plus background jobs.
//!
//! This crate may depend on drivers (databases, queues, HTTP clients); the domain may not.
//! Every adapter translates driver errors into the domain's `RepositoryError` and never lets a
//! driver type leak upwards. Nothing here is referenced by `acme-svc-api`; only the composition
//! root sees both.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]

pub mod clock;
pub mod events;
pub mod jobs;
pub mod memory;

/// The names the composition root wants in scope.
pub mod prelude {
    pub use crate::clock::SystemClock;
    pub use crate::events::{EventBusPublisher, EventLogger};
    pub use crate::jobs::CleanupJob;
    pub use crate::memory::InMemoryTodoRepository;
}
