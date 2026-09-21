//! One error type leaves this crate: [`ApiError`], rendered as RFC 9457 problem details.
//!
//! Any error implementing [`Classify`] converts into it: the kind decides the status code and
//! the log level; client errors carry their message as `detail`, internal errors never do.

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use acme_svc_core::error::{Classify, ErrorKind, Severity, chain};
use acme_svc_domain::error::DomainError;

/// Problem type URI prefix; the kind's slug is appended.
pub const PROBLEM_TYPE_PREFIX: &str = "urn:acme-svc:problem:";

/// An error ready to be sent to the client.
#[derive(Debug)]
pub struct ApiError {
    kind: ErrorKind,
    detail: Option<String>,
    retryable: bool,
}

impl ApiError {
    /// An error of `kind` with no detail.
    #[must_use]
    pub const fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            detail: None,
            retryable: kind.default_retryable(),
        }
    }

    /// An error of `kind` with a client-visible detail.
    #[must_use]
    pub fn with_detail(kind: ErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: Some(detail.into()),
            retryable: kind.default_retryable(),
        }
    }

    /// Converts any classified error, logging it (with its cause chain) at the severity the
    /// kind demands. Internal errors reach the client without detail.
    pub fn from_classified<E: Classify + std::error::Error>(error: &E) -> Self {
        let kind = error.kind();
        let rendered = chain(error);
        match error.severity() {
            Severity::Error => tracing::error!(error = %rendered, %kind, "request failed"),
            Severity::Warn => tracing::warn!(error = %rendered, %kind, "request failed"),
            Severity::Info => tracing::info!(error = %rendered, %kind, "request rejected"),
            Severity::Debug => tracing::debug!(error = %rendered, %kind, "request rejected"),
        }
        let detail = kind.is_client_error().then(|| error.to_string());
        Self {
            kind,
            detail,
            retryable: error.retryable(),
        }
    }

    /// The classification.
    #[must_use]
    pub const fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The HTTP status.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        StatusCode::from_u16(self.kind.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        Self::from_classified(&error)
    }
}

/// RFC 9457 body.
#[derive(Debug, Serialize)]
struct ProblemDetails {
    #[serde(rename = "type")]
    type_: String,
    title: &'static str,
    status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    kind: &'static str,
    retryable: bool,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ProblemDetails {
            type_: format!("{PROBLEM_TYPE_PREFIX}{}", self.kind.as_str()),
            title: self.kind.title(),
            status: status.as_u16(),
            detail: self.detail,
            kind: self.kind.as_str(),
            retryable: self.retryable,
        };
        let json = serde_json::to_vec(&body).unwrap_or_else(|_| b"{}".to_vec());
        (
            status,
            [(header::CONTENT_TYPE, "application/problem+json")],
            json,
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use acme_svc_domain::error::RepositoryError;

    use super::*;

    #[test]
    fn internal_errors_hide_detail_and_client_errors_show_it() {
        let internal = ApiError::from_classified(&DomainError::from(RepositoryError::Storage(
            "disk on fire".into(),
        )));
        assert_eq!(internal.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(internal.detail.is_none());

        let client = ApiError::from(DomainError::TitleEmpty);
        assert_eq!(client.status(), StatusCode::BAD_REQUEST);
        assert_eq!(client.detail.as_deref(), Some("title must not be empty"));
    }
}
