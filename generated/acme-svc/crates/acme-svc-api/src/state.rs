//! What handlers can reach. Cloned per request by axum, so everything inside is a cheap handle.

use std::sync::Arc;

use acme_svc_core::build_info::BuildInfo;
use acme_svc_core::health::HealthRegistry;
use acme_svc_core::server::PhaseWatch;
use acme_svc_core::telemetry::PrometheusHandle;
use acme_svc_domain::todo::TodoService;

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
