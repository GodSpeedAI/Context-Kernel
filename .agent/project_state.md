# Project State

## Last Updated
- Date: 2026-06-02 18:00 EDT
- By: deployment-readiness session

## Current Repository Status

### Crates
- ck-core: implemented — domain entities, port traits (StoragePort, SessionMemoryPort, AuthPort, SchedulerPort, EmbeddingPort, VectorSearchPort, RankerPort with RankContext, ArtifactPort with delete_artifact_payload and upsert_artifact), enums, status types
- ck-config: implemented — TOML config loading, env var interpolation, defaults, McpConfig, SseConfig, AuthConfig, SchedulerConfig, EmbeddingsConfig, ObservabilityConfig, ReloadConfig, RetentionConfig, AuthorizationConfig
- ck-app: implemented — use cases with semantic retrieval, intent-aware ranking, source scope filtering, artifact payload GC orchestration, upsert_artifact in store_context
- ck-storage-sqlite: implemented — StoragePort + SessionMemoryPort with full CRUD for all entities, list_artifacts_older_than, upsert_artifact
- ck-search-fts: implemented — LexicalSearchPort with FTS5/BM25
- ck-ranker: implemented — LexicalRanker with intent-specific weight profiles, session affinity, de-duplication, debug mode
- ck-artifacts: implemented — FileArtifactStore implementing ArtifactPort including delete_artifact_payload
- ck-adapter-filesystem: implemented — FilesystemSourceAdapter, MarkdownNormalizer, HeadingChunker
- ck-embeddings: implemented — NoopEmbeddingProvider and OpenAIEmbeddingProvider (ureq-based sync HTTP)
- ck-vector: implemented — SqliteVectorStore with brute-force cosine similarity, corpus/library/document filtering, remove_chunk_embeddings_by_document, upsert_chunk_embedding_with_metadata
- ck-adapter-github: implemented — GitHubSourceAdapter with tree API and static file list, GitHubNormalizer, truncated tree detection
- ck-adapter-web: implemented — WebSourceAdapter with SSRF protections (DnsResolver trait, SystemDnsResolver, StaticResolver, validate_url_with_resolver), HTML-to-text, WebNormalizer, conditional request headers (If-None-Match, If-Modified-Since)
- ck-mcp: implemented — all 8 MCP tools + 3 compatibility aliases, ToolTelemetry DTO with tracing instrumentation
- ck-transport-stdio: implemented — JSON-RPC stdio framing
- ck-transport-sse: implemented — axum HTTP server with auth gating, ReloadableSseState (RwLock<Arc<...>>) for config reload
- ck-auth: implemented — BearerAuth with fail-closed validation and allowlist authz by tool/corpus
- ck-observability: implemented — ObservabilityRuntimeConfig, init_observability (text/JSON tracing), ObservabilityGuard
- ck-scheduler: implemented — RefreshScheduler with storage-backed active-job coalescing
- ck-memory: implemented — EventRetentionClass, classify_event_kind, SessionRetentionPolicy
- ck-bin: implemented — wires all crates including embeddings/vector/github/web adapters, ck-observability init, SSE reloadable state

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
  - SqliteVectorStore (brute-force cosine similarity with corpus/library/document filtering)
  - EmbeddingsConfig in ck-config (enabled, provider, api_base, api_key, model, dimensions)
  - Semantic retrieval integrated into query_reference (hybrid mode merges FTS + vector results)
  - Graceful degradation: service works fully without embeddings configured
- Milestone 9: GitHub/web adapters
  - GitHubSourceAdapter: discovers files via GitHub tree API or static file list, fetches via contents API
  - GitHubNormalizer: extracts title, preserves GitHub metadata
  - Truncated tree detection: errors on truncated: true in tree responses
  - WebSourceAdapter: discovers units from URL list, fetches via HTTP with conditional request headers
  - SSRF protections: DnsResolver trait, SystemDnsResolver, StaticResolver for tests, validate_url_with_resolver, blocks private IPs, non-HTTP schemes, enforces allowed_domains
  - HTML-to-text conversion: strips tags, removes script/style, decodes entities, preserves block breaks
  - WebNormalizer: extracts title from HTML, converts to Document entities
  - WebConfig supports previous_etag and previous_last_modified for conditional fetches
- Milestone 10: Hardening and packaging
  - Config reload section parsed; stdio serve loop reloads file-backed config when `[reload].enabled = true`
  - SSE reloadable state: ReloadableSseState with RwLock<Arc<...>> for live config updates
  - Retention GC over SQLite metadata AND artifact payload files for unpinned session events, inactive sessions, finished refresh jobs, artifact metadata rows, and disabled sources
  - Authz by tool/corpus through AuthPort and BearerAuth allowlists; SSE calls use authorized MCP dispatch
  - `export-schema` emits embedded SQLite schema SQL
  - Structured tracing logs for retention/config reload and existing runtime paths
  - Release install docs and GitHub Actions CI example
- Deployment Readiness (all 18 tasks complete):
  - `.github/copilot-instructions.md` created
  - `ck-observability` crate with text/JSON tracing init
  - `ToolTelemetry` DTO with tracing instrumentation in dispatch_tool
  - `ReloadableSseState` for SSE config reload
  - Artifact payload file GC (delete_artifact_payload, list_artifacts_older_than, run_retention_gc)
  - Source scope filtering (private corpus rejection, source_scope filter)
  - Vector filtering (corpus_id/library_id/document_id columns, search_chunks filtering, remove_chunk_embeddings_by_document)
  - SSRF DNS resolution (DnsResolver trait, SystemDnsResolver, StaticResolver, validate_url_with_resolver)
  - GitHub truncated tree detection (units_from_tree_response errors on truncated: true)
  - Web incremental refresh (previous_etag/previous_last_modified, conditional request headers)
  - `ck-memory` crate (EventRetentionClass, classify_event_kind, SessionRetentionPolicy)
  - Config templates (context-kernal.example.toml, .local.example.toml, .sse.example.toml)
  - Deployment smoke tests (4 new CLI tests: schema export, migrate+doctor, retention, config template)
  - Operations docs (operations.md, configuration.md, security.md)
  - Release workflow (.github/workflows/release.yml, docs/release.md)

### Partially Implemented Features
- Observability: structured tracing is present and `[observability]` config supports `otel_endpoint`, but no OpenTelemetry exporter/metrics backend is wired. ck-observability provides text/JSON tracing init only.

### Not Yet Implemented
- OpenTelemetry exporter/metrics backend wiring
- Context7 dataset adapter (removed from v1 scope; compatibility via tool aliases only)
- PDF source adapter (removed from v1 scope)

### Database / Migrations
- Schema version 1 in ck-storage-sqlite
- Tables: schema_migrations, sources, corpora, corpus_sources, library_identities, library_aliases, documents, chunks, artifacts, sessions, session_events, refresh_jobs
- FTS5: chunks_fts, session_events_fts, library_aliases_fts
- Vector tables (in ck-vector): chunk_embeddings (with corpus_id, library_id, document_id columns), session_event_embeddings

### MCP Tools
- `get_status`: implemented
- `store_context`: implemented (with artifact upsert)
- `recall_context`: implemented (with intent-aware ranking)
- `ingest_source`: implemented
- `query_reference`: implemented (with semantic retrieval when configured, intent-aware ranking, source scope filtering)
- `resolve_reference`: implemented (with intent-aware ranking, source scope filtering)
- `refresh_source`: implemented
- Compatibility aliases: `resolve-library-id`, `get-library-docs`, `query-docs`: implemented
- SSE authz policy: implemented for bearer-authenticated tool calls

### Source Adapters
- `ck-adapter-filesystem`: implemented (filesystem_tree)
- `ck-adapter-github`: implemented (github_repo, with truncated tree detection)
- `ck-adapter-web`: implemented (website, with SSRF DNS resolution and conditional request headers)

### Verification Summary
- `cargo test --workspace`: 186 tests pass
- `cargo fmt --check`: passes
- `cargo clippy --workspace --all-targets -- -D warnings`: zero warnings
- `cargo build --release -p ck-bin`: produces `context-kernal` binary
- Integration tests (16) verify MCP tools end-to-end via stdio, CLI schema export, migrate+doctor, retention, config templates
- Release binary supports `migrate`, `doctor`, `export-schema`, and `serve` commands
