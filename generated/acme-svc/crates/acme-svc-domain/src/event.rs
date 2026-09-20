//! Facts the domain announces after they happened. Past tense, immutable, serialisable.

use serde::Serialize;

use crate::todo::TodoId;

/// Everything that can be published on the event bus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    /// A todo was created.
    TodoCreated {
        /// Its id.
        id: TodoId,
    },
    /// A todo was marked done.
    TodoCompleted {
        /// Its id.
        id: TodoId,
    },
    /// Completed todos were purged by the cleanup job.
    TodosPurged {
        /// How many.
        count: usize,
    },
}

impl DomainEvent {
    /// Stable `snake_case` name for logs and metrics labels.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::TodoCreated { .. } => "todo_created",
            Self::TodoCompleted { .. } => "todo_completed",
            Self::TodosPurged { .. } => "todos_purged",
        }
    }
}
