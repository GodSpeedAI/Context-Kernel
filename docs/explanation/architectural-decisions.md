# Explanation: Architectural Decisions

This document records the fundamental design decisions that shape Context Kernel, distinguishing verified design rationales from inferred rationales, and examining the trade-offs, constraints, and rejected alternatives.

---

## 1. Ports-and-Adapters (Hexagonal Architecture)

### The Decision
Context Kernel isolates all domain logic and entities inside `ck-core`, use-case coordination inside `ck-app`, and all concrete IO mechanisms inside adapter crates (`ck-storage-sqlite`, `ck-search-fts`, `ck-transport-*`, `ck-adapter-*`).

### Rationale & Constraints (Verified)
* **Swappability Without Core Impact**: Context Kernel is designed to support local developer environments (SQLite, stdio) and enterprise internal self-hosted environments (SSE, potential future PostgreSQL or cloud storage) using the exact same domain contracts.
* **Testability Without Mocks**: The domain and application use cases can be fully verified in unit tests by substituting lightweight test implementations for any port (e.g. testing `AppServices` with in-memory storage or mock adapters).
* **Prevention of Architectural Drift**: Strict crate boundaries prevent infrastructure types (e.g. `rusqlite::Connection`, `axum::Router`, `tokio::TcpListener`) from leaking into core logic.

### Trade-offs
* **Indirection Overhead**: Requires defining trait interfaces in `ck-core` and boxing them (`Box<dyn Port>`) in `AppServices`, introducing small dynamic dispatch costs and boilerplate DTO mappings.
* **Earnedness Assessment**: This complexity is earned because Context Kernel is an infrastructure service designed to support multiple transports and swappable backends over a multi-year lifecycle.

---

## 2. Single-Binary Delivery with Embedded SQLite & FTS5

### The Decision
Context Kernel compiles into a single Rust binary (`context-kernal`), embedding all SQLite migration DDL (`SCHEMA_V1`), running with SQLite's bundled C source, and using the built-in SQLite FTS5 module.

### Rationale & Constraints (Verified)
* **Zero-Dependency Deployment**: AI agents and developers should not need to provision a PostgreSQL cluster, Redis cache, or Elasticsearch instance just to run a context service on their workstation or CI runner.
* **SQLite WAL Mode Concurrency**: SQLite with `PRAGMA journal_mode = WAL` allows concurrent readers to proceed without blocking writers, providing sufficient throughput for local coding agents and internal team servers.
* **Built-In BM25 Lexical Engine**: SQLite FTS5 provides robust BM25 ranking natively, eliminating the operational overhead of a separate Lucene or Meilisearch process.

### Rejected Alternatives (Verified from Specification)
* **Dedicated Vector DB (e.g., Qdrant, Chroma)**: Rejected for v1 to avoid external operational dependencies. Context Kernel provides brute-force cosine search in SQLite (`ck-vector`), which is sufficient for small documentation corpora.
* **Client-Server Database (PostgreSQL)**: Rejected as the default baseline because it breaks local single-command developer setup.

---

## 3. Dedicated Filesystem Artifact Store

### The Decision
Large text and binary payloads (command outputs, AST logs, code diffs, images) are not stored as database BLOBs in SQLite. Instead, they are offloaded to disk files (`<data_dir>/artifacts/<artifact_id>.bin`) and referenced by SQLite metadata rows.

### Rationale & Constraints (Verified)
* **Database Bloat Prevention**: Storing large binary payloads inside SQLite increases database fragmentation and degrades backup and query performance.
* **Prompt Token Minimization**: Storing payloads on disk ensures agents interact with lightweight metadata handles (`artifact_id`, size, checksum) rather than stuffing megabytes of raw text into the conversation context window.

### Trade-offs & Mitigations
* **Two-Phase Consistency Risk**: Writing to disk and writing to SQLite are separate operations. A crash after writing disk files could leave orphaned payloads.
* **Mitigation**: `AppServices::store_context` implements explicit compensation logic: if the database insert fails, the disk payload is deleted immediately. Furthermore, retention GC cleans up unreferenced artifact files on disk.

---

## 4. Multi-Transport Serving: Local Stdio vs. Remote SSE

### The Decision
Context Kernel supports both standard input/output (stdio) JSON-RPC and HTTP Server-Sent Events (SSE).

### Rationale & Constraints (Verified)
* **Stdio for Local Trust**: When running locally inside an IDE (VS Code, Cursor) or via CLI, the process boundary is the trust boundary. Stdio provides zero-configuration, zero-port-conflict communication without network overhead.
* **SSE for Shared Deployments**: When deployed as a shared internal documentation service for a team or CI pipeline, HTTP SSE allows multiple agents to query the service over a network socket with bearer token authentication.

---

## 5. Causal Lineage & E2/E3 Boundary Separation

### The Decision
Context Kernel adheres strictly to the frozen E2/E3 contract with SWE_SEED and SEA-Forge.

### Rationale (Verified)
Context Kernel acts exclusively as contextual authority. If Context Kernel attempted to generate action authorizations (e.g., granting permissions to execute code), authority would become fragmented across multiple servers. By restricting Context Kernel to retrieval and citation assembly, the system maintains a single source of action authority (SEA-Forge) and a single source of work orchestration (SWE_SEED).
