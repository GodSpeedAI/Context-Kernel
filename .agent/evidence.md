# Evidence Log

## Claim
- Claim: Cargo workspace exists for Milestone 0 crates.
- Status: implemented
- Evidence type: code
- Files: `Cargo.toml`
- Lines: 1-9
- Notes: Workspace includes ck-core, ck-config, ck-storage-sqlite, ck-search-fts, ck-artifacts, ck-ranker, ck-adapter-filesystem, ck-app, ck-mcp, ck-transport-stdio, ck-bin.
- Confidence: high

## Claim
- Claim: SQLite migration framework creates schema version 1 and FTS5 tables.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Lines: SCHEMA_V1 constant, migrate(), tests
- Notes: `SqliteStore::migrate` executes schema v1; test verifies `schema_migrations` and `chunks_fts`.
- Confidence: high

## Claim
- Claim: `get_status` is exposed through the MCP service.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`
- Notes: `McpService::call_tool` returns serialized `KernelStatus` for `get_status`; test verifies contract fields.
- Confidence: high

## Claim
- Claim: stdio `serve` can answer JSON-RPC `tools/call` for `get_status`.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-bin/src/main.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: CLI test sends a JSON-RPC tool call and asserts `context-kernal` plus `fts5_enabled: true`.
- Confidence: high

## Claim
- Claim: StoragePort is fully implemented with CRUD for sources, corpora, documents, chunks, library identities, aliases, and refresh jobs.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Notes: 6 unit tests cover session CRUD, event CRUD, source upsert, document+chunk CRUD, library identity+alias CRUD, refresh job CRUD.
- Confidence: high

## Claim
- Claim: SessionMemoryPort is implemented with session creation, event persistence, and active session counting.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Notes: Tests verify session creation, event addition, listing, and count.
- Confidence: high

## Claim
- Claim: LexicalSearchPort is implemented with FTS5/BM25 for chunks, session events, and library aliases.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-search-fts/src/lib.rs`
- Notes: 5 tests cover chunk indexing/search, removal, session event indexing/search, library alias indexing/search, BM25 score positivity.
- Confidence: high

## Claim
- Claim: ArtifactPort is implemented with filesystem storage and SHA-256 checksums.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-artifacts/src/lib.rs`
- Notes: 3 tests cover store+read, checksum verification, non-existent returns None.
- Confidence: high

## Claim
- Claim: RankerPort is implemented with lexical ranking and score breakdowns.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-ranker/src/lib.rs`
- Notes: 3 tests cover resolution candidates, retrieval results, recall results.
- Confidence: high

## Claim
- Claim: Filesystem source adapter, markdown normalizer, and heading-aware chunker are implemented.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-filesystem/src/lib.rs`
- Notes: 6 tests cover file discovery, content fetching, normalization, heading-based chunking, text without headings.
- Confidence: high

## Claim
- Claim: `store_context` MCP tool persists session events and handles large payloads as artifacts.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test verifies store_context returns event_id and stored_as fields.
- Confidence: high

## Claim
- Claim: `recall_context` MCP tool returns BM25-ranked session events.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test verifies recall returns query_id and results.
- Confidence: high

## Claim
- Claim: `ingest_source` MCP tool indexes local filesystem documents.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test creates a temp docs directory, ingests it, and verifies job_id and completed status.
- Confidence: high

## Claim
- Claim: `query_reference` MCP tool returns lexical search results over indexed chunks.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test ingests docs then queries, verifying query_id and results.
- Confidence: high

## Claim
- Claim: `resolve_reference` MCP tool and compatibility aliases work.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration tests verify resolve-library-id and get-library-docs aliases return candidates/query_id.
- Confidence: high

## Claim
- Claim: All 37 workspace tests pass, cargo fmt is clean, cargo clippy has zero warnings.
- Status: implemented
- Evidence type: test
- Files: workspace
- Notes: `cargo test --workspace` = 37 passed, `cargo fmt --check` = clean, `cargo clippy --workspace` = zero warnings.
- Confidence: high

## Claim
- Claim: LexicalRanker implements hybrid scoring with normalized FTS + trust + freshness weights.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-ranker/src/lib.rs`
- Reproducible command: `cargo test -p ck-ranker`
- Result excerpt: `running 5 tests ... test result: ok. 5 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: focused-slice
- Notes: Tests include `fts_score_is_normalized_before_combining` and `trust_score_influences_ranking`; raw and normalized FTS values are included in score breakdowns.
- Confidence: high

## Claim
- Claim: AuthPort trait exists in ck-core with is_local_trust_mode and validate_token methods.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-core/src/ports/auth.rs`
- Reproducible command: `cargo test -p ck-auth`
- Result excerpt: `running 4 tests ... test result: ok. 4 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: focused-slice
- Notes: Trait defined with two methods; BearerAuth in ck-auth implements it.
- Confidence: high

## Claim
- Claim: BearerAuth implements AuthPort with fail-closed token validation and explicit no-auth mode.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-auth/src/lib.rs`, `crates/ck-auth/Cargo.toml`
- Reproducible command: `cargo test -p ck-auth`
- Result excerpt: `running 4 tests ... empty_tokens_fail_closed_without_local_trust_mode ... ok ... test result: ok. 4 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: focused-slice
- Notes: Empty token lists only allow all when local trust mode is explicit; configured token checks use `subtle::ConstantTimeEq`.
- Confidence: high

## Claim
- Claim: SchedulerPort trait exists in ck-core with enqueue, claim, complete, fail, list_stale methods.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-core/src/ports/scheduler.rs`
- Reproducible command: `cargo test -p ck-scheduler`
- Result excerpt: `running 9 tests ... test result: ok. 9 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: focused-slice
- Notes: Trait defined with full job lifecycle methods; claim docs now state atomicity is provided through storage.
- Confidence: high

## Claim
- Claim: RefreshScheduler implements SchedulerPort with storage-backed active-job coalescing, atomic claim, and stale detection.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-scheduler/src/lib.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-core/src/ports/storage.rs`
- Reproducible command: `cargo test -p ck-scheduler && cargo test -p ck-storage-sqlite`
- Result excerpt: `ck_scheduler ... 9 passed; ck_storage_sqlite ... 9 passed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: focused-slice
- Notes: Scheduler calls `insert_active_refresh_job` and `claim_next_refresh_job_for_source`; SQLite tests cover single-use claim and active job coalescing.
- Confidence: high

## Claim
- Claim: SSE transport serves HTTP endpoints with auth gating.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-transport-sse/src/lib.rs`, `crates/ck-bin/src/main.rs`
- Reproducible command: `cargo test --workspace`
- Result excerpt: `test sse_healthz_endpoint_works ... ok ... test result: ok. 65 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: release-gate-proof
- Notes: Routes: POST /mcp, GET /healthz, GET /readyz, GET /api/v1/status. Auth checked on mcp_handler; ck-bin now fails fast if SSE requires auth and no tokens are configured.
- Confidence: high

## Claim
- Claim: refresh_source MCP tool re-ingests existing sources into their persisted corpus link.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`, `crates/ck-storage-sqlite/src/lib.rs`
- Reproducible command: `cargo test -p ck-app && cargo test --workspace`
- Result excerpt: `refresh_source_reuses_persisted_corpus_id ... ok`; workspace `test result: ok. 65 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: release-gate-proof
- Notes: Unit test verifies corpus reuse via `find_corpus_id_for_source`; integration test `refresh_source_tool_works` verifies ingest then refresh returns completed job.
- Confidence: high

## Claim
- Claim: Periodic refresh background thread runs in ck-bin when config.scheduler.enable_periodic_refresh is true.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-bin/src/main.rs`
- Lines: 126-182
- Reproducible command: `cargo test --workspace`
- Result excerpt: `test sse_healthz_endpoint_works ... ok ... test result: ok. 65 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: local-confidence
- Notes: Static code path creates a background thread when scheduler config is enabled; workspace tests compile this path but do not run a periodic refresh loop.
- Confidence: high

## Claim
- Claim: Dual transport (stdio + SSE) works simultaneously.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-bin/src/main.rs`, `crates/ck-bin/tests/cli.rs`
- Reproducible command: `cargo test --workspace`
- Result excerpt: `test sse_healthz_endpoint_works ... ok ... test result: ok. 65 passed; 0 failed`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: release-gate-proof
- Notes: Integration test `sse_healthz_endpoint_works` verifies binary starts with --sse flag and still responds to stdio.
- Confidence: high

## Claim
- Claim: Workspace tests pass, cargo fmt is clean, cargo clippy completes with zero warnings.
- Status: implemented
- Evidence type: test
- Files: workspace
- Reproducible command: `cargo test --workspace && cargo fmt --check && cargo clippy --workspace`
- Result excerpt: `cargo test --workspace`: `test result: ok. 65 passed; 0 failed`; `cargo fmt --check`: no diff; `cargo clippy --workspace`: `Finished dev profile`
- Timestamp/context: 2026-06-02 07:59 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: release-gate-proof
- Notes: Workspace count is now 65 tests after review regression coverage was added.
- Confidence: high

## Claim
- Claim: GitHub source adapter crate (ck-adapter-github) implements SourceAdapter and Normalizer ports.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-github/src/lib.rs`, `crates/ck-adapter-github/Cargo.toml`
- Reproducible command: `cargo test -p ck-adapter-github`
- Result excerpt: `running 18 tests ... test result: ok. 18 passed; 0 failed`
- Timestamp/context: 2026-06-02 12:30 EDT
- Proof classification: focused-slice
- Notes: GitHubSourceAdapter implements SourceAdapter (kind="github_repo") with static file list and tree API discovery. GitHubNormalizer implements Normalizer with GitHub metadata passthrough. Uses ureq for HTTP, base64 for content decoding. Token supports $ENV_VAR syntax.
- Confidence: high

## Claim
- Claim: Milestone 7 — LexicalRanker implements intent-specific weight profiles, session affinity, de-duplication, and debug mode.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-ranker/src/lib.rs`, `crates/ck-core/src/ports/ranker.rs`
- Reproducible command: `cargo test -p ck-ranker`
- Result excerpt: `running 15 tests ... test result: ok. 15 passed; 0 failed`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: focused-slice
- Notes: Tests cover intent_weighting_changes_ranking_order, session_affinity_boosts_matching_session_items, deduplication_collapses_same_document_chunks, debug_mode_includes_detailed_breakdown, semantic_score_included_in_weighted_combination.
- Confidence: high

## Claim
- Claim: Milestone 8 — EmbeddingPort and VectorSearchPort traits defined in ck-core.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-core/src/ports/embedding.rs`, `crates/ck-core/src/ports/vector_search.rs`, `crates/ck-core/src/ports/mod.rs`
- Reproducible command: `cargo check -p ck-core`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: code-presence
- Notes: EmbeddingPort has embed/embed_batch/model_id/dimensions/is_available. VectorSearchPort has upsert_chunk_embedding/upsert_session_event_embedding/search_chunks/search_session_events/remove_chunk_embeddings_by_document.
- Confidence: high

## Claim
- Claim: Milestone 8 — ck-embeddings provides NoopEmbeddingProvider and OpenAIEmbeddingProvider.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-embeddings/src/lib.rs`, `crates/ck-embeddings/Cargo.toml`
- Reproducible command: `cargo test -p ck-embeddings`
- Result excerpt: `running 5 tests ... test result: ok. 5 passed; 0 failed`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: focused-slice
- Notes: NoopEmbeddingProvider returns None/unavailable. OpenAIEmbeddingProvider uses ureq for sync HTTP to OpenAI-compatible API. Graceful fallback when not configured.
- Confidence: high

## Claim
- Claim: Milestone 8 — ck-vector provides SqliteVectorStore with brute-force cosine similarity.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-vector/src/lib.rs`, `crates/ck-vector/Cargo.toml`
- Reproducible command: `cargo test -p ck-vector`
- Result excerpt: `running 14 tests ... test result: ok. 14 passed; 0 failed`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: focused-slice
- Notes: Tests cover encode_decode_roundtrip, cosine_similarity correctness, upsert_and_search, model filtering, limit enforcement, clone reopens database.
- Confidence: high

## Claim
- Claim: Milestone 8 — Semantic retrieval integrated into query_reference with graceful fallback.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-app/src/lib.rs`
- Reproducible command: `cargo test --workspace`
- Result excerpt: `test result: ok. 153 passed; 0 failed`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: release-gate-proof
- Notes: AppServices has optional embedding/vector_search fields. query_reference merges FTS + vector results when semantic is available. semantic_available() method for status reporting.
- Confidence: high

## Claim
- Claim: Milestone 9 — ck-adapter-web provides WebSourceAdapter with SSRF protections.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-web/src/lib.rs`, `crates/ck-adapter-web/Cargo.toml`
- Reproducible command: `cargo test -p ck-adapter-web`
- Result excerpt: `running 34 tests ... test result: ok. 34 passed; 0 failed`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: focused-slice
- Notes: 12 SSRF-specific tests covering private IP blocking, scheme blocking, IPv6 blocking, allowed_domains enforcement, localhost blocking. HTML-to-text conversion and title extraction tested.
- Confidence: high

## Claim
- Claim: Milestone 9 — ck-adapter-github provides GitHubSourceAdapter with tree API and static file list.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-github/src/lib.rs`, `crates/ck-adapter-github/Cargo.toml`
- Reproducible command: `cargo test -p ck-adapter-github`
- Result excerpt: `running 18 tests ... test result: ok. 18 passed; 0 failed`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: focused-slice
- Notes: Supports both static `files` array and dynamic tree API discovery. Token auth with $ENV_VAR syntax. Base64 content decoding. GitHubNormalizer preserves metadata.
- Confidence: high

## Claim
- Claim: Workspace tests pass, cargo fmt is clean, cargo clippy completes with zero warnings after M7-M9.
- Status: implemented
- Evidence type: test
- Files: workspace
- Reproducible command: `cargo test --workspace && cargo fmt --check && cargo clippy --workspace`
- Result excerpt: `cargo test --workspace`: `test result: ok. 153 passed; 0 failed`; `cargo fmt --check`: no diff; `cargo clippy --workspace`: `Finished dev profile`
- Timestamp/context: 2026-06-02 10:30 EDT
- Proof classification: release-gate-proof
- Notes: Test count increased from 65 to 153 after M7-M9 implementation.
- Confidence: high
