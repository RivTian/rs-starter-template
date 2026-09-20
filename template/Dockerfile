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
RUN cargo build --release --locked -p {{project-name}}

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/{{project-name}} /usr/local/bin/{{project-name}}
COPY config/default.toml /etc/{{project-name}}/default.toml
ENV {{env_prefix}}__HTTP__BIND=0.0.0.0:8080 \
    {{env_prefix}}__TELEMETRY__LOG_FORMAT=json \
    {{env_prefix}}__APP__ENVIRONMENT=production
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/{{project-name}}", "probe", "http://127.0.0.1:8080/readyz"]
ENTRYPOINT ["/usr/local/bin/{{project-name}}"]
CMD ["--config", "/etc/{{project-name}}/default.toml"]
