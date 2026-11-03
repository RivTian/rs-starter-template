//! 全局运行时状态
//!
//! 由 [`crate::boot::boot_strap`] 一次性装配；除 `ConfigStore` 写端外全部以
//! `Arc`/Clone 注入各面。**进程内不再有任何业务全局态**——测试并行化的前提。
//!
//! # 字段按需补齐
//!
//! 仅在出现真实消费者时添加字段，不预放无实义的占位类型。

use tokio_util::sync::CancellationToken;

use starter_core::config::ConfigStore;
use starter_core::events::EventBus;

/// 运行期共享资源集合；生命周期统一归 monitor 管理。
pub struct RuntimeState {
    /// watch 写端（`handle()` 派生只读句柄注入各任务）
    pub config: ConfigStore,
    pub events: EventBus,
    /// 根令牌；一切任务用 `child_token()`（级联取消）
    pub shutdown: CancellationToken,
}
