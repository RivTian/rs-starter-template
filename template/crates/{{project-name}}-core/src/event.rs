//! A minimal in-process event bus.
//!
//! Semantics: broadcast. Publishers never wait. A subscriber that falls behind by more than the
//! bus capacity **skips** events (counted in `events_lagged_total`) instead of blocking the
//! publisher. Use it to announce facts (`TodoCreated`), never to ask for work with a reply.

use tokio::sync::broadcast;
use tracing::warn;

/// The bus. Cheap to clone; all clones share one channel.
pub struct EventBus<E> {
    tx: broadcast::Sender<E>,
}

impl<E> Clone for EventBus<E> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<E: Clone + Send + 'static> EventBus<E> {
    /// A bus buffering `capacity` events per subscriber (at least 1).
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity.max(1));
        Self { tx }
    }

    /// A handle for publishing.
    #[must_use]
    pub fn publisher(&self) -> Publisher<E> {
        Publisher {
            tx: self.tx.clone(),
        }
    }

    /// A new subscription; it only sees events published after this call.
    #[must_use]
    pub fn subscribe(&self) -> Subscription<E> {
        Subscription {
            rx: self.tx.subscribe(),
        }
    }

    /// Currently attached subscriptions.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// Publishing half.
pub struct Publisher<E> {
    tx: broadcast::Sender<E>,
}

impl<E> Clone for Publisher<E> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

impl<E: Clone + Send + 'static> Publisher<E> {
    /// Publishes to every current subscriber; returns how many there were.
    /// No subscribers is not an error — the event is simply dropped.
    pub fn publish(&self, event: E) -> usize {
        self.tx.send(event).unwrap_or(0)
    }
}

/// Subscribing half.
pub struct Subscription<E> {
    rx: broadcast::Receiver<E>,
}

impl<E: Clone + Send + 'static> Subscription<E> {
    /// The next event, or `None` once the bus is gone. Lag is logged and counted, then skipped.
    pub async fn next(&mut self) -> Option<E> {
        loop {
            match self.rx.recv().await {
                Ok(event) => return Some(event),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    metrics::counter!("events_lagged_total").increment(skipped);
                    warn!(skipped, "event subscriber lagged; events skipped");
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn delivers_to_every_subscriber() {
        let bus = EventBus::<u8>::new(8);
        let mut a = bus.subscribe();
        let mut b = bus.subscribe();
        assert_eq!(bus.publisher().publish(7), 2);
        assert_eq!(a.next().await, Some(7));
        assert_eq!(b.next().await, Some(7));
    }

    #[tokio::test]
    async fn lagging_subscriber_skips_instead_of_blocking() {
        let bus = EventBus::<u32>::new(2);
        let mut sub = bus.subscribe();
        let publisher = bus.publisher();
        for i in 0..5 {
            publisher.publish(i);
        }
        assert_eq!(sub.next().await, Some(3));
        assert_eq!(sub.next().await, Some(4));
    }

    #[tokio::test]
    async fn ends_when_the_bus_is_dropped() {
        let bus = EventBus::<u8>::new(1);
        let mut sub = bus.subscribe();
        drop(bus);
        assert_eq!(sub.next().await, None);
    }

    #[test]
    fn publishing_without_subscribers_is_fine() {
        let bus = EventBus::<u8>::new(1);
        assert_eq!(bus.publisher().publish(1), 0);
        assert_eq!(bus.subscriber_count(), 0);
    }
}
