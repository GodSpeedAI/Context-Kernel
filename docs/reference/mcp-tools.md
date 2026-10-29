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
| [`ingest_context_effects`](#9-ingest_context_effects) | Calibration | — | Records settlement-backed ranking effects for one pinned world |

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
  "filters": {"source_scope": ["core_docs"]}
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
Fulfills the E2/E3 convergence boundary with SWE_SEED. Takes a canonical `ContextRequired` envelope and emits a CEP `context_bundle` envelope.

#### Request Arguments
Must be a canonical v1 envelope stamped by `swe-seed` (or `swe_seed`). Identity and correlation fields live in `payload`; `required` (default `true`) sits beside the envelope:
```json
{
  "schema_version": "v1",
  "event_id": "evt_swe_01",
  "source_agent": "swe-seed",
  "event_type": "ContextRequired",
  "occurred_at": "2026-10-05T00:00:00Z",
  "payload": {
    "domain_model_hash": "<64 lowercase hex>",
    "world_ref": "world:acme@sha256:<64 lowercase hex>",
    "work_request_id": "wr-123",
    "context_requirement_id": "cr-456",
    "corpus_id": "core_docs",
    "query": "connection pooling",
    "max_results": 10,
    "required_concepts": ["pool exhaustion"],
    "scope_claim": "core_docs",
    "is_private": false,
    "exclusions": [],
    "freshness_max_age_secs": 86400,
    "token_budget": 1500,
    "expected_refs": [],
    "correlation_id": "corr-789"
  },
  "required": true
}
```
`world_ref` is required and must be well formed; an alias such as `world:acme` is rejected before any retrieval. The handler rejects a wrong `schema_version`, a producer other than `swe_seed`, an empty `event_id`, a placeholder or malformed `domain_model_hash`, and missing correlation ids with `invalid_arguments`. A caller-declared private corpus is never auto-ingested: it must already be registered, otherwise the result is `corpus_not_found`.

#### Response Result Schema
```json
{
  "context_bundle": {
    "envelope_id": "env-ctx_<ulid>",
    "cep_version": "1.0.0",
    "envelope_version": "1.0",
    "envelope_kind": "context_bundle",
    "created_at": "2026-10-24T09:14:02+00:00",
    "created_by": "context-kernel",
    "scope": {
      "world_ref": "world:acme@sha256:<64 lowercase hex>",
      "evaluation_context": "context-kernel:core_docs"
    },
    "boundary_record": {
      "scope": "context for wr-123",
      "included_sections": ["references", "omissions"],
      "excluded_sections": [],
      "known_omissions": ["max_results_reached"],
      "unknowns": [],
      "redactions": [],
      "compression_notes": [],
      "out_of_scope_entities": [],
      "limitations": []
    },
    "completeness_status": "partial",
    "omission_status": "marked",
    "provenance_refs": ["prov-ctx_<ulid>"],
    "validation_status": "valid",
    "lineage_refs": [],
    "provenance": [
      {
        "provenance_id": "prov-ctx_<ulid>",
        "source_system_refs": ["corpus:core_docs", "source:ctxfs:core_docs"],
        "producer_refs": ["context-kernel"],
        "production_method": "retrieval",
        "created_at": "2026-10-24T09:14:02+00:00"
      }
    ],
    "extensions": {
      "cep.profile": {"profile_id": "godspeed.context_bundle", "profile_version": "1.0.0"},
      "godspeed.context_bundle": {
        "work_request_ref": "wr-123",
        "retrieval_completeness": "partial",
        "context_packet_id": "ctx_<ulid>",
        "context_requirement_id": "cr-456",
        "domain_model_hash": "<64 lowercase hex>",
        "retrieval_policy": "ck-app.resolve_context/v1",
        "query_mode": "hybrid",
        "caused_by": "evt_swe_01",
        "correlation_id": "corr-789",
        "budget": {"requested_results": 10, "consumed_results": 10, "requested_tokens": 1500, "consumed_tokens": 1490},
        "evaluations": [{"ref_id": "ref-chk_01", "score": 0.71, "score_breakdown": {"fts": 0.71}}]
      }
    },
    "references": [
      {
        "ref_id": "ref-chk_01",
        "ref_type": "chunk",
        "target_uri_or_id": "docs/db.md#connection-pooling",
        "availability_status": "available",
        "integrity_status": "verifiable",
        "target_version": "v3",
        "target_hash": "sha256:<64 lowercase hex>",
        "scope": "core_docs",
        "limitations": ["content_truncated"]
      }
    ],
    "omissions": [
      {
        "omission_id": "om-1",
        "omission_type": "max_results_reached",
        "reason": "max_results_reached",
        "description": "requested 10 of 14 candidates"
      }
    ],
    "integrity": {
      "content_hash": "<64 lowercase hex>",
      "verification_method": "sha256-canonical-json-v1",
      "verification_status": "verified",
      "tamper_status": "intact"
    }
  },
  "context_envelope": {"...": "DEPRECATED legacy ContextPacketCreated envelope"},
  "context_packet": {"...": "DEPRECATED packet payload, also embedded in the legacy envelope"},
  "outcome": "cited",
  "reason": null,
  "access": {"result": "accessible"}
}
```

`outcome` is `cited`, `no_context_governed` (required, zero references) or `empty_optional`. `reason` is `null` when cited; otherwise one of `corpus_root_unconfigured`, `corpus_not_found`, `no_matching_content`, `corpus_unavailable`, `corpus_scope_unresolved`, `corpus_excluded`, `budget_exhausted`, `no_fresh_content`. The reason is always appended to `boundary_record.known_omissions` and to `omissions`, so a minimal bundle still explains itself.

`completeness_status` is `complete_for_context` (references, no omissions), `partial` (references plus omissions) or `minimal` (no references, questions plus omissions). The projection refuses unexplained incompleteness and a `complete` bundle with omissions.

`access` reports context access only, never action authority:
- `{"result": "accessible"}`
- `{"result": "inaccessible", "corpus_id": "...", "reason": "excluded" | "out_of_scope"}`
- `{"result": "requires_scope", "corpus_id": "..."}`

Omission codes are `max_results_reached`, `token_budget_exhausted`, `stale_source_excluded`, `required_distinction_absent`, and `citation_content_truncated` (legacy path only). `integrity.content_hash` is SHA-256 over the canonical JSON of the envelope with the `integrity` block removed: keys sorted, compact separators, floats normalized to six decimals, negative zero flattened to `0.0`.

---

### 9. `ingest_context_effects`
Recalibrates future retrieval for one pinned semantic world. This is prospective ranking calibration, not developmental memory: historical bundles are never rewritten, and a bare self-report is rejected.

#### Request Arguments
```json
{
  "effects": [
    {
      "effect_id": "eff_01",
      "recorded_at": "2026-10-24T09:20:00Z",
      "kind": "source_contradicted",
      "world_ref": "world:acme@sha256:<64 lowercase hex>",
      "work_request_ref": "wr-123",
      "bundle_envelope_id": "env-ctx_<ulid>",
      "bundle_content_hash": "sha256:<64 lowercase hex>",
      "affected_sources": ["ctxfs:core_docs"],
      "evidence_ref": "settlement:stl_42",
      "detail": "Settlement contradicted the pinned retention claim."
    }
  ]
}
```
`kind` is one of `required_distinction_absent`, `source_contradicted`, `source_stale`, `distinction_material_to_settlement`, `context_item_unused`, `excessive_retrieval_burden`, `omission_consequential`. Only `source_contradicted` (0.20) and `source_stale` (0.10) demote scores; penalties accumulate per source and cap at 0.50.

`evidence_ref` is required and must be `<kind>:<value>` with kind in `settlement`, `evidence`, `execution`, `observation`, `proof`. `world_ref` must be pinned and `bundle_content_hash` must be `sha256:<64 lowercase hex>`; anything else is rejected. The handler rejects an empty `effects` array.

#### Response Result Schema
```json
{
  "accepted": 1,
  "skipped_duplicates": 0,
  "rejected": 0
}
```
Ingest is idempotent by `effect_id`: a repeated effect is skipped, never overwritten. The ledger is append-only per world, so unreadable or unavailable storage surfaces as an internal error rather than silently dropping calibration.

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
