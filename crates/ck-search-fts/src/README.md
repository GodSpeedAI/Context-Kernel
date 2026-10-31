# ck-search-fts (src)

## Overview

This crate is the lexical-search adapter of Context Kernel: it implements the `ck-core` `LexicalSearchPort` trait (imported from `ck_core::ports`) over SQLite FTS5 virtual tables via `rusqlite`. The single public type `FtsSearch` holds only the database file path and opens a fresh connection per operation; the crate creates no schema itself and relies on the FTS tables already existing (they are created by `ck-storage-sqlite`'s `SCHEMA_V1`). It covers three indexes: document chunks (`chunks_fts`), session events (`session_events_fts`), and library aliases (`library_aliases_fts`), returning results as `ck_core::entities::ScoredItem` values.

## Interface

- [lib.rs:FtsSearch] - Adapter struct holding the database file path; the only public type in the crate.
- [lib.rs:FtsSearch::open] - Constructs the adapter for a database path without touching the database.
- [lib.rs:FtsSearch::conn] - Private helper opening a new `rusqlite::Connection` per call with `PRAGMA journal_mode=WAL`.
- [lib.rs:FtsSearch::fts_query] - Private helper turning raw query text into an FTS MATCH expression (see Invariants).
- [lib.rs:FtsSearch::collect_scored] - Private helper mapping `(id, raw_score)` rows to `ScoredItem` with a `{"fts": score}` `score_breakdown`.
- [lib.rs:LexicalSearchPort for FtsSearch] - Full trait implementation: `index_chunk`, `remove_chunks_by_document`, `search_chunks`, `index_session_event`, `search_session_events`, `index_library_alias`, `search_library_aliases`. Indexing methods issue plain `INSERT`s into the FTS tables; searches rank with `bm25()` and apply corpus/library/session filters by joining the canonical `chunks`, `documents`, and `session_events` tables.

## Invariants

- Query sanitization: user query text never reaches `MATCH` raw; `fts_query` splits on whitespace, drops empty tokens, quotes each token with embedded quotes doubled, and joins tokens with `OR`; an all-empty query yields no expression [lib.rs:FtsSearch::fts_query].
- Score polarity: SQLite `bm25()` returns lower-is-better (negative) values, so `collect_scored` negates the raw score, making `ScoredItem.score` higher-is-better and positive for matches [lib.rs:FtsSearch::collect_scored].
- Empty chunk queries are browses, not refusals: when `search_chunks` receives an empty/whitespace query it performs rank-neutral chunk enumeration (score 0.0, ordered by `chunk_id`, bounded by `limit`) over the canonical `chunks`/`documents` tables with the same corpus/library filters [lib.rs:FtsSearch::search_chunks]; in contrast, blank queries to `search_session_events` and `search_library_aliases` return an empty result set [lib.rs:FtsSearch::search_session_events] [lib.rs:FtsSearch::search_library_aliases].
- No schema ownership: the crate issues no `CREATE` statements; index writes are plain `INSERT`s with no conflict handling into `chunks_fts`, `session_events_fts`, and `library_aliases_fts`, so those tables must exist beforehand [lib.rs:LexicalSearchPort for FtsSearch].
- Deletion resolution via canonical rows: `remove_chunks_by_document` looks up chunk IDs in the canonical `chunks` table by `document_id` and then deletes each from `chunks_fts` [lib.rs:FtsSearch::remove_chunks_by_document].

## Side Effects

- Opens a new file-backed SQLite connection for every operation via `FtsSearch::conn` and sets `PRAGMA journal_mode=WAL` on it [lib.rs:FtsSearch::conn].
- Writes rows into the FTS5 virtual tables `chunks_fts`, `session_events_fts`, and `library_aliases_fts` on indexing calls, and deletes rows from `chunks_fts` on document removal [lib.rs:LexicalSearchPort for FtsSearch].
- Reads the canonical tables `chunks`, `documents`, and `session_events` for filters, joins, and browse enumeration, assuming they are kept in sync by the storage adapter [lib.rs:FtsSearch::search_chunks] [lib.rs:FtsSearch::remove_chunks_by_document].

## Verification

- Run: `cargo test -p ck-search-fts`
