//! 应用级错误基座。

use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    // ── 通用 ──────────────────────────────────────────────────────────────────
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    // ── 配置相关 ───────────────────────────────────────────────────────────────
    #[error("directory error: {0}")]
    DirError(String),
    #[error("config error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("config parse error: {0}")]
    ConfigParseError(#[from] toml::de::Error),
    #[error("config serialize error: {0}")]
    ConfigSerializeError(#[from] toml::ser::Error),

    // ── 内部错误（兜底） ────────────────────────────────────────────────────────
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}
