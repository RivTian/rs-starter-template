use serde::{Deserialize, Serialize};

/// Identity of this deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    /// Service name used in logs, metrics and thread names. `[a-z0-9_-]+`.
    pub name: String,
    /// Deployment environment; toggles a few defaults (e.g. log format in production).
    pub environment: Environment,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "acme-svc".to_owned(),
            environment: Environment::Development,
        }
    }
}

impl AppConfig {
    pub(crate) fn validate(&self, problems: &mut Vec<String>) {
        if self.name.is_empty() {
            problems.push("app.name must not be empty".to_owned());
        } else if !self
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            problems.push(format!(
                "app.name must match [a-z0-9_-]+, got {:?}",
                self.name
            ));
        }
    }
}

/// Where the service is running.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// A developer machine.
    Development,
    /// Pre-production.
    Staging,
    /// Serving real traffic.
    Production,
}

impl Environment {
    /// `true` when serving real traffic.
    #[must_use]
    pub const fn is_production(self) -> bool {
        matches!(self, Self::Production)
    }
}
