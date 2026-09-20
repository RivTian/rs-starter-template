# Contributing

## Before you push

```bash
just ci
```

That is exactly what CI runs: `cargo fmt --check`, `cargo clippy --all-targets --all-features -D warnings`,
`cargo nextest run`, doctests, `cargo deny check`, `typos`.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/): `feat(api): …`, `fix(core): …`,
`docs: …`, `chore: …`. The changelog is generated from them with `git cliff`.

## Lints are the coding standard

The `[workspace.lints]` block in `Cargo.toml` is the rulebook. `clippy::all`, `pedantic`,
`nursery` and `cargo` are **denied**, as are `unwrap`, `expect`, `panic`, `todo`, `dbg!` and
printing to stdout/stderr in library code.

* Prefer fixing the code over silencing the lint.
* If a lint is wrong for a specific line, use `#[expect(clippy::name, reason = "…")]` — never a bare
  `#[allow]`. `expect` fails once the lint no longer fires, so silencers cannot rot.
* Tests may `unwrap`/`expect`/`panic`: every `lib.rs` has a `cfg_attr(test, allow(...))` and every
  `tests/*.rs` starts with `#![allow(..., reason = "tests may panic")]`.

## Where does code go?

| I am adding… | It goes in | Because |
|---|---|---|
| a type with rules (invariants) | `acme-svc-domain` | rules must not depend on I/O |
| something the domain needs from outside (storage, time, messaging) | a trait in `acme-svc-domain::ports` | the domain owns its ports |
| an implementation of such a trait | `acme-svc-infra` | adapters may depend on drivers |
| an HTTP route, DTO or middleware | `acme-svc-api` | transport only |
| a background job | `acme-svc-infra::jobs`, implementing `core::Service` | jobs are services |
| a configuration key | `acme-svc-config::sections` + `config/default.toml` | one source of truth |
| a lifecycle / observability facility | `acme-svc-core` | must stay business-free |
| a pure helper needed by ≥ 2 crates | `acme-svc-util` | otherwise keep it local |

Rule of thumb: if adding a dependency would make an arrow in `docs/architecture.md` point
upwards, the code belongs one layer higher.

## Concurrency conventions

| Situation | Use | Not |
|---|---|---|
| short critical section, no `.await` inside | `std::sync::Mutex` / `RwLock` | `tokio::sync::Mutex` |
| must hold a lock across `.await` | `tokio::sync::Mutex` | a std lock |
| read-mostly, replaced wholesale (routing tables, config snapshots) | `ArcSwap` | `RwLock` |
| concurrent map | `DashMap` | `Mutex<HashMap>` |
| CPU-heavy work | `tokio::task::spawn_blocking` (or rayon + `mpsc` back) | computing on a worker thread |
| anything that outlives a request | a `core::Service` registered in `bootstrap.rs` | bare `tokio::spawn` |
| a poisoned lock | decide explicitly (`PoisonError::into_inner` with a comment) | silent unwrap chains |

## Errors

Each layer has its own `thiserror` enum. Anything that crosses a layer implements
`Classify` (`kind`, `source`, `retryable`). The API layer turns a classified error into
`application/problem+json`; internal errors are logged in full and never leak details.
`anyhow` is only allowed in the binary's startup path.
