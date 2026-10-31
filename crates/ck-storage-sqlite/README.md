# ck-storage-sqlite

## Overview

Adapter crate providing the SQLite-backed implementation of the persistence boundary of Context Kernel. It depends on `ck-core` for the `StoragePort` and `SessionMemoryPort` traits and their entity types, and on `rusqlite` for the embedded database engine. The crate owns the canonical relational schema (`SCHEMA_V1`, recorded as schema version 1 in `schema_migrations`) including the FTS5 virtual tables that `ck-search-fts` queries, plus helpers for migration, status reporting, and transactional FTS index rebuilding. It contains no transport, MCP, or HTTP code, per the ports-and-adapters layering.

## Subdirectories & Boundaries

- src: [./src/README.md](./src/README.md) - Component contract for `lib.rs`: `SqliteStore`, the `StoragePort`/`SessionMemoryPort` implementations, the embedded `SCHEMA_V1` DDL (tables `sources`, `corpora`, `corpus_sources`, `library_identities`, `library_aliases`, `documents`, `chunks`, `artifacts`, `sessions`, `session_events`, `refresh_jobs`, `schema_migrations`; FTS5 tables `chunks_fts`, `session_events_fts`, `library_aliases_fts`), and the FTS rebuild/status helpers.

## Verification

- Run: `cargo test -p ck-storage-sqlite`
