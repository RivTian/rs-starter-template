//! 顶层任务监管
//!
//! 统一跟踪运行期顶层任务（采集 / 上报 / HTTP 等），并在关停超时时中止底层
//! 任务。`JoinSet` 中运行的是等待已有 `JoinHandle` 的监控任务，因此还需保留
//! 底层任务的 `AbortHandle`——否则仅中止 `JoinSet` 内的监控任务会使底层任务
//! 脱离管理。
//!
//! 收敛模式（参见 [`crate::task`] 模块文档的三模式之一）：JoinSet + 超时后
//! `abort` + drain；`Drop` 兜底 abort 防 monitor 自身被取消时任务脱管。

use std::time::Duration;

use tokio::task::{AbortHandle, JoinError, JoinHandle, JoinSet};

/// 运行期顶层任务意外退出的原因。
#[derive(Debug, thiserror::Error)]
pub enum TaskExit {
    #[error("runtime task {name} unexpectedly exited: {result:?}")]
    Task {
        name: &'static str,
        result: Result<(), JoinError>,
    },
    #[error("runtime task watcher unexpectedly exited: {0}")]
    Watcher(JoinError),
    #[error("all runtime tasks unexpectedly exited")]
    Empty,
}

/// 顶层任务监管器。
pub struct TaskSupervisor {
    joins: JoinSet<(&'static str, Result<(), JoinError>)>,
    aborts: Vec<(&'static str, AbortHandle)>,
}

impl Default for TaskSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskSupervisor {
    pub fn new() -> Self {
        Self {
            joins: JoinSet::new(),
            aborts: Vec::new(),
        }
    }

    /// 登记一个顶层任务。名字重复属编程错误（debug 断言拦截）。
    pub fn register(&mut self, name: &'static str, handle: JoinHandle<()>) {
        debug_assert!(
            self.aborts.iter().all(|(existing, _)| *existing != name),
            "duplicate runtime task name: {name}"
        );
        self.aborts.push((name, handle.abort_handle()));
        self.joins.spawn(async move { (name, handle.await) });
    }

    /// 已登记的顶层任务数。
    pub fn len(&self) -> usize {
        self.aborts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.aborts.is_empty()
    }

    /// 等到任意一个顶层任务退出（first-failure 收敛的感知点）。
    pub async fn next_exit(&mut self) -> TaskExit {
        match self.joins.join_next().await {
            Some(Ok((name, result))) => TaskExit::Task { name, result },
            Some(Err(error)) => TaskExit::Watcher(error),
            None => TaskExit::Empty,
        }
    }

    /// 关停收敛：给全部任务一个宽限期，超时逐个 abort 并回收。
    pub async fn wait_for_shutdown(&mut self, grace: Duration) {
        if self.joins.is_empty() {
            return;
        }

        tracing::info!(grace = ?grace, "waiting for runtime tasks to stop gracefully");
        let graceful = async {
            while let Some(exit) = self.joins.join_next().await {
                log_task_exit(exit);
            }
        };

        if tokio::time::timeout(grace, graceful).await.is_err() {
            for (name, abort) in &self.aborts {
                if !abort.is_finished() {
                    tracing::warn!(task = name, "did not stop within timeout, forcing abort");
                    abort.abort();
                }
            }

            // 等待底层任务确认中止，同时回收 JoinSet 中的监控任务。
            while let Some(exit) = self.joins.join_next().await {
                log_task_exit(exit);
            }
        }
    }
}

impl Drop for TaskSupervisor {
    fn drop(&mut self) {
        // monitor 本身若被取消，也不能让底层任务因 JoinHandle 被丢弃而变成 detached。
        for (_, abort) in &self.aborts {
            abort.abort();
        }
    }
}

fn log_task_exit(exit: Result<(&'static str, Result<(), JoinError>), JoinError>) {
    match exit {
        Ok((name, Ok(()))) => tracing::info!(task = name, "stopped"),
        Ok((name, Err(error))) if error.is_cancelled() => {
            tracing::info!(task = name, "aborted");
        }
        Ok((name, Err(error))) => {
            tracing::warn!(task = name, error = %error, "stopped with join error");
        }
        Err(error) => {
            tracing::warn!(error = %error, "runtime task watcher stopped with join error")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future;
    use std::time::Duration;

    use tokio::sync::oneshot;

    use super::{TaskExit, TaskSupervisor};

    #[tokio::test]
    async fn reports_the_named_task_that_exits() {
        let mut supervisor = TaskSupervisor::new();
        supervisor.register("worker", tokio::spawn(async {}));

        match supervisor.next_exit().await {
            TaskExit::Task { name, result } => {
                assert_eq!(name, "worker");
                result.expect("worker should exit normally");
            }
            other => panic!("unexpected exit: {other:?}"),
        }
    }

    #[tokio::test]
    async fn shutdown_timeout_aborts_stuck_tasks() {
        let (started_tx, started_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = started_tx.send(());
            future::pending::<()>().await;
        });
        let abort = handle.abort_handle();
        let mut supervisor = TaskSupervisor::new();
        supervisor.register("stuck", handle);
        started_rx.await.expect("task should start");

        supervisor.wait_for_shutdown(Duration::ZERO).await;

        assert!(abort.is_finished());
        assert!(supervisor.joins.is_empty());
    }

    #[tokio::test]
    async fn dropping_supervisor_aborts_managed_tasks() {
        let (started_tx, started_rx) = oneshot::channel();
        let handle = tokio::spawn(async move {
            let _ = started_tx.send(());
            future::pending::<()>().await;
        });
        let abort = handle.abort_handle();
        let mut supervisor = TaskSupervisor::new();
        supervisor.register("stuck", handle);
        started_rx.await.expect("task should start");

        drop(supervisor);
        tokio::task::yield_now().await;

        assert!(abort.is_finished());
    }
}
