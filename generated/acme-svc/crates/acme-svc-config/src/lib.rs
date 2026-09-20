//! Configuration schema and layered loading for `acme-svc`.
//!
//! Precedence, lowest to highest:
//!
//! 1. code defaults (`impl Default` on every section — the service starts with no files at all),
//! 2. `config/default.toml`, then `config/local.toml` (or exactly one `--config <file>`),
//! 3. environment variables `ACME_SVC__<SECTION>__<KEY>` (double underscore separates levels),
//! 4. explicit overrides passed by the CLI.
//!
//! Loading is **fail-fast**: an unreadable file, an unknown key, a wrong type or a failed
//! validation rule aborts startup with every problem listed. There is no silent fallback.
//!
//! This crate deliberately denies `missing_docs`: configuration keys are user interface.

#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::unreachable
    )
)]
#![deny(missing_docs)]

mod error;
mod sections;

use std::path::PathBuf;

use config::FileFormat;
use serde::{Deserialize, Serialize};

pub use error::ConfigError;
pub use sections::{
    AppConfig, CleanupJobConfig, Environment, EventsConfig, HttpConfig, JobsConfig, LogFormat,
    RuntimeConfig, ServerConfig, TelemetryConfig,
};

/// Default configuration files, searched relative to the working directory when no explicit
/// file is given. Neither has to exist.
pub const DEFAULT_FILES: [&str; 2] = ["config/default", "config/local"];

/// The complete, validated configuration of the service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Identity of this deployment.
    pub app: AppConfig,
    /// Tokio runtime sizing.
    pub runtime: RuntimeConfig,
    /// Lifecycle timeouts.
    pub server: ServerConfig,
    /// HTTP listener.
    pub http: HttpConfig,
    /// Logs and metrics.
    pub telemetry: TelemetryConfig,
    /// In-process event bus.
    pub events: EventsConfig,
    /// Background jobs.
    pub jobs: JobsConfig,
}

/// Where configuration comes from. Built by the CLI, consumed by [`Config::load`].
#[derive(Debug, Clone)]
pub struct Sources {
    /// An explicit file. When set, [`DEFAULT_FILES`] are **not** read.
    pub file: Option<PathBuf>,
    /// Environment prefix, e.g. `ACME_SVC` for `ACME_SVC__HTTP__BIND`.
    pub env_prefix: String,
    /// Highest-priority overrides as dotted key / value pairs, e.g. `("http.bind", "0.0.0.0:80")`.
    pub overrides: Vec<(String, String)>,
}

impl Sources {
    /// Sources with only the environment prefix set.
    #[must_use]
    pub fn new(env_prefix: impl Into<String>) -> Self {
        Self {
            file: None,
            env_prefix: env_prefix.into(),
            overrides: Vec::new(),
        }
    }

    /// Read exactly this file instead of the default files.
    #[must_use]
    pub fn with_file(mut self, file: Option<PathBuf>) -> Self {
        self.file = file;
        self
    }

    /// Add a highest-priority override.
    #[must_use]
    pub fn with_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides.push((key.into(), value.into()));
        self
    }
}

impl Config {
    /// Loads, merges and validates configuration from all [`Sources`].
    pub fn load(sources: &Sources) -> Result<Self, ConfigError> {
        let mut builder = config::Config::builder();

        builder = match &sources.file {
            Some(path) => builder.add_source(
                config::File::from(path.as_path())
                    .format(FileFormat::Toml)
                    .required(true),
            ),
            None => DEFAULT_FILES.iter().fold(builder, |b, name| {
                b.add_source(
                    config::File::with_name(name)
                        .format(FileFormat::Toml)
                        .required(false),
                )
            }),
        };

        builder = builder.add_source(
            config::Environment::with_prefix(&sources.env_prefix)
                .prefix_separator("__")
                .separator("__")
                .try_parsing(true),
        );

        for (key, value) in &sources.overrides {
            builder = builder.set_override(key.as_str(), value.as_str())?;
        }

        let cfg: Self = builder.build()?.try_deserialize()?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Checks cross-field rules and reports **all** violations at once.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut problems = Vec::new();
        self.app.validate(&mut problems);
        self.runtime.validate(&mut problems);
        self.server.validate(&mut problems);
        self.http.validate(&mut problems);
        self.telemetry.validate(&mut problems);
        self.events.validate(&mut problems);
        self.jobs.validate(&mut problems);
        if problems.is_empty() {
            Ok(())
        } else {
            Err(ConfigError::Invalid(problems))
        }
    }

    /// The effective configuration as TOML, with secrets replaced by `***`.
    pub fn redacted(&self) -> Result<String, ConfigError> {
        Ok(toml::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// A prefix no real environment variable starts with.
    const PREFIX: &str = "ACME_SVC_TEST_NONE";

    #[test]
    fn defaults_are_valid_and_round_trip() {
        let cfg = Config::default();
        cfg.validate().expect("defaults must validate");
        let text = cfg.redacted().expect("render");
        let back: Config = toml::from_str(&text).expect("parse rendered defaults");
        assert_eq!(back.http.bind, cfg.http.bind);
        assert_eq!(back.server.grace_period, Duration::from_secs(30));
    }

    #[test]
    fn overrides_beat_defaults() {
        let sources = Sources::new(PREFIX)
            .with_file(None)
            .with_override("http.bind", "127.0.0.1:1")
            .with_override("server.grace_period", "5s");
        let cfg = Config::load(&sources).expect("load");
        assert_eq!(cfg.http.bind.port(), 1);
        assert_eq!(cfg.server.grace_period, Duration::from_secs(5));
    }

    #[test]
    fn validation_reports_every_problem() {
        let sources = Sources::new(PREFIX)
            .with_override("server.grace_period", "0s")
            .with_override("events.capacity", "0")
            .with_override("app.name", "");
        let err = Config::load(&sources).expect_err("must fail");
        let ConfigError::Invalid(problems) = err else {
            panic!("expected Invalid, got {err}")
        };
        assert_eq!(problems.len(), 3, "{problems:?}");
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let sources = Sources::new(PREFIX).with_override("http.bnid", "x");
        assert!(Config::load(&sources).is_err());
    }
}
