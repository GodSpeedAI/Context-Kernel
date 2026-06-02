# Project State

## Last Updated
- Date: 2026-06-01
- By: Kilo (implementation agent)

## Current Repository Status

### Crates
- ck-core: implemented — domain entities, port traits, enums, status types
- ck-config: implemented — TOML config loading, env var interpolation, defaults
- ck-app: implemented — use cases (StoreContext, RecallContext, IngestSource, QueryReference, ResolveReference, GetStatus)
- ck-storage-sqlite: implemented — StoragePort + SessionMemoryPort with full CRUD for all entities
- ck-search-fts: implemented — LexicalSearchPort with FTS5/BM25 for chunks, session events, library aliases
- ck-ranker: implemented — LexicalRanker implementing RankerPort with score breakdowns
- ck-artifacts: implemented — FileArtifactStore implementing ArtifactPort with filesystem storage
- ck-adapter-filesystem: implemented — FilesystemSourceAdapter, MarkdownNormalizer, HeadingChunker
- ck-mcp: implemented — all 7 MCP tools + 3 compatibility aliases
- ck-transport-stdio: implemented — JSON-RPC stdio framing for all tools
- ck-transport-sse: not implemented
- ck-auth: not implemented
- ck-observability: not implemented (tracing subscriber only)
- ck-scheduler: not implemented
- ck-memory: not implemented (session memory logic in ck-app and ck-storage-sqlite)
- ck-bin: implemented — wires all crates, serves stdio MCP

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

### Partially Implemented Features
- CLI surface: `export-schema` is present but prints a not-implemented message

### Not Yet Implemented
- SSE transport
- Auth
- Scheduler (periodic refresh)
- Observability beyond tracing subscriber
- Semantic/embedding retrieval
- GitHub/web/PDF source adapters
- Hybrid ranking with freshness/trust/session-affinity priors
- Config reload
- Retention GC

### Database / Migrations
- Schema version 1 in ck-storage-sqlite
- Tables: schema_migrations, sources, corpora, corpus_sources, library_identities, library_aliases, documents, chunks, artifacts, sessions, session_events, refresh_jobs
- FTS5: chunks_fts, session_events_fts, library_aliases_fts

### MCP Tools
- `get_status`: implemented
- `store_context`: implemented
- `recall_context`: implemented
- `ingest_source`: implemented
- `query_reference`: implemented
- `resolve_reference`: implemented
- `refresh_source`: not implemented (ingest_source handles one-time indexing)
- Compatibility aliases: `resolve-library-id`, `get-library-docs`, `query-docs`: implemented

### Source Adapters
- `ck-adapter-filesystem`: implemented (filesystem_tree)

### Verification Summary
- `cargo test --workspace`: 37 tests pass
- `cargo fmt --check`: passes
- `cargo clippy --workspace`: zero warnings
- Integration tests verify all MCP tools end-to-end via stdio
