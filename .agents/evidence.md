# Evidence Log

## Technical Knowledge System (IMPLEMENTED)
- Claim: The repository contains a self-contained, Diátaxis-structured, progressive-disclosure technical knowledge system with DeepWiki architecture guides and Google Code Wiki source traceability.
- Status: implemented
- Evidence type: documentation, code, config, schema, test
- Files:
  - Navigation & Index: `docs/README.md`, `docs/documentation-map.md`, `docs/architecture.md`, `docs/source-map.md`
  - Foundations: `docs/mental-model.md`, `docs/concepts.md`
  - Subsystems: `docs/subsystems/core-and-domain.md`, `docs/subsystems/storage-and-persistence.md`, `docs/subsystems/retrieval-and-ranking.md`, `docs/subsystems/source-adapters.md`, `docs/subsystems/application-orchestration.md`, `docs/subsystems/mcp-and-protocols.md`, `docs/subsystems/runtime-and-cli.md`
  - Workflows: `docs/workflows/service-lifecycle-and-startup.md`, `docs/workflows/documentation-ingestion-and-refresh.md`, `docs/workflows/reference-query-and-resolution.md`, `docs/workflows/session-context-continuity.md`, `docs/workflows/agentic-capability-loop.md`
  - Explanation: `docs/explanation/architectural-decisions.md`, `docs/explanation/retrieval-and-ranking-design.md`, `docs/explanation/retention-and-memory-model.md`, `docs/explanation/known-limits-and-debt.md`
  - Tutorials: `docs/tutorials/01-first-steps.md`, `docs/tutorials/02-local-documentation-indexing.md`
  - How-To Guides: `docs/how-to/configure-sse-with-auth.md`, `docs/how-to/manage-database-and-migrations.md`, `docs/how-to/configure-semantic-retrieval.md`, `docs/how-to/implement-a-source-adapter.md`
  - Reference: `docs/reference/cli.md`, `docs/reference/configuration.md`, `docs/reference/mcp-tools.md`, `docs/reference/database-schema.md`, `docs/troubleshooting.md`
- Notes: All documented traits, structs, tables, tools, and error codes are verified against crate implementations and tests; unwired adapters and residual debts are explicitly acknowledged rather than asserted as implemented.
- Confidence: high

## Architecture
- Claim: Primary crate dependency direction mostly conforms to ports and adapters.
- Status: partial (unchanged this pass)
- Evidence type: code, config
- Files: `crates/ck-core/Cargo.toml`, `crates/ck-app/Cargo.toml`, `crates/ck-bin/src/main.rs`, `crates/ck-mcp/Cargo.toml`
- Notes: ACL context path still depends directly on `ck-adapter-filesystem` (deferred).
- Confidence: high

## SQLite enum round-trips (RESOLVED)
- Claim: All domain enum variants now round-trip through SQLite.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Test: `enum_roundtrips_all_writable_variants` (added)
- Confidence: high

## Reload auth + context agent (RESOLVED)
- Claim: Config reload now preserves tool/corpus authorization policy and the ACL context agent.
- Status: implemented
- Evidence type: code, integration test
- Files: `crates/ck-bin/src/main.rs`
- Test: `sse_reload_preserves_tool_authorization` — disallowed tool rejected after reload; allowed tool still succeeds.
- Confidence: high

## context_required auth + containment (RESOLVED)
- Claim: `context_required` corpus id is checked against the allowlist; corpus paths are contained under corpus_root.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-mcp/src/agentic_capability_loop.rs`
- Tests: `context_required_corpus_id_uses_authorization_arm`, `context_required_rejects_path_traversal_outside_corpus_root`
- Residual: `is_private`/`scope_claim` still client-trusted (mitigated by allowlist + containment).
- Confidence: high

## Write consistency (MITIGATED)
- Claim: Refresh no longer accumulates duplicates; payload compensated on metadata-write failure.
- Status: partial
- Evidence type: code, test
- Files: `crates/ck-app/src/lib.rs`
- Tests: `refresh_does_not_accumulate_duplicate_documents`; payload-compensation path in `store_context`.
- Residual: FTS↔DB still non-transactional (separate connections).
- Confidence: high

## SSRF fail-closed (RESOLVED)
- Claim: `validate_url_with_resolver` bails on resolver Err and empty resolution.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-web/src/lib.rs`
- Tests: `validate_url_with_resolver_fails_closed_on_resolver_error`, `…rejects_unresolvable_host`; 4 bug-ratifying tests corrected.
- Confidence: high

## Observability (RESOLVED)
- Claim: Subscriber installed process-wide; config-driven.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-observability/src/lib.rs`, `crates/ck-bin/src/main.rs` (`init_logging`)
- Confidence: high

## SSE lifecycle (MITIGATED)
- Claim: SSE bind failure fails startup; readiness probes the live service.
- Status: partial
- Evidence type: code, test
- Files: `crates/ck-bin/src/main.rs`, `crates/ck-transport-sse/src/lib.rs`
- Tests: `sse_bind_failure_fails_startup`, `sse_healthz_endpoint_works`
- Residual: graceful worker shutdown/join not implemented.
- Confidence: high

## Freshness (RESOLVED)
- Claim: `compute_freshness` is age-vs-TTL.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-app/src/lib.rs`
- Test: `compute_freshness_reflects_age_and_ttl`
- Confidence: high

## Verification gate
- Claim: Full gate green.
- Status: implemented
- Evidence type: test, config
- Commands: `cargo test --workspace` (213 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings` (clean); `cargo fmt --check` (clean).
- Notes: baseline was 198 tests; +8 harden, +7 Task 0 evidence = 213. `cargo fmt` touched only edited files. No new runtime dependencies (ck-artifacts/tempfile are dev-only for ck-app).
- Confidence: high

## Task 0 baseline evidence (audit-corrections plan)
- Claim: Every retained harden fix has a direct runtime/persistence regression test whose failure exposes the corrected branch.
- Status: implemented
- Evidence type: code, test
- Files: `scripts/run-exact-test.sh`; `crates/ck-storage-sqlite/src/lib.rs`; `crates/ck-app/src/lib.rs`; `crates/ck-app/Cargo.toml`; `crates/ck-bin/tests/cli.rs`
- Tests (12 gate exact-tests, all green):
  - `tests::persisted_enums_roundtrip_every_domain_variant` (ck-storage-sqlite) — persists Sse/Pinned/Compacted/DependencyChange/Cancelled through real tables; RED proven by removing `pinned` decoder arm.
  - `sse_reload_preserves_tool_and_corpus_authorization` (ck-bin cli) — tool deny + corpus deny + context-agent preservation + auth/service atomic swap + reject `require_auth=true` w/ no tokens at startup.
  - `tests::artifact_payload_is_compensated_on_metadata_failure` (ck-app) — real FileArtifactStore + injectable upsert failure; asserts payload+sidecar deleted; RED proven by commenting out `delete_artifact_payload`.
  - `json_logging_records_success_and_failure` (ck-bin cli) — JSON stderr for success+error tool telemetry.
  - `json_logging_covers_sse_and_refresh_threads` (ck-bin cli) — JSON stderr from spawned SSE + refresh threads (proves `set_global_default`).
  - `sse_readyz_reflects_live_service` (ck-bin cli) — real HTTP GET /readyz returns `{"ready":true}`; /healthz ok.
  - `tests::compute_freshness_table` (ck-app) — half/exact/past/future/zero/negative TTL.
  - `tests::context_required_corpus_id_uses_authorization_arm` (ck-mcp) — pre-existing, retained.
  - `tests::context_required_rejects_path_traversal_outside_corpus_root` (ck-mcp) — pre-existing, retained.
  - `tests::refresh_does_not_accumulate_duplicate_documents` (ck-app) — pre-existing, retained.
  - `tests::validate_url_with_resolver_fails_closed_on_resolver_error` (ck-adapter-web) — pre-existing, retained.
  - `sse_bind_failure_fails_startup` (ck-bin cli) — pre-existing, retained.
- Notes: RED→GREEN mutation-verified for the two locally-mutable corrections (enum decoder, artifact compensation). Reload-preservation RED is covered by the pre-existing `sse_reload_preserves_tool_authorization` test (same invariant).
- Confidence: high

## Deferred-debt execution status
- Claim: All eight items in the harden report's open/deferred section have an implementation stage with a failing-first gate, but none has been implemented by the planning change.
- Status: not implemented
- Evidence type: config
- Files: `.agents/plans/DEV_PLAN_NEATCODE_DEFERRED_DEBT_2026-07-29.md`, `.agents/reports/neatcode-harden-2026-07-29.md:34-42`
- Notes: Dependency order is ACL resolver port → server policy; atomic writes → storage-port split → adapter wiring → scheduler ownership → graceful shutdown → config semantics.
- Confidence: high

## Full audit-correction plan
- Claim: Every correction in the NeatCode audit has a dependency-ordered implementation task constrained by SWE_SEED execution/proof ownership, SEA-Forge action authority, GodSpeed developmental authority, and Context Kernel retrieval ownership.
- Status: not implemented
- Evidence type: config
- Files: `.agents/plans/DEV_PLAN_NEATCODE_AUDIT_CORRECTIONS_2026-07-29.md`, `.agents/reports/neatcode-audit-2026-07-29.md:28-196`, `/home/sprime01/projects/godspeed_agent/docs/explanations/SWE_SEED_to_godspeed_agent.md`, `/home/sprime01/projects/godspeed_agent/docs/explanations/SEA_Forge_to_godspeed_agent.md`
- Notes: Plan includes RED/GREEN gates, generated-contract correction, signed SEA evidence verification, cross-repository CI, atomic writes, runtime wiring, lifecycle, config, and final conformance.
- Confidence: high

---

## Historical Evidence (2026-06 deployment-readiness era)

## Claim
- Claim: README expresses the combined value propositions of current docs, token-efficient reference retrieval, and durable session continuity.
- Status: implemented
- Evidence type: documentation
- Files: `README.md`
- Lines: sections `The Problem`, `What Context Kernel Does`, `Retrieval`, `Lineage`
- Notes: README copy was revised after inspecting upstream README material for Context7, Ref MCP, and Context Mode. It avoids presenting those projects as direct dependencies.
- Confidence: high

## Claim
- Claim: Project license metadata is aligned to Apache-2.0.
- Status: implemented
- Evidence type: config, documentation
- Files: `Cargo.toml`, `LICENSE`, `README.md`
- Lines: `Cargo.toml` workspace package license; root Apache 2.0 `LICENSE`; README License section
- Notes: `cargo metadata --no-deps --format-version 1` reports workspace crates with `license: "Apache-2.0"`. Search found no remaining MIT license claim.
- Confidence: high

## Claim
- Claim: Root README exists and follows the project voice guide and SEA-style structure.
- Status: implemented
- Evidence type: documentation
- Files: `README.md`, `voice-guide.md`
- Lines: full files
- Notes: README includes problem statement, capabilities, quick start, configuration, observability, architecture, status, development, release, and license sections. It explicitly marks OpenTelemetry exporter/metrics wiring as not implemented.
- Confidence: high

## Claim
- Claim: Web adapter supports conditional request headers (If-None-Match, If-Modified-Since) for incremental refresh.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-web/src/lib.rs`
- Lines: WebConfig struct (previous_etag, previous_last_modified fields), fetch_content method (conditional header injection), tests config_parsing_with_conditional_headers, config_parsing_conditional_headers_default_none, fetch_content_records_etag_and_last_modified_in_metadata
- Notes: ureq Request::set consumes self; reassignment pattern used. 304 handling already existed.
- Confidence: high

## Claim
- Claim: Milestone 10 hardening config is implemented with reload, retention, authorization, and observability sections.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-config/src/lib.rs`
- Lines: ReloadConfig, RetentionConfig, AuthorizationConfig, ObservabilityConfig; test `loads_milestone_10_hardening_config`
- Notes: Defaults are conservative: reload and retention disabled, authorization allowlists default to `*`, observability log format defaults to text.
- Confidence: high

## Claim
- Claim: Retention GC is implemented for SQLite metadata.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-core/src/ports/storage.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-app/src/lib.rs`
- Lines: StoragePort retention methods; SqliteStore delete methods; AppServices::run_retention_gc
- Notes: Tests cover preserving pinned events and active jobs while deleting eligible rows. Artifact payload files are not deleted.
- Confidence: high

## Claim
- Claim: Tool and corpus authorization is implemented for SSE MCP calls.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-core/src/ports/auth.rs`, `crates/ck-auth/src/lib.rs`, `crates/ck-mcp/src/lib.rs`, `crates/ck-transport-sse/src/lib.rs`
- Lines: AuthPort authorize methods; BearerAuth policy; McpService::call_tool_authorized; SSE handler passes bearer header.
- Notes: Unit tests cover denied tool and denied corpus. Stdio direct calls remain local/internal and bypass policy.
- Confidence: high

## Claim
- Claim: `export-schema` emits the live embedded SQLite schema.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-bin/src/main.rs`, `crates/ck-bin/tests/cli.rs`
- Lines: SqliteStore::schema_sql; Commands::ExportSchema; test `export_schema_prints_sqlite_schema`
- Notes: CLI test asserts core table and FTS table SQL appear in output.
- Confidence: high

## Claim
- Claim: Release install documentation and CI integration examples exist.
- Status: implemented
- Evidence type: config, code
- Files: `docs/install.md`, `docs/ci.md`, `.github/workflows/ci.yml`
- Lines: full files
- Notes: CI example runs fmt, clippy, tests, and release binary build.
- Confidence: high

## Claim
- Claim: Milestone 10 verification passed.
- Status: implemented
- Evidence type: test
- Files: workspace
- Lines: n/a
- Notes: `cargo test -p ck-config -p ck-auth -p ck-storage-sqlite -p ck-app -p ck-mcp -p ck-bin` passed. `cargo fmt --check && cargo clippy --workspace --all-targets && cargo test --workspace` passed.
- Confidence: high

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
- Timestamp/context: 2026-06-02 10:30 EDT, local workspace `/home/sprime01/projects/Context_Kernel_Service_MCP`
- Proof classification: release-gate-proof
- Notes: Test count increased from 65 to 153 after M7-M9 implementation.
- Confidence: high

## Claim
- Claim: ck-observability crate provides text/JSON tracing initialization.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-observability/src/lib.rs`, `crates/ck-observability/Cargo.toml`
- Notes: ObservabilityRuntimeConfig with log_format (text/json) and log_level. init_observability sets up tracing-subscriber. ObservabilityGuard for flush-on-drop. ck-bin uses ck_observability instead of direct tracing init.
- Confidence: high

## Claim
- Claim: ToolTelemetry DTO added to ck-mcp with tracing instrumentation in dispatch_tool.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-mcp/src/lib.rs`
- Notes: ToolTelemetry struct with tool_name, duration_ms, success, error fields. dispatch_tool records tracing::info! with telemetry data on every tool call.
- Confidence: high

## Claim
- Claim: ReloadableSseState enables SSE config reload without restart.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-transport-sse/src/lib.rs`
- Notes: ReloadableSseState wraps RwLock<Arc<SseState>>. SSE handlers read through the RwLock. ck-bin updates the reloadable state when config changes.
- Confidence: high

## Claim
- Claim: Artifact payload file GC deletes disk files during retention.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-core/src/ports/artifact.rs`, `crates/ck-core/src/ports/storage.rs`, `crates/ck-artifacts/src/lib.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-app/src/lib.rs`
- Notes: delete_artifact_payload on ArtifactPort, list_artifacts_older_than on StoragePort, run_retention_gc in AppServices orchestrates both metadata and payload deletion. RetentionGcReport includes artifact_payload_errors count.
- Confidence: high

## Claim
- Claim: Source scope filtering rejects private corpus in query_reference and supports source_scope filter in resolve_reference.
- Status: implemented
- Evidence type: code
- Files: `crates/ck-app/src/lib.rs`
- Notes: query_reference filters out private corpus results. resolve_reference accepts source_scope parameter for filtering.
- Confidence: high

## Claim
- Claim: Vector filtering by corpus_id, library_id, document_id is implemented in ck-vector.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-vector/src/lib.rs`
- Notes: chunk_embeddings table has corpus_id, library_id, document_id columns. search_chunks filters by these fields. remove_chunk_embeddings_by_document implemented. upsert_chunk_embedding_with_metadata stores filter columns.
- Confidence: high

## Claim
- Claim: SSRF DNS resolution uses DnsResolver trait with SystemDnsResolver and StaticResolver for tests.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-web/src/lib.rs`
- Notes: DnsResolver trait with resolve method. SystemDnsResolver uses std::net::ToSocketAddrs. StaticResolver for deterministic test assertions. validate_url_with_resolver checks resolved IPs against private ranges.
- Confidence: high

## Claim
- Claim: GitHub adapter errors on truncated tree responses.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-github/src/lib.rs`
- Notes: units_from_tree_response extracted as separate function. Returns error when truncated: true in tree API response. Test verifies error on truncated tree.
- Confidence: high

## Claim
- Claim: Web adapter supports conditional request headers for incremental refresh.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-web/src/lib.rs`
- Notes: WebConfig has previous_etag and previous_last_modified fields. fetch_content sends If-None-Match and If-Modified-Since headers. Tests verify conditional header injection and metadata recording.
- Confidence: high

## Claim
- Claim: ck-memory crate provides event retention classification and session retention policy.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-memory/src/lib.rs`, `crates/ck-memory/Cargo.toml`
- Notes: EventRetentionClass enum (Critical, Normal, Ephemeral). classify_event_kind maps event kinds to retention classes. SessionRetentionPolicy with max_age and max_events. ck-app re-exports from ck-memory.
- Confidence: high

## Claim
- Claim: Config templates exist for stdio, local, and SSE modes.
- Status: implemented
- Evidence type: config
- Files: `config/context-kernal.example.toml`, `config/context-kernal.local.example.toml`, `config/context-kernal.sse.example.toml`
- Notes: All three templates parse successfully (verified by ck-config test example_configs_parse).
- Confidence: high

## Claim
- Claim: Deployment smoke tests verify CLI commands and config templates.
- Status: implemented
- Evidence type: test
- Files: `crates/ck-bin/tests/cli.rs`
- Notes: 4 new tests: export_schema_prints_sqlite_schema, migrate_then_doctor_works, retention_gc_cli_works, example_configs_parse. Total CLI tests: 16.
- Confidence: high

## Claim
- Claim: Operations documentation exists for deployment, configuration, and security.
- Status: implemented
- Evidence type: docs
- Files: `docs/operations.md`, `docs/configuration.md`, `docs/security.md`, `docs/release.md`
- Notes: Covers deployment steps, full config reference, security model, and release process.
- Confidence: high

## Claim
- Claim: Release workflow exists as GitHub Actions YAML.
- Status: implemented
- Evidence type: config
- Files: `.github/workflows/release.yml`
- Notes: Builds release binary for Linux, creates GitHub release with binary artifact.
- Confidence: high

## Claim
- Claim: Final release gate passes — 186 tests, zero warnings, release binary builds and runs.
- Status: implemented
- Evidence type: test
- Files: workspace
- Reproducible command: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo build --release -p ck-bin`
- Result excerpt: `cargo test --workspace`: `test result: ok. 186 passed; 0 failed`; `cargo clippy`: zero warnings; `cargo build --release`: produces `target/release/context-kernal`
- Timestamp/context: 2026-06-02 deployment readiness session
- Proof classification: release-gate-proof
- Notes: Binary supports `migrate`, `doctor`, `export-schema`, and `serve` commands. All 18 deployment readiness tasks complete.
- Confidence: high
