use serde::{Deserialize, Serialize};

/// The in-process event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EventsConfig {
    /// Events buffered per subscriber before the slowest subscriber starts lagging.
    pub capacity: usize,
}

impl Default for EventsConfig {
    fn default() -> Self {
        Self { capacity: 1024 }
    }
}

impl EventsConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        if self.capacity == 0 {
            problems.push("events.capacity must be at least 1".to_owned());
        }
    }
}
