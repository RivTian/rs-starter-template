//! Bridges the domain's `EventPublisher` port to the core event bus, and a subscriber that logs.

use async_trait::async_trait;
use tracing::info;

use acme_svc_core::event::{Publisher, Subscription};
use acme_svc_core::readiness::Readiness;
use acme_svc_core::service::{Service, ServiceError};
use acme_svc_core::shutdown::ShutdownToken;
use acme_svc_domain::event::DomainEvent;
use acme_svc_domain::ports::EventPublisher;

/// Adapter: `domain::EventPublisher` on top of `core::Publisher<DomainEvent>`.
pub struct EventBusPublisher {
    inner: Publisher<DomainEvent>,
}

impl EventBusPublisher {
    /// Wraps a bus publisher.
    #[must_use]
    pub const fn new(inner: Publisher<DomainEvent>) -> Self {
        Self { inner }
    }
}

impl EventPublisher for EventBusPublisher {
    fn publish(&self, event: DomainEvent) {
        metrics::counter!("domain_events_total", "type" => event.name()).increment(1);
        self.inner.publish(event);
    }
}

/// Example subscriber: logs every domain event. Replace with cache invalidation, audit
/// trails, outbound webhooks — anything that reacts to facts without the publisher waiting.
pub struct EventLogger {
    subscription: Subscription<DomainEvent>,
}

impl EventLogger {
    /// Logs events from `subscription` until shutdown.
    #[must_use]
    pub const fn new(subscription: Subscription<DomainEvent>) -> Self {
        Self { subscription }
    }
}

#[async_trait]
impl Service for EventLogger {
    fn name(&self) -> &'static str {
        "event-logger"
    }

    async fn run(
        mut self: Box<Self>,
        shutdown: ShutdownToken,
        ready: Readiness,
    ) -> Result<(), ServiceError> {
        ready.ready();
        loop {
            tokio::select! {
                () = shutdown.cancelled() => return Ok(()),
                next = self.subscription.next() => match next {
                    Some(event) => info!(event = event.name(), payload = ?event, "domain event"),
                    None => return Ok(()),
                },
            }
        }
    }
}
