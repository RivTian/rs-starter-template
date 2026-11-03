//! 进程级 rustls `CryptoProvider` 安装
//!
//! # 为什么需要这个模块
//!
//! 项目里凡是走 TLS 的链路（`reqwest` HTTP 客户端，以及后续可能接入的 MQTT/
//! 数据库连接）都跑在 `rustls` 0.23 上，而 rustls 0.23 要求**必须**指定一个
//! 密码学 provider（`ring` 或 `aws-lc-rs`）。
//!
//! 默认情况下 `reqwest` 的 `rustls` feature 会展开成 `__rustls-aws-lc-rs`，把
//! `aws-lc-sys` 拖进依赖图——那是一坨需要 `cmake` 和 C 工具链的绑定，交叉编译到
//! `linux-x64 / linux-arm / macOS` 时纯属负担。因此 `Cargo.toml` 里三方全部改用
//! `*-no-provider` 变体，改由本模块显式安装 **`ring`**（纯 Rust，无 C 依赖）。
//!
//! 代价是：provider 不再由 feature 自动注入，**首次用到 TLS 之前必须调用
//! [`ensure_crypto_provider`]**，否则 rustls 会在建连时 panic。
//!
//! # 为什么用 `Once` 而不是只在 `boot_strap` 里装一次
//!
//! 单元测试、集成测试和各类 CLI 子命令都可能绕过 `boot_strap` 直接构建 HTTP
//! 客户端。把安装动作下沉到每个客户端构造函数里，并用 [`std::sync::Once`] 保证
//! 幂等，才能做到「任何入口都不会漏装」。

use std::sync::Once;

static INSTALL: Once = Once::new();

/// 确保进程级默认 `CryptoProvider` 已安装为 `ring`。
///
/// 幂等且线程安全：重复调用只有第一次生效。所有构造 TLS 客户端的代码路径
/// （如 `build_reqwest_client`）都应在建连前调用一次。
///
/// # Panics
///
/// 不会 panic。若进程中已经由别处装过 provider（例如宿主程序把本 crate 当库用），
/// `install_default` 返回 `Err`，这里按「已有可用 provider」处理并只记一条 debug 日志。
pub fn ensure_crypto_provider() {
    INSTALL.call_once(|| {
        if rustls::crypto::ring::default_provider()
            .install_default()
            .is_err()
        {
            // 已被别处安装。不是错误：我们只关心「装了且可用」，不关心是谁装的。
            tracing::debug!("rustls CryptoProvider already installed by another component");
        } else {
            tracing::debug!("rustls CryptoProvider installed: ring");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_crypto_provider_is_idempotent() {
        // 连调三次不应 panic，且事后一定能取到默认 provider
        ensure_crypto_provider();
        ensure_crypto_provider();
        ensure_crypto_provider();

        assert!(
            rustls::crypto::CryptoProvider::get_default().is_some(),
            "默认 CryptoProvider 应已就位"
        );
    }
}
