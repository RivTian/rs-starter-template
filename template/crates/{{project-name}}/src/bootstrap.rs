//! Wires adapters into ports, ports into services, services into the server.
//!
//! Two entry points: [`wire`] builds the object graph (tests call it with their own listener
//! and phase watch); [`run`] is what the binary does after the runtime exists.

use std::sync::Arc;

use axum::Router;
use tracing::info;

use {{crate_name}}_api::prelude::{AppState, HttpService};
use {{crate_name}}_config::Config;
use {{crate_name}}_core::event::EventBus;
use {{crate_name}}_core::health::HealthRegistry;
use {{crate_name}}_core::server::{PhaseWatch, Server};
use {{crate_name}}_core::service::Service;
use {{crate_name}}_core::telemetry::{self, PrometheusHandle, RuntimeMetricsService};
use {{crate_name}}_domain::event::DomainEvent;
use {{crate_name}}_domain::todo::TodoService;
use {{crate_name}}_infra::prelude::*;

use crate::{LOG_ENV_VAR, build_info};

/// Everything [`wire`] produces. The HTTP listener is deliberately **not** included: the caller
/// decides the socket (configured address in production, `127.0.0.1:0` in tests).
pub struct Wiring {
    /// Handler state.
    pub state: AppState,
    /// The complete router with middleware.
    pub router: Router,
    /// Background services to register with the server.
    pub services: Vec<Box<dyn Service>>,
    /// The bus, for tests that want to subscribe.
    pub events: EventBus<DomainEvent>,
}

/// Builds the object graph from configuration.
#[must_use]
pub fn wire(cfg: &Config, phase: PhaseWatch, metrics: Option<PrometheusHandle>) -> Wiring {
    // ── adapters (infra) ─────────────────────────────────────────────────
    let clock = Arc::new(SystemClock);
    let repository = Arc::new(InMemoryTodoRepository::new());
    let events = EventBus::<DomainEvent>::new(cfg.events.capacity);
    let publisher = Arc::new(EventBusPublisher::new(events.publisher()));

    // ── domain services (know ports only) ────────────────────────────────
    let todos = Arc::new(TodoService::new(repository.clone(), clock, publisher));

    // ── health ───────────────────────────────────────────────────────────
    let health = HealthRegistry::new().register("todo_repository", repository);

    // ── transport ────────────────────────────────────────────────────────
    let state = AppState {
        todos: todos.clone(),
        health,
        phase,
        metrics,
        build: build_info::current(),
    };
    let router = {{crate_name}}_api::router::router(state.clone(), &cfg.http);

    // ── background services ──────────────────────────────────────────────
    let mut services: Vec<Box<dyn Service>> = vec![Box::new(EventLogger::new(events.subscribe()))];
    if cfg.jobs.cleanup.enabled {
        services.push(Box::new(CleanupJob::new(cfg.jobs.cleanup.clone(), todos)));
    }

    Wiring {
        state,
        router,
        services,
        events,
    }
}

/// Runs the service until shutdown. Assumes a Tokio runtime and no telemetry yet.
pub async fn run(cfg: Config) -> anyhow::Result<()> {
    let telemetry = telemetry::init(&cfg.telemetry, LOG_ENV_VAR)?;
    let build = build_info::current();
    info!(
        version = build.version,
        commit = build.git_sha,
        built = build.build_timestamp,
        rustc = build.rustc,
        environment = ?cfg.app.environment,
        "starting {}",
        cfg.app.name
    );

    let mut server = Server::new(cfg.server.clone());
    let wiring = wire(&cfg, server.phase_watch(), telemetry.metrics.clone());

    server.add_service(HttpService::new(&cfg.http, wiring.router));
    for service in wiring.services {
        server.add_boxed(service);
    }
    if cfg.telemetry.metrics {
        server.add_service(RuntimeMetricsService::new(
            cfg.telemetry.runtime_metrics_interval,
            telemetry.metrics,
        ));
    }

    server.run().await?;
    Ok(())
}
