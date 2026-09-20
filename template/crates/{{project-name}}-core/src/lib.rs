//! Runtime core: everything a long-running service needs that has **nothing** to do with the
//! business. This crate never imports a domain type; it only defines traits (`Service`,
//! `HealthCheck`), handles (`ShutdownToken`, `Readiness`) and generic facilities (`EventBus<E>`).
//!
//! Start at [`server::Server`].

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]

pub mod build_info;
pub mod error;
pub mod event;
pub mod health;
pub mod prelude;
pub mod readiness;
pub mod runtime;
pub mod server;
pub mod service;
pub mod shutdown;
pub mod telemetry;
