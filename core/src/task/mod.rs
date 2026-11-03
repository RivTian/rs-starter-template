//! 任务框架：顶层监管 + 收敛策略。
//!
//! - [`TaskSupervisor`]：顶层任务（采集 / 上报 / HTTP 等）的注册、
//!   first-failure 感知与关停收敛。
//!
//! # 任务收敛三模式（何处用何种）
//!
//! 1. **JoinSet + 超时 abort + drain**（任务组）：[`TaskSupervisor`]——先给
//!    宽限期，超时逐个 abort，绝不留 detached task；
//! 2. **`select!` on cancel**（循环体）：顶层任务主循环
//!    `select! { _ = cancel.cancelled() => break, ... }`，取消是第一分支
//!    （`biased` 时关停优先于新工作）；
//! 3. **`Drop` 兜底 abort**（长活结构）：[`TaskSupervisor`] 的 Drop——
//!    monitor 自身被取消时任务也不脱管。

mod supervisor;

pub use supervisor::{TaskExit, TaskSupervisor};
