//! The HTTP listener as a [`Service`]: bind, announce readiness, serve until shutdown.

use std::net::SocketAddr;

use {{crate_name}}_config::HttpConfig;
use {{crate_name}}_core::readiness::Readiness;
use {{crate_name}}_core::service::{Service, ServiceError};
use {{crate_name}}_core::shutdown::ShutdownToken;
use async_trait::async_trait;
use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

/// Serves a [`Router`] on a TCP listener.
pub struct HttpService {
    bind: SocketAddr,
    listener: Option<TcpListener>,
    router: Router,
}

impl HttpService {
    /// Binds `cfg.bind` when the service starts.
    #[must_use]
    pub const fn new(cfg: &HttpConfig, router: Router) -> Self {
        Self {
            bind: cfg.bind,
            listener: None,
            router,
        }
    }

    /// Serves on an already bound listener (tests bind `127.0.0.1:0` and read the port back).
    pub fn from_listener(listener: TcpListener, router: Router) -> std::io::Result<Self> {
        let bind = listener.local_addr()?;
        Ok(Self {
            bind,
            listener: Some(listener),
            router,
        })
    }

    /// The address this service will (or does) listen on.
    #[must_use]
    pub const fn bind_addr(&self) -> SocketAddr {
        self.bind
    }
}

#[async_trait]
impl Service for HttpService {
    fn name(&self) -> &'static str {
        "http"
    }

    async fn run(
        self: Box<Self>,
        shutdown: ShutdownToken,
        ready: Readiness,
    ) -> Result<(), ServiceError> {
        let listener = match self.listener {
            Some(listener) => listener,
            None => TcpListener::bind(self.bind)
                .await
                .map_err(|source| ServiceError::Bind {
                    addr: self.bind,
                    source,
                })?,
        };
        let addr = listener
            .local_addr()
            .map_err(|e| ServiceError::caused_by("listener has no address", e))?;
        info!(%addr, "http listening");
        ready.ready();

        // Stops accepting on shutdown, then lets in-flight requests finish.
        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown.cancelled_owned())
            .await
            .map_err(|e| ServiceError::caused_by("http server failed", e))
    }
}
