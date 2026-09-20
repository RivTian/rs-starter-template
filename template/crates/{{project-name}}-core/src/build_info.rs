//! Build provenance, filled in by the binary crate's `build.rs` and shown by `--version`,
//! the first log line and `GET /version`.

use serde::Serialize;

/// Static facts about the running binary.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct BuildInfo {
    /// Cargo package name.
    pub name: &'static str,
    /// Cargo package version.
    pub version: &'static str,
    /// Short git commit hash, or `unknown` outside a git checkout.
    pub git_sha: &'static str,
    /// `git describe` output, or `unknown`.
    pub git_describe: &'static str,
    /// Build timestamp (RFC 3339), or `unknown`.
    pub build_timestamp: &'static str,
    /// `rustc` version used to build.
    pub rustc: &'static str,
    /// `debug` or `release`.
    pub profile: &'static str,
}

impl BuildInfo {
    /// One line suitable for `--version` and the first log line.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{} {} ({} {})",
            self.name, self.version, self.git_describe, self.build_timestamp
        )
    }
}
