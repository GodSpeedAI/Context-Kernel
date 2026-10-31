# ck-observability

## Overview
`ck-observability` (package `ck-observability`) owns process-wide logging setup for Context Kernel. Its `init_observability` entry point installs a global `tracing-subscriber` that writes JSON or text logs to stderr with environment-driven filtering, and it reports configured-but-unsupported OpenTelemetry endpoints with a warning rather than exporting. The crate is composed into the runtime by `ck-bin` and has no direct source files outside its `src` module.

## Subdirectories & Boundaries
- `src`: [./src/README.md](./src/README.md) - Component contract for `ObservabilityRuntimeConfig`, `ObservabilityGuard`, and `init_observability`: format selection rules, OTEL non-implementation, env-based filtering, stderr output, and best-effort global subscriber installation.

## Verification
- Run: `cargo test -p ck-observability`
