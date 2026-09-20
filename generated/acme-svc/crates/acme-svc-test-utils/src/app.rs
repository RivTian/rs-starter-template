//! An in-process HTTP server on an ephemeral port.

use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use acme_svc_core::shutdown::ShutdownToken;

/// A running router on `127.0.0.1:<random>`, torn down on drop.
pub struct TestApp {
    /// Where it listens.
    pub addr: SocketAddr,
    /// `http://127.0.0.1:<port>`.
    pub base_url: String,
    /// A client; no TLS, no proxy.
    pub client: reqwest::Client,
    shutdown: ShutdownToken,
    task: JoinHandle<()>,
}

impl TestApp {
    /// Binds an ephemeral port and serves `router` in the background.
    pub async fn spawn(router: Router) -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let shutdown = ShutdownToken::new();
        let signal = shutdown.cancelled_owned();
        let task = tokio::spawn(async move {
            // A serve error in a test means the test fails when its request fails.
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(signal)
                .await;
        });
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .map_err(std::io::Error::other)?;
        Ok(Self {
            addr,
            base_url: format!("http://{addr}"),
            client,
            shutdown,
            task,
        })
    }

    /// Absolute URL for `path` (must start with `/`).
    #[must_use]
    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        self.shutdown.cancel();
        self.task.abort();
    }
}
