# Reference: Model Context Protocol (MCP) Tools

Context Kernel implements the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/). It serves tools over standard input/output (`tools/call` JSON-RPC 2.0) and HTTP Server-Sent Events (SSE).

---

## 1. Tool Index

| Tool Name | Tool Category | Aliases | Description |
| :--- | :--- | :--- | :--- |
| [`get_status`](#1-get_status) | System | — | Returns service health, storage stats, and counters |
| [`store_context`](#2-store_context) | Memory | — | Persists session events and offloads large payloads to disk |
| [`recall_context`](#3-recall_context) | Memory | — | Recalls prior events matching a query with session affinity |
| [`ingest_source`](#4-ingest_source) | Ingestion | — | Ingests a new documentation tree into SQLite and FTS5 |
| [`refresh_source`](#5-refresh_source) | Ingestion | — | Re-indexes a source, replacing prior documents cleanly |
| [`query_reference`](#6-query_reference) | Retrieval | `get-library-docs`, `query-docs` | Queries chunks using hybrid intent-weighted ranking |
| [`resolve_reference`](#7-resolve_reference)| Retrieval | `resolve-library-id` | Resolves colloquial library queries to canonical identities |
| [`context_required`](#8-context_required) | Convergence | — | E2/E3 boundary tool for SWE_SEED work contracts |

---

## 2. Canonical Tool Specifications

### 1. `get_status`
Returns service operational statistics, database connectivity, and subsystem counters.

#### Request Arguments
```json
{}
```

#### Response Result Schema
```json
{
  "service": {
    "name": "context-kernel",
    "version": "0.1.0",
    "started_at": "2026-09-02T12:00:00Z",
    "healthy": true
  },
  "storage": {
    "kind": "sqlite",
    "path": "./data/kernel.db",
    "fts5_enabled": true
  },
  "sources": { "total": 1, "enabled": 1, "degraded": 0 },
  "sessions": { "active": 2 },
  "jobs": { "queued": 0, "running": 0 },
  "retrieval": {
    "default_mode": "hybrid",
    "semantic_enabled": false
  }
}
```

---

### 2. `store_context`
Stores an interaction event within a session. Large text or binary content can be offloaded to the filesystem artifact store via `artifact_ref`.

#### Request Arguments
```json
{
  "session_id": "sess-01",
  "kind": "decision",
  "text": "We chose Rust 2021 for the backend implementation",
  "metadata": { "ticket": "PROJ-123" },
  "artifact_ref": {
    "content": "base64-encoded-bytes-or-raw-utf8",
    "content_type": "text/plain"
  }
}
```

#### Response Result Schema
```json
{
  "event": {
    "event_id": "evt_01H...",
    "session_id": "sess-01",
    "kind": "decision",
    "stored_as": "inline",
    "artifact_id": null,
    "created_at": "2026-09-02T12:05:00Z"
  }
}
```

---

### 3. `recall_context`
Recalls prior session events matching a search query.

#### Request Arguments
```json
{
  "session_id": "sess-01",
  "query": "Rust backend decision",
  "top_k": 5,
  "scope": "current"
}
```
* `scope`: `"current"` (limits search to caller's session), `"recent"`, or `"all"`.

#### Response Result Schema
```json
{
  "events": [
    {
      "event_id": "evt_01H...",
      "session_id": "sess-01",
      "kind": "decision",
      "text": "We chose Rust 2021 for the backend implementation",
      "pinned": false,
      "created_at": "2026-09-02T12:05:00Z"
    }
  ],
  "total_hits": 1
}
```

---

### 4. `ingest_source`
Ingests an external documentation tree into a specified corpus.

#### Request Arguments
```json
{
  "corpus_id": "core_docs",
  "kind": "filesystem",
  "path": "/path/to/docs",
  "display_name": "Core Documentation"
}
```

#### Response Result Schema
```json
{
  "source_id": "src_01H...",
  "corpus_id": "core_docs",
  "documents_indexed": 12,
  "chunks_indexed": 48,
  "status": "active"
}
```

---

### 5. `refresh_source`
Re-indexes an existing source, purging old document chunks to prevent stale accumulation.

#### Request Arguments
```json
{
  "source_id": "src_01H..."
}
```

#### Response Result Schema
```json
{
  "job": {
    "job_id": "job_01H...",
    "source_id": "src_01H...",
    "status": "completed",
    "trigger_kind": "manual"
  }
}
```

---

### 6. `query_reference`
Executes intent-weighted hybrid retrieval over indexed documentation chunks.

#### Request Arguments
```json
{
  "query": "axum graceful shutdown",
  "intent": "query_docs",
  "top_k": 5,
  "source_scope": ["core_docs"]
}
```
* `intent`: `"query_docs"`, `"search_refs"`, `"resolve_library"`, `"general"`.

#### Response Result Schema
```json
{
  "query_id": "qry_01H...",
  "mode": "lexical",
  "matches": [
    {
      "chunk_id": "chk_01H...",
      "document_id": "doc_01H...",
      "title": "Axum Server Operations",
      "heading": "Graceful Shutdown",
      "uri": "docs/operations.md",
      "text": "... excerpt text ...",
      "score": 0.88,
      "score_breakdown": {
        "fts": 0.95,
        "freshness": 1.0,
        "trust": 0.0
      }
    }
  ],
  "total_hits": 1
}
```

---

### 7. `resolve_reference`
Resolves a colloquial library or framework name into a canonical library identity.

#### Request Arguments
```json
{
  "query": "tokio-rs",
  "kind": "library",
  "limit": 5
}
```

#### Response Result Schema
```json
{
  "candidates": [
    {
      "canonical_id": "lib_01H...",
      "provider": "crates.io",
      "name": "tokio",
      "version": "1.38.0",
      "summary": "An asynchronous runtime for the Rust programming language",
      "trust_score": 0.95,
      "score": 0.92
    }
  ],
  "query_id": "qry_01H..."
}
```

---

### 8. `context_required`
Fulfills the E2/E3 convergence boundary with SWE_SEED. Takes a canonical `ContextRequired` envelope and emits a `ContextPacketCreated` envelope.

#### Request Arguments
Must be a canonical v1 envelope stamped by `swe_seed`:
```json
{
  "schema_version": "1.0.0",
  "event_type": "ContextRequired",
  "source_agent": "swe_seed",
  "event_id": "evt_swe_01",
  "domain_model_hash": "a1b2c3...64-char-sha256...",
  "work_request_id": "wr-123",
  "context_requirement_id": "cr-456",
  "corpus_id": "core_docs",
  "query": "connection pooling",
  "max_results": 10
}
```

#### Response Result Schema
```json
{
  "schema_version": "1.0.0",
  "event_type": "ContextPacketCreated",
  "source_agent": "context-kernel",
  "event_id": "evt_ck_01",
  "caused_by": "evt_swe_01",
  "context_packet": {
    "context_packet_id": "ctx_01",
    "corpus_id": "core_docs",
    "outcome": "cited",
    "citations": [
      {
        "uri": "docs/db.md",
        "heading": "Connection Pooling",
        "excerpt": "..."
      }
    ]
  }
}
```

---

## 3. Error Handling & JSON-RPC Error Format

Context Kernel formats errors following the MCP standard:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": "invalid_arguments",
    "message": "invalid_arguments: unauthorized tool: store_context",
    "retryable": false,
    "details": {}
  }
}
```

Common error strings:
* `unknown tool: <name>`: Requested tool name does not exist.
* `invalid_arguments: unauthorized tool: <name>`: Blocked by `allow_tools`.
* `invalid_arguments: unauthorized corpus: <id>`: Blocked by `allow_corpora`.
* `auth_required`: Missing or invalid bearer token over HTTP SSE.
