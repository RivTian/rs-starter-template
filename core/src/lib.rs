//! `starter-core` —— Rust Starter Template workspace 的叶子内核。
//!
//! 承载所有 crate 共享、且**不含常驻任务**的内容：配置类型（[`config`]）、
//! 错误基座（[`error`]）、事件总线（[`events`]）、任务监管（[`task`]）、
//! HTTP 客户端基建（[`http`]）、rustls provider 安装（[`tls`]）与构建元数据/
//! 目录约定（[`util`]）。
//!
//! # 纪律
//!
//! - 不依赖任何兄弟 crate（叶子）；
//! - 不初始化日志：只 `use tracing`，subscriber 安装仅在 `starter-app`；
//! - 无 `tokio::spawn` 的常驻任务；无 SQL。

pub mod config;
pub mod error;
pub mod events;
pub mod http;
pub mod task;
pub mod tls;
pub mod util;

// 窄门面例外：错误基座沿用顶层导出口径，`use starter_core::{AppError, AppResult}`。
pub use error::{AppError, AppResult};

/// workspace 统一版本（来自 `workspace.package.version`）。
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
