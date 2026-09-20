//! Handlers, grouped by resource. Handlers are thin: extract, call the domain, map the result.

pub mod health;
pub mod metrics;
pub mod todos;
pub mod version;

use acme_svc_core::error::ErrorKind;

use crate::error::ApiError;

/// Uniform `404` problem for unknown routes.
pub async fn not_found() -> ApiError {
    ApiError::with_detail(ErrorKind::NotFound, "no such route")
}
