# {{project-name}}

{{project-description}}

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
  {{project-name}}/             bin: CLI, configuration, runtime, dependency wiring (bootstrap.rs)
  {{project-name}}-api/         HTTP: router, middleware, handlers, DTOs, problem+json errors
  {{project-name}}-infra/       adapters: in-memory repository, system clock, event bus bridge, jobs
  {{project-name}}-core/        runtime core: Server, Service, shutdown, readiness, health, telemetry, events
  {{project-name}}-domain/      pure model: entities, ports, domain errors, domain events
  {{project-name}}-config/      configuration schema, layered loading, validation
  {{project-name}}-util/        shared kernel: error classification, ids, Secret<T>
  {{project-name}}-test-utils/  dev-only: in-process test server, fake clock, recording publisher
config/default.toml     every key with its default and a comment
docs/architecture.md    the rules, the diagrams, how to add a feature
```

Dependencies only point downwards (`bin → api/infra → core → domain/config → util`); a test
(`crates/{{project-name}}/tests/layering.rs`) fails the build if that changes.

## Operating it

| Concern | How |
|---|---|
| Configuration | `config/default.toml` → `config/local.toml` → `{{env_prefix}}__SECTION__KEY` env → `--bind`; `--check-config` prints the effective result |
| Logs | `pretty` or `json` (`telemetry.log_format`); filter via `{{env_prefix}}_LOG`, then `RUST_LOG`, then `telemetry.log_filter` |
| Probes | `GET /healthz` (alive), `GET /readyz` (ready; `503` while draining or a check fails) |
| Metrics | `GET /metrics` — Prometheus text; `http_requests_total`, `http_request_duration_seconds`, tokio gauges |
| Version | `GET /version`, `{{project-name}} --version` |
| Shutdown | `SIGTERM`/`SIGINT`: stop accepting, finish in-flight work within `server.grace_period`, exit 0; second signal exits at once |
| Container | `just docker-build`; distroless, non-root, `HEALTHCHECK` via `{{project-name}} probe` |

## Adding a feature

See [docs/architecture.md](docs/architecture.md#adding-a-feature). Short version: type and
rule in `domain`, port trait in `domain`, adapter in `infra`, route in `api`, wire it in
`crates/{{project-name}}/src/bootstrap.rs`. `core` should not need to change.

## License

{% if license == "None" -%}
No license chosen at generation time. Add one with `license = "..."` in `Cargo.toml` and a `LICENSE` file.
{%- else -%}
{{license}} — see [LICENSE](LICENSE).
{%- endif %}
