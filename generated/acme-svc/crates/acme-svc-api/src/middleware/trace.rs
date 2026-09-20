use std::time::Duration;

use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::{Request, Response};
use tracing::{Span, field};

use crate::context::RequestContext;

/// One span per request, named `http.request`, carrying the correlation id from the start.
pub fn make_span(request: &Request<Body>) -> Span {
    let request_id = request
        .extensions()
        .get::<RequestContext>()
        .map_or("-", |c| c.request_id.as_str());
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or("unmatched", MatchedPath::as_str);
    tracing::info_span!(
        "http.request",
        method = %request.method(),
        path = %request.uri().path(),
        route,
        request_id,
        status = field::Empty,
        latency_ms = field::Empty,
    )
}

/// Records the outcome on the span and emits the access-log line.
pub fn on_response(response: &Response<Body>, latency: Duration, span: &Span) {
    let status = response.status().as_u16();
    let latency_ms = u64::try_from(latency.as_millis()).unwrap_or(u64::MAX);
    span.record("status", status);
    span.record("latency_ms", latency_ms);
    tracing::info!(parent: span, status, latency_ms, "request completed");
}
