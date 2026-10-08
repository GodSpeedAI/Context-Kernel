# Current Plan — CEP context_bundle architectural completion

## Objective
Make Context Kernel the bounded representation compiler of the GodSpeed loop:
typed Context Requirement → real CK retrieval/ranking/budgeting/scope enforcement →
CEP-0008 `godspeed.context_bundle` (complete/partial/none truthful, integrity-hashed,
world_ref-pinned) → SWE_SEED dual-read consumption → SEA-Forge/RealityTrace lineage →
settlement-backed Context Effect calibration. No parallel protocol, no duplicate
authority, no second semantic identity.

## Baselines (captured 2026-10-07)
- Context_Kernel: branch `migration/cep-context-bundle`, HEAD `270e911`, clean except `?? .jolli/`; `cargo test --workspace` → 237 passed / 0 failed (EXIT=0, /tmp/ck-baseline-test.log).
- SWE_SEED-cep: branch `e2e/world-loop`, HEAD `86ef9d3`, clean; baseline `cargo test -p swe-seed-core` → 428 passed / 0 failed (EXIT=0, /tmp/swe-baseline.log).
- sxr: branch `migration/cep-world-evidence`, HEAD `0461b50`, clean.
- sea-rs (local): branch `casework/live-wiring` @ 7c65be7, DIRTY (452 files) and DIVERGED from origin (origin has CEP Stages 4–11 commits). Local worktree NOT to be modified; a clean git worktree of `origin/casework/live-wiring` will be created under `.tmp/ref/SEA-Forge` for the narrowly-required SEA-Forge lineage change + e2e driver.
- cep: branch `profiles/context-bundle`, HEAD `d7591c1`, clean (read-only reference + validator).
- godspeed_agent: branch `migration/cep-world-required-evidence` @ 4bfed12 (2 modified fixtures).

## Phases
1. **Context_Kernel**
   - `ck-core`: shared `RetrievalCompleteness`, `NoContextReason`, omission codes, `world_ref` syntax gate (re-exported by ck-mcp for compat).
   - `ck-app`: `context.rs` typed `ContextRequirement` + `ContextResolution` + `ContextAccess` + `AppServices::resolve_context` (real query_reference pipeline: corpus filter, private-scope enforcement incl. POL-ACL-002 strict deny, budgets, freshness, required-concept accounting, completeness calc); adapter registry by kind (wires `LearningProposalsAdapter`); `context_effect.rs` Context Effect ledger (JSONL, evidence-gated ingestion, prospective calibration applied in resolve_context); empty-query match-all support.
   - `ck-mcp`: `AclContextAgent` routes GodSpeed path through `app.resolve_context` (auto-ingest corpus_root/ledger corpora first); delete ad-hoc substring retrieval; `handle_context_required` parses typed requirement, returns `context_bundle` (CEP, canonical) + legacy dual-read fields + `access` result (accessible/inaccessible/requires_scope — no allow/deny authority language); `cep_bundle` v2 projector (native resolution → bundle: references w/ doc/chunk/source ids, versions, digests, locators; provenance; omissions; questions; lineage/correlation extension; `integrity.content_hash` sha256-canonical-JSON + verify fn + structural validator).
2. **SWE_SEED-cep**: dual-read (CEP bundle preferred, legacy fallback) adjudication with bundle integrity recompute + cross-gates; `context_bundle_ref` carried into E4 and E7; persistent HTTP client mode (`SWE_SEED_CONTEXT_KERNEL_URL`) alongside spawn-stdio; get_status capability/version negotiation.
3. **SEA-Forge** (worktree `.tmp/ref/SEA-Forge` @ origin/casework/live-wiring): narrow ingress change — optional `context_bundle_ref` on E4 verified against supplied bundle (world/work_request/integrity/packet-id consistency, fail closed) recorded on `GovernedWorkIntent`; e2e assertions.
4. **sxr**: carry `context_bundle_ref` from E7 into E8 payload (optional field, existing envelope machinery); test.
5. **Composition**: run `scripts/e2e-world-loop.sh` (SEA-Forge worktree driver) with all real hops + CEP_REPO validator step + bundle-identity lineage assertions + calibration step; negative/falsification tests (tampered bundle, wrong profile, unknown world, private scope, budget, empty retrieval, CK unavailable, access-as-authority misuse).

## Validation steps
- `cargo test --workspace` (CK), `cargo fmt/clippy` where gates exist.
- `cargo test -p swe-seed-core` (+ context/e2e tests), `cargo test --workspace` (sxr), SEA-Forge targeted tests (`e2e_world_loop`, ingress/CEP conformance).
- CEP_REPO-gated conformance test (`uv run` reference validator) for complete/partial/none + negative fixtures.
- Full `e2e-world-loop.sh` run.

## Files expected to change
- CK: crates/ck-core/src/{types,lib}.rs (+world/completeness), crates/ck-app/src/{lib,context,context_effect}.rs + tests, crates/ck-search-fts/src/lib.rs (empty-query match-all), crates/ck-mcp/src/{lib,agentic_capability_loop,cep_bundle}.rs + tests/{acl_context_agent,e2e_world_loop,context_pipeline}, crates/ck-bin/{src/main,tests/cli}.rs, docs/mcp-contracts.md, integration/agentic_capability_loop/*, .agents/*.
- SWE_SEED-cep: crates/swe-seed-core/src/federation/{context_client,governed_submission}.rs + tests.
- SEA-Forge worktree: crates/sea-forge-server/src/governed_work_ingress.rs (+ intent) + tests/e2e_world_loop.rs.
- sxr: sxr-core/src/evidence_emit.rs + tests/e2e_world_loop.rs.
