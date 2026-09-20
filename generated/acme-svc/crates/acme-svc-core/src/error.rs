//! Error classification, re-exported from the shared kernel plus logging glue.

pub use acme_svc_util::error::{Classify, ErrorKind};
pub use acme_svc_util::error::{ErrorSource, Severity};

/// Maps a [`Severity`] to the `tracing` level to log at.
#[must_use]
pub const fn tracing_level(severity: Severity) -> tracing::Level {
    match severity {
        Severity::Debug => tracing::Level::DEBUG,
        Severity::Info => tracing::Level::INFO,
        Severity::Warn => tracing::Level::WARN,
        Severity::Error => tracing::Level::ERROR,
    }
}
