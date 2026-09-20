//! The composition root, exposed as a library so integration tests can wire exactly the graph
//! the binary runs. Nothing here contains business logic; it only decides *which* adapters
//! plug into *which* ports and in what order services start.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]

pub mod bootstrap;
pub mod build_info;
pub mod cli;
pub mod probe;

/// Prefix for environment overrides: `ACME_SVC__<SECTION>__<KEY>`.
pub const ENV_PREFIX: &str = "ACME_SVC";
/// Log filter variable; wins over `RUST_LOG` and the configured filter.
pub const LOG_ENV_VAR: &str = "ACME_SVC_LOG";
