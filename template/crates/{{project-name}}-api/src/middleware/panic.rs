use std::any::Any;

use {{crate_name}}_core::error::ErrorKind;
use axum::response::{IntoResponse, Response};

use crate::error::ApiError;

/// Turns a panicking handler into a `500` problem instead of a dropped connection.
/// The panic itself is already logged (with a backtrace) by the process-wide panic hook.
#[expect(
    clippy::needless_pass_by_value,
    reason = "signature dictated by `CatchPanicLayer::custom`"
)]
pub fn handle(payload: Box<dyn Any + Send + 'static>) -> Response {
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "<non-string panic payload>".to_owned());
    tracing::error!(message, "handler panicked");
    ApiError::new(ErrorKind::Internal).into_response()
}
