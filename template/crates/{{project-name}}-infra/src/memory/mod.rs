//! In-memory adapters: zero setup, perfect for the skeleton and for tests. Swap for a real
//! store by adding a sibling module (or an `{{project-name}}-infra-<driver>` crate) that implements
//! the same port.

mod todo_repository;

pub use todo_repository::InMemoryTodoRepository;
