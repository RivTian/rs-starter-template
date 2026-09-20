//! The business model, and nothing else.
//!
//! * **Entities and value objects** enforce their own invariants (`Title::new` cannot produce an
//!   empty title).
//! * **Ports** are traits the outside world implements (`TodoRepository`, `Clock`,
//!   `EventPublisher`); this crate never implements them with real I/O.
//! * **Domain events** are facts in the past tense.
//!
//! Dependency rule: no Tokio, no HTTP, no database — only `std`, `serde`, `time`, `uuid` and
//! the shared kernel. If you need I/O, you are in the wrong crate.
//!
//! The `todo` module is the worked example; delete it once you have your own.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]

pub mod error;
pub mod event;
pub mod ports;
pub mod todo;

/// The names most consumers want in scope.
pub mod prelude {
    pub use crate::error::{DomainError, RepositoryError};
    pub use crate::event::DomainEvent;
    pub use crate::ports::{Clock, EventPublisher};
    pub use crate::todo::{Title, Todo, TodoId, TodoRepository, TodoService};
}
