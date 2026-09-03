# Source Map & Traceability Index

This document maps all high-level concepts, architectural ports, adapters, CLI commands, configuration keys, database tables, and MCP tools directly to their concrete implementation locations in the repository.

---

## 1. Domain Entities & Port Traits

| Domain Concept / Port | Code Symbol | File Path | Line References | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Source Entity** | `struct Source` | [`crates/ck-core/src/entities/source.rs`](file:///crates/ck-core/src/entities/source.rs) | L8–L25 | External documentation provider configuration and refresh state |
| **Corpus Entity** | `struct Corpus` | [`crates/ck-core/src/entities/corpus.rs`](file:///crates/ck-core/src/entities/corpus.rs) | L6–L15 | Logical retrieval grouping and access-control partition |
| **Document Entity** | `struct Document` | [`crates/ck-core/src/entities/document.rs`](file:///crates/ck-core/src/entities/document.rs) | L8–L26 | Ingested file or web page content record |
| **Chunk Entity** | `struct Chunk` | [`crates/ck-core/src/entities/chunk.rs`](file:///crates/ck-core/src/entities/chunk.rs) | L8–L23 | Bounded text excerpt (~800 words) with heading and anchors |
| **Library Identity** | `struct LibraryIdentity` | [`crates/ck-core/src/entities/library.rs`](file:///crates/ck-core/src/entities/library.rs) | L6–L20 | Canonical package name, version, and trust score |
| **Library Alias** | `struct LibraryAlias` | [`crates/ck-core/src/entities/library.rs`](file:///crates/ck-core/src/entities/library.rs) | L22–L28 | Alternative search term mapped to a library identity |
| **Session Entity** | `struct Session` | [`crates/ck-core/src/entities/session.rs`](file:///crates/ck-core/src/entities/session.rs) | L6–L16 | Conversational memory container |
| **Session Event** | `struct SessionEvent` | [`crates/ck-core/src/entities/session.rs`](file:///crates/ck-core/src/entities/session.rs) | L18–L29 | Timestamped semantic memory item |
| **Artifact Entity** | `struct Artifact` | [`crates/ck-core/src/entities/artifact.rs`](file:///crates/ck-core/src/entities/artifact.rs) | L6–L18 | Filesystem payload metadata record |
| **Refresh Job** | `struct RefreshJob` | [`crates/ck-core/src/entities/job.rs`](file:///crates/ck-core/src/entities/job.rs) | L6–L20 | Asynchronous source re-indexing task |
| **Storage Port** | `trait StoragePort` | [`crates/ck-core/src/ports/storage.rs`](file:///crates/ck-core/src/ports/storage.rs) | L7–L88 | CRUD interface for entities, corpora, and retention sweeps |
| **Session Memory Port**| `trait SessionMemoryPort`| [`crates/ck-core/src/ports/session_memory.rs`](file:///crates/ck-core/src/ports/session_memory.rs) | L6–L38 | Session and event persistence interface |
| **Lexical Search Port**| `trait LexicalSearchPort`| [`crates/ck-core/src/ports/lexical_search.rs`](file:///crates/ck-core/src/ports/lexical_search.rs) | L6–L45 | FTS5 index update and query interface |
| **Ranker Port** | `trait RankerPort` | [`crates/ck-core/src/ports/ranker.rs`](file:///crates/ck-core/src/ports/ranker.rs) | L14–L41 | Hybrid scoring and result de-duplication interface |
| **Artifact Port** | `trait ArtifactPort` | [`crates/ck-core/src/ports/artifact.rs`](file:///crates/ck-core/src/ports/artifact.rs) | L5–L17 | Content-addressed filesystem artifact storage interface |
| **Source Adapter** | `trait SourceAdapter` | [`crates/ck-core/src/ports/source_adapter.rs`](file:///crates/ck-core/src/ports/source_adapter.rs) | L6–L13 | Ingestion discovery and content fetching interface |
| **Normalizer Port** | `trait Normalizer` | [`crates/ck-core/src/ports/normalizer.rs`](file:///crates/ck-core/src/ports/normalizer.rs) | L5–L8 | Text cleaning and document normalization interface |
| **Chunker Port** | `trait Chunker` | [`crates/ck-core/src/ports/chunker.rs`](file:///crates/ck-core/src/ports/chunker.rs) | L6–L8 | Document structural heading / token splitting interface |
| **Embedding Port** | `trait EmbeddingPort` | [`crates/ck-core/src/ports/embedding.rs`](file:///crates/ck-core/src/ports/embedding.rs) | L5–L20 | Vector embedding generation interface |
| **Vector Search Port** | `trait VectorSearchPort` | [`crates/ck-core/src/ports/vector_search.rs`](file:///crates/ck-core/src/ports/vector_search.rs) | L8–L44 | SQLite brute-force cosine vector search interface |
| **Auth Port** | `trait AuthPort` | [`crates/ck-core/src/ports/auth.rs`](file:///crates/ck-core/src/ports/auth.rs) | L3–L13 | Bearer token and allowlist validation interface |
| **Scheduler Port** | `trait SchedulerPort` | [`crates/ck-core/src/ports/scheduler.rs`](file:///crates/ck-core/src/ports/scheduler.rs) | L6–L27 | Queueing and claiming background refresh jobs |

---

## 2. Adapters & Implementations

| Subsystem / Capability | Concrete Type | Implementing File | Trait Implemented |
| :--- | :--- | :--- | :--- |
| **SQLite Storage** | `SqliteStore` | [`crates/ck-storage-sqlite/src/lib.rs`](file:///crates/ck-storage-sqlite/src/lib.rs) | `StoragePort`, `SessionMemoryPort` |
| **FTS5 Search** | `FtsSearch` | [`crates/ck-search-fts/src/lib.rs`](file:///crates/ck-search-fts/src/lib.rs) | `LexicalSearchPort` |
| **Lexical Ranker** | `LexicalRanker` | [`crates/ck-ranker/src/lib.rs`](file:///crates/ck-ranker/src/lib.rs) | `RankerPort` |
| **Filesystem Artifacts** | `FileArtifactStore` | [`crates/ck-artifacts/src/lib.rs`](file:///crates/ck-artifacts/src/lib.rs) | `ArtifactPort` |
| **Filesystem Ingestion** | `FilesystemSourceAdapter`| [`crates/ck-adapter-filesystem/src/lib.rs`](file:///crates/ck-adapter-filesystem/src/lib.rs) | `SourceAdapter` |
| **Markdown Normalizer** | `MarkdownNormalizer` | [`crates/ck-adapter-filesystem/src/lib.rs`](file:///crates/ck-adapter-filesystem/src/lib.rs) | `Normalizer` |
| **Heading Chunker** | `HeadingChunker` | [`crates/ck-adapter-filesystem/src/lib.rs`](file:///crates/ck-adapter-filesystem/src/lib.rs) | `Chunker` |
| **Learning Proposals** | `LearningProposalsAdapter`| [`crates/ck-adapter-filesystem/src/learning_proposals.rs`](file:///crates/ck-adapter-filesystem/src/learning_proposals.rs) | `SourceAdapter` |
| **GitHub Ingestion (Library)** | `GitHubSourceAdapter` | [`crates/ck-adapter-github/src/lib.rs`](file:///crates/ck-adapter-github/src/lib.rs) | `SourceAdapter` |
| **Web Ingestion (Library)** | `WebSourceAdapter` | [`crates/ck-adapter-web/src/lib.rs`](file:///crates/ck-adapter-web/src/lib.rs) | `SourceAdapter` |
| **OpenAI Embeddings** | `OpenAIEmbeddingProvider`| [`crates/ck-embeddings/src/lib.rs`](file:///crates/ck-embeddings/src/lib.rs) | `EmbeddingPort` |
| **No-Op Embeddings** | `NoopEmbeddingProvider` | [`crates/ck-embeddings/src/lib.rs`](file:///crates/ck-embeddings/src/lib.rs) | `EmbeddingPort` |
| **Vector Storage** | `SqliteVectorStore` | [`crates/ck-vector/src/lib.rs`](file:///crates/ck-vector/src/lib.rs) | `VectorSearchPort` |
| **Bearer Authentication** | `BearerAuth` | [`crates/ck-auth/src/lib.rs`](file:///crates/ck-auth/src/lib.rs) | `AuthPort` |
| **Refresh Scheduler** | `RefreshScheduler` | [`crates/ck-scheduler/src/lib.rs`](file:///crates/ck-scheduler/src/lib.rs) | `SchedulerPort` |
| **Application Services** | `AppServices` | [`crates/ck-app/src/lib.rs`](file:///crates/ck-app/src/lib.rs) | Orchestration Core |
| **MCP Protocol Service** | `McpService` | [`crates/ck-mcp/src/lib.rs`](file:///crates/ck-mcp/src/lib.rs) | Tool Dispatcher |
| **ACL Context Agent** | `AclContextAgent` | [`crates/ck-mcp/src/agentic_capability_loop.rs`](file:///crates/ck-mcp/src/agentic_capability_loop.rs) | E2/E3 Boundary Agent |
| **Stdio Transport** | `handle_line` | [`crates/ck-transport-stdio/src/lib.rs`](file:///crates/ck-transport-stdio/src/lib.rs) | JSON-RPC Handler |
| **SSE HTTP Transport** | `serve_sse` | [`crates/ck-transport-sse/src/lib.rs`](file:///crates/ck-transport-sse/src/lib.rs) | Axum Server |

---

## 3. MCP Tools & Dispatch Handlers

| MCP Tool Name | Dispatch Method in `ck-mcp` | Application Method in `ck-app` | Description |
| :--- | :--- | :--- | :--- |
| `get_status` | `dispatch_tool` (L122) | `AppServices::get_status` (L1912) | Reports service health, storage path, FTS5 status, and counts |
| `store_context` | `handle_store_context` (L183) | `AppServices::store_context` (L158) | Stores an event in session memory, offloading payload to artifacts |
| `recall_context` | `handle_recall_context` (L237) | `AppServices::recall_context` (L293) | Recalls historical events with session affinity weighting |
| `ingest_source` | `handle_ingest_source` (L278) | `AppServices::ingest_source` (L348) | Ingests a new documentation tree into SQLite and FTS5 |
| `refresh_source` | `handle_refresh_source` (L304) | `AppServices::refresh_source` (L1881)| Re-ingests an existing source, replacing prior documents |
| `query_reference` | `handle_query_reference` (L325)| `AppServices::query_reference` (L536) | Queries documentation chunks with intent-weighted hybrid ranking |
| `resolve_reference` | `handle_resolve_reference` (L382)| `AppServices::resolve_reference` (L1785)| Resolves a colloquial library name/query to canonical identity |
| `context_required` | `handle_context_required` (L449)| `AclContextAgent::acquire_context` | E2/E3 convergence tool for SWE_SEED work contract |
| `resolve-library-id`| Alias for `resolve_reference` | `AppServices::resolve_reference` | Context7 compatibility alias |
| `get-library-docs` | Alias for `query_reference` | `AppServices::query_reference` | Context7 compatibility alias |
| `query-docs` | Alias for `query_reference` | `AppServices::query_reference` | Context7 compatibility alias |

---

## 4. CLI Subcommands & Binary Entry Points

All subcommands are parsed in [`crates/ck-bin/src/main.rs`](file:///crates/ck-bin/src/main.rs) using `clap`:

| CLI Command | Function in `ck-bin` | Purpose | Key Flags |
| :--- | :--- | :--- | :--- |
| `serve` | `fn serve` (L65) | Starts the MCP service over stdio and optionally SSE | `--config`, `--database`, `--data-dir`, `--sse` |
| `migrate` | `fn migrate` (L310) | Executes embedded SQLite `SCHEMA_V1` DDL | `--config`, `--database` |
| `doctor` | `fn doctor` (L330) | Checks database connectivity, path, and FTS5 availability | `--config`, `--database` |
| `export-schema` | `Commands::ExportSchema` (L57) | Prints embedded SQLite schema DDL to stdout | None |
| `rebuild-index` | `fn rebuild_index` (L318) | Repopulates FTS5 tables from canonical SQLite rows | `--config`, `--database` |

---

## 5. Persistence & Schema Mapping

The database schema is defined as a static SQL constant `SCHEMA_V1` in [`crates/ck-storage-sqlite/src/lib.rs`](file:///crates/ck-storage-sqlite/src/lib.rs#L1092-L1297):

| Table Name | Entity Mapped | Key Primary / Foreign Keys | Indexes / Virtual Table Details |
| :--- | :--- | :--- | :--- |
| `schema_migrations`| Migration tracking | `version INTEGER PRIMARY KEY` | Records version 1 installation timestamp |
| `sources` | `Source` | `source_id TEXT PRIMARY KEY` | Stores kind, config JSON, freshness TTL, and health |
| `corpora` | `Corpus` | `corpus_id TEXT PRIMARY KEY` | Stores name, kind, and visibility |
| `corpus_sources` | Junction | `PRIMARY KEY (corpus_id, source_id)` | Foreign keys cascade on delete |
| `library_identities`| `LibraryIdentity`| `library_id TEXT PRIMARY KEY` | Stores provider, name, version, and trust score |
| `library_aliases` | `LibraryAlias` | `PRIMARY KEY (alias, library_id)` | Cascade delete on `library_id` |
| `documents` | `Document` | `document_id TEXT PRIMARY KEY` | Indexed on `source_id`, `corpus_id`, `library_id`, `checksum` |
| `chunks` | `Chunk` | `chunk_id TEXT PRIMARY KEY` | Indexed on `document_id`, `corpus_id`, `ordinal`, `checksum` |
| `artifacts` | `Artifact` | `artifact_id TEXT PRIMARY KEY` | Stores storage URI, size bytes, and SHA-256 checksum |
| `sessions` | `Session` | `session_id TEXT PRIMARY KEY` | Stores transport, continuation mode, and session status |
| `session_events` | `SessionEvent` | `event_id TEXT PRIMARY KEY` | Indexed on `session_id`, `kind`, `created_at`; FK to `artifacts` |
| `refresh_jobs` | `RefreshJob` | `job_id TEXT PRIMARY KEY` | Unique partial index `idx_refresh_jobs_one_active_per_source` |
| `chunks_fts` | `Chunk` FTS5 | Virtual Table (fts5) | Unindexed IDs; indexed `title`, `heading`, `text` (`unicode61`) |
| `session_events_fts`| `SessionEvent` FTS5| Virtual Table (fts5) | Unindexed IDs; indexed `kind`, `text` (`unicode61`) |
| `library_aliases_fts`| `LibraryAlias` FTS5| Virtual Table (fts5) | Indexed `alias`, `name`, `provider` (`unicode61`) |
| `chunk_embeddings` | Vector Storage | `PRIMARY KEY (chunk_id, model_id)` | Created by `ck-vector`; stores raw binary vector blob |
| `session_event_embeddings` | Vector Storage | `PRIMARY KEY (event_id, model_id)` | Created by `ck-vector`; stores raw binary vector blob |

---

## 6. Test Suites & Verification Anchors

| Test Target | File Path | Test Types | Key Invariants Proven |
| :--- | :--- | :--- | :--- |
| **CLI Integration** | [`crates/ck-bin/tests/cli.rs`](file:///crates/ck-bin/tests/cli.rs) | Integration (assert_cmd) | Migration, doctor, schema export, stdio `get_status`, stdio `store_context`/`recall_context`, SSE HTTP listener, config reload |
| **ACL Context Agent**| [`crates/ck-mcp/tests/acl_context_agent.rs`](file:///crates/ck-mcp/tests/acl_context_agent.rs)| Integration | E2 envelope ingress gate, domain model hash verification, zero-citation governance |
| **Learning Proposals**|[`crates/ck-adapter-filesystem/tests/learning_proposals.rs`](file:///crates/ck-adapter-filesystem/tests/learning_proposals.rs)| Integration | Ledger JSON parsing, document creation, chunk generation |
| **Storage Unit Tests**|[`crates/ck-storage-sqlite/src/lib.rs`](file:///crates/ck-storage-sqlite/src/lib.rs#L1300)| Unit (rusqlite) | Foreign key cascade, enum round-tripping, FTS index rebuild, job coalescing, retention deletion |
| **FTS Unit Tests** |[`crates/ck-search-fts/src/lib.rs`](file:///crates/ck-search-fts/src/lib.rs#L190)| Unit (rusqlite) | BM25 query construction, punctuation escaping, score inversion, chunk removal |
| **Ranker Unit Tests**|[`crates/ck-ranker/src/lib.rs`](file:///crates/ck-ranker/src/lib.rs#L240)| Unit | Intent weighting profiles, session affinity, document de-duplication |
| **Artifact Tests** |[`crates/ck-artifacts/src/lib.rs`](file:///crates/ck-artifacts/src/lib.rs#L130)| Unit (tempfile) | Path traversal protection, checksum calculation, payload deletion |
| **Auth Unit Tests** |[`crates/ck-auth/src/lib.rs`](file:///crates/ck-auth/src/lib.rs#L91)| Unit | Constant-time token equality, tool allowlists, corpus allowlists |
| **Web Adapter Tests**|[`crates/ck-adapter-web/src/lib.rs`](file:///crates/ck-adapter-web/src/lib.rs#L300)| Unit | Private IP rejection (SSRF), DNS resolution failure handling, ETag caching |
| **GitHub Adapter Tests**|[`crates/ck-adapter-github/src/lib.rs`](file:///crates/ck-adapter-github/src/lib.rs#L280)| Unit | GitHub API tree truncation detection, URL percent-encoding |
| **Scheduler Tests** |[`crates/ck-scheduler/src/lib.rs`](file:///crates/ck-scheduler/src/lib.rs#L160)| Unit | Job enqueueing, coalescing into active jobs, job claiming |
| **Exact Test Harness**|[`scripts/run-exact-test.sh`](file:///scripts/run-exact-test.sh) | Bash Harness | Enforces exact 1-to-1 test match; fails closed on zero or multiple test matches |
