//! The worked example: a todo list. Model + port + service, one concept per file.

mod model;
mod repository;
mod service;

pub use model::{Title, Todo, TodoId};
pub use repository::TodoRepository;
pub use service::TodoService;
