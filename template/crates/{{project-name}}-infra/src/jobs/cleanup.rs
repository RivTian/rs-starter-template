use std::sync::Arc;

use async_trait::async_trait;
use tokio::time::MissedTickBehavior;
use tracing::{debug, info, warn};

use {{crate_name}}_config::CleanupJobConfig;
use {{crate_name}}_core::error::chain;
use {{crate_name}}_core::readiness::Readiness;
use {{crate_name}}_core::service::{Service, ServiceError};
use {{crate_name}}_core::shutdown::ShutdownToken;
use {{crate_name}}_domain::todo::TodoService;

/// Periodically purges completed todos older than the configured retention.
pub struct CleanupJob {
    cfg: CleanupJobConfig,
    todos: Arc<TodoService>,
}

impl CleanupJob {
    /// A job driven by `cfg`.
    #[must_use]
    pub const fn new(cfg: CleanupJobConfig, todos: Arc<TodoService>) -> Self {
        Self { cfg, todos }
    }

    async fn tick(&self) {
        match self.todos.purge_completed(self.cfg.retention).await {
            Ok(0) => debug!("cleanup: nothing to purge"),
            Ok(count) => info!(count, "cleanup: purged completed todos"),
            Err(error) => {
                warn!(error = %chain(&error), "cleanup: purge failed; will retry next tick");
            }
        }
    }
}

#[async_trait]
impl Service for CleanupJob {
    fn name(&self) -> &'static str {
        "cleanup-job"
    }

    async fn run(
        self: Box<Self>,
        shutdown: ShutdownToken,
        ready: Readiness,
    ) -> Result<(), ServiceError> {
        ready.ready();
        let mut ticker = tokio::time::interval(self.cfg.interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
        // The first tick fires immediately; skip it so startup stays quiet.
        ticker.tick().await;
        loop {
            tokio::select! {
                () = shutdown.cancelled() => return Ok(()),
                _ = ticker.tick() => self.tick().await,
            }
        }
    }
}
