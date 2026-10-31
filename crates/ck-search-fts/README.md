# ck-search-fts

## Overview

Adapter crate providing SQLite FTS5-backed lexical search for Context Kernel. It depends on `ck-core` for the `LexicalSearchPort` trait and the `ScoredItem` entity, and on `rusqlite` for database access. The crate implements chunk search, session-event search, and library-alias search with bm25 ranking, corpus/library/session filtering, and rank-neutral browsing for empty queries. It owns no schema: the FTS5 tables it reads and writes are created by the `ck-storage-sqlite` adapter's `SCHEMA_V1`.

## Subdirectories & Boundaries

- src: [./src/README.md](./src/README.md) - Component contract for `lib.rs`: `FtsSearch` and its `LexicalSearchPort` implementation over the FTS5 tables `chunks_fts`, `session_events_fts`, and `library_aliases_fts`, including query sanitization, bm25 score negation, and browse-mode behavior.

## Verification

- Run: `cargo test -p ck-search-fts`
