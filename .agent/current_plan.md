# Current Plan

## Objective
- Implement Milestones 7-9 from Context_Kernel_Service_Plan.md

## Status
- In progress.

## Scope
- M7: Hybrid ranking — query intent weighting, session affinity, de-duplication, result collapsing
- M8: Optional embeddings — EmbeddingPort, VectorSearchPort, pluggable provider, vector store, semantic/hybrid retrieval, graceful fallback
- M9: GitHub/web adapters — GitHub repo adapter, website fetch adapter, SSRF protections, incremental refresh hints

## Constraints
- Preserve ports-and-adapters boundaries
- Keep ck-core free of infrastructure dependencies (no tokio, no SQL, no HTTP)
- Synchronous core (rusqlite style); async only in transport layer
- Match existing code patterns and naming conventions
- All new crates must have tests

## Files / Crates Expected To Change
- `crates/ck-core/src/ports/` — add embedding.rs, vector_search.rs; update ranker.rs, mod.rs
- `crates/ck-ranker/src/lib.rs` — enhance with intent weighting, affinity, de-dup
- `crates/ck-embeddings/` — NEW crate
- `crates/ck-vector/` — NEW crate
- `crates/ck-adapter-github/` — NEW crate
- `crates/ck-adapter-web/` — NEW crate
- `crates/ck-config/src/lib.rs` — add embeddings config
- `crates/ck-app/src/lib.rs` — wire semantic retrieval, intent-aware ranking
- `crates/ck-bin/src/main.rs` — wire new crates
- `Cargo.toml` — add new workspace members and dependencies
- `crates/ck-bin/tests/cli.rs` — integration tests

## Steps
1. Add EmbeddingPort and VectorSearchPort traits to ck-core
2. Update RankerPort to accept query intent and session context
3. Enhance ck-ranker with intent weighting, session affinity, de-duplication
4. Create ck-embeddings crate with OpenAI-compatible provider and no-op fallback
5. Create ck-vector crate with SQLite brute-force cosine similarity
6. Create ck-adapter-github crate with GitHub API adapter
7. Create ck-adapter-web crate with HTTP fetch adapter + SSRF protections
8. Update ck-config with embeddings configuration section
9. Update ck-app to integrate semantic retrieval and intent-aware ranking
10. Update ck-bin to wire all new crates
11. Add integration tests
12. Verify: cargo test --workspace, cargo fmt --check, cargo clippy --workspace

## Validation Plan
- All existing 65 tests continue to pass
- New unit tests for each new crate
- New integration tests for M7-M9 features
- cargo fmt --check clean
- cargo clippy --workspace zero warnings
