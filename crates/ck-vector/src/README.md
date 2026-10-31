# ck-vector Source

## Overview
This crate implements the vector-search stage of Context Kernel as a SQLite-backed adapter over the `VectorSearchPort` defined in `ck-core`. `SqliteVectorStore` persists chunk and session-event embeddings as length-prefixed little-endian f64 blobs in two tables and performs brute-force cosine-similarity search in Rust over the matching rows - there is no native vector index or SQLite extension. Every embedding is keyed jointly by its id and embedding `model_id`, so vectors from different embedding models never mix. The store is composed into the runtime by `ck-bin` (`use ck_vector::SqliteVectorStore` in `crates/ck-bin/src/main.rs`).

## Interface
- [lib.rs:SqliteVectorStore] - Public `Clone` handle implementing `ck_core::ports::VectorSearchPort`; it holds only the database path and opens a fresh connection per operation.
- [lib.rs:SqliteVectorStore::open] - Opens (or creates) the database, creates parent directories, and applies the schema.
- [lib.rs:SqliteVectorStore::upsert_chunk_embedding] - Stores/replaces a chunk embedding without metadata (metadata columns set to NULL).
- [lib.rs:SqliteVectorStore::upsert_chunk_embedding_with_metadata] - Stores/replaces a chunk embedding with optional `corpus_id`, `library_id`, and `document_id` metadata.
- [lib.rs:SqliteVectorStore::upsert_session_event_embedding] - Stores/replaces a session-event embedding.
- [lib.rs:SqliteVectorStore::search_chunks] - Brute-force cosine search over chunk embeddings, optionally filtered by `corpus_id` and `library_id`, returning `Vec<ScoredItem>` with `{"semantic": sim}` breakdowns.
- [lib.rs:SqliteVectorStore::search_session_events] - Brute-force cosine search over session-event embeddings, optionally filtered by `session_id`.
- [lib.rs:SqliteVectorStore::remove_chunk_embeddings_by_document] - Deletes all chunk embeddings carrying the given `document_id`.
- Private machinery: [lib.rs:SCHEMA] (DDL), [lib.rs:encode_vector] / [lib.rs:decode_vector] (blob codec), [lib.rs:cosine_similarity] (scoring).

## Invariants
- Schema: [lib.rs:SCHEMA] defines `chunk_embeddings` with primary key `(chunk_id, model_id)` and `session_event_embeddings` with primary key `(event_id, model_id)`, plus indexes on `corpus_id`, `library_id`, `document_id`, and `session_id`; DDL is idempotent (`CREATE ... IF NOT EXISTS`) and applied on every `open`.
- Upsert semantics: writes use `INSERT OR REPLACE` keyed by `(id, model_id)`, so re-embedding the same chunk/event under the same model replaces the prior vector ([lib.rs:SqliteVectorStore::upsert_chunk_embedding] and siblings).
- Model isolation: every search query filters rows by `model_id` before scoring ([lib.rs:SqliteVectorStore::search_chunks], [lib.rs:SqliteVectorStore::search_session_events]).
- Blob format: [lib.rs:encode_vector] writes a 4-byte little-endian `u32` dimension prefix followed by little-endian `f64` values; [lib.rs:decode_vector] rejects any blob whose length is not exactly `4 + dim * 8` with an error.
- Similarity semantics: [lib.rs:cosine_similarity] returns 0.0 when dimensions differ or either vector has zero magnitude; scores are plain cosine values in [-1.0, 1.0].
- Search ordering and limits: both searches sort descending by score (NaN compares as equal via `partial_cmp` fallback) and truncate to `limit`, treating a negative limit as zero ([lib.rs:SqliteVectorStore::search_chunks], [lib.rs:SqliteVectorStore::search_session_events]).
- Foreign keys: every connection enables `PRAGMA foreign_keys = ON` ([lib.rs:SqliteVectorStore::conn]).

## Side Effects
- SQLite file I/O: each operation opens a new `rusqlite::Connection` to the database file and runs `PRAGMA foreign_keys = ON` ([lib.rs:SqliteVectorStore::conn]).
- Filesystem: [lib.rs:SqliteVectorStore::open] creates the database's parent directory (`create_dir_all`) and executes the schema DDL against the database.
- No network, thread, lock, or global-state side effects; concurrency is delegated to SQLite's own locking.

## Verification
- Run: `cargo test -p ck-vector`
