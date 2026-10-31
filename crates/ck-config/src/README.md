# ck-config src

## Overview

Single-module crate implementing Context Kernel configuration. [lib.rs] defines the typed TOML configuration tree rooted at `KernelConfig`, canonical defaults via per-struct `Default` impls plus `default_config()`, and `load_config()`, which reads a file from disk, expands `${VAR}` environment references in the raw text, and parses TOML. All sections beyond `service`/`storage`/`retrieval` are optional through `#[serde(default)]`, so minimal configuration files parse.

## Interface

- [lib.rs:KernelConfig] - root config: required `service` ([lib.rs:ServiceConfig]), `storage` ([lib.rs:StorageConfig]), `retrieval` ([lib.rs:RetrievalConfig]); optional via `#[serde(default)]`: `mcp`, `auth`, `scheduler`, `embeddings`, `reload`, `retention`, `authorization`, `observability`.
- [lib.rs:McpConfig] - `enable_stdio` (default true), `enable_sse` (default false), `sse` ([lib.rs:SseConfig]).
- [lib.rs:SseConfig] - `bind` (default "127.0.0.1:8765"), `base_path` (default "/mcp"), `require_auth` (default false).
- [lib.rs:AuthConfig] - `trust_mode` (default "internal"), `tokens` (bearer token list, default empty).
- [lib.rs:SchedulerConfig] - `enable_periodic_refresh` (default false), `refresh_interval_secs` (default 1800).
- [lib.rs:EmbeddingsConfig] - `enabled` (default false), `provider` (default "none"), `api_base`, `api_key`, `model` (default "text-embedding-3-small"), `dimensions` (default 1536).
- [lib.rs:RetentionConfig] - `enabled` (default false) plus day thresholds: `session_event_days` (90), `inactive_session_days` (30), `refresh_job_days` (30), `artifact_days` (90), `disabled_source_days` (30).
- [lib.rs:AuthorizationConfig] - `allow_tools` (default `["*"]`), `allow_corpora` (default `["*"]`).
- [lib.rs:ObservabilityConfig] - `log_format` (default "text"), `otel_endpoint` (Option&lt;String&gt;).
- [lib.rs:ReloadConfig] - reload toggle (`enabled`, default false).
- [lib.rs:load_config(path)] - reads the file (`fs::read_to_string`), expands `${VAR}` references, then parses with `toml::from_str` into `KernelConfig`.
- [lib.rs:default_config] - canonical built-in defaults: service name "context-kernal", `database_path` "kernel.db", retrieval `default_mode` "hybrid" with `semantic_enabled` false, stdio enabled, SSE disabled.

## Invariants

- [lib.rs:KernelConfig]: every section except `service`, `storage`, and `retrieval` is annotated `#[serde(default)]`, so a TOML file containing only those three sections parses successfully.
- [lib.rs:McpConfig] `Default` impl enables stdio transport and disables SSE; `default_enable_stdio()` pins `enable_stdio` to true even when the `[mcp]` table is present without the key.
- [lib.rs:SseConfig] defaults bind SSE to loopback "127.0.0.1:8765" at base path "/mcp" with `require_auth` false.
- [lib.rs:AuthConfig] defaults to trust_mode "internal" with an empty token list; [lib.rs:AuthorizationConfig] defaults to wildcard allow-all (`["*"]`) for both tools and corpora.
- [lib.rs:EmbeddingsConfig] `Default` impl is disabled with provider "none"; [lib.rs:RetentionConfig] `Default` impl is disabled with thresholds 90/30/30/90/30 days.
- [lib.rs:default_config] pins the canonical defaults (name "context-kernal", "kernel.db", "hybrid", semantic disabled), distinct from the runtime-reported service name "context-kernel" used in `ck-core` status types.

## Side Effects

- [lib.rs:load_config] performs filesystem I/O: `fs::read_to_string` on the configured path.
- [lib.rs:expand_env_vars] reads process environment variables (`std::env::var`) for every `${VAR}` occurrence in the raw config text; unset variables expand to the empty string (`unwrap_or_default()`), and a `${` without a closing `}` is preserved as literal text.

## Verification

- Run: `cargo test -p ck-config`
