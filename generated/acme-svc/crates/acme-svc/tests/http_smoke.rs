//! End-to-end over real TCP: operational endpoints, the todo API, error shapes, correlation ids.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "tests may panic"
)]

use std::sync::OnceLock;

use acme_svc::bootstrap;
use acme_svc_config::Config;
use acme_svc_core::server::Phase;
use acme_svc_core::telemetry::PrometheusHandle;
use acme_svc_test_utils::TestApp;
use serde_json::{Value, json};
use tokio::sync::watch;

/// One global recorder per test binary; `install_recorder` may only run once per process.
fn metrics() -> PrometheusHandle {
    static HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();
    HANDLE
        .get_or_init(|| {
            metrics_exporter_prometheus::PrometheusBuilder::new()
                .install_recorder()
                .expect("install recorder")
        })
        .clone()
}

async fn app() -> TestApp {
    let cfg = Config::default();
    let (phase_tx, phase_rx) = watch::channel(Phase::Ready);
    let wiring = bootstrap::wire(&cfg, phase_rx, Some(metrics()));
    // Keep the sender alive for the whole test binary; dropping it would not change the
    // observed phase, but leaking is the simplest way to make that explicit.
    std::mem::forget(phase_tx);
    TestApp::spawn(wiring.router).await.expect("spawn")
}

#[tokio::test]
async fn operational_endpoints() {
    let app = app().await;

    let res = app.client.get(app.url("/healthz")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(res.json::<Value>().await.unwrap()["status"], "ok");

    let res = app.client.get(app.url("/readyz")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["status"], "ready");
    assert_eq!(body["checks"]["todo_repository"]["status"], "ok");

    let res = app.client.get(app.url("/version")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["name"], "acme-svc");
    assert!(body["version"].as_str().is_some_and(|v| !v.is_empty()));

    let res = app.client.get(app.url("/metrics")).send().await.unwrap();
    assert_eq!(res.status(), 200);
    let text = res.text().await.unwrap();
    assert!(text.contains("http_requests_total"), "{text}");
}

#[tokio::test]
async fn readyz_is_503_while_not_accepting_work() {
    let cfg = Config::default();
    let (_tx, rx) = watch::channel(Phase::Draining);
    let wiring = bootstrap::wire(&cfg, rx, None);
    let app = TestApp::spawn(wiring.router).await.expect("spawn");
    let res = app.client.get(app.url("/readyz")).send().await.unwrap();
    assert_eq!(res.status(), 503);
    assert_eq!(res.json::<Value>().await.unwrap()["phase"], "draining");
}

#[tokio::test]
async fn todo_lifecycle_and_error_shapes() {
    let app = app().await;
    let todos = app.url("/api/v1/todos");

    let res = app
        .client
        .post(&todos)
        .json(&json!({ "title": "  ship it  " }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let created: Value = res.json().await.unwrap();
    assert_eq!(created["title"], "ship it");
    assert_eq!(created["done"], false);
    let id = created["id"].as_str().unwrap().to_owned();

    let res = app.client.get(&todos).send().await.unwrap();
    assert_eq!(res.json::<Vec<Value>>().await.unwrap().len(), 1);

    let res = app
        .client
        .post(app.url(&format!("/api/v1/todos/{id}/complete")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let done: Value = res.json().await.unwrap();
    assert_eq!(done["done"], true);
    assert!(done["completed_at"].is_string());

    // second completion → 409 problem+json
    let res = app
        .client
        .post(app.url(&format!("/api/v1/todos/{id}/complete")))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    assert_eq!(res.headers()["content-type"], "application/problem+json");
    let problem: Value = res.json().await.unwrap();
    assert_eq!(problem["kind"], "conflict");
    assert_eq!(problem["status"], 409);
    assert!(problem["detail"].as_str().unwrap().contains("already done"));

    // unknown id → 404; malformed id → 400
    let res = app
        .client
        .get(app.url("/api/v1/todos/018f0000-0000-7000-8000-000000000000"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
    let res = app
        .client
        .get(app.url("/api/v1/todos/not-a-uuid"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    // rule violation and malformed json → 400 with detail
    let res = app
        .client
        .post(&todos)
        .json(&json!({ "title": "   " }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert_eq!(
        res.json::<Value>().await.unwrap()["detail"],
        "title must not be empty"
    );
    let res = app
        .client
        .post(&todos)
        .header("content-type", "application/json")
        .body("{not json")
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
    assert_eq!(res.json::<Value>().await.unwrap()["kind"], "invalid_input");

    // unknown route → 404 problem
    let res = app.client.get(app.url("/nope")).send().await.unwrap();
    assert_eq!(res.status(), 404);
    assert_eq!(res.json::<Value>().await.unwrap()["kind"], "not_found");
}

#[tokio::test]
async fn request_id_is_echoed_or_generated() {
    let app = app().await;
    let res = app
        .client
        .get(app.url("/healthz"))
        .header("x-request-id", "abc-123")
        .send()
        .await
        .unwrap();
    assert_eq!(res.headers()["x-request-id"], "abc-123");

    let res = app.client.get(app.url("/healthz")).send().await.unwrap();
    let generated = res.headers()["x-request-id"].to_str().unwrap();
    assert_eq!(generated.len(), 36, "uuid expected, got {generated}");
}

#[tokio::test]
async fn oversized_bodies_are_rejected() {
    let app = app().await;
    let huge = "x".repeat(2 * 1024 * 1024);
    let res = app
        .client
        .post(app.url("/api/v1/todos"))
        .json(&json!({ "title": huge }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 413);
}
