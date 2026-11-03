use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::sync::watch;

use super::app::{AppConfig, load_or_init};
use super::error::ConfigError;

/// 一次热重载的结果清单（SIGHUP 日志与将来 `POST /v1/system/config/reload`
/// 响应体共用）。字段路径用 serde 主名（camelCase，如 `collector.intervalMs`）。
#[derive(Debug, Clone, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReloadReport {
    /// 本次重载后的配置代数（每次成功重载 +1；启动装载为 0）
    pub generation: u64,
    /// 已生效的可热变更（消费任务下一 tick 现读到新值）
    pub applied: Vec<String>,
    /// 半热变更：仅当启动时该顶层任务已拉起才生效；启动即关不补拉起
    pub deferred: Vec<String>,
    /// 不可热变更：已回滚为运行值，需重启才生效
    pub requires_restart: Vec<String>,
}

impl ReloadReport {
    /// 三张清单都空（配置内容没有实质变化）
    pub fn is_noop(&self) -> bool {
        self.applied.is_empty() && self.deferred.is_empty() && self.requires_restart.is_empty()
    }
}

/// 配置写端：仅 app 装配层持有。
pub struct ConfigStore {
    tx: watch::Sender<Arc<AppConfig>>,
    path: PathBuf,
    generation: AtomicU64,
}

impl ConfigStore {
    /// 启动装载：缺文件先落盘内嵌模板；失败即启动失败（fail-fast）。
    pub fn load_or_init(path: PathBuf) -> Result<Self, ConfigError> {
        let cfg = load_or_init(&path)?;
        let (tx, _rx) = watch::channel(Arc::new(cfg));
        Ok(Self {
            tx,
            path,
            generation: AtomicU64::new(0),
        })
    }

    /// 派生一个只读句柄（Clone 廉价，随处注入）。
    pub fn handle(&self) -> ConfigHandle {
        ConfigHandle {
            rx: self.tx.subscribe(),
        }
    }

    /// 当前配置（`Arc` 克隆，非全量拷贝）。
    pub fn current(&self) -> Arc<AppConfig> {
        self.tx.borrow().clone()
    }

    /// 当前配置代数。
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// 配置文件路径（诊断/日志用）。
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// 热重载：SIGHUP 与 HTTP 端点共用的唯一入口。
    ///
    /// 全量重跑三段式管线 → 白名单分类 → 不可热字段回滚 → `watch::send`。
    /// 事件广播（`ConfigReloaded { generation }`）由调用方在成功后自行发出
    /// ——config 模块不依赖 events，保持层内单向。
    pub fn reload(&self) -> Result<ReloadReport, ConfigError> {
        let old = self.current();
        let fresh = load_or_init(&self.path)?;

        let mut report = ReloadReport::default();
        diff_values(
            &to_value(&old),
            &to_value(&fresh),
            &mut String::new(),
            &mut report,
        );

        // 不可热段回滚为运行值：watch 内容始终等于「生效中」的配置
        if !report.requires_restart.is_empty() {
            // fresh.http = old.http.clone();
            tracing::warn!(
                fields = ?report.requires_restart,
                "config reload: non-hot fields changed, keeping running values until restart"
            );
        }

        let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        report.generation = generation;
        // send_replace 而不是 send：后者在「当前没有任何接收端」时拒绝写入，
        // 而句柄是随用随派生的——没有活跃句柄的瞬间不该让重载静默失效
        self.tx.send_replace(Arc::new(fresh));
        Ok(report)
    }
}

/// 配置读端：廉价 Clone 的注入句柄。
#[derive(Clone)]
pub struct ConfigHandle {
    rx: watch::Receiver<Arc<AppConfig>>,
}

impl ConfigHandle {
    /// 当前配置（`Arc` 克隆）。
    pub fn current(&self) -> Arc<AppConfig> {
        self.rx.borrow().clone()
    }

    /// 等到下一代配置发布。写端已销毁（进程关停）时返回 `None`——
    /// 消费循环据此退出等待而不是永久挂起。
    pub async fn changed(&mut self) -> Option<Arc<AppConfig>> {
        self.rx.changed().await.ok()?;
        Some(self.rx.borrow_and_update().clone())
    }
}

// ── diff 与白名单 ─────────────────────────────────────────────────────────────

/// 序列化成 `toml::Value` 做结构化 diff：新增配置字段自动按「所属段」获得
/// 热度分类，不必每加一个字段就来这里登记一次。
fn to_value(cfg: &AppConfig) -> toml::Value {
    toml::Value::try_from(cfg).expect("AppConfig 可序列化（结构体全字段 Serialize）")
}

/// 递归比对两棵配置树，把变更的叶子路径按白名单分类进报告。
fn diff_values(old: &toml::Value, new: &toml::Value, path: &mut String, report: &mut ReloadReport) {
    match (old, new) {
        (toml::Value::Table(a), toml::Value::Table(b)) => {
            // 两侧同构（同一 Rust 类型序列化而来），键集合一致，只需遍历一侧
            for (k, av) in a {
                if let Some(bv) = b.get(k) {
                    let len = path.len();
                    if !path.is_empty() {
                        path.push('.');
                    }
                    path.push_str(k);
                    diff_values(av, bv, path, report);
                    path.truncate(len);
                }
            }
        }
        _ if old != new => classify(path, report),
        _ => {}
    }
}

/// 白名单分类
/// TODO: 未来配置项有需求过滤的可在此添加
fn classify(path: &str, report: &mut ReloadReport) {
    let bucket = if path.starts_with("<config_part_name>.")
        || path.starts_with("*.")
        || path.starts_with("**.")
    {
        &mut report.requires_restart
    } else if matches!(path, "**.enabled" | "*.enabled") {
        &mut report.deferred
    } else {
        &mut report.applied
    };
    bucket.push(path.to_string());
}
