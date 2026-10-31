# Work Log

## 2026-10-30 — EGM bootstrap: contract READMEs across the repo tree
- Task: Applied the `egm-bootstrap` skill (/home/sprime01/projects/elephant-goldfish-model/skills/egm-bootstrap/SKILL.md) to the repository: generated EGM-compliant `README.md` contract files (Overview/Interface/Invariants/Side Effects/Verification) for every pruned directory, then ran the `egm-drift-report` skill over the result. Documentation only; no code, schema, config, or CI changes; no git operations.
- Safety gate: 34+ directories discovered (>15); user approved full-tree bootstrapping with sequential execution (RAM-constrained machine, ~1.9GiB free at start; one agent at a time; no compilation).
- Execution: 8 sequential worker subagents (W1 ck-core tree + ck-config, W2 ck-app, W3 ck-mcp + ck-transport-stdio, W4 ck-storage-sqlite + ck-search-fts + ck-memory, W5 adapters/artifacts/auth, W6 ranker/vector/scheduler/embeddings/observability/transport-sse, W7 integration/ + scripts/, W8 ck-bin — W8 added after root compile caught that ck-bin had been dropped from the partition regroup). Root orchestrator wrote `crates/README.md` and the root README Blueprint block (Correction Policy: existing content preserved, `## System Overview` heading added).
- Deliverables: 48 contract files — root `README.md`, `crates/README.md`, 20 crate Hays, 20 `src/` Peanuts, 3 ck-core leaf Peanuts, `integration/` Hay + leaf Peanut, `scripts/` Peanut. Verification commands: `cargo test --workspace` (root/blueprints) and `cargo test -p <pkg>` per crate — documented, NOT run.
- Drift check (user-authorized): `.agents/reports/2026-10-30_EGM_DRIFT_REPORT.md`. Findings: ck-mcp/src contract missing public types `DomainIdentityError`/`EventEnvelope` (fixed in place); root template heading fixed; 0 broken links, 0 ungrounded citations, 53/53 specific claims verified, pub-symbol coverage scan clean after fix.
- Memory files updated: current_plan.md (pre-work), work_log.md, evidence.md, decisions.md, OBSERVED_DEBT.md, LESSONS.md, project_state.md, current_status.yml.
- Tests run: none (documentation-only change; cargo intentionally not executed per build-lock/RAM discipline). Verification was static: link/section/citation scripts + source greps.

## 2026-09-02 — Repository Documentation Architecture Knowledge System
- Task: Designed and implemented a self-contained technical knowledge system for Context Kernel fulfilling both newcomer and experienced engineer needs through progressive disclosure (Layers 0 to 8: Orientation, Mental Model, Architecture, Subsystems, Workflows, Explanation, Tutorials, How-To, Reference).
- Information Architecture:
  - Landing & Navigation: `docs/README.md` (Orientation, system in one picture, 8 concepts, representative journey, intent-based navigation), `docs/documentation-map.md` (Full page catalog with Diátaxis classes and prerequisites).
  - Foundations: `docs/mental-model.md` (Conceptual boundaries, state ownership, invariants), `docs/concepts.md` (Domain vocabulary).
  - Canonical Architecture: `docs/architecture.md` (Logical, runtime, dependency, data, control flow, security), `docs/source-map.md` (Symbol-level traceability to codebase).
  - Subsystems: 7 guides (`core-and-domain.md`, `storage-and-persistence.md`, `retrieval-and-ranking.md`, `source-adapters.md`, `application-orchestration.md`, `mcp-and-protocols.md`, `runtime-and-cli.md`).
  - Workflows: 5 execution traces (`service-lifecycle-and-startup.md`, `documentation-ingestion-and-refresh.md`, `reference-query-and-resolution.md`, `session-context-continuity.md`, `agentic-capability-loop.md`).
  - Explanation: 4 design rationale pages (`architectural-decisions.md`, `retrieval-and-ranking-design.md`, `retention-and-memory-model.md`, `known-limits-and-debt.md`).
  - Tutorials: 2 step-by-step guides (`01-first-steps.md`, `02-local-documentation-indexing.md`).
  - How-To Guides: 4 operational guides (`configure-sse-with-auth.md`, `manage-database-and-migrations.md`, `configure-semantic-retrieval.md`, `implement-a-source-adapter.md`).
  - Technical Reference: 4 reference manuals (`cli.md`, `configuration.md`, `mcp-tools.md`, `database-schema.md`), plus `docs/troubleshooting.md`.
- Verification: Static audit of all links, symbols, schemas, CLI commands, and error codes against repository source code and existing tests. Explicitly highlighted verified vs unwired adapters and tracked debt.
- Memory files updated: `work_log.md`, `decisions.md`, `evidence.md`, `project_state.md`, `current_status.yml`, `current_plan.md`.

## 2026-08-24 — AGENTS.md governance revision

- Task: Superseded the six-step Agent Workflow section with the Workflow-and-Permissions approach (investigate-first tooling order, ask-before gates for deps/schema/public interfaces/CI/deletions/milestone growth, spec-or-ADR duty on contract changes, diff review criteria, secrets hygiene, authority/sandbox rules, small-focused-commit discipline). Added the Handoff section (`current_status.yml` resumable state, OBSERVED_DEBT/LESSONS/open_questions upkeep, `just context-check`) including build-lock and artifact discipline. Preserved evidence-priority ordering as a subsection of Implementation Verification Rules. Extended required memory files with `.agents/LESSONS.md` and `.agents/current_status.yml`, added their templates, and extended the completion checklist.
- Files changed: `AGENTS.md`, `.agents/current_status.yml` (new), `.agents/LESSONS.md` (new); removed stale empty `.agents/CURRENT_STATUS.md`
- Tests run: None (governance/documentation only)
- Results: No competing workflow guidance remains; old Agent Workflow fully superseded
- Notes: `just` binary is installed but the repository has no justfile defining a `context-check` recipe yet

## 2026-08-24 — Memory migration: legacy `.agent` directory merged into `.agents`

- Task: Consolidated duplicate agent-memory directories. Migrated all memory files from the legacy `.agent` directory into `.agents`, following its conventions (plans/, reports/, skills/, specs/, root memory files); merged diverged histories; updated every repository reference from the legacy path to `.agents`; deleted the legacy directory. (This entry names the legacy path once, as a historical record; it is not a live reference.)
- Files changed: `.agents/work_log.md`, `.agents/project_state.md`, `.agents/decisions.md`, `.agents/evidence.md`, `.agents/open_questions.md`, `.agents/OBSERVED_DEBT.md`, `.agents/current_plan.md`, `.agents/plans/2026-06-02_deployment_readiness_completed.md`, `AGENTS.md`, `.github/copilot-instructions.md`, `README.md`, `docs/plans/2026-06-02-deployment-readiness.md`
- Tests run: None (memory/documentation migration only). Verified by repo-wide grep that no live references to the legacy directory remain.
- Results: Single canonical `.agents/` memory directory. Newer 2026-07-29/30 audit-era entries lead each merged file; 2026-06 era history preserved beneath. Completed 2026-06-02 deployment-readiness plan archived to `.agents/plans/2026-06-02_deployment_readiness_completed.md`.
- Notes: Empty duplicate `.agents/product_marketing.md` removed in favor of populated `product-marketing.md`.

## 2026-07-30 — Tasks 1 and 4 (audit-corrections plan)
- Task: Implement Task 1 (SEA/SWE context authority contract correction) and Task 4 (atomic writes + stable IDs + FTS rebuild) from `.agents/plans/2026-07-29_audit_corrections.md`.
- **Task 1 — cross-repo contract correction:**
  - SEA source (`agentic_capability_loop.sea`): removed `ContextAuthorized` from CK ownership; moved `EvidenceRecorded` from SEA-Forge to SWE_SEED; updated `PrivateCorpusRequiresExplicitScope` policy to require prior signed SEA `AuthorityChecked` allow (CK no longer decides authority).
  - `generate_contracts.py` FIELD_TYPES: removed `ContextAuthorized`; added `AuthorityChecked` (`key_id`, `signed_payload`, `signature`); dropped `is_private`/`scope_claim` from `ContextRequired`; replaced `authorized: bool` with `authority_decision_id`/`authority_reference` in `ContextPacketCreated`.
  - Regenerated all artifacts (Rust, Python, TypeScript, JSON Schema). Deleted stale `ContextAuthorized.schema.json`. Verified zero `ContextAuthorized` in generated output.
  - CK Rust (`ck-mcp/src/agentic_capability_loop.rs`): removed `ContextAuthorizedPayload`, `AuthorizationResult`, `ContextAgent` trait; `AclContextAgent::handle_context_required` now takes `ContextRequiredPayload` + optional authority references and returns only `ContextPacketCreatedPayload`.
  - CK MCP handler (`ck-mcp/src/lib.rs`): no longer constructs authority events; passes authority references through from caller arguments.
  - CK tests (`ck-mcp/tests/acl_context_agent.rs`, `ck-mcp/src/lib.rs`, `ck-bin/tests/cli.rs`): updated to new API — no `is_private`/`scope_claim`/`authorized` assertions.
  - SWE_SEED client (`context_client.rs`): `context_required()` now takes `authority_decision_id`/`authority_reference` instead of `is_private`/`scope_claim`; `ContextPacket` no longer carries `authorization` field or `authorized()` method.
  - SWE_SEED tests (`context_kernel_client.rs`): updated to new API.
- **Task 4 — data integrity (partial: fetch-before-delete + stable IDs + FTS rebuild):**
  - Fetch-before-delete: `ingest_source` now discovers, fetches, normalizes, and chunks an entire snapshot BEFORE deleting previous rows. An adapter/fetch failure leaves the old snapshot untouched.
  - Stable document IDs: added `stable_document_id(source_id, external_id, uri)` using deterministic hashing; overwrites normalizer-generated random IDs before chunking.
  - `rebuild-index` CLI command: `SqliteStore::rebuild_fts_index()` clears and repopulates `chunks_fts`, `session_events_fts`, `library_aliases_fts` from canonical tables.
  - Tests: `stable_document_id_is_deterministic`, `document_identity_survives_discovery_reorder_and_insertion` (ck-app), `rebuild_index_repairs_search_from_canonical_rows` (ck-bin cli).
  - NOT DONE: full transactional atomicity (single-connection canonical+FTS writes); failure-injection rollback tests. These require a port restructure (new `IndexWritePort`) that is Task 4 steps 5-6.
- Files changed (Context Kernel): `crates/ck-app/src/lib.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-mcp/src/agentic_capability_loop.rs`, `crates/ck-mcp/src/lib.rs`, `crates/ck-mcp/tests/acl_context_agent.rs`, `crates/ck-bin/src/main.rs`, `crates/ck-bin/tests/cli.rs`.
- Files changed (SEA): `docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea`, `docs/specs/domains/agentic_capability_loop/tools/generate_contracts.py`, regenerated AST/IR/manifest/rust/python/typescript/schemas.
- Files changed (SWE_SEED): `crates/swe-seed-core/src/federation/context_client.rs`, `crates/swe-seed-core/tests/context_kernel_client.rs`.
- Tests run: `cargo test --workspace` → **215 passed, 0 failed**. `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --check` clean. SWE_SEED `cargo build` + `cargo test --no-run` compile clean.
- Notes: Tasks 2 (SEA evidence verifier + app-owned retrieval) and 3 (GodSpeed canonical proposal migration) remain not started — they depend on Task 1 completing (done) and are larger architectural changes.

## 2026-07-30 — Task 0 (audit-corrections plan): baseline evidence
- Task: Implement Task 0 of `.agents/plans/2026-07-29_audit_corrections.md` — add direct runtime/persistence regression tests for every retained harden fix so residual structural work (Tasks 1-11) cannot regress them.
- Method: Read full plan + all `.agents/` memory files; recorded baseline (206 tests, clippy/fmt green); added 7 new evidence tests across ck-storage-sqlite, ck-app, ck-bin; proved RED→GREEN on the two tests whose corrected branch is locally mutable (enum decoder, artifact compensation); kept all 9 existing harden files untouched.
- Files changed:
  - `scripts/run-exact-test.sh` (new) — asserts exactly one discovered test, runs `--exact --nocapture`; zero-test cargo success is a failed gate.
  - `crates/ck-storage-sqlite/src/lib.rs` — `tests::persisted_enums_roundtrip_every_domain_variant`: persists every SessionTransport/ContinuationMode/SessionStatus (incl. Sse/Pinned/Compacted via SQL UPDATE+get_session) and every JobTrigger/JobStatus (incl. DependencyChange/Cancelled) through real tables, reads typed values back. RED proven by removing the `pinned` decoder arm.
  - `crates/ck-app/Cargo.toml` — added dev-dependencies `ck-artifacts`, `tempfile`.
  - `crates/ck-app/src/lib.rs` — `tests::artifact_payload_is_compensated_on_metadata_failure`: real `FileArtifactStore` + `TestStorage.fail_upsert_artifact` toggle; asserts payload+sidecar deleted from `artifacts/` subdir and exactly one upsert call. RED proven by commenting out `delete_artifact_payload`. Also `tests::compute_freshness_table`: half/exact/past/future/zero/negative TTL coverage. Added `WorkingSessionMemory` + `test_app_with_artifact_store` helpers; added `fail_upsert_artifact`/`artifact_upsert_calls` fields to `AppTestState`; made `TestStorage::upsert_artifact` injectable.
  - `crates/ck-bin/tests/cli.rs` — `sse_reload_preserves_tool_and_corpus_authorization` (tool deny + corpus deny + context-agent preservation + auth/service atomic swap + reject `require_auth=true` w/ no tokens at startup); `sse_readyz_reflects_live_service` (real HTTP GET /readyz + /healthz); `json_logging_records_success_and_failure` (JSON stderr for success+error tool telemetry); `json_logging_covers_sse_and_refresh_threads` (JSON stderr from spawned SSE + refresh threads). Added `http_get` + `wait_until_listening_with_child` helpers.
- Tests run: `cargo test --workspace` → **213 passed, 0 failed** (was 206; +7 new). `cargo clippy --workspace --all-targets -- -D warnings` clean. `cargo fmt --check` clean. All 12 Task 0 gate exact-tests pass.
- Results: Task 0 baseline complete. Existing harden fixes now have direct regression evidence; residual audit items (Tasks 1-11) remain explicitly partial/open and unchanged.
- Notes: RED→GREEN mutation verified for the two locally-mutable corrections (enum decoder, artifact compensation). The reload-preservation RED is already covered by the pre-existing `sse_reload_preserves_tool_authorization` test (same invariant); the new comprehensive test extends coverage to corpus denial + context-agent preservation + no-tokens startup rejection. Cross-repo work (Tasks 1, 3, 11) not started.

## 2026-07-29 — Full audit correction plan
- Task: Produced an exact-template implementation plan for all NeatCode audit corrections, considering the controlling SWE_SEED and SEA-Forge relationship documents in the GodSpeed-Agent workspace.
- Files changed: `.agents/plans/DEV_PLAN_NEATCODE_AUDIT_CORRECTIONS_2026-07-29.md`, `.agents/current_plan.md`, `.agents/work_log.md`, `.agents/project_state.md`, `.agents/evidence.md`, `.agents/decisions.md`.
- Tests run: None; planning-only change. Inspected all audit findings, current post-harden code, SEA source/generated contracts, SWE_SEED client/tests, GodSpeed canonical storage/proposal paths, and CK integration fixtures.
- Results: Eleven dependency-ordered implementation tasks plus baseline retention evidence cover all 12 findings and the architecture correction without assigning SEA, SWE_SEED, or GodSpeed authority to Context Kernel.
- Notes: The requested underscore audit filename resolves to the repository's canonical hyphenated `.agents/reports/neatcode-audit-2026-07-29.md`; explanation docs live in `/home/sprime01/projects/godspeed_agent/docs/explanations/`.

## 2026-07-29
- Task: Produced a repository-grounded execution plan for all eight residual/deferred NeatCode harden items.
- Files changed: `.agents/plans/DEV_PLAN_NEATCODE_DEFERRED_DEBT_2026-07-29.md`, `.agents/current_plan.md`, `.agents/work_log.md`, `.agents/project_state.md`, `.agents/evidence.md`, `.agents/decisions.md`.
- Tests run: None; planning-only change. Inspected current traits, adapters, transaction paths, runtime composition, worker lifecycle, config structs, tests, manifests, and documentation claims.
- Results: Dependency order established as ACL port → server policy; atomic writes → StoragePort split → adapter wiring → scheduler ownership → graceful shutdown → config semantics. Each stage includes a failing-first gate and delegation authority.
- Notes: No deferred item was marked resolved; execution has not started.

## 2026-07-29 (audit)
- Task: Deep NeatCode audit of the repository.
- Files changed: created `.agents/reports/neatcode-audit-2026-07-29.md` + initialized `.agents/` memory; no production source changed.
- Results: 198 tests passed; clippy clean; `cargo fmt --check` failed on committed formatting in `ck-bin`/`ck-mcp`.
- Notes: 4 S1, 4 S2, 3 S3, 1 S4 findings.

## 2026-07-29 (harden: implement audit corrections)
- Task: Implement the audit corrections + run the harden pass.
- Method: 5 subagent implementers on disjoint crates (ck-storage-sqlite, ck-adapter-web, ck-mcp, ck-app, ck-observability) with verbatim specs + failing-first tests; interdependent ck-bin/ck-transport-sse lifecycle work done directly.
- Files changed:
  - `crates/ck-storage-sqlite/src/lib.rs` — added 5 enum decoder arms + round-trip test.
  - `crates/ck-adapter-web/src/lib.rs` — SSRF fail-closed on DNS Err/empty resolution; corrected 4 bug-ratifying tests; +2 fail-closed tests.
  - `crates/ck-mcp/src/lib.rs` + `agentic_capability_loop.rs` — `context_required` corpus allowlist arm + canonicalize/contain path check; +2 tests.
  - `crates/ck-app/src/lib.rs` — refresh replaces source docs (no accumulation), payload compensation, real freshness score, TestStorage fixes; +2 tests.
  - `crates/ck-observability/src/lib.rs` — `set_global_default` (all threads log).
  - `crates/ck-bin/src/main.rs` — reload preserves auth+context agent; observability wired from config; SSE listener bound in main (bind failure fails startup).
  - `crates/ck-bin/tests/cli.rs` — `sse_bind_failure_fails_startup`, `sse_reload_preserves_tool_authorization` (+helpers).
  - `crates/ck-transport-sse/src/lib.rs` — accepts pre-bound listener; `/readyz` probes `get_status`.
- Tests run: `cargo test --workspace` (206 passed, 0 failed); `cargo clippy --workspace --all-targets -- -D warnings` (clean); `cargo fmt --check` (clean).
- Results: all green. +8 net new tests.
- Notes: deferred (documented in harden report + OBSERVED_DEBT) — FTS transactional atomicity, is_private server-validation, GitHub/web wiring, graceful shutdown, remaining dead config fields, scheduler ownership, broad StoragePort.

## 2026-10-13 — Devin configuration
- Task: Inspected code, tests, workflow, spec and memory paths. Added low-effort DeepWiki and Git-backed Rust blueprint. Corrected agent memory path instructions.
- Files: .devin/, AGENTS.md, .github/copilot-instructions.md, .agents/ continuity files.
- Tests: Configuration/static checks only; no Rust tests or Devin snapshot run.
- Result: Repository configuration submitted for GitHub review. External Devin execution pending.


## 2026-06-02 18:00 EDT

- Task: Updated README value proposition after reading upstream Context7, Ref MCP, and Context Mode READMEs.
- Files changed: `README.md`, `.agents/work_log.md`, `.agents/evidence.md`
- Tests run: Not run; documentation-only change.
- Results: README now describes current/version-aware documentation retrieval, token-efficient reference reading, and persistent session continuity as first-class value props.
- Notes: Upstream projects are reflected as product patterns, not named as dependencies or direct implementations.

## 2026-06-02 18:00 EDT

- Task: Fixed license discrepancy so repository metadata matches the Apache-2.0 root `LICENSE`.
- Files changed: `Cargo.toml`, `README.md`, `.agents/work_log.md`, `.agents/evidence.md`
- Tests run: `cargo metadata --no-deps --format-version 1`; `rg -n "MIT|license =|Apache-2.0|Apache License|MPL" Cargo.toml crates README.md docs .github .agents`
- Results: Cargo metadata reports workspace crates with `license: "Apache-2.0"`; scan found no remaining MIT license claim.
- Notes: All crate manifests inherit `license.workspace = true`, so changing workspace package license aligns all crates.

## 2026-06-02 18:00 EDT

- Task: Created root `README.md` using `voice-guide.md` and the SEA README structure as references.
- Files changed: `README.md`, `.agents/work_log.md`, `.agents/evidence.md`
- Tests run: Not run; documentation-only change.
- Results: README added with project overview, problem statement, capabilities, quick start, architecture, status, development rules, release notes, and known limits.
- Notes: README avoids claiming OTEL exporter support because code only implements text/JSON tracing and OTEL endpoint detection.

## 2026-06-02 18:00 EDT

- Task: Updated all `.agents/` memory files to reflect completed deployment readiness state.
- Files changed: `.agents/project_state.md`, `.agents/current_plan.md`, `.agents/evidence.md`, `.agents/decisions.md`, `.agents/open_questions.md`, `.agents/work_log.md`
- Tests run: None (documentation-only update)
- Results: All memory files now accurately reflect the completed 18-task deployment readiness plan.
- Notes: This was the final step — updating memory files to match actual repository state after all deployment readiness work was completed.

## 2026-06-02 17:00 EDT

- Task: Deployment readiness tasks 14-18 — config templates, smoke tests, operations docs, release workflow, final release gate.
- Files changed: `config/context-kernal.example.toml`, `config/context-kernal.local.example.toml`, `config/context-kernal.sse.example.toml`, `crates/ck-bin/tests/cli.rs` (4 new tests), `docs/operations.md`, `docs/configuration.md`, `docs/security.md`, `docs/release.md`, `.github/workflows/release.yml`
- Tests run: `cargo test --workspace` (186 tests pass), `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo build --release -p ck-bin`
- Results: All pass. Release binary `context-kernal` supports `migrate`, `doctor`, `export-schema`, and `serve` commands.
- Notes: Final release gate verified. All 18 deployment readiness tasks complete.

## 2026-06-02 16:00 EDT

- Task: Deployment readiness tasks 11-13 — ck-memory crate, Context7/PDF adapter scope decisions.
- Files changed: `crates/ck-memory/Cargo.toml`, `crates/ck-memory/src/lib.rs`, `Cargo.toml` (added workspace member), `crates/ck-app/Cargo.toml` (added ck-memory dep), `crates/ck-app/src/lib.rs` (re-exports)
- Tests run: `cargo test -p ck-memory`, `cargo test --workspace`
- Results: ck-memory tests pass. Context7 and PDF adapters removed from v1 scope.
- Notes: ck-memory provides EventRetentionClass, classify_event_kind, SessionRetentionPolicy.

## 2026-06-02 15:00 EDT

- Task: Deployment readiness tasks 5-10 — artifact payload GC, source scope filtering, vector filtering, SSRF DNS resolution, GitHub truncated tree, web incremental refresh.
- Files changed: `crates/ck-core/src/ports/artifact.rs`, `crates/ck-core/src/ports/storage.rs`, `crates/ck-core/src/ports/vector_search.rs`, `crates/ck-artifacts/src/lib.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-vector/src/lib.rs`, `crates/ck-adapter-web/src/lib.rs`, `crates/ck-adapter-github/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-scheduler/src/lib.rs`
- Tests run: `cargo test --workspace`
- Results: All tests pass. Artifact payload GC, source scope filtering, vector filtering, SSRF DNS resolution, truncated tree detection, and conditional request headers all implemented.
- Notes: StoragePort and ArtifactPort trait changes required updating all implementors (SqliteStore, MockStorage, TestStorage, NoopArtifactStore).

## 2026-06-02 14:30 EDT

- Task: Deployment readiness tasks 1-4 — copilot-instructions, ck-observability, ToolTelemetry, ReloadableSseState.
- Files changed: `.github/copilot-instructions.md`, `crates/ck-observability/Cargo.toml`, `crates/ck-observability/src/lib.rs`, `Cargo.toml` (added workspace member), `crates/ck-mcp/src/lib.rs`, `crates/ck-transport-sse/src/lib.rs`, `crates/ck-bin/Cargo.toml`, `crates/ck-bin/src/main.rs`
- Tests run: `cargo test --workspace`
- Results: All tests pass. ck-observability provides text/JSON tracing init. ToolTelemetry instruments dispatch_tool. ReloadableSseState enables SSE config reload.
- Notes: ck-bin now uses ck_observability::init_observability instead of direct tracing init.

## 2026-06-02 14:00 EDT

- Task: Task 10 — Add web incremental refresh hints (conditional request headers).
- Files changed: `crates/ck-adapter-web/src/lib.rs`
- Tests run: `cargo test -p ck-adapter-web` (41 tests pass)
- Results: All 41 tests pass (3 new). Added `previous_etag` and `previous_last_modified` fields to `WebConfig`, conditional `If-None-Match` / `If-Modified-Since` headers in `fetch_content`, and 3 new tests.
- Notes: ureq `Request::set` consumes self and returns `Self`; used reassignment pattern for conditional header injection.

## 2026-06-02 10:05 EDT

- Task: Created deployment readiness plan for remaining partials and release blockers.
- Files changed: `docs/plans/2026-06-02-deployment-readiness.md`, `.agents/current_plan.md`.
- Tests run: Not run; planning-only change.
- Results: Plan created with release standard, TDD tasks, file targets, and validation commands.
- Notes: Plan covers OTEL, SSE reload, artifact payload GC, source-scope authz, vector cleanup/filtering, adapter hardening, `ck-memory`, adapter scope decisions, config templates, smoke tests, docs, and release workflow.

## 2026-06-02 10:05 EDT

- Task: Implemented Milestone 10 hardening and packaging.
- Files changed: `.agents/current_plan.md`, `.agents/project_state.md`, `.agents/work_log.md`, `.agents/decisions.md`, `.agents/open_questions.md`, `.agents/evidence.md`, `.github/workflows/ci.yml`, `crates/ck-core/src/ports/auth.rs`, `crates/ck-core/src/ports/storage.rs`, `crates/ck-config/src/lib.rs`, `crates/ck-auth/src/lib.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-mcp/src/lib.rs`, `crates/ck-transport-sse/src/lib.rs`, `crates/ck-bin/src/main.rs`, `crates/ck-bin/tests/cli.rs`, `crates/ck-scheduler/src/lib.rs`, `docs/schema.md`, `docs/mcp-contracts.md`, `docs/install.md`, `docs/ci.md`.
- Tests run: `cargo test -p ck-config -p ck-auth -p ck-storage-sqlite -p ck-app -p ck-mcp -p ck-bin`; `cargo fmt --check && cargo clippy --workspace --all-targets && cargo test --workspace`.
- Results: All focused tests passed; full fmt/clippy/workspace test gate passed.
- Notes: Implemented typed M10 config, retention GC, authz policy, schema export, startup retention application, stdio-loop file config reload, SSE policy enforcement, install docs, CI example. OTEL endpoint is represented in config and docs, but no OTEL exporter crate is wired.

## 2026-06-02 10:30 EDT

- Task: Implement Milestones 7-9 from Context_Kernel_Service_Plan.md
- Files changed:
  - `Cargo.toml` — added 4 new workspace members (ck-embeddings, ck-vector, ck-adapter-github, ck-adapter-web) and new dependencies (ureq, url)
  - `crates/ck-core/src/ports/` — added embedding.rs (EmbeddingPort), vector_search.rs (VectorSearchPort), updated ranker.rs (RankContext with intent/session_id/debug), updated mod.rs
  - `crates/ck-ranker/src/lib.rs` — rewrote with intent-specific weight profiles, session affinity, de-duplication, debug mode, semantic score integration (15 tests)
  - `crates/ck-embeddings/` — NEW crate with NoopEmbeddingProvider and OpenAIEmbeddingProvider (5 tests)
  - `crates/ck-vector/` — NEW crate with SqliteVectorStore (brute-force cosine similarity) (14 tests)
  - `crates/ck-adapter-github/` — NEW crate with GitHubSourceAdapter and GitHubNormalizer (18 tests)
  - `crates/ck-adapter-web/` — NEW crate with WebSourceAdapter, SSRF protections, HTML-to-text, WebNormalizer (34 tests)
  - `crates/ck-config/src/lib.rs` — added EmbeddingsConfig section with provider/api_base/api_key/model/dimensions
  - `crates/ck-app/src/lib.rs` — added optional EmbeddingPort/VectorSearchPort, semantic retrieval in query_reference, intent-aware ranking in all use cases
  - `crates/ck-bin/src/main.rs` — wires embeddings/vector when configured, passes config to wire_app
  - `crates/ck-bin/Cargo.toml` — added ck-embeddings and ck-vector dependencies
- Tests run: `cargo test --workspace` (153 tests pass), `cargo fmt --check` (clean), `cargo clippy --workspace` (zero warnings)
- Results: All milestones 7-9 acceptance criteria met. Test count increased from 65 to 153.
- Notes: Used parallel subagent delegation for 5 independent crate implementations (ck-ranker, ck-embeddings, ck-vector, ck-adapter-github, ck-adapter-web).

## 2026-06-02 12:30 EDT

- Task: Milestone 9 — Create ck-adapter-github crate (GitHub repo source adapter)
- Files changed: `Cargo.toml` (added workspace member), `crates/ck-adapter-github/Cargo.toml`, `crates/ck-adapter-github/src/lib.rs`
- Tests run: `cargo check -p ck-adapter-github` (clean, zero warnings), `cargo test -p ck-adapter-github` (18 tests pass)
- Results: Crate compiles and all 18 unit tests pass. Implements GitHubSourceAdapter (SourceAdapter trait, kind="github_repo"), GitHubNormalizer (Normalizer trait), and config parsing for static file lists and tree API discovery.
- Notes: Uses ureq for HTTP requests (already in workspace deps). Supports both static `files` array in config and dynamic tree API discovery. Token supports `$ENV_VAR` syntax.

## 2026-06-02 08:06 EDT

- Task: Addressed 11 review findings from `tmp/cr.md` and struck completed review text.
- Files changed: `tmp/cr.md`, `Cargo.toml`, `Cargo.lock`, `.agents/current_plan.md`, `.agents/project_state.md`, `.agents/work_log.md`, `.agents/decisions.md`, `.agents/open_questions.md`, `.agents/evidence.md`, `crates/ck-core/src/ports/storage.rs`, `crates/ck-core/src/ports/scheduler.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-scheduler/src/lib.rs`, `crates/ck-ranker/src/lib.rs`, `crates/ck-config/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-auth/Cargo.toml`, `crates/ck-auth/src/lib.rs`, `crates/ck-bin/src/main.rs`.
- Tests run: initial red runs for `ck-config`, `ck-auth`, `ck-ranker`, `ck-app`, `ck-storage-sqlite`; focused green runs for `ck-config`, `ck-auth`, `ck-ranker`, `ck-app`, `ck-storage-sqlite`, `ck-scheduler`; final `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace`.
- Results: Final workspace validation passed; `cargo test --workspace` reports 65 passing tests, `cargo fmt --check` clean, `cargo clippy --workspace` finished with zero warnings.
- Notes: `.github/copilot-instructions.md` is required by AGENTS.md but missing in this checkout. Review references to `source.corpus_id` and `SourceHealth::Failing` were not valid against current code; implementation used `corpus_sources` and `SourceHealth::Degraded`.

## 2026-06-01 15:00 EDT

- Task: Started Milestone 0 implementation from plan.
- Files changed: `.gitignore` in parent worktree committed before feature work; initialized `.agents/` memory in feature worktree.
- Tests run: None yet.
- Results: Repository has no Rust workspace yet.
- Notes: `.github/copilot-instructions.md` is missing.

## 2026-06-01 15:06 EDT

- Task: Implemented Milestone 0 Rust skeleton.
- Files changed: `.gitignore`, `Cargo.toml`, `Cargo.lock`, `crates/ck-core`, `crates/ck-config`, `crates/ck-storage-sqlite`, `crates/ck-mcp`, `crates/ck-transport-stdio`, `crates/ck-bin`, `context-kernal.toml.example`, `docs/schema.md`, `docs/mcp-contracts.md`, `.agents/*`.
- Tests run: `cargo test --workspace`; `cargo fmt --check`; manual stdio smoke command through `cargo run`.
- Results: All tests and format check passed. Smoke command returned `get_status` JSON with SQLite FTS5 enabled.
- Notes: Work is in `.worktrees/implement-spec` on branch `implement-spec`.

## 2026-06-01 15:09 EDT

- Task: Expanded `.gitignore` for project runtime and development artifacts.
- Files changed: `.gitignore`, `.agents/work_log.md`.
- Tests run: `git check-ignore -v` against ignored runtime/build paths and non-ignored source/docs/memory paths.
- Results: `target/`, local SQLite files, local config, `.env.*`, runtime data directories, and logs are ignored; `Cargo.lock`, `.agents/`, examples, docs, and source files are not ignored.

## 2026-06-01 (Session 2) — Milestones 1-3 Implementation

- Task: Implement Milestones 1-3 from Context_Kernel_Service_Plan.md.
- Files changed:
  - `Cargo.toml` — added 5 new workspace members (ck-app, ck-search-fts, ck-artifacts, ck-ranker, ck-adapter-filesystem) and new dependencies (ulid, sha2, hex)
  - `crates/ck-core/` — expanded with entities (Source, Corpus, LibraryIdentity, Document, Chunk, Session, SessionEvent, Artifact, RefreshJob, RetrievalQuery, RetrievalResult, ScoredItem, etc.), ports (StoragePort, SessionMemoryPort, ArtifactPort, LexicalSearchPort, SourceAdapter, Normalizer, Chunker, RankerPort), types (enums for RetrievalMode, SessionTransport, ContinuationMode, SessionStatus, JobTrigger, JobStatus, QueryIntent, RecallScope)
  - `crates/ck-storage-sqlite/` — expanded with full StoragePort and SessionMemoryPort implementations (CRUD for all entities)
  - `crates/ck-search-fts/` — new crate implementing LexicalSearchPort with FTS5/BM25
  - `crates/ck-artifacts/` — new crate implementing ArtifactPort with filesystem storage
  - `crates/ck-ranker/` — new crate implementing RankerPort with lexical ranking
  - `crates/ck-adapter-filesystem/` — new crate with FilesystemSourceAdapter, MarkdownNormalizer, HeadingChunker
  - `crates/ck-app/` — new crate with use cases (StoreContext, RecallContext, IngestSource, QueryReference, ResolveReference, GetStatus)
  - `crates/ck-mcp/` — expanded with all 7 MCP tools + 3 compatibility aliases
  - `crates/ck-bin/` — expanded to wire all new crates
  - `crates/ck-bin/tests/cli.rs` — expanded with 6 new integration tests
- Tests run: `cargo test --workspace` (37 tests pass), `cargo fmt --check` (clean), `cargo clippy --workspace` (zero warnings)
- Results: All milestones 1-3 acceptance criteria met.
- Notes: Used parallel subagent delegation for independent crate implementations.

## 2026-06-01 (Session 3) — Milestones 4-6 Implementation

- Task: Implement Milestones 4-6 from Context_Kernel_Service_Plan.md.
- Files changed:
  - `Cargo.toml` — added 3 new workspace members (ck-transport-sse, ck-auth, ck-scheduler) and new dependencies (axum, tokio, tower, hyper, http-body-util)
  - `crates/ck-core/` — added SourceHealth enum, AuthPort trait, SchedulerPort trait, 5 new StoragePort methods
  - `crates/ck-ranker/src/lib.rs` — rewrote with hybrid scoring (FTS 0.7 + trust 0.2 + freshness 0.1)
  - `crates/ck-storage-sqlite/src/lib.rs` — implemented all new StoragePort methods (source health, stale detection, chunk+document join, job counting)
  - `crates/ck-scheduler/` — new crate with RefreshScheduler (coalescing, job lifecycle, stale detection)
  - `crates/ck-auth/` — new crate with BearerAuth (token validation, no-auth mode)
  - `crates/ck-transport-sse/` — new crate with axum HTTP server (POST /mcp, GET /healthz, GET /readyz, GET /api/v1/status)
  - `crates/ck-config/src/lib.rs` — added McpConfig, SseConfig, AuthConfig, SchedulerConfig
  - `crates/ck-app/src/lib.rs` — added refresh_source use case, enriched query_reference with citations/excerpts/trust/freshness
  - `crates/ck-mcp/src/lib.rs` — added refresh_source tool handler
  - `crates/ck-bin/src/main.rs` — rewrote with SSE transport, auth wiring, dual transport, periodic refresh thread
  - `crates/ck-bin/tests/cli.rs` — added 3 new integration tests (refresh_source, SSE mode, citations)
- Tests run: `cargo test --workspace` (57 tests pass), `cargo fmt --check` (clean), `cargo clippy --workspace` (zero warnings)
- Results: All milestones 4-6 acceptance criteria met.

## 2026-10-07 — CEP context_bundle architectural completion (GodSpeed boundary)

- Objective: make CK the bounded representation compiler: typed Context
  Requirement -> real retrieval pipeline -> CEP-0008 `godspeed.context_bundle`
  -> SWE_SEED/SEA-Forge/RealityTrace lineage -> settlement-backed calibration.
- Changed (CK):
  - `crates/ck-core/src/types/context.rs` (new): shared `RetrievalCompleteness`,
    `NoContextReason` (incl. `corpus_scope_unresolved`, `budget_exhausted`,
    `no_fresh_content`), omission codes, `verify_world_ref` gate.
  - `crates/ck-app/src/context.rs` (new): typed `ContextRequirement`,
    `ContextAccess` (accessible/inaccessible/requires_scope — never action
    authority), `ContextResolution`, `AppServices::resolve_context` (real
    query_reference pipeline; POL-ACL-002 strict private scope; budgets;
    freshness; required-concept accounting; calibration hook).
  - `crates/ck-app/src/context_effect.rs` (new): settlement-backed Context
    Effect ledger (append-only JSONL, evidence-gated ingestion, world-scoped
    prospective demotion) + `AppServices::ingest_context_effects`.
  - `crates/ck-app/src/lib.rs`: kind-selected source adapters
    (`with_source_adapter_kind`), visibility-preserving ingestion,
    `with_context_effect_store`, `corpus()`.
  - `crates/ck-storage-sqlite/src/lib.rs`: `upsert_source`/`upsert_corpus` now
    UPSERT (INSERT OR REPLACE cascade-deleted the index).
  - `crates/ck-ranker/src/lib.rs`: compact score_breakdown keeps participating
    components (fts/trust_score/freshness/semantic/calibration).
  - `crates/ck-search-fts/src/lib.rs`: empty query = bounded browse.
  - `crates/ck-mcp`: `AclContextAgent` resolves through `resolve_context`
    (ad hoc substring retrieval deleted); `handle_context_required` emits the
    canonical CEP `context_bundle` (integrity-stamped, truthful
    complete/partial/none) + deprecated dual-read legacy fields + `access`;
    `ingest_context_effects` tool; `cep_bundle` projector with
    `canonical_json` (float-normalized) + `content_hash` +
    `verify_envelope_integrity` + `validate_context_bundle`.
  - `crates/ck-bin`: `LearningProposalsAdapter` wiring, `CK_CONTEXT_EFFECT_LEDGER`.
  - docs/mcp-contracts.md + integration Python adapter: authorization language
    retired (`ContextAccessEvaluated`), canonical bundle documented.
  - Tests: `context_pipeline.rs` (12), `context_bundle_conformance.rs` (9),
    `context_calibration.rs` (6), rewritten `acl_context_agent.rs` (10),
    `e2e_world_loop.rs`, ck-mcp lib tests, ck-bin cli tests.
- Changed (SWE_SEED-cep): `context_bundle.rs` (bundle adjudication + integrity
  recompute, float-normalized), dual-read `adjudicate_context_response`,
  `context_bundle_ref` bound into E4/E7, persistent-service client mode
  (`SWE_SEED_CONTEXT_KERNEL_URL`, std-only HTTP) + `negotiate()`; new tests
  `context_bundle_adjudication.rs` (9) + e2e hop assertions.
- Changed (SEA-Forge worktree `.tmp/ref/SEA-Forge`, branch `cep-bundle-lineage`):
  `GovernedWorkIntent.context_bundle_ref` (world-checked, content-addressed)
  + e2e lineage assertions + a wrong-world falsification test.
- Changed (sxr): `EvidenceEmission.context_bundle_ref` carried into E8 payload
  + e2e assertions.
- Verified: CK 265 tests green (baseline 237); SWE_SEED 437 green (baseline
  428 + new); sxr workspace green; sea-forge-server green; live CK<->SWE_SEED
  test green (real binary); `scripts/e2e-world-loop.sh` EXIT=0 (all 7 hops,
  CEP reference validator, 3 refusal paths); CEP_REPO-gated conformance green.
- Notes: Fixed clippy warnings (option_map_unit_fn, unnecessary_lazy_evaluations). Used parallel subagent delegation for integration tests and periodic refresh implementation.

## 2026-10-24 — Documentation reconciliation for the context pipeline

- Scope: `docs/**`, root `README.md`, `.agents/**`. No Rust, tests, config, deps, CI, or build changes.
- Reconciled bundle-first wire semantics across `docs/README.md`, `concepts.md`, `mental-model.md`, `source-map.md`, `security.md`, `reference/mcp-tools.md`, `subsystems/{mcp-and-protocols,application-orchestration,retrieval-and-ranking}.md`, `explanation/{retrieval-and-ranking-design,known-limits-and-debt}.md`, `troubleshooting.md`, `workflows/agentic-capability-loop.md`, `mcp-contracts.md`, `documentation-map.md`, root `README.md`.
- Added `docs/workflows/context-requirement-resolution.md` (stage-by-stage trace, reason/omission tables, calibration loop) and `docs/explainers/context-requirement-explorer.html` (offline, single file, in-page canonical-JSON SHA-256 + tamper detection, 11-stage stepper, presets for every terminal branch).
- Fixed 33 broken `file:///crates/` links across 9 docs pages; scripted relative-link check now resolves every `docs/**/*.md` link and every root README link.
- Corrected the stale single-adapter claim in `known-limits-and-debt.md`: kind routing exists; GitHub/Web registration is the remaining gap.
- Verified: HTML JS passes `node --check`; SHA-256 matches node crypto on known vectors; headless Chromium screenshots at 1280x2000, 1280x4200, and 430x2600 plus DOM dump confirm layout and state transitions; behavior harness covers complete/budget/cap/scope/excluded/freshness/no-match/corpus-missing/ingress/calibration/concept-absent/optional branches (all passed).
- Not run: `cargo test` (no code changed; last recorded 265 passed on 2026-10-07) and `just context-check` (no justfile exists; recorded in `open_questions.md`).
