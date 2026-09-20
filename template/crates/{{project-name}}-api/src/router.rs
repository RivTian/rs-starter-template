//! The route table and the middleware stack.

use {{crate_name}}_config::HttpConfig;
use axum::Router;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::get;
use tower::ServiceBuilder;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

use crate::state::AppState;
use crate::{middleware, routes};

/// Builds the application router. Operational endpoints sit at the root; the versioned API
/// lives under `/api/v1`.
pub fn router(state: AppState, cfg: &HttpConfig) -> Router {
    let trace = TraceLayer::new_for_http()
        .make_span_with(middleware::trace::make_span)
        .on_response(middleware::trace::on_response);

    // `ServiceBuilder` applies layers top-down: the first one is the outermost.
    let stack = ServiceBuilder::new()
        .layer(from_fn(middleware::request_id::request_id))
        .layer(trace)
        .layer(from_fn(middleware::metrics::observe))
        .layer(from_fn_with_state(
            cfg.request_timeout,
            middleware::timeout::timeout,
        ))
        .layer(RequestBodyLimitLayer::new(cfg.body_limit_bytes()))
        .layer(CatchPanicLayer::custom(middleware::panic::handle));

    Router::new()
        .route("/healthz", get(routes::health::live))
        .route("/readyz", get(routes::health::ready))
        .route("/version", get(routes::version::get))
        .route("/metrics", get(routes::metrics::get))
        .nest("/api/v1", routes::todos::router())
        .fallback(routes::not_found)
        .layer(stack)
        .with_state(state)
}
