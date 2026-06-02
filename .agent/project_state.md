# Project State

## Last Updated
- Date: 2026-06-02 08:06 EDT
- By: Codex

## Current Repository Status

### Crates
- ck-core: implemented — domain entities, port traits (StoragePort, SessionMemoryPort, AuthPort, SchedulerPort), enums (SourceHealth, JobStatus, etc.), status types
- ck-config: implemented — TOML config loading, env var interpolation, defaults, McpConfig, SseConfig, AuthConfig, SchedulerConfig; default MCP stdio is enabled consistently
- ck-app: implemented — use cases (StoreContext, RecallContext, IngestSource, QueryReference, ResolveReference, GetStatus, RefreshSource); refresh reuses persisted corpus links and ingest marks source degraded on failures
- ck-storage-sqlite: implemented — StoragePort + SessionMemoryPort with full CRUD for all entities, source health, stale detection, corpus/source links, atomic active refresh job insert and claim
- ck-search-fts: implemented — LexicalSearchPort with FTS5/BM25 for chunks, session events, library aliases
- ck-ranker: implemented — LexicalRanker implementing RankerPort with hybrid scoring (normalized FTS + trust + freshness)
- ck-artifacts: implemented — FileArtifactStore implementing ArtifactPort with filesystem storage
- ck-adapter-filesystem: implemented — FilesystemSourceAdapter, MarkdownNormalizer, HeadingChunker
- ck-mcp: implemented — all 8 MCP tools + 3 compatibility aliases
- ck-transport-stdio: implemented — JSON-RPC stdio framing for all tools
- ck-transport-sse: implemented — axum HTTP server with POST /mcp, GET /healthz, GET /readyz, GET /api/v1/status, auth gating
- ck-auth: implemented — BearerAuth implementing AuthPort with fail-closed token validation, constant-time comparison, and explicit no-auth mode
- ck-observability: not implemented (tracing subscriber only)
- ck-scheduler: implemented — RefreshScheduler implementing SchedulerPort with storage-backed active-job coalescing, atomic claim, job lifecycle, stale detection
- ck-memory: not implemented (session memory logic in ck-app and ck-storage-sqlite)
- ck-bin: implemented — wires all crates, serves stdio + SSE MCP, periodic refresh background thread

### Implemented Features
- Milestone 0: Cargo workspace, config, migrations, stdio MCP, get_status
- Milestone 1: Storage + memory foundation
  - SQLite schema v1 with all tables and FTS5 virtual tables
  - Session/event/artifact persistence via StoragePort and SessionMemoryPort
  - Filesystem artifact storage via ArtifactPort
  - `store_context` MCP tool — persists events, handles large payloads as artifacts
  - `recall_context` MCP tool — BM25-ranked session event recall via FTS5
- Milestone 2: Local document ingestion
  - Filesystem source adapter — discovers .md/.txt/.toml/.rs/.json files
  - Markdown normalizer — converts raw content to Document entities
  - Heading-aware chunker — splits by markdown headings, sub-splits large sections
  - `ingest_source` MCP tool — discovers, normalizes, chunks, indexes local docs
  - `query_reference` MCP tool — lexical search over indexed chunks
- Milestone 3: Unified identity layer
  - Library identity and alias persistence
  - `resolve_reference` MCP tool — FTS search over library aliases
  - Compatibility aliases: `resolve-library-id`, `get-library-docs`, `query-docs`
- Milestone 4: Ref-style search/read unification
  - Enhanced ranker with trust + freshness hybrid scoring (normalized FTS 0.7, trust 0.2, freshness 0.1)
  - Citations and excerpts in query_reference results
  - SourceHealth enum and storage methods
  - Expanded StoragePort with 5 new methods
- Milestone 5: Scheduler and refresh
  - `ck-scheduler` crate with RefreshScheduler (storage-backed active-job coalescing, atomic claim, job lifecycle, stale detection)
  - `refresh_source` MCP tool — re-ingests existing sources into persisted corpus links
  - Periodic refresh background thread in ck-bin
  - Stale source detection via SQL datetime arithmetic
- Milestone 6: SSE remote serving
  - `ck-transport-sse` crate with axum HTTP server
  - POST /mcp (JSON-RPC), GET /healthz, GET /readyz, GET /api/v1/status
  - `ck-auth` crate with BearerAuth (fail-closed token validation, constant-time comparison, explicit no-auth mode)
  - Dual transport: stdio + SSE coexist, SSE runs in background thread with tokio runtime
  - Auth gating on SSE endpoints

### Partially Implemented Features
- CLI surface: `export-schema` is present but prints a not-implemented message

### Not Yet Implemented
- Observability beyond tracing subscriber
- Semantic/embedding retrieval
- GitHub/web/PDF source adapters
- Source scope filtering (public/private corpus visibility)
- Config reload
- Retention GC

### Database / Migrations
- Schema version 1 in ck-storage-sqlite
- Tables: schema_migrations, sources, corpora, corpus_sources, library_identities, library_aliases, documents, chunks, artifacts, sessions, session_events, refresh_jobs
- FTS5: chunks_fts, session_events_fts, library_aliases_fts
- Refresh job indexes include partial unique index `idx_refresh_jobs_one_active_per_source` for one queued/running job per source.

### MCP Tools
- `get_status`: implemented
- `store_context`: implemented
- `recall_context`: implemented
- `ingest_source`: implemented
- `query_reference`: implemented (with citations, excerpts, trust/freshness enrichment)
- `resolve_reference`: implemented
- `refresh_source`: implemented
- Compatibility aliases: `resolve-library-id`, `get-library-docs`, `query-docs`: implemented

### Source Adapters
- `ck-adapter-filesystem`: implemented (filesystem_tree)

### Verification Summary
- `cargo test --workspace`: 65 tests pass
- `cargo fmt --check`: passes
- `cargo clippy --workspace`: zero warnings
- Integration tests (11) verify all MCP tools end-to-end via stdio, including refresh_source and SSE mode
