# 3. Error model: layered enums plus a shared classification

Date: 2026-09-20

## Status

Accepted

## Context

Two extremes were on the table: one rich error struct for the whole workspace (as in
`pingora-error`) or an unrelated `thiserror` enum per crate. The first loses exhaustive
matching; the second loses the ability to map any error to a status code and a log level
without knowing its type.

## Decision

Each layer keeps a precise `thiserror` enum. Every error that crosses a layer implements
`Classify` (`kind`, `source`, `retryable`), defined in `{{project-name}}-util` so that `domain` can
implement it without depending on `core`. `ErrorKind` maps to HTTP status, RFC 9457 title and
log severity in one place. `anyhow` is confined to the binary's startup path.

## Consequences

* Handlers write `?` and get correct status codes and logging for free.
* Adding an `ErrorKind` variant forces every mapping table to be updated (compile error).
* Internal errors are never shown to clients; only client-caused errors carry `detail`.
