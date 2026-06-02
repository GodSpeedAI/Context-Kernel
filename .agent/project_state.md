# Project State

## Last Updated
- Date: 2026-06-02 10:30 EDT
- By: Kilo

## Current Repository Status

### Crates
- ck-core: implemented — domain entities, port traits (StoragePort, SessionMemoryPort, AuthPort, SchedulerPort, EmbeddingPort, VectorSearchPort, RankerPort with RankContext), enums, status types
- ck-config: implemented — TOML config loading, env var interpolation, defaults, McpConfig, SseConfig, AuthConfig, SchedulerConfig, EmbeddingsConfig
- ck-app: implemented — use cases with semantic retrieval support, intent-aware ranking, optional EmbeddingPort/VectorSearchPort
- ck-storage-sqlite: implemented — StoragePort + SessionMemoryPort with full CRUD for all entities
- ck-search-fts: implemented — LexicalSearchPort with FTS5/BM25
- ck-ranker: implemented — LexicalRanker with intent-specific weight profiles, session affinity, de-duplication, debug mode
- ck-artifacts: implemented — FileArtifactStore implementing ArtifactPort
- ck-adapter-filesystem: implemented — FilesystemSourceAdapter, MarkdownNormalizer, HeadingChunker
- ck-embeddings: implemented — NoopEmbeddingProvider and OpenAIEmbeddingProvider (ureq-based sync HTTP)
- ck-vector: implemented — SqliteVectorStore with brute-force cosine similarity
- ck-adapter-github: implemented — GitHubSourceAdapter with tree API and static file list, GitHubNormalizer
- ck-adapter-web: implemented — WebSourceAdapter with SSRF protections, HTML-to-text, WebNormalizer
- ck-mcp: implemented — all 8 MCP tools + 3 compatibility aliases
- ck-transport-stdio: implemented — JSON-RPC stdio framing
- ck-transport-sse: implemented — axum HTTP server with auth gating
- ck-auth: implemented — BearerAuth with fail-closed validation
- ck-observability: not implemented (tracing subscriber only)
- ck-scheduler: implemented — RefreshScheduler with storage-backed active-job coalescing
- ck-memory: not implemented (session memory logic in ck-app and ck-storage-sqlite)
- ck-bin: implemented — wires all crates including embeddings/vector/github/web adapters

### Implemented Features
- Milestone 0: Cargo workspace, config, migrations, stdio MCP, get_status
- Milestone 1: Storage + memory foundation
- Milestone 2: Local document ingestion
- Milestone 3: Unified identity layer
- Milestone 4: Ref-style search/read unification
- Milestone 5: Scheduler and refresh
- Milestone 6: SSE remote serving
- Milestone 7: Hybrid ranking
  - Intent-specific weight profiles (ResolveLibrary, QueryDocs, SearchRefs, RecallContext, General)
  - Session affinity scoring (boost items from current session)
  - De-duplication by document_id (collapse chunks from same document)
  - Debug mode with detailed score breakdowns
  - Semantic score integration in weighted combination
- Milestone 8: Optional embeddings
  - EmbeddingPort trait in ck-core
  - VectorSearchPort trait in ck-core
  - NoopEmbeddingProvider (graceful fallback when not configured)
  - OpenAIEmbeddingProvider (synchronous ureq HTTP to OpenAI-compatible API)
  - SqliteVectorStore (brute-force cosine similarity over stored vectors)
  - EmbeddingsConfig in ck-config (enabled, provider, api_base, api_key, model, dimensions)
  - Semantic retrieval integrated into query_reference (hybrid mode merges FTS + vector results)
  - Graceful degradation: service works fully without embeddings configured
- Milestone 9: GitHub/web adapters
  - GitHubSourceAdapter: discovers files via GitHub tree API or static file list, fetches via contents API
  - GitHubNormalizer: extracts title, preserves GitHub metadata
  - WebSourceAdapter: discovers units from URL list, fetches via HTTP
  - SSRF protections: blocks private IPs (10/8, 172.16/12, 192.168/16, 127/8, 169.254/16, 0/8), IPv6 loopback/link-local, non-HTTP schemes, enforces allowed_domains
  - HTML-to-text conversion: strips tags, removes script/style, decodes entities, preserves block breaks
  - WebNormalizer: extracts title from HTML, converts to Document entities

### Partially Implemented Features
- CLI surface: `export-schema` is present but prints a not-implemented message

### Not Yet Implemented
- Observability beyond tracing subscriber
- ck-memory as separate crate
- Source scope filtering (public/private corpus visibility)
- Config reload
- Retention GC
- PDF source adapter
- Context7 dataset adapter

### Database / Migrations
- Schema version 1 in ck-storage-sqlite
- Tables: schema_migrations, sources, corpora, corpus_sources, library_identities, library_aliases, documents, chunks, artifacts, sessions, session_events, refresh_jobs
- FTS5: chunks_fts, session_events_fts, library_aliases_fts
- Vector tables (in ck-vector): chunk_embeddings, session_event_embeddings

### MCP Tools
- `get_status`: implemented
- `store_context`: implemented
- `recall_context`: implemented (with intent-aware ranking)
- `ingest_source`: implemented
- `query_reference`: implemented (with semantic retrieval when configured, intent-aware ranking)
- `resolve_reference`: implemented (with intent-aware ranking)
- `refresh_source`: implemented
- Compatibility aliases: `resolve-library-id`, `get-library-docs`, `query-docs`: implemented

### Source Adapters
- `ck-adapter-filesystem`: implemented (filesystem_tree)
- `ck-adapter-github`: implemented (github_repo)
- `ck-adapter-web`: implemented (website)

### Verification Summary
- `cargo test --workspace`: 153 tests pass
- `cargo fmt --check`: passes
- `cargo clippy --workspace`: zero warnings
- Integration tests (11) verify all MCP tools end-to-end via stdio
