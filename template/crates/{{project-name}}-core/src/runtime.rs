//! Builds the Tokio runtime from configuration.
//!
//! The rule is *configuration decides the runtime, never the reverse*: `main` stays synchronous
//! until [`build`] returns, so the number of worker threads is a config key like any other.

use tokio::runtime::{Builder, Runtime};

use {{crate_name}}_config::RuntimeConfig;

/// Failure to construct the runtime.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// The operating system refused to create the worker threads.
    #[error("failed to build the tokio runtime")]
    Build(#[source] std::io::Error),
}

/// Builds a multi-threaded runtime. `thread_name` becomes the OS thread name of the workers.
pub fn build(cfg: &RuntimeConfig, thread_name: &str) -> Result<Runtime, RuntimeError> {
    let mut builder = Builder::new_multi_thread();
    builder.enable_all().thread_name(thread_name);
    if cfg.worker_threads > 0 {
        builder.worker_threads(cfg.worker_threads);
    }
    if cfg.max_blocking_threads > 0 {
        builder.max_blocking_threads(cfg.max_blocking_threads);
    }
    builder.build().map_err(RuntimeError::Build)
}
