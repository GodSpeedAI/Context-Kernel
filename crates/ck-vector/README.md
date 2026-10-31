# ck-vector

## Overview
`ck-vector` (package `ck-vector`) is a ports-and-adapters adapter crate providing vector search for Context Kernel. It implements the `VectorSearchPort` from `ck-core` with `SqliteVectorStore`, a SQLite-backed store of chunk and session-event embeddings that performs brute-force cosine-similarity search scoped by embedding `model_id`. The crate is composed into the runtime by `ck-bin` and has no direct source files outside its `src` module.

## Subdirectories & Boundaries
- `src`: [./src/README.md](./src/README.md) - Component contract for `SqliteVectorStore`: schema and upsert semantics keyed by `(id, model_id)`, length-prefixed LE-f64 blob codec, brute-force cosine search with model/filter scoping, and per-document embedding deletion.

## Verification
- Run: `cargo test -p ck-vector`
