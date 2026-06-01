# Project State

## Last Updated
- Date: 2026-06-01 15:06 EDT
- By: Codex

## Current Repository Status

### Crates
- ck-core: implemented for Milestone 0 status DTOs only
- ck-config: implemented for basic TOML file loading, env var interpolation, and defaults
- ck-app: not implemented
- ck-storage-sqlite: implemented for schema v1 migration bootstrap and storage status
- ck-search-fts: not implemented
- ck-ranker: not implemented
- ck-memory: not implemented
- ck-artifacts: not implemented
- ck-mcp: implemented for `get_status` only
- ck-transport-stdio: implemented for one-line JSON-RPC `tools/call` dispatch
- ck-transport-sse: not implemented
- ck-auth: not implemented
- ck-observability: not implemented
- ck-scheduler: not implemented
- ck-bin: implemented for `serve`, `migrate`, `doctor`, and stubbed `export-schema`

### Implemented Features
- Cargo workspace with Milestone 0 crates.
- `context-kernal migrate --database <path>` creates and migrates a SQLite database.
- `context-kernal serve --database <path>` accepts stdio JSON-RPC `tools/call` and returns `get_status`.
- SQLite schema v1 creates core metadata tables and FTS5 virtual tables.
- Basic TOML config loading and defaults.

### Partially Implemented Features
- CLI surface: `export-schema` is present but prints a Milestone 0 not-implemented message.
- MCP surface: only `get_status` is implemented; all other planned tools are not implemented.

### Not Yet Implemented
- `store_context`
- `recall_context`
- `resolve_reference`
- `query_reference`
- `ingest_source`
- `refresh_source`
- Source adapters
- Context persistence
- Retrieval/indexing
- SSE transport
- Auth
- Scheduler
- Observability beyond tracing subscriber initialization

### Database / Migrations
- Implemented schema version 1 directly in `ck-storage-sqlite`.
- Migration creates `schema_migrations`, source/corpus/library/document/chunk/session/artifact/job tables, and `chunks_fts`, `session_events_fts`, `library_aliases_fts`.

### MCP Tools
- `get_status`: implemented.
- All compatibility aliases and retrieval/context tools: not implemented.

### Source Adapters
- Not implemented.

### Verification Summary
- `cargo fmt --check`: passed.
- `cargo test --workspace`: passed.
- Manual stdio smoke check returned `get_status` JSON with `service.name = context-kernal` and `storage.fts5_enabled = true`.
