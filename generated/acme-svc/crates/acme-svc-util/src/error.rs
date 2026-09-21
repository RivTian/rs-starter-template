//! Transport-agnostic error classification shared by every layer.
//!
//! Each layer keeps its own precise `thiserror` enum (domain, infra, core, api). What they have in
//! common is answered here, once: *what kind of failure is this, whose fault is it, and may the
//! caller retry?* The API layer turns a [`Classify`] implementation into an HTTP status code and a
//! log level without knowing the concrete error type.

use core::fmt;

/// Coarse classification of a failure. Adding a variant makes the compiler point at every
/// `match` that must decide what the new kind means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// The caller sent something we cannot accept.
    InvalidInput,
    /// The request body exceeds the configured limit.
    PayloadTooLarge,
    /// The caller is not authenticated.
    Unauthorized,
    /// The caller is authenticated but not allowed.
    Forbidden,
    /// The referenced resource does not exist.
    NotFound,
    /// The request conflicts with current state (duplicate, stale version, already done).
    Conflict,
    /// The caller exceeded a rate limit; retrying later may succeed.
    RateLimited,
    /// A dependency did not answer in time.
    Timeout,
    /// A dependency (or this service during shutdown) is not available right now.
    Unavailable,
    /// A bug or an unexpected condition on our side.
    Internal,
}

impl ErrorKind {
    /// Every kind, for tests and exhaustive tables.
    pub const ALL: [Self; 10] = [
        Self::InvalidInput,
        Self::PayloadTooLarge,
        Self::Unauthorized,
        Self::Forbidden,
        Self::NotFound,
        Self::Conflict,
        Self::RateLimited,
        Self::Timeout,
        Self::Unavailable,
        Self::Internal,
    ];

    /// The HTTP status code this kind maps to.
    #[must_use]
    pub const fn http_status(self) -> u16 {
        match self {
            Self::InvalidInput => 400,
            Self::Unauthorized => 401,
            Self::Forbidden => 403,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::PayloadTooLarge => 413,
            Self::RateLimited => 429,
            Self::Internal => 500,
            Self::Unavailable => 503,
            Self::Timeout => 504,
        }
    }

    /// Short human-readable title (RFC 9457 `title`).
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::InvalidInput => "Invalid input",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden => "Forbidden",
            Self::NotFound => "Not found",
            Self::Conflict => "Conflict",
            Self::PayloadTooLarge => "Payload too large",
            Self::RateLimited => "Rate limited",
            Self::Timeout => "Upstream timeout",
            Self::Unavailable => "Service unavailable",
            Self::Internal => "Internal error",
        }
    }

    /// Stable `snake_case` identifier for metrics labels and problem types.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::PayloadTooLarge => "payload_too_large",
            Self::RateLimited => "rate_limited",
            Self::Timeout => "timeout",
            Self::Unavailable => "unavailable",
            Self::Internal => "internal",
        }
    }

    /// Who is responsible, unless the concrete error says otherwise.
    #[must_use]
    pub const fn default_source(self) -> ErrorSource {
        match self {
            Self::InvalidInput
            | Self::PayloadTooLarge
            | Self::Unauthorized
            | Self::Forbidden
            | Self::NotFound
            | Self::Conflict
            | Self::RateLimited => ErrorSource::Client,
            Self::Timeout | Self::Unavailable => ErrorSource::Dependency,
            Self::Internal => ErrorSource::Internal,
        }
    }

    /// Whether an identical retry could plausibly succeed.
    #[must_use]
    pub const fn default_retryable(self) -> bool {
        matches!(self, Self::RateLimited | Self::Timeout | Self::Unavailable)
    }

    /// How loudly to log an error of this kind.
    #[must_use]
    pub const fn severity(self) -> Severity {
        match self {
            Self::InvalidInput
            | Self::PayloadTooLarge
            | Self::Unauthorized
            | Self::Forbidden
            | Self::NotFound => Severity::Debug,
            Self::Conflict | Self::RateLimited => Severity::Info,
            Self::Timeout | Self::Unavailable => Severity::Warn,
            Self::Internal => Severity::Error,
        }
    }

    /// `true` for 4xx-class kinds.
    #[must_use]
    pub const fn is_client_error(self) -> bool {
        matches!(self.default_source(), ErrorSource::Client)
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Who caused the failure. Orthogonal to [`ErrorKind`]: a `Timeout` is normally a dependency's
/// fault but a concrete error may reclassify it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSource {
    /// The remote caller.
    Client,
    /// Something we depend on (database, upstream service, queue).
    Dependency,
    /// Ourselves.
    Internal,
}

impl ErrorSource {
    /// Stable `snake_case` identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Dependency => "dependency",
            Self::Internal => "internal",
        }
    }
}

/// Log level for an error, without depending on a logging crate here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Expected client mistakes.
    Debug,
    /// Noteworthy but routine.
    Info,
    /// Degraded dependencies.
    Warn,
    /// Bugs.
    Error,
}

/// Implemented by every error type that can cross a layer boundary.
///
/// Only [`Classify::kind`] is required; `source` and `retryable` default to the kind's defaults.
pub trait Classify {
    /// The coarse kind of this error.
    fn kind(&self) -> ErrorKind;

    /// Who caused it.
    fn source(&self) -> ErrorSource {
        self.kind().default_source()
    }

    /// Whether an identical retry could plausibly succeed.
    fn retryable(&self) -> bool {
        self.kind().default_retryable()
    }

    /// How loudly to log it.
    fn severity(&self) -> Severity {
        self.kind().severity()
    }
}

/// Renders `error` and its `source()` chain as `a: b: c`.
///
/// A hop whose text equals the previous one (a transparent wrapper) is printed once. Use it
/// wherever an error is logged — `error = %chain(&e)` — so the root cause lands in the same log
/// line as the summary; a bare `%e` shows only the outermost message.
pub fn chain(error: &dyn std::error::Error) -> impl std::fmt::Display + '_ {
    Chain(error)
}

struct Chain<'a>(&'a dyn std::error::Error);

impl fmt::Display for Chain<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut previous = self.0.to_string();
        f.write_str(&previous)?;
        let mut cursor = self.0.source();
        while let Some(cause) = cursor {
            let text = cause.to_string();
            if text != previous {
                write!(f, ": {text}")?;
                previous = text;
            }
            cursor = cause.source();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-rolled error so the test needs no extra dependency.
    #[derive(Debug)]
    struct Layer {
        text: &'static str,
        source: Option<Box<Self>>,
    }

    impl fmt::Display for Layer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.text)
        }
    }

    impl std::error::Error for Layer {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.source
                .as_deref()
                .map(|s| -> &(dyn std::error::Error + 'static) { s })
        }
    }

    fn layer(text: &'static str, source: Option<Layer>) -> Layer {
        Layer {
            text,
            source: source.map(Box::new),
        }
    }

    #[test]
    fn chain_joins_every_hop_with_a_colon() {
        let err = layer(
            "service failed",
            Some(layer("failed to bind", Some(layer("address in use", None)))),
        );
        assert_eq!(
            chain(&err).to_string(),
            "service failed: failed to bind: address in use"
        );
    }

    #[test]
    fn chain_folds_a_hop_that_repeats_the_previous_text() {
        let err = layer("same", Some(layer("same", Some(layer("root", None)))));
        assert_eq!(chain(&err).to_string(), "same: root");
    }

    #[test]
    fn chain_of_a_single_error_is_its_display() {
        let err = layer("alone", None);
        assert_eq!(chain(&err).to_string(), "alone");
    }

    #[test]
    fn every_kind_maps_to_a_distinct_status_family() {
        for kind in ErrorKind::ALL {
            let status = kind.http_status();
            assert!((400..600).contains(&status), "{kind}: {status}");
            assert_eq!(kind.is_client_error(), status < 500, "{kind}");
        }
    }

    #[test]
    fn retryable_kinds_are_never_client_errors_except_rate_limit() {
        for kind in ErrorKind::ALL {
            if kind.default_retryable() {
                assert!(
                    kind == ErrorKind::RateLimited || !kind.is_client_error(),
                    "{kind}"
                );
            }
        }
    }

    #[test]
    fn as_str_is_unique_and_snake_case() {
        let mut seen = std::collections::HashSet::new();
        for kind in ErrorKind::ALL {
            let s = kind.as_str();
            assert!(s.chars().all(|c| c.is_ascii_lowercase() || c == '_'), "{s}");
            assert!(seen.insert(s), "duplicate {s}");
        }
    }
}
