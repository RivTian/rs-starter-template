//! What handlers can reach. Cloned per request by axum, so everything inside is a cheap handle.

use std::sync::Arc;

use {{crate_name}}_core::build_info::BuildInfo;
use {{crate_name}}_core::health::HealthRegistry;
use {{crate_name}}_core::server::PhaseWatch;
use {{crate_name}}_core::telemetry::PrometheusHandle;
use {{crate_name}}_domain::todo::TodoService;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// The todo use cases.
    pub todos: Arc<TodoService>,
    /// Health checks behind `/readyz`.
    pub health: HealthRegistry,
    /// Lifecycle phase; `/readyz` answers 503 while draining.
    pub phase: PhaseWatch,
    /// Prometheus renderer; `None` when metrics are disabled.
    pub metrics: Option<PrometheusHandle>,
    /// Build provenance for `/version`.
    pub build: BuildInfo,
}
