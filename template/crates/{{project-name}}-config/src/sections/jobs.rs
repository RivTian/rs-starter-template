use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::require_at_least;

/// Background jobs. Add a struct per job; keep every job individually switchable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct JobsConfig {
    /// Purges completed todos.
    pub cleanup: CleanupJobConfig,
}

impl JobsConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        self.cleanup.validate(problems);
    }
}

/// The example periodic job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CleanupJobConfig {
    /// Run the job at all.
    pub enabled: bool,
    /// Time between runs.
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    /// Completed todos older than this are purged.
    #[serde(with = "humantime_serde")]
    pub retention: Duration,
}

impl Default for CleanupJobConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_mins(1),
            retention: Duration::from_hours(24),
        }
    }
}

impl CleanupJobConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        if self.enabled {
            require_at_least(
                problems,
                "jobs.cleanup.interval",
                self.interval,
                Duration::from_secs(1),
            );
        }
    }
}
