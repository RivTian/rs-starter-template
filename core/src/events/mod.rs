//! 轻量事件总线

use tokio::sync::broadcast;

/// 进程内事件（只带键与元数据，不带载荷）。
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// runtime.toml 热重载完成（携带代际号，消费者按需读 ConfigHandle）
    ConfigReloaded { generation: u64 },
    /// 进程进入关停（先于 CancellationToken 广播，供旁路提前 flush）
    ShuttingDown,
}

/// 事件总线：Clone 廉价（共享同一 broadcast 通道）。
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<AgentEvent>,
}

impl EventBus {
    /// `cap` 为通道容量（生产路径用 [`Default`] 的 256；测试可给小值验证 Lagged 行为）。
    pub fn new(cap: usize) -> Self {
        Self {
            tx: broadcast::channel(cap).0,
        }
    }

    /// 发布：永不阻塞、永不失败；无订阅者时静默丢弃。
    pub fn publish(&self, ev: AgentEvent) {
        let _ = self.tx.send(ev);
    }

    pub fn subscribe(&self) -> EventStream {
        EventStream {
            rx: self.tx.subscribe(),
            lagged_total: 0,
        }
    }
}

impl Default for EventBus {
    /// 默认容量 256：事件只载键不载荷，单条开销极小；该深度足以吸收
    /// 热重载、关停等低频事件的瞬时峰值，又不至于让慢消费者长期滞后。
    fn default() -> Self {
        Self::new(256)
    }
}

/// 单个订阅者的接收流。
pub struct EventStream {
    rx: broadcast::Receiver<AgentEvent>,
    lagged_total: u64,
}

impl EventStream {
    /// 收下一个事件。
    ///
    /// `Lagged(n)` 不是错误：记 debug 计数后继续。事件只载键，丢事件不丢数据，
    /// 消费者兜底轮询会追平——这是「通知不载荷」原则换来的免背压设计。
    ///
    /// 返回 `None` 表示总线已整体销毁（进程关停途中），消费循环据此退出。
    pub async fn recv(&mut self) -> Option<AgentEvent> {
        loop {
            match self.rx.recv().await {
                Ok(ev) => return Some(ev),
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    self.lagged_total += n;
                    tracing::debug!(
                        skipped = n,
                        total = self.lagged_total,
                        "event stream lagged; polling fallback will catch up"
                    );
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    }

    /// 累计因滞后被跳过的事件数（自省用）。
    pub fn lagged_total(&self) -> u64 {
        self.lagged_total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_subscribe_roundtrip() {
        let bus = EventBus::new(8);
        let mut stream = bus.subscribe();
        bus.publish(AgentEvent::ShuttingDown);
        match stream.recv().await {
            Some(AgentEvent::ShuttingDown) => {}
            other => panic!("unexpected: {other:?}"),
        }
    }

    /// 无订阅者时发布不阻塞不报错
    #[test]
    fn publish_without_subscribers_is_silent() {
        let bus = EventBus::new(8);
        bus.publish(AgentEvent::ConfigReloaded { generation: 1 });
    }

    /// 慢消费者：Lagged 被吸收，后续事件照常接收，计数可见
    #[tokio::test]
    async fn lagged_receiver_skips_and_continues() {
        let bus = EventBus::new(2);
        let mut stream = bus.subscribe();
        for g in 0..5 {
            bus.publish(AgentEvent::ConfigReloaded { generation: g });
        }
        // 容量 2：最早的事件被挤掉，第一次 recv 吸收 Lagged 后返回仍在队里的
        let ev = stream.recv().await.expect("bus alive");
        match ev {
            AgentEvent::ConfigReloaded { generation } => assert!(generation >= 3),
            other => panic!("unexpected: {other:?}"),
        }
        assert!(stream.lagged_total() > 0);
    }

    /// 总线销毁后 recv 返回 None（消费循环退出的判据）
    #[tokio::test]
    async fn recv_returns_none_after_bus_drop() {
        let bus = EventBus::new(2);
        let mut stream = bus.subscribe();
        drop(bus);
        assert!(stream.recv().await.is_none());
    }
}
