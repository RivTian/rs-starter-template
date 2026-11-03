//! 通用 HTTP 客户端模块
//!
//! 提供基于 `reqwest` 的通用 HTTP 客户端，供采集、上报等业务模块复用。
//!
//! 功能包括：
//! - GET、POST、PUT、DELETE 等常见 JSON 请求
//! - 统一的请求配置（超时、重试、连接池）
//! - 默认 JSON 请求头
//! - 自动重试机制（默认重试延迟为 100ms 和 500ms）
//! - 统一的错误处理，包含 method/path 上下文
//! - JSON 响应体字段路径错误定位，不包含完整响应体
//!
//! # 重试策略
//!
//! 当遇到连接错误、请求错误或超时时，客户端会自动重试。
//! 默认的重试延迟为 `[100, 500]` 毫秒，即：
//! - 第一次失败后等待 100ms 重试
//! - 第二次失败后等待 500ms 重试
//! - 第三次失败则返回错误
//!
//! 这一策略在排查偶发耗时问题时需要考虑，实际请求耗时可能包含多次重试的累积延迟。
//!
//! # base_url 的落位
//!
//! `base_url` 只作为 [`HttpClient::with_config`] 的**独立入参**，不进
//! [`HttpClientConfig`]。理由：一份配置（超时/TLS/请求头）常需复用到多个不同
//! 地址的对端（例如探测时对同一 IP 试多个端点），把地址塞进配置会迫使调用方
//! 每次克隆改写，且留下「config.base_url 与实际入参不一致」的歧义源。
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use starter_core::http::{HttpClient, HttpClientConfig};
//!
//! let client = HttpClient::with_config(
//!     "https://api.example.com",
//!     HttpClientConfig::default(),
//! )?;
//! let data: MyResponse = client.get_json("/endpoint").await?;
//! ```

mod builder;
mod client;

pub use builder::build_reqwest_client;
pub use client::{HttpClient, HttpClientConfig, HttpClientError};
