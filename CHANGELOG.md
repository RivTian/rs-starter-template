# Changelog

All notable changes to the template are documented here. Generated projects have their own
changelog (git-cliff).

## [0.1.0] - 2026-09-20

### Added

- `template/`: cargo-generate template for an 8-crate Rust service skeleton
  (`bin`, `api`, `infra`, `core`, `domain`, `config`, `util`, `test-utils`).
- Generated projects ship with: layered configuration with fail-fast validation, lifecycle
  coordinator with graceful shutdown and a readiness state machine, health registry,
  in-process event bus, `tracing` + Prometheus metrics, RFC 9457 error responses,
  request-id propagation, an example `todo` domain with end-to-end tests, an architecture
  guard test, `just ci`, Dockerfile (cargo-chef + distroless + self-probe), GitHub Actions
  (ci / release / docker), cargo-deny, typos, git-cliff, ADRs.
- Placeholders: `project-description`, `license` (MIT / Apache-2.0 / both), `include_docker`,
  `repository`; fully non-interactive with `--silent -d ...`.
- `generated/acme-svc`: committed snapshot; `just regen` / `just drift` / `just matrix` for
  template maintenance; `template-ci.yml` checks docs, snapshot, drift and a three-name matrix.
- `docs/design/`: research (Pumpkin, Pingora, ratatui/templates), principles, architecture,
  template engineering, roadmap with implementation log.
