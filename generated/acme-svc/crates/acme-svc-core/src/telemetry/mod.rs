//! Logs, metrics and the panic hook.
//!
//! Order at startup: [`install_panic_hook`] first (before anything can panic), then the
//! runtime, then [`init`] inside the runtime. `init` must run exactly once per process.

mod logging;
mod panic_hook;
mod prom;

use acme_svc_config::TelemetryConfig;

pub use self::panic_hook::install_panic_hook;
pub use self::prom::RuntimeMetricsService;
pub use metrics_exporter_prometheus::PrometheusHandle;

/// Handles produced by [`init`].
pub struct Telemetry {
    /// Renders the Prometheus text exposition; `None` when metrics are disabled.
    pub metrics: Option<PrometheusHandle>,
}

/// Telemetry could not be initialised.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// The log filter directive is malformed.
    #[error("invalid log filter directive")]
    Filter(#[from] tracing_subscriber::filter::ParseError),
    /// A global subscriber was already installed (calling `init` twice).
    #[error("tracing subscriber already installed")]
    Subscriber(#[from] tracing_subscriber::util::TryInitError),
    /// The metrics recorder could not be installed.
    #[error("metrics recorder could not be installed")]
    Metrics(#[from] metrics_exporter_prometheus::BuildError),
}

/// Installs the global `tracing` subscriber and, if enabled, the Prometheus recorder.
///
/// `log_env_var` is the project-specific filter variable (e.g. `ACME_SVC_LOG`); it wins over
/// `RUST_LOG`, which wins over the configured filter.
pub fn init(cfg: &TelemetryConfig, log_env_var: &str) -> Result<Telemetry, TelemetryError> {
    logging::init(cfg, log_env_var)?;
    let metrics = if cfg.metrics {
        Some(prom::install()?)
    } else {
        None
    };
    Ok(Telemetry { metrics })
}
