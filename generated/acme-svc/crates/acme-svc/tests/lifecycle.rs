//! The whole server: start → ready → serve → SIGTERM → drained within the grace period.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests may panic"
)]

use std::time::Duration;

use tokio::net::TcpListener;

use acme_svc::bootstrap;

use acme_svc_api::prelude::HttpService;
use acme_svc_config::Config;
use acme_svc_core::error::chain;
use acme_svc_core::server::{Phase, Server, ServerError};
use acme_svc_core::shutdown::{ChannelSignals, ShutdownSignal};

#[tokio::test]
async fn start_serve_terminate() {
    let mut cfg = Config::default();
    cfg.server.grace_period = Duration::from_secs(2);
    cfg.server.readiness_timeout = Duration::from_secs(2);

    let mut server = Server::new(cfg.server.clone());
    let mut phase = server.phase_watch();
    let wiring = bootstrap::wire(&cfg, server.phase_watch(), None);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let http = HttpService::from_listener(listener, wiring.router).unwrap();
    let addr = http.bind_addr();
    server.add_service(http);
    for service in wiring.services {
        server.add_boxed(service);
    }

    let (trigger, signals) = ChannelSignals::new();
    let run = tokio::spawn(server.run_with(signals));

    phase.wait_for(|p| *p == Phase::Ready).await.expect("ready");
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    let res = client
        .get(format!("http://{addr}/healthz"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let res = client
        .get(format!("http://{addr}/readyz"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    assert!(trigger.send(ShutdownSignal::Terminate).await);
    let outcome = tokio::time::timeout(Duration::from_secs(5), run)
        .await
        .expect("drained in time")
        .expect("join");
    outcome.expect("clean shutdown");
    assert_eq!(*phase.borrow(), Phase::Stopped);

    // the listener is gone
    assert!(
        client
            .get(format!("http://{addr}/healthz"))
            .send()
            .await
            .is_err()
    );
}

#[tokio::test]
async fn bind_conflict_is_fatal_and_reports_the_os_error() {
    let mut cfg = Config::default();
    cfg.server.grace_period = Duration::from_secs(2);
    cfg.server.readiness_timeout = Duration::from_secs(2);

    // Occupy a port, then ask the service to bind the same one.
    let taken = TcpListener::bind("127.0.0.1:0").await.unwrap();
    cfg.http.bind = taken.local_addr().unwrap();

    let mut server = Server::new(cfg.server.clone());
    let wiring = bootstrap::wire(&cfg, server.phase_watch(), None);
    server.add_service(HttpService::new(&cfg.http, wiring.router));

    let (_trigger, signals) = ChannelSignals::new();
    let err = server.run_with(signals).await.expect_err("bind must fail");
    assert!(
        matches!(err, ServerError::ServiceFailed { name: "http", .. }),
        "{err}"
    );

    // The top-level message alone hides the OS reason; the rendered chain carries it.
    let rendered = chain(&err).to_string();
    assert!(
        rendered.starts_with("service `http` failed: failed to bind "),
        "{rendered}"
    );
    assert!(rendered.contains("os error"), "{rendered}");
    assert_ne!(rendered, err.to_string());
}
