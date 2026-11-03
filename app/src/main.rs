//! `rs-starter-template` 引导入口
//!
//! 全部逻辑在 [`boot::boot_strap`]；这里只负责把结果变成进程退出码。

mod boot;
mod cli;
mod signals;
mod state;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match boot::boot_strap().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            // tracing 可能尚未初始化（如配置加载失败前），stderr 双保险
            eprintln!("rs-starter-template: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
