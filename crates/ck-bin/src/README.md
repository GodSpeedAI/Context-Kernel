# ck-bin/src

## Overview

This directory contains the single production source file of the `ck-bin` crate, `main.rs`, which is the entry point and composition root for the `context-kernal` binary target declared in `crates/ck-bin/Cargo.toml`. It defines the clap CLI surface (`Serve`, `Migrate`, `Doctor`, `ExportSchema`, `RebuildIndex`) and wires concrete adapters from the workspace crates into `ck_app::AppServices` and `ck_mcp::McpService`. Registered source adapters are the default `FilesystemSourceAdapter` plus the kind-selected `LearningProposalsAdapter` for the `learning_proposals_jsonl` corpus kind; no GitHub/web adapters or `ck-scheduler` are wired in this crate. In `serve`, the process always serves MCP over stdio (stdin line loop) and additionally binds an SSE listener when `--sse` is passed or `config.mcp.enable_sse` is set.

## Interface

`ck-bin` is a binary crate, so these items are crate-private; the effective public boundary is the CLI surface plus the composition functions reachable from `main`:

- [main.rs:Cli] - clap `Parser` root (`#[command(name = "context-kernel")]`) with a required subcommand.
- [main.rs:Commands] - clap `Subcommand` enum: `Serve(RuntimeArgs)`, `Migrate(RuntimeArgs)`, `Doctor(RuntimeArgs)`, `ExportSchema`, `RebuildIndex(RuntimeArgs)` (CLI names in kebab-case per clap defaults).
- [main.rs:RuntimeArgs] - shared runtime flags: `--config`, `--database`, `--data-dir`, `--sse` (all optional).
- [main.rs:main] - entry point; parses `Cli` and dispatches to the subcommand handlers, returning `anyhow::Result<()>`.
- [main.rs:serve] - long-running serve subcommand: store open/migrate, adapter wiring, retention GC, auth setup, SSE bind/spawn, periodic refresh thread, and the blocking stdio request loop with config reload.
- [main.rs:migrate] - opens the store, runs `store.migrate()`, prints the migrated database path.
- [main.rs:doctor] - opens the store, migrates, prints storage kind, path, and `fts5_enabled` from `store.status()`.
- [main.rs:rebuild_index] - opens the store, migrates, rebuilds the FTS index via `store.rebuild_fts_index()`, prints chunk/event/alias counts.
- [main.rs:open_store] - loads `ck_config::KernelConfig` from `--config` (or `default_config()`), applies the `--database` override, and opens `SqliteStore::open`.
- [main.rs:wire_app] - composition root: builds `AppServices` with `SqliteStore` (storage and session ports), `FileArtifactStore`, `FtsSearch::open`, `LexicalRanker`, `FilesystemSourceAdapter`, `MarkdownNormalizer`, `HeadingChunker`, registers `LearningProposalsAdapter` under kind `learning_proposals_jsonl` via `with_source_adapter_kind`, and conditionally attaches semantic retrieval (`OpenAIEmbeddingProvider` or `NoopEmbeddingProvider` fallback plus `SqliteVectorStore`).
- [main.rs:init_logging] - installs the process-wide tracing subscriber via `ck_observability::init_observability` from config (log format, OTel endpoint, service name).
- [main.rs:build_context_agent] - env-driven factory for `ck_mcp::AclContextAgent` used to attach the ACL `context_required` tool to `McpService`.

## Invariants

- SSE bind failure fails startup: [main.rs:serve] binds `std::net::TcpListener` on the main thread so a bind failure aborts startup via `?`, per the in-code comment that this avoids the bind error being swallowed inside the spawned thread (which previously left a "started" log with no live endpoint).
- Listener must be non-blocking before tokio registration: [main.rs:serve] calls `set_nonblocking(true)` on the std listener before `tokio::net::TcpListener::from_std` inside the spawned thread.
- SSE auth tokens are mandatory when required: [main.rs:serve] bails with "SSE auth is required but no auth tokens are configured" when `config.mcp.sse.require_auth` is set and `config.auth.tokens` is empty.
- Config reload preserves auth policy and the ACL context agent: [main.rs:serve] rebuilds `McpService` with `with_app_and_auth` and re-attaches the context agent on reload; the in-code comment notes that the previous `with_app` path reset internal auth to `None`, so SSE tool/corpus allowlist checks silently passed after the first reload.
- Kind-selected ingestion goes through the real pipeline: [main.rs:wire_app] routes the learning-proposals ledger corpus through discovery + fetch + normalization + chunking + FTS + ranking, not an ad hoc reader, per the in-code comment.
- Context-effect calibration is never faked: [main.rs:wire_app] enables the `ContextEffectStore` only when `CK_CONTEXT_EFFECT_LEDGER` names an append-only JSONL ledger; unset means calibration is disabled.
- The ACL context agent is optional and self-reporting: [main.rs:build_context_agent] returns `None` when neither `CK_CONTEXT_CORPUS_ROOT` nor `CK_LEARNING_PROPOSALS_LEDGER` is set; the `context_required` tool then reports itself as unconfigured rather than disabling the whole server, per the in-code comments.
- Unknown embedding providers degrade explicitly: [main.rs:wire_app] falls back to `NoopEmbeddingProvider` and logs a warning when `config.embeddings.provider` is neither `openai` nor `none`.
- Logging is initialized once per subcommand after config load: [main.rs:init_logging] doc comment states this ordering is required so spawned threads (SSE, refresh) also emit logs.

## Side Effects

- Process startup and stdout output: [main.rs:main] parses CLI args and dispatches; `ExportSchema` prints `SqliteStore::schema_sql()`, and the migrate/doctor/rebuild_index handlers print status lines to stdout.
- SQLite open and migrate: [main.rs:open_store] opens `SqliteStore` at the configured or `--database` path; [main.rs:serve], [main.rs:migrate], [main.rs:doctor], and [main.rs:rebuild_index] all run `store.migrate()`; [main.rs:serve] re-opens and migrates the store again on config reload.
- FTS index rewrite: [main.rs:rebuild_index] calls `store.rebuild_fts_index()`.
- Semantic vector store open: [main.rs:wire_app] opens `SqliteVectorStore` on the same database file when embeddings are enabled.
- Filesystem artifact storage: [main.rs:wire_app] roots `FileArtifactStore` at `--data-dir`, defaulting to the parent directory of the database path or `.` ([main.rs:serve]).
- Network listener: [main.rs:serve] binds the SSE TCP listener at `--sse` or `config.mcp.sse.bind` and serves it via `ck_transport_sse::serve_sse` on a spawned thread running its own tokio runtime; `ReloadableSseState` allows live service/auth replacement on reload.
- Stdio transport: [main.rs:serve] always reads stdin line-by-line and prints `ck_transport_stdio::handle_line` responses to stdout; the loop ends on stdin EOF.
- Thread spawns: [main.rs:serve] spawns the SSE runtime thread and, when `config.scheduler.enable_periodic_refresh` is set, a periodic refresh thread that polls `list_stale_sources()` every `refresh_interval_secs` and calls `refresh_source` per stale source until the `running` flag is cleared after the stdio loop exits.
- Startup retention GC: [main.rs:serve] runs `app.run_retention_gc` with a `RetentionPolicy` built from config when `config.retention.enabled`, deleting expired rows and logging the report.
- Tracing/observability init: [main.rs:init_logging] installs the global subscriber via `ck_observability::init_observability` (log format, OTel endpoint, service name).
- Environment variable reads: [main.rs:wire_app] reads `CK_CONTEXT_EFFECT_LEDGER`; [main.rs:build_context_agent] reads `CK_CONTEXT_CORPUS_ROOT` and `CK_LEARNING_PROPOSALS_LEDGER`.

## Verification

- Run: `cargo test -p ck-bin`
