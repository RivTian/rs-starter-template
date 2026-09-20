use acme_svc_core::health::HealthReport;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::state::AppState;

/// `/healthz` body.
#[derive(Serialize)]
pub struct Liveness {
    status: &'static str,
}

/// `GET /healthz` — liveness: the process is up. Always `200` while we can answer at all.
pub async fn live() -> Json<Liveness> {
    Json(Liveness { status: "ok" })
}

/// `/readyz` body while the lifecycle does not accept work.
#[derive(Serialize)]
pub struct NotReady {
    status: &'static str,
    phase: acme_svc_core::server::Phase,
}

/// `GET /readyz` — readiness: `200` only when the lifecycle accepts work **and** every health
/// check passes; `503` otherwise, so load balancers stop routing to us while we drain.
pub async fn ready(State(state): State<AppState>) -> Response {
    let phase = *state.phase.borrow();
    if !phase.accepts_work() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(NotReady {
                status: "not_ready",
                phase,
            }),
        )
            .into_response();
    }
    let report: HealthReport = state.health.report().await;
    let status = if report.is_ready() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(report)).into_response()
}
