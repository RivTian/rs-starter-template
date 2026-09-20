//! One file per configuration section. Every section:
//!
//! * derives `Serialize` + `Deserialize` with `#[serde(default, deny_unknown_fields)]`,
//! * hand-writes `impl Default` — the values in `config/default.toml` must match,
//! * exposes `validate(&self, problems: &mut Vec<String>)` that appends, never returns early.

mod app;
mod events;
mod http;
mod jobs;
mod runtime;
mod server;
mod telemetry;

pub use app::{AppConfig, Environment};
pub use events::EventsConfig;
pub use http::HttpConfig;
pub use jobs::{CleanupJobConfig, JobsConfig};
pub use runtime::RuntimeConfig;
pub use server::ServerConfig;
pub use telemetry::{LogFormat, TelemetryConfig};

/// Shared helper: a duration must be at least `min`.
pub(crate) fn require_at_least(
    problems: &mut Vec<String>,
    key: &str,
    value: std::time::Duration,
    min: std::time::Duration,
) {
    if value < min {
        problems.push(format!("{key} must be at least {min:?}, got {value:?}"));
    }
}
