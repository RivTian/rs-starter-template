//! The whole server: start → ready → serve → SIGTERM → drained within the grace period.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests may panic"
)]

use std::time::Duration;

use tokio::net::TcpListener;

use {{crate_name}}::bootstrap;

use {{crate_name}}_api::prelude::HttpService;
use {{crate_name}}_config::Config;
use {{crate_name}}_core::server::{Phase, Server};
use {{crate_name}}_core::shutdown::{ChannelSignals, ShutdownSignal};

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
