use starter_core::config::ConfigStore;
use starter_core::events::EventBus;
use starter_core::task::TaskSupervisor;
use starter_core::tls::ensure_crypto_provider;
use starter_core::util::{BuildInfo, get_config_dir};
use starter_core::{AppError, AppResult};
use tokio_util::sync::CancellationToken;

use crate::state::RuntimeState;

use super::cli::{self, CliAction};
use super::signals::monitor;

pub async fn boot_strap() -> AppResult<()> {
    // 0. 解析 CLI
    let CliAction::Run { config } = cli::parse(std::env::args().skip(1)) else {
        // release 构建经 QUASAR_VERSION 注入 tag 版本，未注入回退包版本（旧口径）
        println!("{}", BuildInfo::current().version);
        return Ok(());
    };

    // 1. tracing 初始化（全 workspace 仅此一处；EnvFilter 默认 info；
    //    tracing-subscriber 默认特性含 log 桥接，依赖库的 log 输出一并收编）
    init_tracing();

    // 第一条日志固定打构建串：现场排查只看日志开头就知道跑的哪个版本
    tracing::info!(
        "{}",
        BuildInfo::current().service_description(env!("CARGO_PKG_NAME"))
    );

    // 2. rustls ring provider 尽早安装（所有 TLS 链路共用）
    ensure_crypto_provider();

    // 3. 配置：统一管线加载 → ConfigStore（watch 写端）；缺文件落盘内嵌模板
    let config_path = match config {
        Some(p) => p,
        None => get_config_dir()
            .map_err(AppError::DirError)?
            .join("runtime.toml"),
    };
    let config = ConfigStore::load_or_init(config_path)?;
    tracing::info!(path = %config.path().display(), "config loaded");
    let _cfg = config.current();

    // 4. 共享状态装配
    let state = RuntimeState {
        config,
        events: EventBus::default(),
        shutdown: CancellationToken::new(),
    };

    // 5. 顶层任务注册（全部 child_token；
    //    双闸的 TOML 侧： enabled=false 时不注册并打 warn——「接口全都在但数据不更新」
    //    是最难猜的一种故障，关掉必须在日志里喊出来
    let supervisor = TaskSupervisor::new();

    monitor(state, supervisor).await
}

/// tracing 初始化：`RUST_LOG` 过滤（缺省 info）；输出到 stdout。
///
/// crate 边界即过滤单元（默认 module-path target）：
/// `RUST_LOG=starter_core=debug,rs_starter_template=info`。
fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
