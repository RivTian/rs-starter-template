//! Per-request context, created by the request-id middleware and available to every handler.

use std::time::Instant;

use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;

/// Facts about the current request that every layer may want.
#[derive(Clone, Debug)]
pub struct RequestContext {
    /// Correlation id: taken from the `x-request-id` request header or generated.
    pub request_id: String,
    /// When the request entered the middleware stack.
    pub started_at: Instant,
}

impl<S: Send + Sync> FromRequestParts<S> for RequestContext {
    type Rejection = StatusCode;

    // Nothing to await: the middleware already put the context into the extensions.
    fn from_request_parts(
        parts: &mut Parts,
        _: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        std::future::ready(
            parts
                .extensions
                .get::<Self>()
                .cloned()
                .ok_or(StatusCode::INTERNAL_SERVER_ERROR),
        )
    }
}
