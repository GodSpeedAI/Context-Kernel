# ck-ranker

## Overview
`ck-ranker` (package `ck-ranker`) is a ports-and-adapters adapter crate providing the ranking stage of Context Kernel. It implements the `RankerPort` from `ck-core` with a single stateless ranker, `LexicalRanker`, which combines lexical FTS, trust, freshness, semantic, session-affinity, and alias-exactness signals under intent-specific weight profiles and de-duplicates retrieval results per document. The crate is composed into the runtime by `ck-bin` and has no direct source files outside its `src` module.

## Subdirectories & Boundaries
- `src`: [./src/README.md](./src/README.md) - Component contract for `LexicalRanker`: intent-weighted hybrid scoring, sigmoid-normalized FTS, per-document de-duplication, and debug/compact score breakdowns.

## Verification
- Run: `cargo test -p ck-ranker`
