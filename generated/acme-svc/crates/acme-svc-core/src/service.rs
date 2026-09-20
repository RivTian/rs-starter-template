//! The unit the server runs: anything that lives until shutdown.

use std::net::SocketAddr;

use async_trait::async_trait;

use crate::readiness::Readiness;
use crate::shutdown::ShutdownToken;

/// A long-running component owned by the [`Server`](crate::server::Server).
///
/// Contract:
/// * run until `shutdown` is cancelled, then return promptly (within the grace period);
/// * call `ready.ready()` once you can do your job (a listener is bound, a pool is warm);
/// * return `Err` only for failures that should take the whole process down — the server
///   then cancels every other service.
#[async_trait]
pub trait Service: Send + Sync + 'static {
    /// Stable name for logs and metrics.
    fn name(&self) -> &'static str;

    /// Runs the service to completion.
    async fn run(
        self: Box<Self>,
        shutdown: ShutdownToken,
        ready: Readiness,
    ) -> Result<(), ServiceError>;
}

/// A fatal failure of a service.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// A listener could not be bound.
    #[error("failed to bind {addr}")]
    Bind {
        /// The address that was requested.
        addr: SocketAddr,
        /// The OS error.
        #[source]
        source: std::io::Error,
    },
    /// Any other fatal condition.
    #[error("{reason}")]
    Failed {
        /// Human-readable explanation.
        reason: String,
        /// Underlying cause, if any.
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
}

impl ServiceError {
    /// A fatal failure with a plain reason.
    #[must_use]
    pub fn failed(reason: impl Into<String>) -> Self {
        Self::Failed {
            reason: reason.into(),
            source: None,
        }
    }

    /// A fatal failure wrapping a cause.
    #[must_use]
    pub fn caused_by(
        reason: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Failed {
            reason: reason.into(),
            source: Some(Box::new(source)),
        }
    }
}
