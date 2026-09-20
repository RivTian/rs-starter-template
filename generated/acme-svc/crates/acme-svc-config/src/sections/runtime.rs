use serde::{Deserialize, Serialize};

/// Tokio runtime sizing. The runtime is built **from** this before anything async runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RuntimeConfig {
    /// Worker threads; `0` means one per CPU core.
    pub worker_threads: usize,
    /// Upper bound of the blocking thread pool (`spawn_blocking`); `0` keeps Tokio's default.
    pub max_blocking_threads: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: 0,
            max_blocking_threads: 512,
        }
    }
}

impl RuntimeConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        if self.worker_threads > 1024 {
            problems.push(format!(
                "runtime.worker_threads is unreasonably high: {}",
                self.worker_threads
            ));
        }
    }
}
