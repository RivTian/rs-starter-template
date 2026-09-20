# syntax=docker/dockerfile:1
# Multi-stage: cargo-chef caches the dependency layer; the runtime image is distroless and
# runs as non-root. The binary carries its own HEALTHCHECK probe (no curl in distroless).

FROM rust:1.96-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --locked -p acme-svc

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/acme-svc /usr/local/bin/acme-svc
COPY config/default.toml /etc/acme-svc/default.toml
ENV ACME_SVC__HTTP__BIND=0.0.0.0:8080 \
    ACME_SVC__TELEMETRY__LOG_FORMAT=json \
    ACME_SVC__APP__ENVIRONMENT=production
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/acme-svc", "probe", "http://127.0.0.1:8080/readyz"]
ENTRYPOINT ["/usr/local/bin/acme-svc"]
CMD ["--config", "/etc/acme-svc/default.toml"]
