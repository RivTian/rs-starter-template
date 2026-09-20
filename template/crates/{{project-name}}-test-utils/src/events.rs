//! An `EventPublisher` that remembers everything.

use std::sync::Mutex;

use {{crate_name}}_domain::event::DomainEvent;
use {{crate_name}}_domain::ports::EventPublisher;

/// Records published events for assertions.
#[derive(Debug, Default)]
pub struct RecordingPublisher {
    events: Mutex<Vec<DomainEvent>>,
}

impl RecordingPublisher {
    /// An empty recorder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Everything published so far, in order.
    #[must_use]
    pub fn events(&self) -> Vec<DomainEvent> {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl EventPublisher for RecordingPublisher {
    fn publish(&self, event: DomainEvent) {
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(event);
    }
}
