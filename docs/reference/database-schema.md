# Reference: SQLite Database Schema (DDL)

This document provides the complete SQLite schema definition installed by `SCHEMA_V1` in `ck-storage-sqlite`, along with index definitions, virtual tables, foreign key relationships, and domain enum string representations.

---

## 1. Global PRAGMA Settings

```sql
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;
PRAGMA synchronous = NORMAL;
```

---

## 2. Table Definitions

### `schema_migrations`
Tracks installed schema version numbers.
```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);
```

### `sources`
Configured origins of external documentation.
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
Logical retrieval and access-control boundaries.
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
Many-to-many junction linking sources to corpora.
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
Canonical framework and package descriptions.
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
Alternative search terms and abbreviations mapped to libraries.
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
Ingested documentation resources.
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

CREATE INDEX IF NOT EXISTS idx_documents_source_id ON documents(source_id);
CREATE INDEX IF NOT EXISTS idx_documents_corpus_id ON documents(corpus_id);
CREATE INDEX IF NOT EXISTS idx_documents_library_id ON documents(library_id);
CREATE INDEX IF NOT EXISTS idx_documents_uri ON documents(uri);
CREATE INDEX IF NOT EXISTS idx_documents_checksum ON documents(checksum);
```

### `chunks`
Structural text extracts (~800 words max) used for retrieval.
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

CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_chunks_corpus_id ON chunks(corpus_id);
CREATE INDEX IF NOT EXISTS idx_chunks_ordinal ON chunks(document_id, ordinal);
CREATE INDEX IF NOT EXISTS idx_chunks_checksum ON chunks(checksum);
```

### `artifacts`
Metadata for large payload files stored on disk.
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
  created_at TEXT NOT NULL
);
```

### `sessions`
Durable conversational and task memory timelines.
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
Timestamped memory events within a session.
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

CREATE INDEX IF NOT EXISTS idx_session_events_session_id ON session_events(session_id);
CREATE INDEX IF NOT EXISTS idx_session_events_kind ON session_events(kind);
CREATE INDEX IF NOT EXISTS idx_session_events_created_at ON session_events(created_at);
```

### `refresh_jobs`
Background source refresh queue.
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

CREATE INDEX IF NOT EXISTS idx_refresh_jobs_source_id ON refresh_jobs(source_id);
CREATE INDEX IF NOT EXISTS idx_refresh_jobs_status ON refresh_jobs(status);
CREATE INDEX IF NOT EXISTS idx_refresh_jobs_created_at ON refresh_jobs(created_at);

-- Partial index enforcing at most one active job per source (coalescing)
CREATE UNIQUE INDEX IF NOT EXISTS idx_refresh_jobs_one_active_per_source
  ON refresh_jobs(source_id)
  WHERE status IN ('queued', 'running');
```

---

## 3. SQLite FTS5 Virtual Tables

Full-text search tables using the `unicode61` tokenizer:

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

CREATE VIRTUAL TABLE IF NOT EXISTS session_events_fts USING fts5(
  event_id UNINDEXED,
  session_id UNINDEXED,
  kind,
  text,
  tokenize = 'unicode61'
);

CREATE VIRTUAL TABLE IF NOT EXISTS library_aliases_fts USING fts5(
  alias,
  library_id UNINDEXED,
  name,
  provider,
  tokenize = 'unicode61'
);
```

---

## 4. Vector Embedding Tables (`ck-vector`)

Installed by `ck-vector` for storing binary float embeddings:

```sql
CREATE TABLE IF NOT EXISTS chunk_embeddings (
  chunk_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  vector_blob BLOB NOT NULL,
  corpus_id TEXT,
  library_id TEXT,
  document_id TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (chunk_id, model_id)
);

CREATE TABLE IF NOT EXISTS session_event_embeddings (
  event_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  vector_blob BLOB NOT NULL,
  session_id TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (event_id, model_id)
);
```

---

## 5. Domain Enum Representations in SQLite

| Rust Enum | Variant | Persisted SQLite String |
| :--- | :--- | :--- |
| **`SessionTransport`** | `Stdio`<br>`Sse` | `"stdio"`<br>`"sse"` |
| **`ContinuationMode`** | `Ephemeral`<br>`Resumeable`<br>`Pinned` | `"ephemeral"`<br>`"resumeable"`<br>`"pinned"` |
| **`SessionStatus`** | `Active`<br>`Closed`<br>`Compacted` | `"active"`<br>`"closed"`<br>`"compacted"` |
| **`JobTrigger`** | `Startup`<br>`Manual`<br>`Scheduled`<br>`Webhook`<br>`DependencyChange` | `"startup"`<br>`"manual"`<br>`"scheduled"`<br>`"webhook"`<br>`"dependency_change"` |
| **`JobStatus`** | `Queued`<br>`Running`<br>`Completed`<br>`Failed`<br>`Cancelled` | `"queued"`<br>`"running"`<br>`"completed"`<br>`"failed"`<br>`"cancelled"` |
| **`SourceHealth`** | `Unknown`<br>`Healthy`<br>`Degraded` | `"unknown"`<br>`"healthy"`<br>`"degraded"` |
