//! 监控循环与统一关停

use std::time::Duration;

use starter_core::events::AgentEvent;
use starter_core::task::{TaskExit, TaskSupervisor};
use starter_core::{AppError, AppResult};

use crate::state::RuntimeState;

/// 顶层任务的关停宽限期。
const SHUTDOWN_GRACE: Duration = Duration::from_secs(10);

/// 运行监控：等待退出信号 / 热重载信号 / 任务异常退出。
pub async fn monitor(state: RuntimeState, mut supervisor: TaskSupervisor) -> AppResult<()> {
    let run_result = run_loop(&state, &mut supervisor).await;
    shutdown_runtime(state, supervisor).await;
    run_result
}

#[cfg(unix)]
async fn run_loop(state: &RuntimeState, supervisor: &mut TaskSupervisor) -> AppResult<()> {
    use tokio::signal::unix::{SignalKind, signal};

    // 尚未登记任何顶层任务：没有监控对象，留痕后正常退出（仍走统一关停序列）。
    // 否则 select 中 next_exit 立即返回 Empty，被 first_failure 判为异常。
    if supervisor.is_empty() {
        tracing::warn!("no runtime tasks registered, exiting");
        return Ok(());
    }

    let mut sigterm = signal(SignalKind::terminate()).map_err(AppError::IoError)?;
    let mut sighup = signal(SignalKind::hangup()).map_err(AppError::IoError)?;

    loop {
        tokio::select! {
            signal = tokio::signal::ctrl_c() => {
                return match signal {
                    Ok(()) => {
                        tracing::info!("shutdown signal received (ctrl_c)");
                        Ok(())
                    }
                    // 信号监听失败同样进入关停序列，避免跳过资源清理
                    Err(error) => Err(AppError::IoError(error)),
                };
            }
            _ = sigterm.recv() => {
                tracing::info!("shutdown signal received (SIGTERM)");
                return Ok(());
            }
            _ = sighup.recv() => {
                // 热重载：失败保留旧配置，绝不半套用（ConfigStore::reload 保证）
                match state.config.reload() {
                    Ok(report) => {
                        state.events.publish(AgentEvent::ConfigReloaded {
                            generation: report.generation,
                        });
                        tracing::info!(
                            generation = report.generation,
                            applied = ?report.applied,
                            deferred = ?report.deferred,
                            requires_restart = ?report.requires_restart,
                            "config reloaded"
                        );
                    }
                    Err(error) => {
                        tracing::warn!(error = %error, "config reload rejected, keeping last good");
                    }
                }
            }
            exit = supervisor.next_exit() => {
                return Err(first_failure(exit));
            }
        }
    }
}

/// 非 unix 平台：无 SIGTERM/SIGHUP，仅 ctrl_c（行为对齐旧实现）。
#[cfg(not(unix))]
async fn run_loop(_state: &RuntimeState, supervisor: &mut TaskSupervisor) -> AppResult<()> {
    // 同 unix 版：空 supervisor 正常退出，避免 next_exit 的 Empty 被判为异常。
    if supervisor.is_empty() {
        tracing::warn!("no runtime tasks registered, exiting");
        return Ok(());
    }

    tokio::select! {
        signal = tokio::signal::ctrl_c() => match signal {
            Ok(()) => {
                tracing::info!("shutdown signal received (ctrl_c)");
                Ok(())
            }
            Err(error) => Err(AppError::IoError(error)),
        },
        exit = supervisor.next_exit() => Err(first_failure(exit)),
    }
}

/// first-failure 收敛：任一顶层任务退出即结束进程。
fn first_failure(exit: TaskExit) -> AppError {
    tracing::error!(exit = %exit, "top-level task exited, shutting down");
    AppError::Internal(anyhow::Error::new(exit))
}

/// 统一关停序列（所有退出路径共用）。
async fn shutdown_runtime(state: RuntimeState, mut supervisor: TaskSupervisor) {
    // ShuttingDown 先于取消广播：给旁路一个在取消风暴前 flush 的机会
    // （事件语义见 starter_core::events::AgentEvent::ShuttingDown）
    state.events.publish(AgentEvent::ShuttingDown);
    state.shutdown.cancel();
    supervisor.wait_for_shutdown(SHUTDOWN_GRACE).await;
}

#[cfg(test)]
mod tests {
    use starter_core::config::ConfigStore;
    use starter_core::events::EventBus;
    use starter_core::task::TaskSupervisor;
    use tokio_util::sync::CancellationToken;

    use crate::state::RuntimeState;

    use super::monitor;

    /// 尚未登记任何顶层任务时应正常退出（含统一关停序列），而非被判为 first-failure。
    #[tokio::test]
    async fn empty_supervisor_exits_normally() {
        let dir = std::env::temp_dir().join(format!("starter-app-signals-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp config dir");
        let config =
            ConfigStore::load_or_init(dir.join("runtime.toml")).expect("init embedded template");
        let state = RuntimeState {
            config,
            events: EventBus::default(),
            shutdown: CancellationToken::new(),
        };

        let result = monitor(state, TaskSupervisor::new()).await;

        let _ = std::fs::remove_dir_all(&dir);
        result.expect("empty supervisor should exit normally");
    }
}
