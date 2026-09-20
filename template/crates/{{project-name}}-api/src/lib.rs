//! HTTP transport: requests in, domain calls, responses out.
//!
//! Rules of this crate:
//! * no business rules — handlers validate shape (DTOs) and delegate to domain services;
//! * no adapters — it never constructs a repository or a clock; the composition root injects
//!   them through [`state::AppState`];
//! * every error leaves as RFC 9457 `application/problem+json` via [`error::ApiError`].
//!
//! Middleware order (outermost first): request id → trace span → metrics → timeout →
//! body limit → panic guard. See [`router::router`].

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]

pub mod context;
pub mod dto;
pub mod error;
pub mod middleware;
pub mod router;
pub mod routes;
pub mod service;
pub mod state;

/// The names the composition root and tests want in scope.
pub mod prelude {
    pub use crate::context::RequestContext;
    pub use crate::error::ApiError;
    pub use crate::router::router;
    pub use crate::service::HttpService;
    pub use crate::state::AppState;
}
