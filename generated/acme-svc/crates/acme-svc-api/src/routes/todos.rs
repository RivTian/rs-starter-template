use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};

use acme_svc_domain::todo::{Title, TodoId};

use crate::dto::todo::{CreateTodoRequest, TodoResponse};
use crate::dto::{AppJson, AppPath};
use crate::error::ApiError;
use crate::state::AppState;

/// Routes under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/todos", post(create).get(list))
        .route("/todos/{id}", get(get_one))
        .route("/todos/{id}/complete", post(complete))
}

async fn create(
    State(state): State<AppState>,
    AppJson(body): AppJson<CreateTodoRequest>,
) -> Result<(StatusCode, Json<TodoResponse>), ApiError> {
    let title = Title::new(body.title)?;
    let todo = state.todos.create(title).await?;
    Ok((StatusCode::CREATED, Json(todo.into())))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<TodoResponse>>, ApiError> {
    let todos = state.todos.list().await?;
    Ok(Json(todos.into_iter().map(TodoResponse::from).collect()))
}

async fn get_one(
    State(state): State<AppState>,
    AppPath(id): AppPath<TodoId>,
) -> Result<Json<TodoResponse>, ApiError> {
    Ok(Json(state.todos.get(id).await?.into()))
}

async fn complete(
    State(state): State<AppState>,
    AppPath(id): AppPath<TodoId>,
) -> Result<Json<TodoResponse>, ApiError> {
    Ok(Json(state.todos.complete(id).await?.into()))
}
