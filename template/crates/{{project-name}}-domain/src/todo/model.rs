use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::DomainError;

/// Identity of a todo. A newtype so ids of different entities cannot be mixed up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TodoId(Uuid);

impl TodoId {
    /// A fresh, time-ordered id.
    #[must_use]
    pub fn new() -> Self {
        Self({{crate_name}}_util::id::new_v7())
    }

    /// The underlying UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }
}

impl Default for TodoId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for TodoId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl FromStr for TodoId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(Self)
    }
}

impl fmt::Display for TodoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// A validated title: trimmed, non-empty, at most [`Title::MAX_LEN`] characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Title(String);

impl Title {
    /// Maximum length in characters.
    pub const MAX_LEN: usize = 200;

    /// Validates and normalises a raw title.
    pub fn new(raw: impl Into<String>) -> Result<Self, DomainError> {
        let raw = raw.into();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::TitleEmpty);
        }
        let actual = trimmed.chars().count();
        if actual > Self::MAX_LEN {
            return Err(DomainError::TitleTooLong {
                max: Self::MAX_LEN,
                actual,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The title text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for Title {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// The entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    /// Identity.
    pub id: TodoId,
    /// What to do.
    pub title: Title,
    /// Whether it is done.
    pub done: bool,
    /// When it was created.
    pub created_at: OffsetDateTime,
    /// When it was completed, if it is done.
    pub completed_at: Option<OffsetDateTime>,
}

impl Todo {
    /// A new, open todo.
    #[must_use]
    pub fn new(title: Title, now: OffsetDateTime) -> Self {
        Self {
            id: TodoId::new(),
            title,
            done: false,
            created_at: now,
            completed_at: None,
        }
    }

    /// Marks the todo done. Doing it twice is a conflict.
    pub const fn complete(&mut self, now: OffsetDateTime) -> Result<(), DomainError> {
        if self.done {
            return Err(DomainError::AlreadyDone(self.id));
        }
        self.done = true;
        self.completed_at = Some(now);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_is_trimmed_and_bounded() {
        assert_eq!(
            Title::new("  ship it  ")
                .map(|t| t.as_str().to_owned())
                .ok(),
            Some("ship it".to_owned())
        );
        assert!(matches!(Title::new("   "), Err(DomainError::TitleEmpty)));
        let long = "x".repeat(Title::MAX_LEN + 1);
        assert!(matches!(
            Title::new(long),
            Err(DomainError::TitleTooLong { .. })
        ));
    }

    #[test]
    fn completing_twice_is_a_conflict() {
        let now = OffsetDateTime::UNIX_EPOCH;
        let mut todo = Todo::new(Title::new("x").unwrap_or_else(|_| unreachable!()), now);
        assert!(todo.complete(now).is_ok());
        assert!(matches!(todo.complete(now), Err(DomainError::AlreadyDone(id)) if id == todo.id));
    }

    #[test]
    fn ids_round_trip_through_text() {
        let id = TodoId::new();
        let parsed: TodoId = id.to_string().parse().unwrap_or_default();
        assert_eq!(parsed, id);
    }
}
