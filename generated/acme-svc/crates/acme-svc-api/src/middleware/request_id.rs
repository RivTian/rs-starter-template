use std::time::Instant;

use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;

use acme_svc_util::id;

use crate::context::RequestContext;

/// The correlation header, honoured inbound and always set outbound.
pub const HEADER: HeaderName = HeaderName::from_static("x-request-id");

const MAX_LEN: usize = 128;

/// Attaches a [`RequestContext`] to the request and echoes the request id on the response.
pub async fn request_id(mut request: Request, next: Next) -> Response {
    let request_id = request
        .headers()
        .get(&HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty() && value.len() <= MAX_LEN && value.is_ascii())
        .map_or_else(|| id::new_v7().to_string(), str::to_owned);

    request.extensions_mut().insert(RequestContext {
        request_id: request_id.clone(),
        started_at: Instant::now(),
    });

    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(HEADER, value);
    }
    response
}
