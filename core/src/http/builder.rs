//! `reqwest::Client` 构建器辅助函数

use super::HttpClientConfig;

/// 根据配置构建 `reqwest::Client`
///
/// 统一处理超时、连接池、TLS 证书验证等配置。
///
/// # 示例
///
/// ```rust,ignore
/// use starter_core::http::{HttpClientConfig, build_reqwest_client};
/// use std::time::Duration;
///
/// let config = HttpClientConfig {
///     connect_timeout: Duration::from_secs(3),
///     request_timeout: Duration::from_secs(10),
///     accept_invalid_certs: true, // 仅用于开发环境
///     ..Default::default()
/// };
///
/// let client = build_reqwest_client(&config)?;
/// ```
pub fn build_reqwest_client(config: &HttpClientConfig) -> Result<reqwest::Client, reqwest::Error> {
    // reqwest 走的是 `rustls-no-provider`，provider 不由 feature 注入；
    // 漏装会在首次 HTTPS 建连时 panic，所以放在这里兜底（幂等）。
    crate::tls::ensure_crypto_provider();

    let mut builder = reqwest::Client::builder()
        .connect_timeout(config.connect_timeout)
        .timeout(config.request_timeout)
        .pool_idle_timeout(config.pool_idle_timeout)
        .no_proxy(); // 禁用系统代理，避免内网设备走公司代理

    // TLS 证书验证：仅开发/自签场景放开
    if config.accept_invalid_certs {
        builder = builder
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true);
    }

    if let Some(keepalive) = config.tcp_keepalive {
        builder = builder.tcp_keepalive(keepalive);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn build_client_with_default_config() {
        assert!(build_reqwest_client(&HttpClientConfig::default()).is_ok());
    }

    #[test]
    fn build_client_with_custom_config() {
        let config = HttpClientConfig {
            connect_timeout: Duration::from_secs(3),
            request_timeout: Duration::from_secs(10),
            pool_idle_timeout: Duration::from_secs(30),
            accept_invalid_certs: true,
            tcp_keepalive: Some(Duration::from_secs(15)),
            extra_headers: Default::default(),
        };

        assert!(build_reqwest_client(&config).is_ok());
    }
}
