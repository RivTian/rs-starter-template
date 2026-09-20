//! Wire types and extractors. DTOs are separate from domain types on purpose: the API can
//! stay stable while the model evolves, and the model never derives `Deserialize` just to
//! please a client.

pub mod todo;

use {{crate_name}}_core::error::ErrorKind;
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use serde::de::DeserializeOwned;

use crate::error::ApiError;

/// `axum::Json` whose rejection is a problem-details `400` instead of plain text.
pub struct AppJson<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for AppJson<T> {
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(request, state).await {
            Ok(axum::Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(ApiError::with_detail(
                ErrorKind::InvalidInput,
                rejection.body_text(),
            )),
        }
    }
}

/// `axum::extract::Path` whose rejection is a problem-details `400`.
pub struct AppPath<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned + Send> FromRequestParts<S> for AppPath<T> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(axum::extract::Path(value)) => Ok(Self(value)),
            Err(rejection) => Err(ApiError::with_detail(
                ErrorKind::InvalidInput,
                rejection.body_text(),
            )),
        }
    }
}
