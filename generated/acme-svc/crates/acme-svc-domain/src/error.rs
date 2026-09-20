//! Domain errors: precise variants for business rules, plus what a port may report.

use acme_svc_util::error::{Classify, ErrorKind, ErrorSource};

use crate::todo::TodoId;

/// A business rule was violated, or a port failed underneath a business operation.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// Titles carry meaning; an empty one is a caller mistake.
    #[error("title must not be empty")]
    TitleEmpty,
    /// Titles are bounded to keep storage and UIs sane.
    #[error("title must be at most {max} characters, got {actual}")]
    TitleTooLong {
        /// The limit.
        max: usize,
        /// What was submitted.
        actual: usize,
    },
    /// The referenced todo does not exist.
    #[error("todo {0} not found")]
    NotFound(TodoId),
    /// Completing twice is a conflict, not a no-op: the second caller has stale state.
    #[error("todo {0} is already done")]
    AlreadyDone(TodoId),
    /// The repository failed.
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

impl Classify for DomainError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::TitleEmpty | Self::TitleTooLong { .. } => ErrorKind::InvalidInput,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::AlreadyDone(_) => ErrorKind::Conflict,
            Self::Repository(inner) => inner.kind(),
        }
    }

    fn source(&self) -> ErrorSource {
        match self {
            Self::Repository(inner) => inner.source(),
            _ => self.kind().default_source(),
        }
    }

    fn retryable(&self) -> bool {
        match self {
            Self::Repository(inner) => inner.retryable(),
            _ => false,
        }
    }
}

/// What a [`TodoRepository`](crate::todo::TodoRepository) implementation may report.
/// Adapters translate their driver's errors into these; the domain never sees driver types.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    /// Insert of an id that already exists.
    #[error("todo {0} already exists")]
    Duplicate(TodoId),
    /// Update of an id that does not exist.
    #[error("todo {0} not found")]
    NotFound(TodoId),
    /// The store cannot be reached right now; retrying may help.
    #[error("storage unavailable: {0}")]
    Unavailable(String),
    /// The store misbehaved; retrying will not help.
    #[error("storage failure: {0}")]
    Storage(String),
}

impl Classify for RepositoryError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Duplicate(_) => ErrorKind::Conflict,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::Unavailable(_) => ErrorKind::Unavailable,
            Self::Storage(_) => ErrorKind::Internal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_errors_are_dependency_faults() {
        let err = DomainError::from(RepositoryError::Unavailable("db down".into()));
        assert_eq!(err.kind(), ErrorKind::Unavailable);
        assert_eq!(err.source(), ErrorSource::Dependency);
        assert!(err.retryable());
    }

    #[test]
    fn rule_violations_are_client_faults() {
        let err = DomainError::TitleEmpty;
        assert_eq!(err.kind(), ErrorKind::InvalidInput);
        assert_eq!(err.source(), ErrorSource::Client);
        assert!(!err.retryable());
    }
}
