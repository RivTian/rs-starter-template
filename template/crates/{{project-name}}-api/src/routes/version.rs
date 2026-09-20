use axum::Json;
use axum::extract::State;

use {{crate_name}}_core::build_info::BuildInfo;

use crate::state::AppState;

/// `GET /version` — build provenance.
pub async fn get(State(state): State<AppState>) -> Json<BuildInfo> {
    Json(state.build)
}
