//! 配置错误
//!
//! 四个变体对应加载管线的四个失败面：读文件 / env 占位符 / 反序列化 /
//! 语义校验。信息一律**教学式**：不只说错了，还要说在哪、怎么修——
//! 配置错误的读者是凌晨被叫起来的运维，不是写这段代码的人。

use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("读取配置失败 {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// 占位符要求的环境变量未设置且无默认值。
    ///
    /// 语法：`{{env.NAME}}`（必须提供）或 `{{env.NAME:default}}`（缺省回退）。
    #[error(
        "环境变量占位符 {{{{env.{name}}}}} 未设置且无默认值；请设置环境变量 {name}，或改写为 {{{{env.{name}:默认值}}}}"
    )]
    EnvMissing { name: String },

    /// 反序列化失败；`path_in_doc` 由 `serde_path_to_error` 提供字段路径
    /// （如 `collector.interval_ms`），比裸 toml 错误的行列号好定位得多。
    #[error("配置解析失败于 {path_in_doc}: {message}")]
    Parse {
        path_in_doc: String,
        message: String,
    },

    /// 语义校验失败（硬错误）。信息含修复建议。
    #[error("配置校验失败: {0}")]
    Invalid(String),
}
