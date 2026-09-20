use std::sync::Arc;
use std::time::Duration;

use super::model::{Title, Todo, TodoId};
use super::repository::TodoRepository;
use crate::error::DomainError;
use crate::event::DomainEvent;
use crate::ports::{Clock, EventPublisher};

/// The use cases. Holds ports as trait objects so the composition root decides the adapters
/// and tests can substitute fakes.
pub struct TodoService {
    repo: Arc<dyn TodoRepository>,
    clock: Arc<dyn Clock>,
    events: Arc<dyn EventPublisher>,
}

impl TodoService {
    /// Wires the use cases to their ports.
    #[must_use]
    pub fn new(
        repo: Arc<dyn TodoRepository>,
        clock: Arc<dyn Clock>,
        events: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            repo,
            clock,
            events,
        }
    }

    /// Creates a todo and announces `TodoCreated`.
    pub async fn create(&self, title: Title) -> Result<Todo, DomainError> {
        let todo = Todo::new(title, self.clock.now());
        self.repo.insert(todo.clone()).await?;
        self.events
            .publish(DomainEvent::TodoCreated { id: todo.id });
        Ok(todo)
    }

    /// Fetches a todo.
    pub async fn get(&self, id: TodoId) -> Result<Todo, DomainError> {
        self.repo.get(id).await?.ok_or(DomainError::NotFound(id))
    }

    /// Lists every todo, oldest first.
    pub async fn list(&self) -> Result<Vec<Todo>, DomainError> {
        Ok(self.repo.list().await?)
    }

    /// Marks a todo done and announces `TodoCompleted`.
    pub async fn complete(&self, id: TodoId) -> Result<Todo, DomainError> {
        let mut todo = self.get(id).await?;
        todo.complete(self.clock.now())?;
        self.repo.update(todo.clone()).await?;
        self.events.publish(DomainEvent::TodoCompleted { id });
        Ok(todo)
    }

    /// Deletes todos completed more than `retention` ago; announces `TodosPurged` if any.
    pub async fn purge_completed(&self, retention: Duration) -> Result<usize, DomainError> {
        let cutoff = self.clock.now() - retention;
        let count = self.repo.purge_completed_before(cutoff).await?;
        if count > 0 {
            self.events.publish(DomainEvent::TodosPurged { count });
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use time::OffsetDateTime;

    use super::*;
    use crate::error::RepositoryError;

    #[derive(Default)]
    struct MemRepo(Mutex<BTreeMap<TodoId, Todo>>);

    #[async_trait]
    impl TodoRepository for MemRepo {
        async fn insert(&self, todo: Todo) -> Result<(), RepositoryError> {
            let mut map = self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if map.contains_key(&todo.id) {
                return Err(RepositoryError::Duplicate(todo.id));
            }
            map.insert(todo.id, todo);
            Ok(())
        }
        async fn get(&self, id: TodoId) -> Result<Option<Todo>, RepositoryError> {
            Ok(self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&id)
                .cloned())
        }
        async fn list(&self) -> Result<Vec<Todo>, RepositoryError> {
            Ok(self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .values()
                .cloned()
                .collect())
        }
        async fn update(&self, todo: Todo) -> Result<(), RepositoryError> {
            let mut map = self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !map.contains_key(&todo.id) {
                return Err(RepositoryError::NotFound(todo.id));
            }
            map.insert(todo.id, todo);
            Ok(())
        }
        async fn purge_completed_before(
            &self,
            cutoff: OffsetDateTime,
        ) -> Result<usize, RepositoryError> {
            let mut map = self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let before = map.len();
            map.retain(|_, t| t.completed_at.is_none_or(|at| at >= cutoff));
            Ok(before - map.len())
        }
    }

    struct FixedClock(OffsetDateTime);

    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            self.0
        }
    }

    #[derive(Default)]
    struct Recorder(Mutex<Vec<DomainEvent>>);

    impl EventPublisher for Recorder {
        fn publish(&self, event: DomainEvent) {
            self.0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(event);
        }
    }

    fn service() -> (TodoService, Arc<Recorder>) {
        let events = Arc::new(Recorder::default());
        let svc = TodoService::new(
            Arc::new(MemRepo::default()),
            Arc::new(FixedClock(
                OffsetDateTime::UNIX_EPOCH + Duration::from_secs(1_000_000),
            )),
            events.clone(),
        );
        (svc, events)
    }

    fn title(s: &str) -> Title {
        Title::new(s).unwrap_or_else(|_| unreachable!("valid test title"))
    }

    #[tokio::test]
    async fn create_then_complete_publishes_two_events() {
        let (svc, events) = service();
        let todo = svc.create(title("ship")).await.expect("create");
        let done = svc.complete(todo.id).await.expect("complete");
        assert!(done.done);
        let recorded = events
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        assert_eq!(
            recorded,
            vec![
                DomainEvent::TodoCreated { id: todo.id },
                DomainEvent::TodoCompleted { id: todo.id }
            ]
        );
    }

    #[tokio::test]
    async fn completing_twice_is_a_conflict_and_unknown_is_not_found() {
        let (svc, _) = service();
        let todo = svc.create(title("x")).await.expect("create");
        svc.complete(todo.id).await.expect("first");
        assert!(matches!(
            svc.complete(todo.id).await,
            Err(DomainError::AlreadyDone(_))
        ));
        assert!(matches!(
            svc.get(TodoId::new()).await,
            Err(DomainError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn purge_only_removes_old_completed_todos() {
        let (svc, events) = service();
        let open = svc.create(title("open")).await.expect("create");
        let done = svc.create(title("done")).await.expect("create");
        svc.complete(done.id).await.expect("complete");
        // completed "now" is not older than 1h
        assert_eq!(
            svc.purge_completed(Duration::from_hours(1))
                .await
                .expect("purge"),
            0
        );
        // retention 0 means "everything completed before now"; completed_at == now is kept
        assert_eq!(svc.purge_completed(Duration::ZERO).await.expect("purge"), 0);
        assert_eq!(svc.list().await.expect("list").len(), 2);
        assert!(svc.get(open.id).await.is_ok());
        // two creations + one completion; purges published nothing
        assert_eq!(
            events
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .len(),
            3
        );
    }
}
