use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::require_at_least;

/// Logs and metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TelemetryConfig {
    /// `pretty` for humans, `json` for log collectors.
    pub log_format: LogFormat,
    /// `tracing` filter directive used when neither `<PREFIX>_LOG` nor `RUST_LOG` is set.
    pub log_filter: String,
    /// Expose Prometheus metrics at `GET /metrics`.
    pub metrics: bool,
    /// How often Tokio runtime gauges are sampled.
    #[serde(with = "humantime_serde")]
    pub runtime_metrics_interval: Duration,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            log_format: LogFormat::Pretty,
            log_filter: "info,{{crate_name}}=debug".to_owned(),
            metrics: true,
            runtime_metrics_interval: Duration::from_secs(15),
        }
    }
}

impl TelemetryConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        if self.log_filter.trim().is_empty() {
            problems.push("telemetry.log_filter must not be empty".to_owned());
        }
        require_at_least(
            problems,
            "telemetry.runtime_metrics_interval",
            self.runtime_metrics_interval,
            Duration::from_secs(1),
        );
    }
}

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable, colored when attached to a terminal.
    Pretty,
    /// One JSON object per line.
    Json,
}
