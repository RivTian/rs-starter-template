# Changelog

All notable changes to the template are documented here. Generated projects have their own
changelog (git-cliff).

## [Unreleased]

### Fixed

- Generated projects: the `domain event` log line emitted the `event` key twice (the `?event`
  shorthand reuses the variable name), which strict JSON parsers reject. The payload is now
  logged as `payload`.
- Generated projects: errors are logged with their full `source()` chain via the new
  `util::error::chain` (re-exported as `core::error::chain`), so a bind failure reads
  `failed to bind 127.0.0.1:8080: Address already in use (os error 48)` in the log line instead
  of only on stderr at exit. `ApiError::from_classified` now requires `std::error::Error`; the
  client-visible `detail` is still the top-level message only.

### Added

- Repository: issue forms (bug / feature) and a pull-request template that encode the
  contribution flow (issue → branch → PR closing the issue → template-ci → merge).
- Generated projects: a `bind_conflict_is_fatal_and_reports_the_os_error` integration test and a
  `## Logs` section in `CONTRIBUTING.md` (one field name per event; log errors with `chain`).

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
