use {{crate_name}}_core::error::ErrorKind;
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;
use crate::state::AppState;

/// `GET /metrics` — Prometheus text exposition, or `404` when metrics are disabled.
pub async fn get(State(state): State<AppState>) -> Response {
    match &state.metrics {
        Some(handle) => (
            [(
                header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            handle.render(),
        )
            .into_response(),
        None => ApiError::with_detail(ErrorKind::NotFound, "metrics are disabled").into_response(),
    }
}
