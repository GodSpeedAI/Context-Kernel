# Subsystem: Application Orchestration

The Application Orchestration subsystem consists of `ck-app`. It implements `AppServices`, the central orchestrator that coordinates domain ports to execute the primary use cases of Context Kernel.

---

## 1. Purpose
`AppServices` is the application layer. It sits between incoming protocol requests (from `ck-mcp` or CLI) and outgoing infrastructure adapters (storage, search, ranking, artifacts, sources). It enforces transactional workflows, calculates composite metrics, and ensures business invariants are preserved across storage boundaries.

---

## 2. Responsibilities
* Coordinate use-case execution for:
  - `store_context`: Stores session events and manages artifact offloading.
  - `recall_context`: Recalls prior session events matching a query with session affinity.
  - `ingest_source`: Ingests an external documentation source, normalizes content, chunks text, and indexes in SQLite and FTS5.
  - `refresh_source`: Re-indexes an existing source, cleanly replacing prior documents without accumulation.
  - `query_reference`: Executes hybrid retrieval, intent-weighted ranking, and de-duplicated citation generation.
  - `resolve_reference`: Resolves library queries to canonical identities using FTS and alias exact-matching.
  - `get_status`: Aggregates service, storage, source, session, job, and retrieval counts.
  - `run_retention_gc`: Executes scheduled garbage collection across unpinned events, inactive sessions, jobs, artifacts, and disabled sources.

---

## 3. Non-Responsibilities
* **No Transport Parsing**: Does not handle JSON-RPC framing, HTTP request headers, or SSE streaming; accepts strongly-typed request structs and returns response structs.
* **No Direct SQL Execution**: Communicates with persistence exclusively through `StoragePort`, `SessionMemoryPort`, and `LexicalSearchPort`.

---

## 4. Position in the System
`ck-app` sits at the center of the architecture, directly invoked by `ck-mcp` and driving all adapters via `ck-core` ports.

```mermaid
graph TD
    MCP["ck-mcp::McpService"] -->|Request Structs| App["ck-app::AppServices"]

    App -->|StoragePort & SessionMemoryPort| Storage["ck-storage-sqlite"]
    App -->|ArtifactPort| Artifacts["ck-artifacts"]
    App -->|LexicalSearchPort| FTS["ck-search-fts"]
    App -->|RankerPort| Ranker["ck-ranker"]
    App -->|SourceAdapter| Adapters["ck-adapter-filesystem"]
    App -->|EmbeddingPort| Embeddings["ck-embeddings"]
    App -->|VectorSearchPort| Vectors["ck-vector"]
```

---

## 5. Core Abstractions

### `AppServices` ([`crates/ck-app/src/lib.rs`](file:///crates/ck-app/src/lib.rs#L15-L26))
The struct holds boxed trait objects for all ports:
```rust
pub struct AppServices {
    storage: Box<dyn StoragePort>,
    session_memory: Box<dyn SessionMemoryPort>,
    artifact_store: Box<dyn ArtifactPort>,
    search: Box<dyn LexicalSearchPort>,
    ranker: Box<dyn RankerPort>,
    source_adapter: Box<dyn SourceAdapter>,
    normalizer: Box<dyn Normalizer>,
    chunker: Box<dyn Chunker>,
    embedding: Option<Box<dyn EmbeddingPort>>,
    vector_search: Option<Box<dyn VectorSearchPort>>,
}
```

### Request & Response DTOs
Every use case is represented by explicit, strongly typed request and response pairs:
* `StoreContextRequest` / `StoreContextResponse`
* `RecallContextRequest` / `RecallContextResponse`
* `IngestSourceRequest` / `IngestSourceResponse`
* `RefreshSourceRequest` / `RefreshSourceResponse`
* `QueryReferenceRequest` / `QueryReferenceResponse`
* `ResolveReferenceRequest` / `ResolveReferenceResponse`
* `RetentionPolicy` / `RetentionGcReport`

---

## 6. Internal Operation

### Context Storage & Artifact Offloading (`store_context`)
1. Verifies that the target session exists; if missing, auto-creates it with default settings.
2. Checks if an `artifact_content` payload is present:
   - If present, writes the raw bytes to the artifact store (`ArtifactPort::store_artifact`).
   - If storing artifact metadata in SQLite fails, `AppServices` executes a compensating delete (`ArtifactPort::delete_artifact_payload`) so orphan files never leak.
3. Inserts the `SessionEvent` into SQLite.
4. Indexes the event text in `session_events_fts` for future recall.

### Documentation Refresh & Non-Accumulation (`refresh_source`)
1. Fetches the existing `Source` metadata.
2. Identifies all documents previously produced by this source (`StoragePort::list_documents_by_source`).
3. For each existing document:
   - Removes its chunks from the FTS5 index (`LexicalSearchPort::remove_chunks_by_document`).
   - Deletes the document from SQLite (foreign key cascades automatically delete canonical chunks).
4. Re-discovers units and re-ingests the source, generating fresh document and chunk records.
5. Re-indexes the new chunks in FTS5.
6. Updates source timestamps (`last_refresh_success_at`, `updated_at`).

### Intent-Weighted Reference Retrieval (`query_reference`)
1. Resolves target corpora from `source_scope`.
2. Queries FTS5 for top lexical matches using SQLite BM25.
3. If semantic embeddings are configured, queries SQLite vector store for cosine similarity.
4. Calls `RankerPort::rank_results`, applying the intent's `WeightProfile`.
5. De-duplicates results by `document_id`.
6. Resolves parent document metadata to construct citations with line and heading anchors.

---

## 7. State
`AppServices` is internally stateless. It holds immutable trait objects (`Box<dyn Port>`). All mutable state is stored in SQLite or on the filesystem via the ports.

---

## 8. Lifecycle
Constructed in `main.rs::wire_app` during service startup and wrapped in an `Arc<McpService>`. Reconstructed dynamically when hot configuration reload is triggered.

---

## 9. Failure Modes
* **Partial Ingestion**: If an individual document fails during ingestion, `AppServices` logs a warning and marks the source `SourceHealth::Degraded`, recording the error in `last_refresh_error`.
* **Artifact Compensation**: If metadata persistence fails after disk write, the compensating delete ensures disk cleanliness.
* **Semantic Failure**: If the embedding provider fails or is unconfigured, queries automatically fall back to pure lexical search.

---

## 10. Extension Points
* **Semantic Retrieval Toggle**: `AppServices::with_semantic(embedding, vector_search)` enables vector search dynamically.
* **Additional Use Cases**: New domain workflows (e.g., bulk document export) can be implemented directly on `AppServices`.

---

## 11. Source Trail
* `crates/ck-app/src/lib.rs` — `AppServices` definition, `store_context` (L158), `recall_context` (L293), `ingest_source` (L348), `query_reference` (L536), `resolve_reference` (L1785), `refresh_source` (L1881), `run_retention_gc` (L50).
