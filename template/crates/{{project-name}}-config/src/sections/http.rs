use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use bytesize::ByteSize;
use serde::{Deserialize, Serialize};

use super::require_at_least;

/// The HTTP listener.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HttpConfig {
    /// Address to bind. Use `0.0.0.0:<port>` inside containers.
    pub bind: SocketAddr,
    /// Whole-request deadline; slower requests get `408`.
    #[serde(with = "humantime_serde")]
    pub request_timeout: Duration,
    /// Maximum request body, e.g. `"1 MiB"`.
    pub body_limit: ByteSize,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            bind: SocketAddr::from((Ipv4Addr::LOCALHOST, 8080)),
            request_timeout: Duration::from_secs(30),
            body_limit: ByteSize::mib(1),
        }
    }
}

impl HttpConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        require_at_least(
            problems,
            "http.request_timeout",
            self.request_timeout,
            Duration::from_secs(1),
        );
        if self.body_limit < ByteSize::kib(1) {
            problems.push(format!(
                "http.body_limit must be at least 1 KiB, got {}",
                self.body_limit
            ));
        }
    }

    /// The body limit in bytes, for middleware that wants an integer.
    #[must_use]
    pub fn body_limit_bytes(&self) -> usize {
        usize::try_from(self.body_limit.as_u64()).unwrap_or(usize::MAX)
    }
}
