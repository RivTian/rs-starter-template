//! Ports: what the domain needs from the outside world, as traits.
//!
//! Adapters live in `{{project-name}}-infra`; the composition root (`{{project-name}}`) connects the two.

use time::OffsetDateTime;

use crate::event::DomainEvent;

/// The current time. Injected so tests can freeze it.
pub trait Clock: Send + Sync + 'static {
    /// Now, in UTC.
    fn now(&self) -> OffsetDateTime;
}

/// Announces domain events. Fire-and-forget: publishing never fails and never waits.
pub trait EventPublisher: Send + Sync + 'static {
    /// Publishes one event.
    fn publish(&self, event: DomainEvent);
}
