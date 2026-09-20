//! Cooperative shutdown.
//!
//! One [`ShutdownToken`] tree flows from [`Server`](crate::server::Server) down to every task;
//! no layer creates its own channel. Shutdown *requests* come from a [`SignalSource`], which is
//! a trait so tests and embedders can inject one instead of sending real OS signals.

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::{CancellationToken, WaitForCancellationFutureOwned};

/// A handle to observe (and, for the owner, trigger) shutdown.
#[derive(Clone, Debug, Default)]
pub struct ShutdownToken {
    inner: CancellationToken,
}

impl ShutdownToken {
    /// A fresh root token.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A child token: cancelled when the parent is, cancellable on its own without affecting
    /// the parent.
    #[must_use]
    pub fn child(&self) -> Self {
        Self {
            inner: self.inner.child_token(),
        }
    }

    /// Requests shutdown for this token and every descendant.
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Whether shutdown has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }

    /// Resolves once shutdown has been requested.
    pub async fn cancelled(&self) {
        self.inner.cancelled().await;
    }

    /// Like [`cancelled`](Self::cancelled) but `'static`, for `with_graceful_shutdown`-style APIs.
    pub fn cancelled_owned(&self) -> WaitForCancellationFutureOwned {
        self.inner.clone().cancelled_owned()
    }
}

/// Why shutdown was requested. Both are handled gracefully; a **second** signal of either kind
/// exits the process immediately with status 130.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownSignal {
    /// `SIGTERM`: an orchestrator wants us gone.
    Terminate,
    /// `SIGINT` / Ctrl-C: a human wants us gone.
    Interrupt,
}

impl ShutdownSignal {
    /// Stable name for logs.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Terminate => "SIGTERM",
            Self::Interrupt => "SIGINT",
        }
    }
}

/// A stream of shutdown requests.
#[async_trait]
pub trait SignalSource: Send + 'static {
    /// Waits for the next request. `None` means the source is exhausted and will never fire.
    async fn recv(&mut self) -> Option<ShutdownSignal>;
}

/// Operating-system signals: `SIGTERM` + `SIGINT` on Unix, Ctrl-C elsewhere.
pub struct OsSignals {
    #[cfg(unix)]
    terminate: tokio::signal::unix::Signal,
    #[cfg(unix)]
    interrupt: tokio::signal::unix::Signal,
}

impl OsSignals {
    /// Installs the handlers. Must be called from within a Tokio runtime.
    pub fn new() -> std::io::Result<Self> {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            Ok(Self {
                terminate: signal(SignalKind::terminate())?,
                interrupt: signal(SignalKind::interrupt())?,
            })
        }
        #[cfg(not(unix))]
        {
            Ok(Self {})
        }
    }
}

#[async_trait]
impl SignalSource for OsSignals {
    async fn recv(&mut self) -> Option<ShutdownSignal> {
        #[cfg(unix)]
        {
            tokio::select! {
                res = self.terminate.recv() => res.map(|()| ShutdownSignal::Terminate),
                res = self.interrupt.recv() => res.map(|()| ShutdownSignal::Interrupt),
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c()
                .await
                .ok()
                .map(|()| ShutdownSignal::Interrupt)
        }
    }
}

/// Programmatic signals for tests and embedding: send through the [`SignalTrigger`].
pub struct ChannelSignals {
    rx: mpsc::Receiver<ShutdownSignal>,
}

/// The sending half of [`ChannelSignals`].
#[derive(Clone, Debug)]
pub struct SignalTrigger {
    tx: mpsc::Sender<ShutdownSignal>,
}

impl ChannelSignals {
    /// A connected trigger / source pair.
    #[must_use]
    pub fn new() -> (SignalTrigger, Self) {
        let (tx, rx) = mpsc::channel(4);
        (SignalTrigger { tx }, Self { rx })
    }
}

impl SignalTrigger {
    /// Requests shutdown. Returns `false` if the server is no longer listening.
    pub async fn send(&self, signal: ShutdownSignal) -> bool {
        self.tx.send(signal).await.is_ok()
    }
}

#[async_trait]
impl SignalSource for ChannelSignals {
    async fn recv(&mut self) -> Option<ShutdownSignal> {
        self.rx.recv().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn child_tokens_follow_the_parent() {
        let root = ShutdownToken::new();
        let child = root.child();
        assert!(!child.is_cancelled());
        root.cancel();
        assert!(child.is_cancelled());
        child.cancelled().await;
    }

    #[tokio::test]
    async fn cancelling_a_child_leaves_the_parent_alone() {
        let root = ShutdownToken::new();
        let child = root.child();
        child.cancel();
        assert!(!root.is_cancelled());
    }

    #[tokio::test]
    async fn channel_signals_deliver_in_order_then_end() {
        let (trigger, mut source) = ChannelSignals::new();
        assert!(trigger.send(ShutdownSignal::Terminate).await);
        assert_eq!(source.recv().await, Some(ShutdownSignal::Terminate));
        drop(trigger);
        assert_eq!(source.recv().await, None);
    }
}
