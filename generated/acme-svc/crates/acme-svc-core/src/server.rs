//! The process lifecycle coordinator.
//!
//! ```text
//! Starting ──all ready──▶ Ready ──signal / failure──▶ Draining ──tasks done──▶ Stopped
//!    │                      ▲                              │
//!    └──timeout──▶ Degraded ─┘ (late ready)                └──grace exceeded──▶ Aborted
//! ```
//!
//! Every [`Service`] runs as a tracked task with a child [`ShutdownToken`]. The first failure
//! or the first shutdown signal cancels the root token; the server then waits at most the
//! configured grace period for the tasks to finish.

use std::time::Duration;

use serde::Serialize;
use tokio::sync::{mpsc, watch};
use tokio_util::task::TaskTracker;
use tracing::{error, info, warn};

use acme_svc_config::ServerConfig;

use crate::readiness::{self, Readiness, ReadinessWatch};
use crate::service::{Service, ServiceError};
use crate::shutdown::{OsSignals, ShutdownToken, SignalSource};

/// Observable lifecycle state. Subscribe with [`Server::phase_watch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Phase {
    /// Services are starting; not all are ready.
    Starting,
    /// Every service reported ready.
    Ready,
    /// The readiness timeout elapsed with services still starting.
    Degraded,
    /// Shutdown requested; no new work is accepted while in-flight work finishes.
    Draining,
    /// Every task finished within the grace period.
    Stopped,
    /// The grace period elapsed with tasks still running.
    Aborted,
}

impl Phase {
    /// Whether new work should be accepted right now.
    #[must_use]
    pub const fn accepts_work(self) -> bool {
        matches!(self, Self::Ready | Self::Degraded)
    }
}

/// The observing half of the lifecycle state.
pub type PhaseWatch = watch::Receiver<Phase>;

/// Fatal outcomes of [`Server::run`].
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Nothing to run.
    #[error("no services registered")]
    NoServices,
    /// OS signal handlers could not be installed.
    #[error("failed to install OS signal handlers")]
    Signals(#[source] std::io::Error),
    /// A service failed; the process drained and stopped because of it.
    #[error("service `{name}` failed")]
    ServiceFailed {
        /// The failing service.
        name: &'static str,
        /// Its error.
        #[source]
        source: ServiceError,
    },
    /// Tasks were still running when the grace period elapsed.
    #[error("grace period of {grace:?} exceeded with {remaining} task(s) still running")]
    GraceExceeded {
        /// The configured grace period.
        grace: Duration,
        /// Tasks that did not finish.
        remaining: usize,
    },
}

struct ServiceFailure {
    name: &'static str,
    error: ServiceError,
}

/// Owns the services and drives them through the lifecycle.
pub struct Server {
    cfg: ServerConfig,
    services: Vec<Box<dyn Service>>,
    phase: watch::Sender<Phase>,
}

impl Server {
    /// A server with no services yet.
    #[must_use]
    pub fn new(cfg: ServerConfig) -> Self {
        let (phase, _) = watch::channel(Phase::Starting);
        Self {
            cfg,
            services: Vec::new(),
            phase,
        }
    }

    /// Registers a service. Services start in registration order.
    pub fn add_service(&mut self, service: impl Service) -> &mut Self {
        self.services.push(Box::new(service));
        self
    }

    /// Registers an already boxed service.
    pub fn add_boxed(&mut self, service: Box<dyn Service>) -> &mut Self {
        self.services.push(service);
        self
    }

    /// Builder-style [`add_service`](Self::add_service).
    #[must_use]
    pub fn service(mut self, service: impl Service) -> Self {
        self.add_service(service);
        self
    }

    /// A watch on the lifecycle [`Phase`]; hand it to `/readyz` and to tests.
    #[must_use]
    pub fn phase_watch(&self) -> PhaseWatch {
        self.phase.subscribe()
    }

    /// Runs until an OS signal or a service failure, then drains.
    pub async fn run(self) -> Result<(), ServerError> {
        let signals = OsSignals::new().map_err(ServerError::Signals)?;
        self.run_with(signals).await
    }

    /// Like [`run`](Self::run) with an injected [`SignalSource`].
    pub async fn run_with<S: SignalSource>(self, signals: S) -> Result<(), ServerError> {
        if self.services.is_empty() {
            return Err(ServerError::NoServices);
        }
        let root = ShutdownToken::new();
        let tracker = TaskTracker::new();
        let (watches, mut failure_rx) = Self::spawn_services(self.services, &root, &tracker);
        let signal_task = tokio::spawn(Self::watch_signals(signals, root.clone()));

        let mut late_ready = Self::await_readiness(&self.cfg, &self.phase, &watches).await;

        // ── steady state ─────────────────────────────────────────────────────
        loop {
            tokio::select! {
                () = root.cancelled() => break,
                () = tracker.wait() => {
                    info!("all services stopped on their own");
                    break;
                }
                () = async {
                    match late_ready.as_mut() {
                        Some(fut) => fut.await,
                        None => std::future::pending().await,
                    }
                } => {
                    info!("late services became ready");
                    self.phase.send_replace(Phase::Ready);
                    late_ready = None;
                }
            }
        }

        // ── drain ────────────────────────────────────────────────────────────
        self.phase.send_replace(Phase::Draining);
        root.cancel();
        info!(grace = ?self.cfg.grace_period, "draining");
        let drained = tokio::time::timeout(self.cfg.grace_period, tracker.wait()).await;
        signal_task.abort();

        if drained.is_err() {
            let remaining = tracker.len();
            self.phase.send_replace(Phase::Aborted);
            warn!(
                remaining,
                "grace period exceeded; abandoning remaining tasks"
            );
            return Err(ServerError::GraceExceeded {
                grace: self.cfg.grace_period,
                remaining,
            });
        }
        self.phase.send_replace(Phase::Stopped);
        if let Ok(ServiceFailure { name, error }) = failure_rx.try_recv() {
            return Err(ServerError::ServiceFailed {
                name,
                source: error,
            });
        }
        info!("clean shutdown");
        Ok(())
    }

    /// Starts every service as a tracked task. The tracker is closed afterwards so `wait()`
    /// resolves once they all finish; closing does not stop tracking.
    fn spawn_services(
        services: Vec<Box<dyn Service>>,
        root: &ShutdownToken,
        tracker: &TaskTracker,
    ) -> (
        Vec<(&'static str, ReadinessWatch)>,
        mpsc::UnboundedReceiver<ServiceFailure>,
    ) {
        let (failure_tx, failure_rx) = mpsc::unbounded_channel::<ServiceFailure>();
        let mut watches = Vec::with_capacity(services.len());
        for service in services {
            let name = service.name();
            let (notifier, watch) = Readiness::new();
            watches.push((name, watch));
            let child = root.child();
            let root = root.clone();
            let failure_tx = failure_tx.clone();
            tracker.spawn(async move {
                info!(service = name, "starting");
                match service.run(child, notifier).await {
                    Ok(()) => info!(service = name, "stopped"),
                    Err(error) => {
                        error!(service = name, error = %error, "service failed; shutting down");
                        let _ = failure_tx.send(ServiceFailure { name, error });
                        root.cancel();
                    }
                }
            });
        }
        tracker.close();
        (watches, failure_rx)
    }

    /// First signal cancels the root token; a second one exits the process immediately.
    async fn watch_signals<S: SignalSource>(mut signals: S, root: ShutdownToken) {
        let Some(first) = signals.recv().await else {
            return;
        };
        info!(signal = first.name(), "shutdown requested");
        root.cancel();
        if let Some(second) = signals.recv().await {
            warn!(
                signal = second.name(),
                "second signal received; exiting immediately"
            );
            std::process::exit(130);
        }
    }

    /// Waits up to the readiness timeout. Returns a future for the stragglers when it elapsed.
    async fn await_readiness(
        cfg: &ServerConfig,
        phase: &watch::Sender<Phase>,
        watches: &[(&'static str, ReadinessWatch)],
    ) -> Option<LateReady> {
        let all = readiness::wait_all(watches.iter().map(|(_, w)| w.clone()).collect());
        if tokio::time::timeout(cfg.readiness_timeout, all)
            .await
            .is_ok()
        {
            info!("all services ready");
            phase.send_replace(Phase::Ready);
            return None;
        }
        let pending: Vec<&str> = watches
            .iter()
            .filter(|(_, w)| !*w.borrow())
            .map(|(n, _)| *n)
            .collect();
        warn!(?pending, timeout = ?cfg.readiness_timeout, "readiness timeout elapsed");
        phase.send_replace(Phase::Degraded);
        Some(Box::pin(readiness::wait_all(
            watches.iter().map(|(_, w)| w.clone()).collect(),
        )))
    }
}

type LateReady = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shutdown::{ChannelSignals, ShutdownSignal};
    use async_trait::async_trait;

    struct Idle;

    #[async_trait]
    impl Service for Idle {
        fn name(&self) -> &'static str {
            "idle"
        }
        async fn run(
            self: Box<Self>,
            shutdown: ShutdownToken,
            ready: Readiness,
        ) -> Result<(), ServiceError> {
            ready.ready();
            shutdown.cancelled().await;
            Ok(())
        }
    }

    struct Failing;

    #[async_trait]
    impl Service for Failing {
        fn name(&self) -> &'static str {
            "failing"
        }
        async fn run(
            self: Box<Self>,
            _: ShutdownToken,
            ready: Readiness,
        ) -> Result<(), ServiceError> {
            ready.ready();
            Err(ServiceError::failed("boom"))
        }
    }

    struct Stubborn;

    #[async_trait]
    impl Service for Stubborn {
        fn name(&self) -> &'static str {
            "stubborn"
        }
        async fn run(
            self: Box<Self>,
            _: ShutdownToken,
            ready: Readiness,
        ) -> Result<(), ServiceError> {
            ready.ready();
            std::future::pending().await
        }
    }

    fn cfg(grace_ms: u64) -> ServerConfig {
        ServerConfig {
            grace_period: Duration::from_millis(grace_ms),
            readiness_timeout: Duration::from_secs(1),
        }
    }

    #[tokio::test]
    async fn signal_drains_cleanly() {
        let server = Server::new(cfg(1000)).service(Idle);
        let mut phase = server.phase_watch();
        let (trigger, signals) = ChannelSignals::new();
        let handle = tokio::spawn(server.run_with(signals));
        phase.wait_for(|p| *p == Phase::Ready).await.expect("ready");
        assert!(trigger.send(ShutdownSignal::Terminate).await);
        handle.await.expect("join").expect("clean");
        assert_eq!(*phase.borrow(), Phase::Stopped);
    }

    #[tokio::test]
    async fn service_failure_takes_the_process_down() {
        let server = Server::new(cfg(1000)).service(Idle).service(Failing);
        let (_trigger, signals) = ChannelSignals::new();
        let err = server.run_with(signals).await.expect_err("must fail");
        assert!(
            matches!(
                err,
                ServerError::ServiceFailed {
                    name: "failing",
                    ..
                }
            ),
            "{err}"
        );
    }

    #[tokio::test]
    async fn grace_period_is_enforced() {
        let server = Server::new(cfg(50)).service(Stubborn);
        let (trigger, signals) = ChannelSignals::new();
        let handle = tokio::spawn(server.run_with(signals));
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(trigger.send(ShutdownSignal::Interrupt).await);
        let err = handle.await.expect("join").expect_err("must exceed grace");
        assert!(
            matches!(err, ServerError::GraceExceeded { remaining: 1, .. }),
            "{err}"
        );
    }

    #[test]
    fn no_services_is_an_error() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        let (_trigger, signals) = ChannelSignals::new();
        let err = rt
            .block_on(Server::new(cfg(10)).run_with(signals))
            .expect_err("empty");
        assert!(matches!(err, ServerError::NoServices));
    }
}
