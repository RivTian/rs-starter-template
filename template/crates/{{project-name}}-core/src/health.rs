//! Health checks and the readiness report behind `/readyz`.
//!
//! Adapters implement [`HealthCheck`]; the composition root registers them in a
//! [`HealthRegistry`]; the API layer renders [`HealthRegistry::report`].

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::Serialize;

/// Outcome of one check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "detail", rename_all = "snake_case")]
pub enum HealthStatus {
    /// Working normally.
    Ok,
    /// Working, but something is off (e.g. a replica is lagging).
    Degraded(String),
    /// Not working; the service should not receive traffic.
    Failed(String),
}

/// A probe into one dependency. Keep checks cheap: they run on every `/readyz` call.
#[async_trait]
pub trait HealthCheck: Send + Sync + 'static {
    /// Probes the dependency.
    async fn check(&self) -> HealthStatus;
}

/// Aggregated outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportStatus {
    /// Every check is `Ok`.
    Ready,
    /// No check failed, at least one is degraded. Still serves traffic.
    Degraded,
    /// At least one check failed.
    Unavailable,
}

/// What `/readyz` returns.
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    /// The aggregate.
    pub status: ReportStatus,
    /// Every check by name.
    pub checks: BTreeMap<String, HealthStatus>,
}

impl HealthReport {
    /// Whether the service should receive traffic.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        !matches!(self.status, ReportStatus::Unavailable)
    }
}

/// The set of registered checks.
#[derive(Clone)]
pub struct HealthRegistry {
    checks: Vec<(String, Arc<dyn HealthCheck>)>,
    timeout: Duration,
}

impl Default for HealthRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthRegistry {
    /// An empty registry with a 2 s per-check timeout.
    #[must_use]
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            timeout: Duration::from_secs(2),
        }
    }

    /// Adds a named check.
    #[must_use]
    pub fn register(mut self, name: impl Into<String>, check: Arc<dyn HealthCheck>) -> Self {
        self.checks.push((name.into(), check));
        self
    }

    /// Changes the per-check timeout; a check that exceeds it counts as failed.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Runs every check and aggregates.
    pub async fn report(&self) -> HealthReport {
        let mut checks = BTreeMap::new();
        let mut status = ReportStatus::Ready;
        for (name, check) in &self.checks {
            let outcome = match tokio::time::timeout(self.timeout, check.check()).await {
                Ok(outcome) => outcome,
                Err(_) => HealthStatus::Failed(format!("timed out after {:?}", self.timeout)),
            };
            match (&outcome, status) {
                (HealthStatus::Failed(_), _) => status = ReportStatus::Unavailable,
                (HealthStatus::Degraded(_), ReportStatus::Ready) => status = ReportStatus::Degraded,
                _ => {}
            }
            checks.insert(name.clone(), outcome);
        }
        HealthReport { status, checks }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixed(HealthStatus);

    #[async_trait]
    impl HealthCheck for Fixed {
        async fn check(&self) -> HealthStatus {
            self.0.clone()
        }
    }

    struct Hanging;

    #[async_trait]
    impl HealthCheck for Hanging {
        async fn check(&self) -> HealthStatus {
            std::future::pending().await
        }
    }

    #[tokio::test]
    async fn aggregates_worst_status() {
        let registry = HealthRegistry::new()
            .register("a", Arc::new(Fixed(HealthStatus::Ok)))
            .register(
                "b",
                Arc::new(Fixed(HealthStatus::Degraded("lagging".into()))),
            );
        let report = registry.report().await;
        assert_eq!(report.status, ReportStatus::Degraded);
        assert!(report.is_ready());

        let registry = registry.register("c", Arc::new(Fixed(HealthStatus::Failed("down".into()))));
        let report = registry.report().await;
        assert_eq!(report.status, ReportStatus::Unavailable);
        assert!(!report.is_ready());
        assert_eq!(report.checks.len(), 3);
    }

    #[tokio::test]
    async fn slow_checks_fail_instead_of_hanging() {
        let registry = HealthRegistry::new()
            .with_timeout(Duration::from_millis(10))
            .register("slow", Arc::new(Hanging));
        let report = registry.report().await;
        assert!(matches!(report.checks["slow"], HealthStatus::Failed(_)));
    }
}
