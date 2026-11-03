//! 应用配置的加载与运行时分发。
//!
//! - [`load`] / [`load_or_init`]：读文件 → env 占位符展开 → 反序列化的统一管线；
//! - [`ConfigStore`]：唯一写端（装配层持有），[`ConfigHandle`]：廉价 Clone 的读端；
//! - [`ConfigError`]：教学式错误（说清在哪、怎么修）。
//!
//! 当前 [`AppConfig`] 暂无业务字段：只在出现真实消费者时按需加入对应配置，
//! 避免提前复制旧实现的完整配置面。

mod app;
mod env_expand;
mod error;
mod store;

pub use app::{AppConfig, default_config_template, load, load_or_init};
pub use env_expand::expand_env_placeholders;
pub use error::ConfigError;
pub use store::{ConfigHandle, ConfigStore};
