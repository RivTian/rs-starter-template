use std::time::Duration;

use acme_svc_core::error::ErrorKind;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;

/// Fails the request with a `504` problem once the configured deadline passes.
pub async fn timeout(State(limit): State<Duration>, request: Request, next: Next) -> Response {
    tokio::time::timeout(limit, next.run(request))
        .await
        .unwrap_or_else(|_| {
            tracing::warn!(limit = ?limit, "request timed out");
            ApiError::with_detail(ErrorKind::Timeout, format!("request exceeded {limit:?}"))
                .into_response()
        })
}
