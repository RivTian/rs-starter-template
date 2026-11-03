//! HTTP 客户端核心实现

use std::time::Duration;

use reqwest::{
    Method, StatusCode, Url,
    header::{HeaderMap, HeaderValue},
};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

use super::build_reqwest_client;

/// 默认重试延迟（毫秒）：失败后依次等待 100ms、500ms，第三次失败即返回错误。
const DEFAULT_RETRY_DELAYS_MS: [u64; 2] = [100, 500];

/// HTTP 客户端配置
///
/// 只描述**传输行为**（超时/连接池/TLS/公共请求头），不含对端地址——
/// `base_url` 由 [`HttpClient::with_config`] 独立传入，便于一份配置复用到多个对端。
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// 连接超时时间
    pub connect_timeout: Duration,
    /// 请求超时时间
    pub request_timeout: Duration,
    /// 连接池空闲超时时间
    pub pool_idle_timeout: Duration,
    /// 是否接受无效的 TLS 证书（仅用于开发/测试）
    pub accept_invalid_certs: bool,
    /// TCP Keep-Alive 间隔
    pub tcp_keepalive: Option<Duration>,
    /// 每个请求都会携带的额外请求头
    pub extra_headers: HeaderMap,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(2),
            request_timeout: Duration::from_secs(5),
            pool_idle_timeout: Duration::from_secs(10),
            accept_invalid_certs: false,
            tcp_keepalive: None,
            extra_headers: HeaderMap::new(),
        }
    }
}

/// 通用 HTTP 客户端请求错误。
///
/// JSON 反序列化错误只包含字段路径和 serde 错误信息，**不包含完整响应体**
/// （响应体可能含令牌等敏感数据，不应进日志）。
#[derive(Debug, Error)]
pub enum HttpClientError {
    /// HTTP 客户端构建错误
    #[error("HTTP client build error: {source}")]
    ClientBuild {
        /// 源错误
        #[source]
        source: reqwest::Error,
    },

    /// URL 解析错误
    #[error("URL error for path {path}: {source}")]
    Url {
        /// 请求路径
        path: String,
        /// 源错误
        #[source]
        source: url::ParseError,
    },

    /// HTTP 请求传输错误
    #[error("Request failed [{method} {path}]: {source}")]
    Transport {
        /// HTTP 方法
        method: Method,
        /// 请求路径
        path: String,
        /// 源错误
        #[source]
        source: reqwest::Error,
    },

    /// HTTP 状态码错误
    #[error("Request failed [{method} {path}]: HTTP {status}: {body}")]
    Status {
        /// HTTP 方法
        method: Method,
        /// 请求路径
        path: String,
        /// HTTP 状态码
        status: StatusCode,
        /// 响应体内容
        body: String,
    },

    /// JSON 响应体解析错误
    #[error("Request failed [{method} {path}]: JSON decode error at {field_path}: {message}")]
    JsonDecode {
        /// HTTP 方法
        method: Method,
        /// 请求路径
        path: String,
        /// 出错字段路径。根节点出错时为 `.`。
        field_path: String,
        /// `serde_json` 返回的错误描述
        message: String,
        /// JSON 行号
        line: usize,
        /// JSON 列号
        column: usize,
    },
}

impl HttpClientError {
    fn client_build(source: reqwest::Error) -> Self {
        Self::ClientBuild { source }
    }

    fn url(path: &str, source: url::ParseError) -> Self {
        Self::Url {
            path: path.to_string(),
            source,
        }
    }

    fn transport(method: &Method, path: &str, source: reqwest::Error) -> Self {
        Self::Transport {
            method: method.clone(),
            path: path.to_string(),
            source,
        }
    }

    fn status(method: &Method, path: &str, status: StatusCode, body: String) -> Self {
        Self::Status {
            method: method.clone(),
            path: path.to_string(),
            status,
            body,
        }
    }

    fn json_decode(
        method: &Method,
        path: &str,
        field_path: String,
        source: serde_json::Error,
    ) -> Self {
        Self::JsonDecode {
            method: method.clone(),
            path: path.to_string(),
            field_path,
            message: source.to_string(),
            line: source.line(),
            column: source.column(),
        }
    }
}

/// 通用 HTTP 客户端。
///
/// 统一处理 URL 拼接、默认 JSON 请求头、重试、状态码检查和 JSON 字段路径错误定位。
///
/// # 示例
///
/// ```rust,ignore
/// use starter_core::http::{HttpClient, HttpClientConfig};
///
/// let client = HttpClient::with_config(
///     "https://api.example.com",
///     HttpClientConfig::default(),
/// )?;
/// let data: MyResponse = client.get_json("/endpoint").await?;
/// ```
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: reqwest::Client,
    base_url: Url,
    retry_delays: Vec<u64>,
    extra_headers: HeaderMap,
}

impl HttpClient {
    /// 以默认配置创建 HTTP 客户端。
    pub fn new(base_url: impl AsRef<str>) -> Result<Self, HttpClientError> {
        Self::with_config(base_url, HttpClientConfig::default())
    }

    /// 使用自定义配置创建 HTTP 客户端。
    pub fn with_config(
        base_url: impl AsRef<str>,
        config: HttpClientConfig,
    ) -> Result<Self, HttpClientError> {
        let base_url = base_url.as_ref();
        let parsed = Url::parse(base_url).map_err(|err| HttpClientError::url(base_url, err))?;
        let client = build_reqwest_client(&config).map_err(HttpClientError::client_build)?;

        Ok(Self {
            client,
            base_url: parsed,
            retry_delays: DEFAULT_RETRY_DELAYS_MS.to_vec(),
            extra_headers: config.extra_headers,
        })
    }

    /// 获取底层 `reqwest::Client` 引用。
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// 获取 `base_url` 引用。
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// 构建完整 URL。
    ///
    /// 入参 `path` 视为**相对 base_url 的相对路径**，前导 `/` 会被剥掉——否则
    /// `Url::join` 会把 base 的路径段整体丢弃（`http://h/api` join `/x` ⇒ `http://h/x`）。
    pub fn build_url(&self, path: &str) -> Result<Url, HttpClientError> {
        let normalized = path.trim().trim_start_matches('/');
        self.base_url()
            .join(normalized)
            .map_err(|err| HttpClientError::url(path, err))
    }

    /// 构建默认请求头（`Accept: application/json` + 配置中的额外请求头）。
    pub fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        headers.extend(self.extra_headers.clone());
        headers
    }

    /// 构建默认请求头并叠加本次请求的额外请求头（同名以本次为准）。
    pub fn headers_with_extra(&self, extra_headers: &HeaderMap) -> HeaderMap {
        let mut headers = self.default_headers();
        for (k, v) in extra_headers.iter() {
            headers.insert(k.clone(), v.clone());
        }
        headers
    }

    /// 判断是否应该重试该错误。
    ///
    /// 只重试**连接/发送/超时**类失败——这些是幂等安全的传输层抖动；
    /// 4xx/5xx 状态码不在此列（由调用方按语义决定）。
    pub fn should_retry(&self, err: &reqwest::Error) -> bool {
        err.is_connect() || err.is_request() || err.is_timeout()
    }

    /// 重试延迟策略（毫秒）。
    pub fn retry_delays(&self) -> &[u64] {
        &self.retry_delays
    }

    async fn send_with_retry(
        &self,
        method: &Method,
        path: &str,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response, HttpClientError> {
        let delays = self.retry_delays();
        let mut next_request = Some(request);
        let mut last_err = None;

        for idx in 0..=delays.len() {
            let Some(builder) = next_request.take() else {
                break;
            };
            // 先克隆备用（带 body 的流式请求不可克隆 ⇒ None ⇒ 不再重试）
            next_request = builder.try_clone();

            match builder.send().await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    let should_retry = self.should_retry(&err);
                    last_err = Some(err);

                    if !should_retry || idx >= delays.len() || next_request.is_none() {
                        break;
                    }

                    tokio::time::sleep(Duration::from_millis(delays[idx])).await;
                }
            }
        }

        Err(HttpClientError::transport(
            method,
            path,
            last_err.expect("at least one attempt"),
        ))
    }

    /// 执行 JSON 请求并解析响应。
    pub async fn execute_json<T>(
        &self,
        method: Method,
        path: &str,
        request: reqwest::RequestBuilder,
    ) -> Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        self.execute_json_with_headers(method, path, request, &HeaderMap::new())
            .await
    }

    /// 执行 JSON 请求并解析响应，允许为本次请求叠加额外请求头。
    pub async fn execute_json_with_headers<T>(
        &self,
        method: Method,
        path: &str,
        mut request: reqwest::RequestBuilder,
        extra_headers: &HeaderMap,
    ) -> Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        request = request.headers(self.headers_with_extra(extra_headers));

        let response = self.send_with_retry(&method, path, request).await?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(HttpClientError::status(&method, path, status, body));
        }

        let body = response
            .bytes()
            .await
            .map_err(|err| HttpClientError::transport(&method, path, err))?;
        self.deserialize_json(&method, path, &body)
    }

    /// 解析 JSON 响应体，错误带**字段路径**定位。
    pub fn deserialize_json<T>(
        &self,
        method: &Method,
        path: &str,
        body: &[u8],
    ) -> Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        let mut deserializer = serde_json::Deserializer::from_slice(body);
        let value = serde_path_to_error::deserialize(&mut deserializer).map_err(|err| {
            HttpClientError::json_decode(method, path, err.path().to_string(), err.into_inner())
        })?;
        // 拒绝尾随内容：`{...} false` 这类畸形响应必须报错而非静默取前半
        deserializer
            .end()
            .map_err(|err| HttpClientError::json_decode(method, path, ".".to_string(), err))?;
        Ok(value)
    }

    /// GET 请求，返回 JSON 响应。
    pub async fn get_json<T>(&self, path: &str) -> Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        let request = self.client().request(Method::GET, url);
        self.execute_json(Method::GET, path, request).await
    }

    /// GET 请求，叠加本次请求的额外请求头并返回 JSON 响应。
    pub async fn get_json_with_headers<T>(
        &self,
        path: &str,
        extra_headers: &HeaderMap,
    ) -> Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        let request = self.client().request(Method::GET, url);
        self.execute_json_with_headers(Method::GET, path, request, extra_headers)
            .await
    }

    /// POST 请求，发送 JSON body，返回 JSON 响应。
    pub async fn post_json<B, T>(&self, path: &str, body: &B) -> Result<T, HttpClientError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        let request = self.client().request(Method::POST, url).json(body);
        self.execute_json(Method::POST, path, request).await
    }

    /// POST 请求，叠加本次请求的额外请求头，发送 JSON body 并返回 JSON 响应。
    pub async fn post_json_with_headers<B, T>(
        &self,
        path: &str,
        body: &B,
        extra_headers: &HeaderMap,
    ) -> Result<T, HttpClientError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        let request = self.client().request(Method::POST, url).json(body);
        self.execute_json_with_headers(Method::POST, path, request, extra_headers)
            .await
    }

    /// PUT 请求，发送 JSON body，返回 JSON 响应。
    pub async fn put_json<B, T>(&self, path: &str, body: &B) -> Result<T, HttpClientError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        let request = self.client().request(Method::PUT, url).json(body);
        self.execute_json(Method::PUT, path, request).await
    }

    /// DELETE 请求，返回 JSON 响应。
    pub async fn delete_json<T>(&self, path: &str) -> Result<T, HttpClientError>
    where
        T: DeserializeOwned,
    {
        let url = self.build_url(path)?;
        let request = self.client().request(Method::DELETE, url);
        self.execute_json(Method::DELETE, path, request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(dead_code)]
    struct Response {
        device_arr: Vec<Device>,
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Device {
        id: u32,
    }

    #[test]
    fn json_decode_error_includes_path_without_body() {
        let client = HttpClient::new("http://127.0.0.1").unwrap();
        let body = br#"{"deviceArr":[{"id":"abc","secret":"do-not-log"}]}"#;

        let err = client
            .deserialize_json::<Response>(&Method::GET, "v1/devices", body)
            .unwrap_err();

        match err {
            HttpClientError::JsonDecode {
                ref field_path,
                ref message,
                ..
            } => {
                assert_eq!(field_path.as_str(), "deviceArr[0].id");
                assert!(message.contains("invalid type"));
                assert!(!err.to_string().contains("do-not-log"));
            }
            other => panic!("expected json decode error, got {other:?}"),
        }
    }

    #[test]
    fn json_decode_error_rejects_trailing_data() {
        let client = HttpClient::new("http://127.0.0.1").unwrap();
        let body = br#"{"deviceArr":[{"id":1}]} false"#;

        let err = client
            .deserialize_json::<Response>(&Method::GET, "v1/devices", body)
            .unwrap_err();

        match err {
            HttpClientError::JsonDecode {
                ref field_path,
                ref message,
                ..
            } => {
                assert_eq!(field_path.as_str(), ".");
                assert!(message.contains("trailing characters"));
            }
            other => panic!("expected json decode error, got {other:?}"),
        }
    }

    #[test]
    fn build_url_keeps_base_path_prefix() {
        let client = HttpClient::new("http://127.0.0.1:8080/api/").unwrap();
        // 前导斜杠被剥掉，base 的 /api/ 前缀得以保留
        assert_eq!(
            client.build_url("/devices").unwrap().as_str(),
            "http://127.0.0.1:8080/api/devices"
        );
    }

    #[test]
    fn invalid_base_url_reports_the_offending_input() {
        let err = HttpClient::new("not-a-url").unwrap_err();
        match err {
            HttpClientError::Url { ref path, .. } => assert_eq!(path, "not-a-url"),
            other => panic!("expected url error, got {other:?}"),
        }
    }

    #[test]
    fn default_headers_include_configured_extra_headers() {
        let mut extra_headers = HeaderMap::new();
        extra_headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer test-token"),
        );
        extra_headers.insert(
            "x-platform",
            HeaderValue::from_static("rs-starter-template"),
        );

        let client = HttpClient::with_config(
            "http://127.0.0.1",
            HttpClientConfig {
                extra_headers,
                ..Default::default()
            },
        )
        .unwrap();

        let headers = client.default_headers();

        assert_eq!(
            headers.get(reqwest::header::ACCEPT).unwrap(),
            "application/json"
        );
        assert_eq!(headers.get("authorization").unwrap(), "Bearer test-token");
        assert_eq!(headers.get("x-platform").unwrap(), "rs-starter-template");
    }

    #[test]
    fn headers_with_extra_overrides_configured_headers() {
        let mut configured_headers = HeaderMap::new();
        configured_headers.insert(
            "x-platform",
            HeaderValue::from_static("rs-starter-template"),
        );

        let client = HttpClient::with_config(
            "http://127.0.0.1",
            HttpClientConfig {
                extra_headers: configured_headers,
                ..Default::default()
            },
        )
        .unwrap();

        let mut request_headers = HeaderMap::new();
        request_headers.insert("x-platform", HeaderValue::from_static("override-platform"));
        request_headers.insert("x-device-id", HeaderValue::from_static("device-001"));

        let headers = client.headers_with_extra(&request_headers);

        assert_eq!(headers.get("x-platform").unwrap(), "override-platform");
        assert_eq!(headers.get("x-device-id").unwrap(), "device-001");

        // 叠加只影响本次请求，不污染默认头
        let default_headers = client.default_headers();
        assert_eq!(
            default_headers.get("x-platform").unwrap(),
            "rs-starter-template"
        );
        assert!(default_headers.get("x-device-id").is_none());
    }
}
