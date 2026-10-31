# ck-storage-sqlite (src)

## Overview

This crate is the SQLite persistence adapter of Context Kernel: it implements the `ck-core` ports `StoragePort` and `SessionMemoryPort` (imported from `ck_core::ports`) on top of an embedded SQLite database via `rusqlite`. The single public type `SqliteStore` holds only the database file path and opens a fresh connection per operation; it never pools or caches connections. The full schema is embedded as the idempotent `SCHEMA_V1` SQL batch (all objects use `CREATE ... IF NOT EXISTS`), and schema application is recorded in a `schema_migrations` table at version 1. Beyond the two port implementations, the crate exposes runtime helpers for schema migration, storage status reporting, and a transactional FTS index rebuild.

## Interface

- [lib.rs:SqliteStore] - `Clone` adapter struct holding the database file path; the only public type in the crate.
- [lib.rs:SqliteStore::open] - Constructs the store for a path and creates the parent directories of the database file.
- [lib.rs:SqliteStore::path] - Returns the configured database path.
- [lib.rs:SqliteStore::conn] - Private helper opening a new `rusqlite::Connection` per call with `PRAGMA foreign_keys = ON`.
- [lib.rs:SqliteStore::migrate] - Executes the `SCHEMA_V1` batch against the database.
- [lib.rs:SqliteStore::schema_sql] - Returns the embedded `SCHEMA_V1` SQL text.
- [lib.rs:SqliteStore::status] - Reports a `StorageStatus`, probing `sqlite_master` for the `chunks_fts` table to set `fts5_enabled`.
- [lib.rs:SqliteStore::rebuild_fts_index] - Clears and repopulates all three FTS tables from canonical rows in one transaction; returns the reindexed counts for chunks, session events, and aliases.
- [lib.rs:StoragePort for SqliteStore] - Full trait implementation covering sources, corpora and the corpus-source link, documents, chunks (transactional whole-document replacement), library identities and aliases, refresh jobs (including single-active-job insertion and atomic job claiming), source health and staleness, artifacts, and retention deletes for events, sessions, jobs, disabled sources, and artifacts.
- [lib.rs:SessionMemoryPort for SqliteStore] - Session lifecycle (`create_session`, `get_session`, `count_active_sessions`) and session events (`add_event`, `get_event`, `list_events_by_session`); `add_event` generates event IDs via `Ulid` and timestamps via `chrono::Utc`.
- [lib.rs:SCHEMA_V1] - Embedded DDL. Exact tables: `schema_migrations`, `sources`, `corpora`, `corpus_sources`, `library_identities`, `library_aliases`, `documents`, `chunks`, `artifacts`, `sessions`, `session_events`, `refresh_jobs`; FTS5 virtual tables `chunks_fts`, `session_events_fts`, `library_aliases_fts` (tokenize = 'unicode61', ID columns `UNINDEXED`); indexes `idx_documents_*`, `idx_chunks_*`, `idx_session_events_*`, `idx_refresh_jobs_*`, and the partial unique index `idx_refresh_jobs_one_active_per_source`.

## Invariants

- Upsert discipline for cascading parents: `upsert_source` and `upsert_corpus` deliberately use `INSERT ... ON CONFLICT(...) DO UPDATE` instead of `INSERT OR REPLACE`, because a REPLACE deletes the existing row and would cascade-delete every child document/chunk under the schema's `ON DELETE CASCADE` foreign keys [lib.rs:SqliteStore::upsert_source] [lib.rs:SqliteStore::upsert_corpus]. (Other upserts such as `upsert_document`, `upsert_library_identity`, `upsert_library_alias`, `upsert_refresh_job`, and `upsert_artifact` do use `INSERT OR REPLACE`.)
- Foreign-key enforcement: every connection opened by the store enables `PRAGMA foreign_keys = ON` [lib.rs:SqliteStore::conn], and the schema additionally sets `PRAGMA journal_mode=WAL` and `PRAGMA synchronous=NORMAL` [lib.rs:SCHEMA_V1].
- FTS rebuild atomicity: `rebuild_fts_index` performs the DELETE of all FTS rows and the repopulating INSERT...SELECT statements inside a single transaction, so a failure part-way leaves the existing index intact rather than cleared or half-populated [lib.rs:SqliteStore::rebuild_fts_index].
- Enum codec round-trip: `SessionTransport`, `ContinuationMode`, `SessionStatus`, `JobTrigger`, and `JobStatus` are persisted as fixed TEXT values via the private `*_str`/`*_from` codec pairs, and decoding an unknown stored string is a conversion error, not a default [lib.rs:transport_from] [lib.rs:cont_mode_from] [lib.rs:sess_status_from] [lib.rs:trigger_from] [lib.rs:job_status_from].
- Pinned-event preservation: retention deletion removes only `pinned = 0` session events [lib.rs:SqliteStore::delete_unpinned_session_events_older_than]; inactive sessions that still own pinned events are kept [lib.rs:SqliteStore::delete_inactive_sessions_older_than]; artifacts referenced by a pinned event are excluded from deletion and from the payload-removal candidate list [lib.rs:SqliteStore::delete_artifacts_older_than] [lib.rs:SqliteStore::list_artifacts_older_than].
- One active refresh job per source: enforced by the partial unique index `idx_refresh_jobs_one_active_per_source` (on `source_id` WHERE `status IN ('queued','running')`) [lib.rs:SCHEMA_V1]; `insert_active_refresh_job` relies on `ON CONFLICT ... DO NOTHING` and falls back to returning the existing active job [lib.rs:SqliteStore::insert_active_refresh_job]; `claim_next_refresh_job_for_source` transitions a queued job to `running` atomically via `UPDATE ... RETURNING *` [lib.rs:SqliteStore::claim_next_refresh_job_for_source].
- Session creation is non-destructive: `create_session` uses `INSERT OR IGNORE`, so an existing session row is left untouched while the newly constructed `Session` (status `Active`) is returned [lib.rs:SqliteStore::create_session].

## Side Effects

- Opens a new file-backed SQLite connection for every operation via `SqliteStore::conn` [lib.rs:SqliteStore::conn].
- Creates parent directories of the database file (and, on first connect, the database file itself) [lib.rs:SqliteStore::open].
- `migrate` mutates the database schema: sets PRAGMAs, creates tables, indexes, and FTS5 virtual tables, and inserts the version record into `schema_migrations` [lib.rs:SqliteStore::migrate] [lib.rs:SCHEMA_V1].
- All port methods issue SQL INSERT/UPDATE/DELETE statements against the tables listed above; `rebuild_fts_index` additionally clears and rewrites the three FTS tables [lib.rs:SqliteStore::rebuild_fts_index].
- Generates wall-clock timestamps with `chrono::Utc::now()` for source health updates, job claiming, session creation, and event creation [lib.rs:SqliteStore::update_source_health] [lib.rs:SqliteStore::claim_next_refresh_job_for_source] [lib.rs:SqliteStore::create_session] [lib.rs:SqliteStore::add_event].

## Verification

- Run: `cargo test -p ck-storage-sqlite`
