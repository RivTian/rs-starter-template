//! Readiness signalling between a service and the server.
//!
//! A [`Readiness`] notifier is consumed by [`Readiness::ready`] so a service can only announce
//! once — and it also announces on `Drop`, so a service that returns early or fails can never
//! leave the server waiting forever. Announcing too early is a bug you will see in `/readyz`;
//! never announcing would be a hang you would not.

use tokio::sync::watch;

/// The observing half: `true` once the service is ready.
pub type ReadinessWatch = watch::Receiver<bool>;

/// Given to every service; call [`ready`](Self::ready) once initialisation is done.
#[derive(Debug)]
pub struct Readiness {
    tx: watch::Sender<bool>,
}

impl Readiness {
    /// A connected notifier / watch pair.
    #[must_use]
    pub fn new() -> (Self, ReadinessWatch) {
        let (tx, rx) = watch::channel(false);
        (Self { tx }, rx)
    }

    /// Announces readiness. Consumes the notifier.
    pub fn ready(self) {
        drop(self);
    }
}

impl Drop for Readiness {
    fn drop(&mut self) {
        // No receivers is fine: nobody was waiting.
        let _ = self.tx.send(true);
    }
}

/// Resolves once every watch reports ready (or its notifier is gone).
pub async fn wait_all(watches: Vec<ReadinessWatch>) {
    for mut watch in watches {
        // An error means the sender was dropped, which itself signalled ready.
        let _ = watch.wait_for(|ready| *ready).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ready_flips_the_watch() {
        let (notifier, watch) = Readiness::new();
        assert!(!*watch.borrow());
        notifier.ready();
        assert!(*watch.borrow());
    }

    #[tokio::test]
    async fn dropping_the_notifier_counts_as_ready() {
        let (notifier, watch) = Readiness::new();
        drop(notifier);
        wait_all(vec![watch]).await;
    }
}
