//! Configuration errors.

/// Anything that can go wrong between "no configuration" and "validated [`Config`](crate::Config)".
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A source could not be read, parsed or deserialized into the schema.
    #[error("failed to read configuration")]
    Source(#[from] config::ConfigError),

    /// One or more validation rules failed. Every problem is listed.
    #[error("configuration is invalid:\n{}", format_problems(.0))]
    Invalid(Vec<String>),

    /// The effective configuration could not be rendered as TOML.
    #[error("failed to render configuration")]
    Render(#[from] toml::ser::Error),
}

fn format_problems(problems: &[String]) -> String {
    problems
        .iter()
        .map(|p| format!("  - {p}"))
        .collect::<Vec<_>>()
        .join("\n")
}
