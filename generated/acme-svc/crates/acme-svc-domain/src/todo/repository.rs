use async_trait::async_trait;
use time::OffsetDateTime;

use super::model::{Todo, TodoId};
use crate::error::RepositoryError;

/// Port: persistence for todos. Implemented by adapters in `acme-svc-infra`.
///
/// Methods take and return owned entities; the adapter decides how to map them.
#[async_trait]
pub trait TodoRepository: Send + Sync + 'static {
    /// Stores a new todo. Fails with [`RepositoryError::Duplicate`] if the id exists.
    async fn insert(&self, todo: Todo) -> Result<(), RepositoryError>;

    /// Looks a todo up. `Ok(None)` when absent.
    async fn get(&self, id: TodoId) -> Result<Option<Todo>, RepositoryError>;

    /// Every todo, oldest first.
    async fn list(&self) -> Result<Vec<Todo>, RepositoryError>;

    /// Replaces a stored todo. Fails with [`RepositoryError::NotFound`] if absent.
    async fn update(&self, todo: Todo) -> Result<(), RepositoryError>;

    /// Deletes completed todos whose `completed_at` is before `cutoff`; returns how many.
    async fn purge_completed_before(
        &self,
        cutoff: OffsetDateTime,
    ) -> Result<usize, RepositoryError>;
}
