use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use acme_svc_domain::todo::{Todo, TodoId};

/// `POST /api/v1/todos` body.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTodoRequest {
    /// The title; validated by the domain.
    pub title: String,
}

/// A todo as the API shows it.
#[derive(Debug, Serialize)]
pub struct TodoResponse {
    /// Identity.
    pub id: TodoId,
    /// Title.
    pub title: String,
    /// Whether it is done.
    pub done: bool,
    /// Creation time, RFC 3339.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    /// Completion time, RFC 3339, when done.
    #[serde(with = "time::serde::rfc3339::option")]
    pub completed_at: Option<OffsetDateTime>,
}

impl From<Todo> for TodoResponse {
    fn from(todo: Todo) -> Self {
        Self {
            id: todo.id,
            title: todo.title.as_str().to_owned(),
            done: todo.done,
            created_at: todo.created_at,
            completed_at: todo.completed_at,
        }
    }
}
