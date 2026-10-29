# Source Map & Traceability Index

This document maps all high-level concepts, architectural ports, adapters, CLI commands, configuration keys, database tables, and MCP tools directly to their concrete implementation locations in the repository.

---

## 1. Domain Entities & Port Traits

| Domain Concept / Port | Code Symbol | File Path | Line References | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Source Entity** | `struct Source` | [`crates/ck-core/src/entities/source.rs`](../crates/ck-core/src/entities/source.rs) | L8–L25 | External documentation provider configuration and refresh state |
| **Corpus Entity** | `struct Corpus` | [`crates/ck-core/src/entities/corpus.rs`](../crates/ck-core/src/entities/corpus.rs) | L6–L15 | Logical retrieval grouping and access-control partition |
| **Document Entity** | `struct Document` | [`crates/ck-core/src/entities/document.rs`](../crates/ck-core/src/entities/document.rs) | L8–L26 | Ingested file or web page content record |
| **Chunk Entity** | `struct Chunk` | [`crates/ck-core/src/entities/chunk.rs`](../crates/ck-core/src/entities/chunk.rs) | L8–L23 | Bounded text excerpt (~800 words) with heading and anchors |
| **Library Identity** | `struct LibraryIdentity` | [`crates/ck-core/src/entities/library.rs`](../crates/ck-core/src/entities/library.rs) | L6–L20 | Canonical package name, version, and trust score |
| **Library Alias** | `struct LibraryAlias` | [`crates/ck-core/src/entities/library.rs`](../crates/ck-core/src/entities/library.rs) | L22–L28 | Alternative search term mapped to a library identity |
| **Session Entity** | `struct Session` | [`crates/ck-core/src/entities/session.rs`](../crates/ck-core/src/entities/session.rs) | L6–L16 | Conversational memory container |
| **Session Event** | `struct SessionEvent` | [`crates/ck-core/src/entities/session.rs`](../crates/ck-core/src/entities/session.rs) | L18–L29 | Timestamped semantic memory item |
| **Artifact Entity** | `struct Artifact` | [`crates/ck-core/src/entities/artifact.rs`](../crates/ck-core/src/entities/artifact.rs) | L6–L18 | Filesystem payload metadata record |
| **Refresh Job** | `struct RefreshJob` | [`crates/ck-core/src/entities/job.rs`](../crates/ck-core/src/entities/job.rs) | L6–L20 | Asynchronous source re-indexing task |
| **Storage Port** | `trait StoragePort` | [`crates/ck-core/src/ports/storage.rs`](../crates/ck-core/src/ports/storage.rs) | L7–L88 | CRUD interface for entities, corpora, and retention sweeps |
| **Session Memory Port**| `trait SessionMemoryPort`| [`crates/ck-core/src/ports/session_memory.rs`](../crates/ck-core/src/ports/session_memory.rs) | L6–L38 | Session and event persistence interface |
| **Lexical Search Port**| `trait LexicalSearchPort`| [`crates/ck-core/src/ports/lexical_search.rs`](../crates/ck-core/src/ports/lexical_search.rs) | L6–L45 | FTS5 index update and query interface |
| **Ranker Port** | `trait RankerPort` | [`crates/ck-core/src/ports/ranker.rs`](../crates/ck-core/src/ports/ranker.rs) | L14–L41 | Hybrid scoring and result de-duplication interface |
| **Artifact Port** | `trait ArtifactPort` | [`crates/ck-core/src/ports/artifact.rs`](../crates/ck-core/src/ports/artifact.rs) | L5–L17 | Content-addressed filesystem artifact storage interface |
| **Source Adapter** | `trait SourceAdapter` | [`crates/ck-core/src/ports/source_adapter.rs`](../crates/ck-core/src/ports/source_adapter.rs) | L6–L13 | Ingestion discovery and content fetching interface |
| **Normalizer Port** | `trait Normalizer` | [`crates/ck-core/src/ports/normalizer.rs`](../crates/ck-core/src/ports/normalizer.rs) | L5–L8 | Text cleaning and document normalization interface |
| **Chunker Port** | `trait Chunker` | [`crates/ck-core/src/ports/chunker.rs`](../crates/ck-core/src/ports/chunker.rs) | L6–L8 | Document structural heading / token splitting interface |
| **Embedding Port** | `trait EmbeddingPort` | [`crates/ck-core/src/ports/embedding.rs`](../crates/ck-core/src/ports/embedding.rs) | L5–L20 | Vector embedding generation interface |
| **Vector Search Port** | `trait VectorSearchPort` | [`crates/ck-core/src/ports/vector_search.rs`](../crates/ck-core/src/ports/vector_search.rs) | L8–L44 | SQLite brute-force cosine vector search interface |
| **Auth Port** | `trait AuthPort` | [`crates/ck-core/src/ports/auth.rs`](../crates/ck-core/src/ports/auth.rs) | L3–L13 | Bearer token and allowlist validation interface |
| **Context Requirement** | `struct ContextRequirement` | [`crates/ck-app/src/context.rs`](../crates/ck-app/src/context.rs#L69-L118) | L69–L118 | Typed retrieval demand: identity, world pin, corpora, constraints, freshness, budget |
| **Context Resolution** | `struct ContextResolution` | [`crates/ck-app/src/context.rs`](../crates/ck-app/src/context.rs#L180-L243) | L180–L243 | Resolved results, omissions, completeness, access, and no-context reason |
| **Context Effect** | `struct ContextEffect` | [`crates/ck-app/src/context_effect.rs`](../crates/ck-app/src/context_effect.rs#L56-L79) | L56–L79 | Settlement-backed calibration record for one pinned world |
| **Context Effect Store** | `struct ContextEffectStore` | [`crates/ck-app/src/context_effect.rs`](../crates/ck-app/src/context_effect.rs#L201-L210) | L201–L210 | Append-only per-world JSONL ledger port |
| **Completeness & Reasons** | `RetrievalCompleteness`, `NoContextReason` | [`crates/ck-core/src/types/context.rs`](../crates/ck-core/src/types/context.rs) | L18–L70 | Complete/partial/none and the eight governed no-context codes |
| **World Pin Gate** | `verify_world_ref` | [`crates/ck-core/src/types/context.rs`](../crates/ck-core/src/types/context.rs#L91-L118) | L91–L118 | Syntax-only pinned-world reference check |
| **Scheduler Port** | `trait SchedulerPort` | [`crates/ck-core/src/ports/scheduler.rs`](../crates/ck-core/src/ports/scheduler.rs) | L6–L27 | Queueing and claiming background refresh jobs |

---

## 2. Adapters & Implementations

| Subsystem / Capability | Concrete Type | Implementing File | Trait Implemented |
| :--- | :--- | :--- | :--- |
| **SQLite Storage** | `SqliteStore` | [`crates/ck-storage-sqlite/src/lib.rs`](../crates/ck-storage-sqlite/src/lib.rs) | `StoragePort`, `SessionMemoryPort` |
| **FTS5 Search** | `FtsSearch` | [`crates/ck-search-fts/src/lib.rs`](../crates/ck-search-fts/src/lib.rs) | `LexicalSearchPort` |
| **Lexical Ranker** | `LexicalRanker` | [`crates/ck-ranker/src/lib.rs`](../crates/ck-ranker/src/lib.rs) | `RankerPort` |
| **Filesystem Artifacts** | `FileArtifactStore` | [`crates/ck-artifacts/src/lib.rs`](../crates/ck-artifacts/src/lib.rs) | `ArtifactPort` |
| **Filesystem Ingestion** | `FilesystemSourceAdapter`| [`crates/ck-adapter-filesystem/src/lib.rs`](../crates/ck-adapter-filesystem/src/lib.rs) | `SourceAdapter` |
| **Markdown Normalizer** | `MarkdownNormalizer` | [`crates/ck-adapter-filesystem/src/lib.rs`](../crates/ck-adapter-filesystem/src/lib.rs) | `Normalizer` |
| **Heading Chunker** | `HeadingChunker` | [`crates/ck-adapter-filesystem/src/lib.rs`](../crates/ck-adapter-filesystem/src/lib.rs) | `Chunker` |
| **Learning Proposals** | `LearningProposalsAdapter`| [`crates/ck-adapter-filesystem/src/learning_proposals.rs`](../crates/ck-adapter-filesystem/src/learning_proposals.rs) | `SourceAdapter` |
| **GitHub Ingestion (Library)** | `GitHubSourceAdapter` | [`crates/ck-adapter-github/src/lib.rs`](../crates/ck-adapter-github/src/lib.rs) | `SourceAdapter` |
| **Web Ingestion (Library)** | `WebSourceAdapter` | [`crates/ck-adapter-web/src/lib.rs`](../crates/ck-adapter-web/src/lib.rs) | `SourceAdapter` |
| **OpenAI Embeddings** | `OpenAIEmbeddingProvider`| [`crates/ck-embeddings/src/lib.rs`](../crates/ck-embeddings/src/lib.rs) | `EmbeddingPort` |
| **No-Op Embeddings** | `NoopEmbeddingProvider` | [`crates/ck-embeddings/src/lib.rs`](../crates/ck-embeddings/src/lib.rs) | `EmbeddingPort` |
| **Vector Storage** | `SqliteVectorStore` | [`crates/ck-vector/src/lib.rs`](../crates/ck-vector/src/lib.rs) | `VectorSearchPort` |
| **Bearer Authentication** | `BearerAuth` | [`crates/ck-auth/src/lib.rs`](../crates/ck-auth/src/lib.rs) | `AuthPort` |
| **Refresh Scheduler** | `RefreshScheduler` | [`crates/ck-scheduler/src/lib.rs`](../crates/ck-scheduler/src/lib.rs) | `SchedulerPort` |
| **Application Services** | `AppServices` | [`crates/ck-app/src/lib.rs`](../crates/ck-app/src/lib.rs) | Orchestration Core |
| **MCP Protocol Service** | `McpService` | [`crates/ck-mcp/src/lib.rs`](../crates/ck-mcp/src/lib.rs) | Tool Dispatcher |
| **ACL Context Agent** | `AclContextAgent` | [`crates/ck-mcp/src/agentic_capability_loop.rs`](../crates/ck-mcp/src/agentic_capability_loop.rs) | E2/E3 Boundary Agent |
| **Context Bundle Projection** | `project_context_bundle`, `assemble` | [`crates/ck-mcp/src/cep_bundle.rs`](../crates/ck-mcp/src/cep_bundle.rs) | CEP-0008 Truthfulness + Integrity Seal |
| **Context Effect Ledger (JSONL)** | `ContextEffectStore::new` | [`crates/ck-app/src/context_effect.rs`](../crates/ck-app/src/context_effect.rs#L201-L230) | Append-only per-world effect ledger, wired at [`main.rs`](../crates/ck-bin/src/main.rs#L430) |
| **Stdio Transport** | `handle_line` | [`crates/ck-transport-stdio/src/lib.rs`](../crates/ck-transport-stdio/src/lib.rs) | JSON-RPC Handler |
| **SSE HTTP Transport** | `serve_sse` | [`crates/ck-transport-sse/src/lib.rs`](../crates/ck-transport-sse/src/lib.rs) | Axum Server |

---

## 3. MCP Tools & Dispatch Handlers

| MCP Tool Name | Dispatch Method in `ck-mcp` | Application Method in `ck-app` | Description |
| :--- | :--- | :--- | :--- |
| `get_status` | `dispatch_tool` (L120) | `AppServices::get_status` (L1964) | Reports service health, storage path, FTS5 status, and counts |
| `store_context` | `handle_store_context` (L185) | `AppServices::store_context` (L206) | Stores an event in session memory, offloading payload to artifacts |
| `recall_context` | `handle_recall_context` (L239) | `AppServices::recall_context` (L339) | Recalls historical events with session affinity weighting |
| `ingest_source` | `handle_ingest_source` (L283) | `AppServices::ingest_source` (L393) | Ingests a new documentation tree into SQLite and FTS5 |
| `refresh_source` | `handle_refresh_source` (L572) | `AppServices::refresh_source` (L1934)| Re-ingests an existing source, replacing prior documents |
| `query_reference` | `handle_query_reference` (L320)| `AppServices::query_reference` (L590) | Queries documentation chunks with intent-weighted hybrid ranking |
| `resolve_reference` | `handle_resolve_reference` (L384)| `AppServices::resolve_reference` (L1840)| Resolves a colloquial library name/query to canonical identity |
| `context_required` | `handle_context_required` (L476)| `AclContextAgent::acquire_context` (L285) | E2/E3 convergence tool; projects `ContextResolution` into a CEP bundle |
| `ingest_context_effects` | `handle_ingest_context_effects` (L441) | `AppServices::ingest_context_effects` ([context.rs L579](../crates/ck-app/src/context.rs#L579)) | Records settlement-backed calibration effects for one pinned world |
| `resolve-library-id`| Alias for `resolve_reference` | `AppServices::resolve_reference` | Context7 compatibility alias |
| `get-library-docs` | Alias for `query_reference` | `AppServices::query_reference` | Context7 compatibility alias |
| `query-docs` | Alias for `query_reference` | `AppServices::query_reference` | Context7 compatibility alias |

---

## 3a. Context Pipeline Entry Points

| Step | Symbol | File Path | Line References | Behavior |
| :--- | :--- | :--- | :--- | :--- |
| Ingress gates | `handle_context_required` | [`crates/ck-mcp/src/lib.rs`](../crates/ck-mcp/src/lib.rs) | L476–L569 | Envelope, producer, identity, world, and correlation checks; fail-closed |
| Corpus resolution | `AclContextAgent::ensure_corpus_ingested` | [`crates/ck-mcp/src/agentic_capability_loop.rs`](../crates/ck-mcp/src/agentic_capability_loop.rs) | L420–L470 | Registers operator-configured corpora once; private claims never auto-ingest |
| Acquisition | `AclContextAgent::acquire_context` | [`crates/ck-mcp/src/agentic_capability_loop.rs`](../crates/ck-mcp/src/agentic_capability_loop.rs) | L288–L354 | Projects payload, resolves corpora, runs the pipeline, selects outcome |
| Resolution | `AppServices::resolve_context` | [`crates/ck-app/src/context.rs`](../crates/ck-app/src/context.rs) | L245–L535 | Eight stages: access, retrieval, freshness, budget, availability, concepts, calibration, verdict |
| Effect ingestion | `AppServices::ingest_context_effects` | [`crates/ck-app/src/context.rs`](../crates/ck-app/src/context.rs) | L579–L600 | Validates settlement evidence, appends to the per-world ledger |
| Projection | `project_context_bundle` | [`crates/ck-mcp/src/cep_bundle.rs`](../crates/ck-mcp/src/cep_bundle.rs) | L345–L430 | Maps resolution and packet onto the CEP bundle |
| Truthfulness + seal | `assemble` | [`crates/ck-mcp/src/cep_bundle.rs`](../crates/ck-mcp/src/cep_bundle.rs) | L179–L331 | Refuses inconsistent truth claims; stamps `integrity` last |
| Canonical hash | `content_hash`, `canonical_json` | [`crates/ck-mcp/src/cep_bundle.rs`](../crates/ck-mcp/src/cep_bundle.rs) | L70–L110 | Sorted keys, compact separators, six-decimal floats, negative zero flattened |
| World pin gate | `verify_world_ref` | [`crates/ck-core/src/types/context.rs`](../crates/ck-core/src/types/context.rs) | L91–L118 | Refuses mutable aliases and malformed digests |

---

## 4. CLI Subcommands & Binary Entry Points

All subcommands are parsed in [`crates/ck-bin/src/main.rs`](../crates/ck-bin/src/main.rs) using `clap`:

| CLI Command | Function in `ck-bin` | Purpose | Key Flags |
| :--- | :--- | :--- | :--- |
| `serve` | `fn serve` (L65) | Starts the MCP service over stdio and optionally SSE | `--config`, `--database`, `--data-dir`, `--sse` |
| `migrate` | `fn migrate` (L310) | Executes embedded SQLite `SCHEMA_V1` DDL | `--config`, `--database` |
| `doctor` | `fn doctor` (L330) | Checks database connectivity, path, and FTS5 availability | `--config`, `--database` |
| `export-schema` | `Commands::ExportSchema` (L57) | Prints embedded SQLite schema DDL to stdout | None |
| `rebuild-index` | `fn rebuild_index` (L318) | Repopulates FTS5 tables from canonical SQLite rows | `--config`, `--database` |

---

## 5. Persistence & Schema Mapping

The database schema is defined as a static SQL constant `SCHEMA_V1` in [`crates/ck-storage-sqlite/src/lib.rs`](../crates/ck-storage-sqlite/src/lib.rs#L1108-L1313):

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
| **CLI Integration** | [`crates/ck-bin/tests/cli.rs`](../crates/ck-bin/tests/cli.rs) | Integration (assert_cmd) | Migration, doctor, schema export, stdio `get_status`, stdio `store_context`/`recall_context`, SSE HTTP listener, config reload |
| **ACL Context Agent**| [`crates/ck-mcp/tests/acl_context_agent.rs`](../crates/ck-mcp/tests/acl_context_agent.rs)| Integration | E2 envelope ingress gate, domain model hash verification, zero-citation governance, result-cap omissions |
| **Context Pipeline** | [`crates/ck-mcp/tests/context_pipeline.rs`](../crates/ck-mcp/tests/context_pipeline.rs)| Integration | Retrieval identity and ranking metadata, semantic merge, private-corpus scope enforcement, `query_reference` parity |
| **Bundle Conformance** | [`crates/ck-mcp/tests/context_bundle_conformance.rs`](../crates/ck-mcp/tests/context_bundle_conformance.rs)| Conformance | Complete/partial/none truthfulness, false completeness refusal, profile and world-ref rejection |
| **Calibration** | [`crates/ck-mcp/tests/context_calibration.rs`](../crates/ck-mcp/tests/context_calibration.rs)| Integration | Prospective demotion, self-report refusal, world isolation, idempotent re-ingest, access semantics |
| **End-to-End World Loop** | [`crates/ck-mcp/tests/e2e_world_loop.rs`](../crates/ck-mcp/tests/e2e_world_loop.rs)| Integration | One full `context_required` hop over a real corpus |
| **Learning Proposals**|[`crates/ck-adapter-filesystem/tests/learning_proposals.rs`](../crates/ck-adapter-filesystem/tests/learning_proposals.rs)| Integration | Ledger JSON parsing, document creation, chunk generation |
| **Storage Unit Tests**|[`crates/ck-storage-sqlite/src/lib.rs`](../crates/ck-storage-sqlite/src/lib.rs#L1300)| Unit (rusqlite) | Foreign key cascade, enum round-tripping, FTS index rebuild, job coalescing, retention deletion |
| **FTS Unit Tests** |[`crates/ck-search-fts/src/lib.rs`](../crates/ck-search-fts/src/lib.rs#L190)| Unit (rusqlite) | BM25 query construction, punctuation escaping, score inversion, chunk removal |
| **Ranker Unit Tests**|[`crates/ck-ranker/src/lib.rs`](../crates/ck-ranker/src/lib.rs#L240)| Unit | Intent weighting profiles, session affinity, document de-duplication |
| **Artifact Tests** |[`crates/ck-artifacts/src/lib.rs`](../crates/ck-artifacts/src/lib.rs#L130)| Unit (tempfile) | Path traversal protection, checksum calculation, payload deletion |
| **Auth Unit Tests** |[`crates/ck-auth/src/lib.rs`](../crates/ck-auth/src/lib.rs#L91)| Unit | Constant-time token equality, tool allowlists, corpus allowlists |
| **Web Adapter Tests**|[`crates/ck-adapter-web/src/lib.rs`](../crates/ck-adapter-web/src/lib.rs#L300)| Unit | Private IP rejection (SSRF), DNS resolution failure handling, ETag caching |
| **GitHub Adapter Tests**|[`crates/ck-adapter-github/src/lib.rs`](../crates/ck-adapter-github/src/lib.rs#L280)| Unit | GitHub API tree truncation detection, URL percent-encoding |
| **Scheduler Tests** |[`crates/ck-scheduler/src/lib.rs`](../crates/ck-scheduler/src/lib.rs#L160)| Unit | Job enqueueing, coalescing into active jobs, job claiming |
| **Exact Test Harness**|[`scripts/run-exact-test.sh`](../scripts/run-exact-test.sh) | Bash Harness | Enforces exact 1-to-1 test match; fails closed on zero or multiple test matches |
