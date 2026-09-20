use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::require_at_least;

/// Lifecycle timeouts of the process.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ServerConfig {
    /// After a shutdown request: how long in-flight work may take before remaining tasks are
    /// abandoned and the process exits with an error.
    #[serde(with = "humantime_serde")]
    pub grace_period: Duration,
    /// How long to wait for every service to report ready after start. Exceeding it only
    /// logs a warning and marks the server degraded; `/readyz` keeps telling the truth.
    #[serde(with = "humantime_serde")]
    pub readiness_timeout: Duration,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            grace_period: Duration::from_secs(30),
            readiness_timeout: Duration::from_secs(10),
        }
    }
}

impl ServerConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        require_at_least(
            problems,
            "server.grace_period",
            self.grace_period,
            Duration::from_secs(1),
        );
        require_at_least(
            problems,
            "server.readiness_timeout",
            self.readiness_timeout,
            Duration::from_secs(1),
        );
    }
}
