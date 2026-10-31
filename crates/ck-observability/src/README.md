# ck-observability Source

## Overview
This crate installs the process-wide observability stack for Context Kernel: it builds a `tracing-subscriber` formatter from an `ObservabilityRuntimeConfig` and registers it as the global tracing subscriber. Log output always goes to stderr, either as JSON or as the default text format, and verbosity is controlled entirely by the environment via `EnvFilter`. OpenTelemetry export is configured-for but not implemented: a configured OTEL endpoint currently only produces a warning. The crate is composed into the runtime by `ck-bin` (`ck_observability::init_observability` in `crates/ck-bin/src/main.rs`).

## Interface
- [lib.rs:ObservabilityRuntimeConfig] - Public configuration struct with public fields `log_format` (String), `otel_endpoint` (Option<String>), and `service_name` (String).
- [lib.rs:ObservabilityRuntimeConfig::otel_enabled] - Returns `true` iff `otel_endpoint` is `Some`.
- [lib.rs:ObservabilityGuard] - Public marker unit struct returned by `init_observability`; it carries no behavior or drop logic.
- [lib.rs:init_observability] - Free function that builds the formatted subscriber and installs it as the process-global tracing subscriber.

## Invariants
- Format selection: [lib.rs:init_observability] selects the JSON formatter when `log_format == "json"`; any other value falls back to the default text formatter. `service_name` is carried in the config but does not alter the subscriber's output.
- OTEL is not implemented: when `otel_endpoint` is set, [lib.rs:init_observability] only emits a `tracing::warn!` ("OTEL endpoint configured but not yet supported; ignoring") and proceeds with local logging; no exporter is built or attached.
- Filter source: verbosity comes exclusively from `EnvFilter::from_default_env()` (environment-driven, e.g. `RUST_LOG`); the config struct does not set the filter ([lib.rs:init_observability]).
- Output destination and detail: the subscriber always writes to stderr with `with_target(true)` ([lib.rs:init_observability]).
- Installation is best-effort: the `Result` of `tracing::subscriber::set_global_default` is discarded (`let _ =`), so a double-init or failed install is intentionally non-fatal for the CLI ([lib.rs:init_observability]).

## Side Effects
- Installs a process-wide global tracing subscriber via `tracing::subscriber::set_global_default`, affecting log emission for all threads in the process ([lib.rs:init_observability]).
- Reads the process environment to build the log filter ([lib.rs:init_observability]).
- Emits a `tracing::warn!` event when an OTEL endpoint is configured ([lib.rs:init_observability]).
- No filesystem, network, lock, or thread side effects of its own.

## Verification
- Run: `cargo test -p ck-observability`
