Here’s a phased implementation plan with crate structure, SQLite schema, MCP JSON contracts, and milestone order. I’m grounding the compatibility layer in the publicly documented upstream tool surfaces and behaviors: Context7’s MCP tools are `resolve-library-id` and `get-library-docs`/`query-docs`, with stdio and remote HTTP transport options; Context Mode centers on SQLite + FTS5/BM25 session continuity and context saving; Ref MCP centers on token-efficient documentation search and URL/document reading across public and private sources. ([github.com](https://github.com/mcp/io.github.upstash/context7?utm_source=openai))

## 1. Delivery strategy

Build **Context Kernal** in 4 major phases:

1. **Core local MVP**
   - single Rust binary
   - stdio MCP
   - SQLite + FTS5
   - source ingestion from local filesystem + static dataset adapter
   - lexical retrieval
   - context persistence

2. **Unified retrieval + compatibility**
   - Context7-compatible library resolution and docs query aliases
   - Ref-style documentation search/read flows
   - hybrid ranker
   - citation/excerpt shaping
   - stable JSON contracts

3. **Remote/internal deployment**
   - SSE transport over HTTP
   - auth
   - OTEL hooks
   - refresh scheduler/jobs
   - GitHub/web adapters

4. **Semantic + production hardening**
   - optional embeddings/vector path
   - better refresh/incremental indexing
   - richer policy/retention/authz
   - performance tuning
   - import/export and CI ergonomics

## 2. Recommended Rust workspace structure

Use one Cargo workspace with multiple crates, even if you ultimately publish a single binary.

```text
context-kernal/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   ├── ck-core/
│   ├── ck-config/
│   ├── ck-app/
│   ├── ck-storage-sqlite/
│   ├── ck-search-fts/
│   ├── ck-ranker/
│   ├── ck-memory/
│   ├── ck-artifacts/
│   ├── ck-mcp/
│   ├── ck-transport-stdio/
│   ├── ck-transport-sse/
│   ├── ck-auth/
│   ├── ck-observability/
│   ├── ck-scheduler/
│   ├── ck-adapter-filesystem/
│   ├── ck-adapter-context7-dataset/
│   ├── ck-adapter-github/
│   ├── ck-adapter-web/
│   ├── ck-adapter-pdf/
│   ├── ck-embeddings/
│   ├── ck-vector/
│   └── ck-bin/
└── docs/
    ├── SPEC.md
    ├── schema.md
    └── mcp-contracts.md
```

## 3. Crate responsibilities

### `ck-core`
Pure domain types and traits.

Contents:
- entities: `Source`, `Corpus`, `LibraryIdentity`, `Document`, `Chunk`, `Session`, `SessionEvent`, `Artifact`, `RefreshJob`
- enums: retrieval mode, trust mode, session mode, job status
- ports/traits:
  - `ConfigPort`
  - `SourceAdapter`
  - `Normalizer`
  - `Chunker`
  - `StoragePort`
  - `LexicalSearchPort`
  - `EmbeddingPort`
  - `VectorSearchPort`
  - `RankerPort`
  - `SessionMemoryPort`
  - `ArtifactPort`
  - `AuthPort`
  - `TelemetryPort`

Rule:
- no Tokio
- no SQL
- no HTTP
- no serde_json-dependent business logic beyond DTO definitions if possible

### `ck-config`
- TOML parsing
- env var interpolation
- defaults
- live reload/watch
- typed config structs

### `ck-app`
Application services/use cases:
- `ResolveReference`
- `QueryReference`
- `StoreContext`
- `RecallContext`
- `IngestSource`
- `RefreshSource`
- `GetStatus`

This crate composes ports and owns use-case orchestration.

### `ck-storage-sqlite`
Implements:
- `StoragePort`
- `SessionMemoryPort`
- metadata persistence for `ArtifactPort`
- migrations

### `ck-search-fts`
Implements `LexicalSearchPort` using SQLite FTS5/BM25.

### `ck-ranker`
Implements:
- lexical-only ranker
- hybrid ranker
- query intent routing
- score breakdowns

### `ck-memory`
Domain helpers around context continuity:
- session retention policy
- event typing
- pinned facts
- session-local affinity scoring

### `ck-artifacts`
Filesystem artifact payload storage:
- blob path layout
- checksum
- truncation/excerpt extraction
- summary hooks

### `ck-mcp`
Protocol-facing tool/resource definitions independent of transport:
- MCP tool registration
- request/response DTOs
- compatibility aliases
- MCP error mapping

### `ck-transport-stdio`
- stdio framing
- request loop
- graceful shutdown

### `ck-transport-sse`
- HTTP server
- SSE stream handling
- auth middleware
- health/readiness endpoints

### `ck-auth`
- bearer token validation
- local trust mode
- corpus/tool authorization hooks

### `ck-observability`
- tracing subscriber setup
- OTEL export
- structured log field conventions
- metrics instruments

### `ck-scheduler`
- refresh job queue
- periodic jobs
- debouncing/coalescing
- optional honker-backed adapter later

### source adapters
Each adapter implements `SourceAdapter` and possibly helper normalizers:
- `ck-adapter-filesystem`
- `ck-adapter-context7-dataset`
- `ck-adapter-github`
- `ck-adapter-web`
- `ck-adapter-pdf`

### optional semantic crates
- `ck-embeddings`
- `ck-vector`

### `ck-bin`
Single binary:
- CLI parsing
- dependency wiring
- runtime bootstrap

## 4. Recommended dependency boundaries

Keep these strict:

- `ck-core`: no dependency on other internal crates
- `ck-app`: depends on `ck-core`
- adapters depend on `ck-core`, maybe `ck-app` DTOs sparingly
- `ck-bin` depends on everything, wires concrete implementations

Avoid:
- MCP logic inside storage crate
- SQL logic inside app crate
- transport code inside ranker or source adapters

## 5. Suggested CLI surface

```text
context-kernal serve [--config path]
context-kernal index <source-id> [--full]
context-kernal refresh <source-id>
context-kernal migrate
context-kernal doctor
context-kernal export-schema
```

This is useful even if MCP remains primary. Public Context7 issues indicate CLI demand exists for agent workflows, even though its current public tooling is MCP-centric. ([github.com](https://github.com/upstash/context7/issues/1788?utm_source=openai))

## 6. SQLite schema, v1

Use SQLite as system-of-record. Context Mode publicly emphasizes SQLite and FTS5 for session continuity, and your design calls for SQLite baseline storage. ([github.com](https://github.com/mksglu/context-mode/blob/main/README.md?utm_source=openai))

### 6.1 Core tables

```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
PRAGMA synchronous=NORMAL;
```

### `schema_migrations`

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
```

### `sources`

```sql
CREATE TABLE IF NOT EXISTS sources (
  source_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  display_name TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  adapter_config_json TEXT NOT NULL,
  auth_ref TEXT,
  first_indexed_at TEXT,
  last_refresh_attempt_at TEXT,
  last_refresh_success_at TEXT,
  last_refresh_error TEXT,
  freshness_ttl_secs INTEGER,
  upstream_revision TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### `corpora`

```sql
CREATE TABLE IF NOT EXISTS corpora (
  corpus_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  visibility TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### `corpus_sources`

```sql
CREATE TABLE IF NOT EXISTS corpus_sources (
  corpus_id TEXT NOT NULL,
  source_id TEXT NOT NULL,
  PRIMARY KEY (corpus_id, source_id),
  FOREIGN KEY (corpus_id) REFERENCES corpora(corpus_id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES sources(source_id) ON DELETE CASCADE
);
```

### `library_identities`

```sql
CREATE TABLE IF NOT EXISTS library_identities (
  library_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  name TEXT NOT NULL,
  version TEXT,
  namespace TEXT,
  language TEXT,
  summary TEXT,
  trust_score REAL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### `library_aliases`

```sql
CREATE TABLE IF NOT EXISTS library_aliases (
  alias TEXT NOT NULL,
  library_id TEXT NOT NULL,
  alias_kind TEXT NOT NULL DEFAULT 'general',
  PRIMARY KEY (alias, library_id),
  FOREIGN KEY (library_id) REFERENCES library_identities(library_id) ON DELETE CASCADE
);
```

### `documents`

```sql
CREATE TABLE IF NOT EXISTS documents (
  document_id TEXT PRIMARY KEY,
  corpus_id TEXT NOT NULL,
  source_id TEXT NOT NULL,
  external_id TEXT,
  library_id TEXT,
  title TEXT NOT NULL,
  uri TEXT,
  version TEXT,
  mime_type TEXT,
  content_text TEXT NOT NULL,
  metadata_json TEXT NOT NULL,
  checksum TEXT NOT NULL,
  superseded_by_document_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (corpus_id) REFERENCES corpora(corpus_id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES sources(source_id) ON DELETE CASCADE,
  FOREIGN KEY (library_id) REFERENCES library_identities(library_id) ON DELETE SET NULL
);
```

Indexes:

```sql
CREATE INDEX IF NOT EXISTS idx_documents_source_id ON documents(source_id);
CREATE INDEX IF NOT EXISTS idx_documents_corpus_id ON documents(corpus_id);
CREATE INDEX IF NOT EXISTS idx_documents_library_id ON documents(library_id);
CREATE INDEX IF NOT EXISTS idx_documents_uri ON documents(uri);
CREATE INDEX IF NOT EXISTS idx_documents_checksum ON documents(checksum);
```

### `chunks`

```sql
CREATE TABLE IF NOT EXISTS chunks (
  chunk_id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  corpus_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  text TEXT NOT NULL,
  token_count_estimate INTEGER NOT NULL,
  heading TEXT,
  anchor TEXT,
  section_path_json TEXT NOT NULL DEFAULT '[]',
  metadata_json TEXT NOT NULL DEFAULT '{}',
  checksum TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (document_id) REFERENCES documents(document_id) ON DELETE CASCADE,
  FOREIGN KEY (corpus_id) REFERENCES corpora(corpus_id) ON DELETE CASCADE
);
```

Indexes:

```sql
CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_chunks_corpus_id ON chunks(corpus_id);
CREATE INDEX IF NOT EXISTS idx_chunks_ordinal ON chunks(document_id, ordinal);
CREATE INDEX IF NOT EXISTS idx_chunks_checksum ON chunks(checksum);
```

### `sessions`

```sql
CREATE TABLE IF NOT EXISTS sessions (
  session_id TEXT PRIMARY KEY,
  client_id TEXT,
  transport TEXT NOT NULL,
  continuation_mode TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
```

### `session_events`

```sql
CREATE TABLE IF NOT EXISTS session_events (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  text TEXT NOT NULL,
  artifact_id TEXT,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  pinned INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE,
  FOREIGN KEY (artifact_id) REFERENCES artifacts(artifact_id) ON DELETE SET NULL
);
```

Indexes:

```sql
CREATE INDEX IF NOT EXISTS idx_session_events_session_id ON session_events(session_id);
CREATE INDEX IF NOT EXISTS idx_session_events_kind ON session_events(kind);
CREATE INDEX IF NOT EXISTS idx_session_events_created_at ON session_events(created_at);
```

### `artifacts`

```sql
CREATE TABLE IF NOT EXISTS artifacts (
  artifact_id TEXT PRIMARY KEY,
  session_id TEXT,
  kind TEXT NOT NULL,
  mime_type TEXT,
  storage_uri TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  summary TEXT,
  checksum TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE SET NULL
);
```

### `refresh_jobs`

```sql
CREATE TABLE IF NOT EXISTS refresh_jobs (
  job_id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  trigger_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT,
  error TEXT,
  stats_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (source_id) REFERENCES sources(source_id) ON DELETE CASCADE
);
```

Indexes:

```sql
CREATE INDEX IF NOT EXISTS idx_refresh_jobs_source_id ON refresh_jobs(source_id);
CREATE INDEX IF NOT EXISTS idx_refresh_jobs_status ON refresh_jobs(status);
CREATE INDEX IF NOT EXISTS idx_refresh_jobs_created_at ON refresh_jobs(created_at);
```

## 7. FTS5 virtual tables

### `chunks_fts`

Use external-content FTS if you want smaller duplication management, or direct content for simplicity. For v1, prefer simplicity.

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
  chunk_id UNINDEXED,
  document_id UNINDEXED,
  corpus_id UNINDEXED,
  library_id UNINDEXED,
  title,
  heading,
  text,
  tokenize = 'unicode61'
);
```

### `session_events_fts`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS session_events_fts USING fts5(
  event_id UNINDEXED,
  session_id UNINDEXED,
  kind,
  text,
  tokenize = 'unicode61'
);
```

### `library_aliases_fts`

```sql
CREATE VIRTUAL TABLE IF NOT EXISTS library_aliases_fts USING fts5(
  alias,
  library_id UNINDEXED,
  name,
  provider,
  tokenize = 'unicode61'
);
```

## 8. Optional vector schema

If embeddings enabled later:

### `embedding_models`

```sql
CREATE TABLE IF NOT EXISTS embedding_models (
  model_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  dimensions INTEGER NOT NULL,
  created_at TEXT NOT NULL
);
```

### `chunk_embeddings`

```sql
CREATE TABLE IF NOT EXISTS chunk_embeddings (
  chunk_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  vector_blob BLOB NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (chunk_id, model_id),
  FOREIGN KEY (chunk_id) REFERENCES chunks(chunk_id) ON DELETE CASCADE,
  FOREIGN KEY (model_id) REFERENCES embedding_models(model_id) ON DELETE CASCADE
);
```

Same for `session_event_embeddings` if needed.

For v1, brute-force cosine over small corpora is acceptable behind a `VectorSearchPort`. Later you can swap in sqlite-vec, LanceDB, Qdrant, or something else without domain changes.

## 9. JSON metadata conventions

Store semi-structured fields as JSON text:
- `adapter_config_json`
- `metadata_json`
- `section_path_json`
- `stats_json`

Recommended internal JSON schema conventions:
- lowercase snake_case keys
- small predictable objects
- avoid storing huge blobs inline if artifact store is better

## 10. Artifact filesystem layout

```text
<data_dir>/
  artifacts/
    sha256/
      ab/
        cd/
          <artifact_id>.bin
  exports/
  logs/
```

Rules:
- path from checksum/artifact ID
- never trust user-supplied filenames
- SQLite holds the lookup metadata

## 11. MCP JSON contracts

Assume MCP-compliant envelope outside these tool payloads. The tool arguments/results below are your stable contracts.

## 12. Required MCP tools and aliases

For compatibility:
- canonical:
  - `resolve_reference`
  - `query_reference`
  - `store_context`
  - `recall_context`
  - `ingest_source`
  - `refresh_source`
  - `get_status`
- aliases:
  - `resolve-library-id` → `resolve_reference`
  - `get-library-docs` / `query-docs` → `query_reference`

Context7’s publicly documented tools include `resolve-library-id` and `get-library-docs`, and public issue text also references `query-docs`, so supporting aliases is sensible for compatibility. ([github.com](https://github.com/mcp/io.github.upstash/context7?utm_source=openai))

## 13. Tool contract: `resolve_reference`

### Request

```json
{
  "name": "resolve_reference",
  "arguments": {
    "query": "react router",
    "kind": "library",
    "version": null,
    "source_scope": ["context7_public", "private_docs"],
    "limit": 10
  }
}
```

### Response

```json
{
  "candidates": [
    {
      "canonical_id": "/remix-run/react-router",
      "provider": "context7",
      "name": "React Router",
      "version": null,
      "aliases": ["react-router", "react router"],
      "summary": "Declarative routing for React applications.",
      "trust_score": 0.93,
      "score": 0.9821,
      "score_breakdown": {
        "alias_exact": 1.0,
        "provider_prior": 0.1,
        "popularity_prior": 0.05
      }
    }
  ],
  "query_id": "qry_01J...",
  "mode": "resolve"
}
```

### Compatibility alias input mapping
If invoked as `resolve-library-id`:
- `libraryName` maps to `query`
- implied `kind = "library"`

## 14. Tool contract: `query_reference`

### Request

```json
{
  "name": "query_reference",
  "arguments": {
    "canonical_id": "/vercel/next.js",
    "query": "app router caching and revalidation",
    "mode": "hybrid",
    "top_k": 8,
    "version": null,
    "filters": {
      "lang": "en",
      "doc_type": ["guide", "api"]
    },
    "budget": {
      "max_tokens": 4000,
      "max_chunks": 8
    }
  }
}
```

### Response

```json
{
  "results": [
    {
      "result_id": "res_01J...",
      "canonical_id": "/vercel/next.js",
      "document_id": "doc_01J...",
      "chunk_id": "chk_01J...",
      "title": "Caching and Revalidating",
      "heading": "Time-based Revalidation",
      "excerpt": "You can set a cache lifetime and revalidate data after the specified duration...",
      "citation": {
        "uri": "https://nextjs.org/docs/app/building-your-application/data-fetching/caching",
        "anchor": "time-based-revalidation"
      },
      "score": 0.913,
      "score_breakdown": {
        "fts": 12.8,
        "freshness": 0.2,
        "semantic": 0.0
      },
      "metadata": {
        "source_id": "context7_public",
        "corpus_id": "library_docs",
        "version": null
      }
    }
  ],
  "query_id": "qry_01J...",
  "mode": "hybrid",
  "truncated": false
}
```

### Compatibility alias mapping
If invoked as `get-library-docs` or `query-docs`:
- `context7CompatibleLibraryID` maps to `canonical_id`
- `topic` maps to `query`
- `page` maps to internal pagination/offset hint if used

## 15. Tool contract: `store_context`

### Request

```json
{
  "name": "store_context",
  "arguments": {
    "session_id": "ses_01J...",
    "kind": "decision",
    "text": "We decided to use axum for HTTP and sqlx for SQLite access.",
    "metadata": {
      "project": "context-kernal",
      "importance": "high",
      "tags": ["architecture", "decision"]
    }
  }
}
```

### Response

```json
{
  "event": {
    "event_id": "evt_01J...",
    "session_id": "ses_01J...",
    "kind": "decision",
    "stored_as": "event",
    "artifact_id": null,
    "created_at": "2026-06-01T12:00:00Z"
  }
}
```

### Large artifact variant request

```json
{
  "name": "store_context",
  "arguments": {
    "session_id": "ses_01J...",
    "kind": "tool_result",
    "artifact_ref": {
      "content_type": "text/plain",
      "content": "very large payload..."
    },
    "metadata": {
      "source_tool": "playwright_snapshot"
    }
  }
}
```

### Response

```json
{
  "event": {
    "event_id": "evt_01J...",
    "session_id": "ses_01J...",
    "kind": "tool_result",
    "stored_as": "artifact",
    "artifact_id": "art_01J...",
    "created_at": "2026-06-01T12:00:00Z"
  }
}
```

## 16. Tool contract: `recall_context`

### Request

```json
{
  "name": "recall_context",
  "arguments": {
    "session_id": "ses_01J...",
    "query": "what database and web framework did we choose?",
    "scope": "current",
    "top_k": 5,
    "mode": "hybrid"
  }
}
```

### Response

```json
{
  "results": [
    {
      "event_id": "evt_01J...",
      "kind": "decision",
      "text": "We decided to use axum for HTTP and sqlx for SQLite access.",
      "artifact_id": null,
      "score": 0.952,
      "score_breakdown": {
        "fts": 9.2,
        "session_affinity": 0.4
      },
      "created_at": "2026-06-01T12:00:00Z"
    }
  ],
  "query_id": "qry_01J..."
}
```

## 17. Tool contract: `ingest_source`

### Request

```json
{
  "name": "ingest_source",
  "arguments": {
    "source": {
      "source_id": "local_docs",
      "kind": "filesystem_tree",
      "path": "./docs"
    },
    "mode": "upsert"
  }
}
```

### Response

```json
{
  "job": {
    "job_id": "job_01J...",
    "source_id": "local_docs",
    "status": "queued",
    "trigger": "manual"
  }
}
```

## 18. Tool contract: `refresh_source`

### Request

```json
{
  "name": "refresh_source",
  "arguments": {
    "source_id": "context7_public"
  }
}
```

### Response

```json
{
  "job": {
    "job_id": "job_01J...",
    "source_id": "context7_public",
    "status": "queued",
    "trigger": "manual"
  }
}
```

## 19. Tool contract: `get_status`

### Request

```json
{
  "name": "get_status",
  "arguments": {}
}
```

### Response

```json
{
  "service": {
    "name": "context-kernal",
    "version": "0.1.0",
    "started_at": "2026-06-01T11:45:00Z",
    "healthy": true
  },
  "storage": {
    "kind": "sqlite",
    "path": "/home/user/.local/share/context-kernal/kernel.db",
    "fts5_enabled": true
  },
  "sources": {
    "total": 4,
    "enabled": 3,
    "degraded": 1
  },
  "sessions": {
    "active": 2
  },
  "jobs": {
    "queued": 1,
    "running": 0
  },
  "retrieval": {
    "default_mode": "hybrid",
    "semantic_enabled": false
  }
}
```

## 20. MCP error contract

Use stable structured errors:

```json
{
  "error": {
    "code": "source_not_found",
    "message": "No source exists with id 'foo'",
    "retryable": false,
    "details": {}
  }
}
```

Recommended codes:
- `invalid_arguments`
- `source_not_found`
- `canonical_id_not_found`
- `query_failed`
- `indexing_failed`
- `storage_unavailable`
- `artifact_too_large`
- `auth_required`
- `forbidden`
- `semantic_unavailable`

## 21. Milestone order

## Milestone 0 — Repo bootstrap
Goal: compile a skeleton.

Deliverables:
- Cargo workspace
- core domain types
- config parsing
- tracing/logging setup
- migrations framework
- empty stdio MCP server with `get_status`

Acceptance:
- `context-kernal serve` starts
- stdio MCP responds to `get_status`
- SQLite file created and migrated

## Milestone 1 — Storage + memory foundation
Goal: persistent local substrate.

Deliverables:
- SQLite schema v1
- storage repository layer
- sessions/events/artifacts persistence
- basic artifact filesystem store
- `store_context`
- `recall_context` using FTS5 over session events

Acceptance:
- can persist session events
- can store large payload as artifact + summary
- recall returns BM25-ranked events

This milestone directly captures the core continuity model that Context Mode publicly describes with SQLite and FTS5/BM25. ([github.com](https://github.com/mksglu/context-mode/blob/main/README.md?utm_source=openai))

## Milestone 2 — Local document ingestion
Goal: useful offline docs index.

Deliverables:
- filesystem source adapter
- document normalizer
- chunker preserving headings/code fences
- `documents`, `chunks`, `chunks_fts`
- `ingest_source`
- `query_reference` lexical mode

Acceptance:
- ingest local docs tree
- retrieve relevant chunks with citations
- chunk updates replace previous chunks cleanly

## Milestone 3 — Unified identity layer
Goal: library/reference resolution.

Deliverables:
- `library_identities`, aliases, alias FTS
- `resolve_reference`
- compatibility aliases:
  - `resolve-library-id`
  - `get-library-docs`/`query-docs`
- source/corpus filtering
- query intent router

Acceptance:
- given “next.js”, can resolve a canonical ID
- can query docs by canonical ID or free-text
- compatibility alias requests succeed

This matches Context7’s publicly documented library-resolution and docs-query outcome surface. ([github.com](https://github.com/mcp/io.github.upstash/context7?utm_source=openai))

## Milestone 4 — Ref-style search/read unification
Goal: one-stop search across sources.

Deliverables:
- generic reference search semantics
- URL/document read abstraction
- public/private source scope filters
- better excerpt shaping and citations
- ranker with lexical + trust + freshness

Acceptance:
- public docs and private/local docs can be searched in one query
- response is token-efficient and cited

This aligns with Ref MCP’s public positioning as a documentation search/read server for public web/GitHub and private resources. ([github.com](https://github.com/ref-tools/ref-tools-mcp?utm_source=openai))

## Milestone 5 — Scheduler and refresh
Goal: freshness and operations.

Deliverables:
- refresh jobs table + queue
- periodic/manual refresh
- stale detection
- refresh coalescing
- source health/degraded state

Acceptance:
- source can be refreshed on interval or command
- failed refresh does not erase old index
- service reports degraded source state

## Milestone 6 — SSE remote serving
Goal: internal remote deployment.

Deliverables:
- HTTP server
- SSE MCP transport
- bearer auth
- `/healthz`, `/readyz`
- optional `/api/v1/status`

Acceptance:
- remote MCP works over SSE
- unauthorized requests are rejected
- stdio and SSE can coexist

Public Context7 registry docs describe remote HTTP transport, and Ref MCP publicly recommends streamable HTTP, so remote serving is a natural compatibility target for your unified server. ([github.com](https://github.com/mcp/io.github.upstash/context7?utm_source=openai))

## Milestone 7 — Hybrid ranking
Goal: better answer quality without semantic dependency.

Deliverables:
- score breakdown model
- freshness/trust/session affinity priors
- query intent weighting
- better de-duplication and result collapsing

Acceptance:
- hybrid mode outperforms plain lexical on eval set
- ranking explanations available in debug mode

## Milestone 8 — Optional embeddings
Goal: semantic enhancement.

Deliverables:
- embedding port
- pluggable provider config
- vector store adapter
- semantic and hybrid-with-semantic retrieval
- fallback when unavailable

Acceptance:
- semantic mode works when configured
- service degrades gracefully when embedding provider is absent

## Milestone 9 — GitHub/web adapters
Goal: parity with broader reference retrieval.

Deliverables:
- GitHub docs/repo adapter
- website fetch adapter
- SSRF protections
- incremental refresh hints (etag, sha, last-modified)

Acceptance:
- remote docs sources can be indexed and refreshed safely

## Milestone 10 — Hardening and packaging
Goal: operationally usable binary.

Deliverables:
- config reload
- retention GC
- authz by corpus/tool
- structured OTEL spans/metrics
- release builds and install docs
- CI integration examples

Acceptance:
- single-binary install
- stable config and migrations
- operational telemetry available

## 22. Recommended implementation order inside milestones

Within each milestone:
1. define trait/DTO in `ck-core`
2. implement use-case wiring in `ck-app`
3. implement adapter
4. add integration test
5. expose MCP tool/CLI wrapper

This preserves ports-and-adapters discipline.

## 23. Testing strategy by layer

### Unit tests
- `ck-core`, `ck-ranker`, chunker, config

### Integration tests
- sqlite migrations
- FTS queries
- source ingestion end-to-end
- MCP tool request/response

### golden tests
- chunking snapshots
- retrieval result shaping
- MCP JSON outputs

### compatibility tests
- Context7 alias input/output
- context memory continuity scenarios
- ref-style public/private search scenarios

## 24. Performance targets for v1

Initial practical targets:
- startup under 500 ms without heavy refresh
- query_reference p50 under 150 ms on warm local index for modest corpus
- recall_context p50 under 50 ms
- ingest throughput acceptable for markdown/docs-first workloads

These are engineering targets, not normative requirements.

## 25. Suggested Rust libraries

Non-binding recommendations:
- CLI: `clap`
- async runtime: `tokio`
- serialization: `serde`, `serde_json`, `toml`
- DB: `sqlx` or `rusqlite`
- tracing: `tracing`, `tracing-subscriber`, OTEL crates
- HTTP/SSE: `axum`
- IDs: `ulid` or `uuid`
- hashing: `sha2`
- markdown/html parsing: `pulldown-cmark`, `scraper`, `html5ever` as needed

My recommendation:
- **`rusqlite`** if you want tight SQLite/FTS5 control early
- **`sqlx`** if you want more async consistency and type-checked queries
- Either works; FTS5 ergonomics may be simpler with `rusqlite`

## 26. Suggested first two weeks

### Week 1
- workspace bootstrap
- config crate
- core traits/types
- sqlite migrations
- stdio MCP with `get_status`
- sessions/events/artifacts tables

### Week 2
- `store_context`
- `recall_context`
- FTS5 integration
- filesystem source adapter
- document/chunk indexing
- lexical `query_reference`

By the end of week 2, you already have a usable local system.

## 27. Biggest design risks to manage

1. **Over-coupling compatibility aliases to internals**
   - keep aliases in `ck-mcp`, not `ck-app`

2. **Letting adapters leak assumptions**
   - keep query semantics in ranker/app layer

3. **Overbuilding semantic search too early**
   - lexical + hybrid priors first

4. **Mixing session memory and docs retrieval too loosely**
   - keep separate corpora with explicit session-affinity scoring

5. **Remote fetch security**
   - plan SSRF restrictions before shipping web adapter

## 28. Recommended next artifact

The best next artifact is a **repo bootstrap package** containing:

- `Cargo.toml` workspace
- crate manifests
- migration SQL files
- `context-kernal.toml.example`
- `docs/mcp-contracts.md`
- `docs/schema.md`
- a `ck-bin` skeleton with `serve` and `migrate`

If you want, I can produce that next as:
1. a **concrete repo tree with starter files**, or
2. the **actual SQL migration files + Rust trait definitions + MCP DTO structs**.