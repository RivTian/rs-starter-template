# acme-svc

A Rust service built from rs-starter-template

## Five minutes

```bash
cp .env.example .env
just dev                          # first log line: version / commit / build; last: "all services ready"
curl -s localhost:8080/healthz    # {"status":"ok"}
curl -s localhost:8080/readyz     # {"status":"ready","checks":{"todo_repository":{"status":"ok"}}}
curl -s -XPOST localhost:8080/api/v1/todos -H 'content-type: application/json' -d '{"title":"ship it"}'
just ci                           # fmt · clippy -D warnings · nextest · doctests · deny · typos
# Ctrl-C → "draining" → "clean shutdown"
```

Prerequisites: stable Rust (see `rust-toolchain.toml`), [`just`](https://just.systems),
`cargo-nextest`, `cargo-deny`, `typos` (`cargo install cargo-nextest cargo-deny typos-cli` or your
package manager).

## Layout

```text
crates/
  acme-svc/             bin: CLI, configuration, runtime, dependency wiring (bootstrap.rs)
  acme-svc-api/         HTTP: router, middleware, handlers, DTOs, problem+json errors
  acme-svc-infra/       adapters: in-memory repository, system clock, event bus bridge, jobs
  acme-svc-core/        runtime core: Server, Service, shutdown, readiness, health, telemetry, events
  acme-svc-domain/      pure model: entities, ports, domain errors, domain events
  acme-svc-config/      configuration schema, layered loading, validation
  acme-svc-util/        shared kernel: error classification, ids, Secret<T>
  acme-svc-test-utils/  dev-only: in-process test server, fake clock, recording publisher
config/default.toml     every key with its default and a comment
docs/architecture.md    the rules, the diagrams, how to add a feature
```

Dependencies only point downwards (`bin → api/infra → core → domain/config → util`); a test
(`crates/acme-svc/tests/layering.rs`) fails the build if that changes.

## Operating it

| Concern | How |
|---|---|
| Configuration | `config/default.toml` → `config/local.toml` → `ACME_SVC__SECTION__KEY` env → `--bind`; `--check-config` prints the effective result |
| Logs | `pretty` or `json` (`telemetry.log_format`); filter via `ACME_SVC_LOG`, then `RUST_LOG`, then `telemetry.log_filter` |
| Probes | `GET /healthz` (alive), `GET /readyz` (ready; `503` while draining or a check fails) |
| Metrics | `GET /metrics` — Prometheus text; `http_requests_total`, `http_request_duration_seconds`, tokio gauges |
| Version | `GET /version`, `acme-svc --version` |
| Shutdown | `SIGTERM`/`SIGINT`: stop accepting, finish in-flight work within `server.grace_period`, exit 0; second signal exits at once |
| Container | `just docker-build`; distroless, non-root, `HEALTHCHECK` via `acme-svc probe` |

## Adding a feature

See [docs/architecture.md](docs/architecture.md#adding-a-feature). Short version: type and
rule in `domain`, port trait in `domain`, adapter in `infra`, route in `api`, wire it in
`crates/acme-svc/src/bootstrap.rs`. `core` should not need to change.

## License

MIT — see [LICENSE](LICENSE).
