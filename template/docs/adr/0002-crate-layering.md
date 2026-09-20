# 2. Crate layering and dependency direction

Date: 2026-09-20

## Status

Accepted

## Context

A single crate absorbs everything and makes boundaries invisible; too many crates make small
changes ceremonial. The dependency graph is the architecture.

## Decision

Eight crates in five layers (`util` → `domain`/`config` → `core` → `api`/`infra` → `bin`, plus
dev-only `test-utils`). Dependencies point downwards only; `api` and `infra` never depend on
each other; `core` never depends on `domain`. The allowed edges are listed in
`crates/{{project-name}}/tests/layering.rs` and checked on every test run.

## Consequences

* Adding a feature touches `domain`, `infra`, `api` and `bootstrap.rs`, never `core`.
* A new dependency direction requires editing the allow-list — a visible, reviewable act.
* `domain` stays free of Tokio and frameworks and can be unit-tested with plain fakes.
