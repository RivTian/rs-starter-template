use async_trait::async_trait;
use dashmap::DashMap;
use time::OffsetDateTime;

use acme_svc_core::health::{HealthCheck, HealthStatus};
use acme_svc_domain::error::RepositoryError;
use acme_svc_domain::todo::{Todo, TodoId, TodoRepository};

/// A process-local todo store. Lost on restart, by design.
#[derive(Debug, Default)]
pub struct InMemoryTodoRepository {
    items: DashMap<TodoId, Todo>,
}

impl InMemoryTodoRepository {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored todos.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Whether the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[async_trait]
impl TodoRepository for InMemoryTodoRepository {
    async fn insert(&self, todo: Todo) -> Result<(), RepositoryError> {
        match self.items.entry(todo.id) {
            dashmap::Entry::Occupied(_) => Err(RepositoryError::Duplicate(todo.id)),
            dashmap::Entry::Vacant(slot) => {
                slot.insert(todo);
                Ok(())
            }
        }
    }

    async fn get(&self, id: TodoId) -> Result<Option<Todo>, RepositoryError> {
        Ok(self.items.get(&id).map(|entry| entry.clone()))
    }

    async fn list(&self) -> Result<Vec<Todo>, RepositoryError> {
        let mut all: Vec<Todo> = self.items.iter().map(|entry| entry.clone()).collect();
        all.sort_by_key(|todo| (todo.created_at, todo.id));
        Ok(all)
    }

    async fn update(&self, todo: Todo) -> Result<(), RepositoryError> {
        match self.items.get_mut(&todo.id) {
            Some(mut slot) => {
                *slot = todo;
                Ok(())
            }
            None => Err(RepositoryError::NotFound(todo.id)),
        }
    }

    async fn purge_completed_before(
        &self,
        cutoff: OffsetDateTime,
    ) -> Result<usize, RepositoryError> {
        let before = self.items.len();
        self.items
            .retain(|_, todo| todo.completed_at.is_none_or(|at| at >= cutoff));
        Ok(before - self.items.len())
    }
}

#[async_trait]
impl HealthCheck for InMemoryTodoRepository {
    async fn check(&self) -> HealthStatus {
        HealthStatus::Ok
    }
}

#[cfg(test)]
mod tests {
    use acme_svc_domain::todo::Title;

    use super::*;

    fn todo(title: &str, at: OffsetDateTime) -> Todo {
        Todo::new(Title::new(title).unwrap_or_else(|_| unreachable!()), at)
    }

    #[tokio::test]
    async fn insert_get_update_list_purge() {
        let repo = InMemoryTodoRepository::new();
        let t0 = OffsetDateTime::UNIX_EPOCH;
        let mut a = todo("a", t0);
        let b = todo("b", t0 + time::Duration::seconds(1));
        repo.insert(a.clone()).await.expect("insert a");
        repo.insert(b.clone()).await.expect("insert b");
        assert!(matches!(
            repo.insert(a.clone()).await,
            Err(RepositoryError::Duplicate(_))
        ));

        assert_eq!(repo.get(a.id).await.expect("get"), Some(a.clone()));
        assert_eq!(
            repo.list()
                .await
                .expect("list")
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![a.id, b.id]
        );

        a.complete(t0 + time::Duration::seconds(2))
            .expect("complete");
        repo.update(a.clone()).await.expect("update");
        assert!(matches!(
            repo.update(todo("ghost", t0)).await,
            Err(RepositoryError::NotFound(_))
        ));

        assert_eq!(
            repo.purge_completed_before(t0 + time::Duration::seconds(2))
                .await
                .expect("purge"),
            0
        );
        assert_eq!(
            repo.purge_completed_before(t0 + time::Duration::seconds(3))
                .await
                .expect("purge"),
            1
        );
        assert_eq!(repo.len(), 1);
    }
}
