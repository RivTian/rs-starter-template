//! Shared, dependency-light building blocks for every other `acme-svc` crate.
//!
//! Admission rule (keep this crate small): a module lives here only if
//! **at least two crates need it** and it is **pure** — no I/O, no runtime, no business rules.

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
pub mod id;
pub mod secret;

/// The handful of names most crates want in scope.
pub mod prelude {
    pub use crate::error::{Classify, ErrorKind, ErrorSource, Severity};
    pub use crate::secret::Secret;
}
