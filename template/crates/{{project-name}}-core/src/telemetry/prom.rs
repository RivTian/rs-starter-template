use std::time::Duration;

use async_trait::async_trait;
use metrics::{describe_counter, describe_gauge, describe_histogram, gauge};
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder, PrometheusHandle};
use tokio::time::MissedTickBehavior;

use crate::readiness::Readiness;
use crate::service::{Service, ServiceError};
use crate::shutdown::ShutdownToken;

pub(super) fn install() -> Result<PrometheusHandle, BuildError> {
    let handle = PrometheusBuilder::new().install_recorder()?;
    describe();
    Ok(handle)
}

/// Metric descriptions show up as `# HELP` lines. Keep this list in sync with the emitters.
fn describe() {
    describe_counter!(
        "http_requests_total",
        "HTTP requests by method, route and status"
    );
    describe_histogram!(
        "http_request_duration_seconds",
        "HTTP request latency by method and route"
    );
    describe_counter!("domain_events_total", "Domain events published, by type");
    describe_counter!("events_lagged_total", "Events skipped by slow subscribers");
    describe_gauge!("tokio_workers", "Tokio worker threads");
    describe_gauge!("tokio_alive_tasks", "Tokio tasks currently alive");
    describe_gauge!(
        "tokio_global_queue_depth",
        "Tasks waiting in the runtime's global queue"
    );
}

/// Samples Tokio runtime gauges and runs exporter upkeep on a fixed interval.
pub struct RuntimeMetricsService {
    interval: Duration,
    handle: Option<PrometheusHandle>,
}

impl RuntimeMetricsService {
    /// A sampler; `handle` enables periodic upkeep of the Prometheus exporter.
    #[must_use]
    pub const fn new(interval: Duration, handle: Option<PrometheusHandle>) -> Self {
        Self { interval, handle }
    }

    fn sample() {
        let m = tokio::runtime::Handle::current().metrics();
        gauge!("tokio_workers").set(m.num_workers() as f64);
        gauge!("tokio_alive_tasks").set(m.num_alive_tasks() as f64);
        gauge!("tokio_global_queue_depth").set(m.global_queue_depth() as f64);
    }
}

#[async_trait]
impl Service for RuntimeMetricsService {
    fn name(&self) -> &'static str {
        "runtime-metrics"
    }

    async fn run(
        self: Box<Self>,
        shutdown: ShutdownToken,
        ready: Readiness,
    ) -> Result<(), ServiceError> {
        ready.ready();
        let mut ticker = tokio::time::interval(self.interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        loop {
            tokio::select! {
                () = shutdown.cancelled() => return Ok(()),
                _ = ticker.tick() => {
                    Self::sample();
                    if let Some(handle) = &self.handle {
                        handle.run_upkeep();
                    }
                }
            }
        }
    }
}
