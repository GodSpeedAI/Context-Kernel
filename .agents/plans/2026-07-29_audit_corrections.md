# Implementation Plan — Complete NeatCode Audit Corrections

**Created:** 2026-07-29
**Source of truth:** `.agents/reports/neatcode-audit-2026-07-29.md`; controlling ownership constraints: `/home/sprime01/projects/godspeed_agent/docs/explanations/SWE_SEED_to_godspeed_agent.md` and `/home/sprime01/projects/godspeed_agent/docs/explanations/SEA_Forge_to_godspeed_agent.md`
**Originating context:** The deep NeatCode audit found 4 S1, 4 S2, 3 S3, 1 S4, and one architecture-conformance correction. The current `harden` worktree already implements or mitigates part of that report; this plan closes every residual while preserving the SWE_SEED, SEA-Forge, GodSpeed-Agent, and Context Kernel authority boundaries.
**Status of the work today:** Reload authorization, enum decoder arms, observability lifetime/config loading, DNS fail-closed behavior, bind-fails-startup, a readiness probe, refresh de-duplication, artifact metadata compensation, path containment, corpus allowlisting, and freshness decay exist in the current uncommitted harden worktree. Server-derived protected-read enforcement, authority-neutral context retrieval, canonical learning-proposal consumption, full write atomicity/repair, DNS address pinning, runtime source routing, scheduler ownership, worker joins, effective config semantics, narrow storage ports, and complete runtime evidence remain partial or not implemented.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies: Task 0 precedes all work; Task 1→2→3; Task 4→5→7; Task 6→7; Tasks 3, 5, and 6→7→8→9→10; Task 11 follows Tasks 1-10. Tasks 1, 4, and 6 may proceed in parallel after Task 0, but shared `ck-core`, `ck-app`, and `ck-bin` edits must be serialized.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **SEA-Forge authorizes protected actions, SWE_SEED owns software execution/proof, GodSpeed-Agent owns settlement/developmental truth, and Context Kernel owns retrieval; adapters only translate and enforce verified decisions.**
- Match surrounding code style; locate the relevant idioms here: `crates/ck-core/src/ports/`, `crates/ck-app/src/lib.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-mcp/src/lib.rs`, and `crates/ck-bin/src/main.rs`.
- The current worktree and failing-first tests are the implementation evidence. Preserve existing uncommitted harden changes, never edit SEA generated files directly, and never treat documentation or generated contracts alone as runtime proof.
- Use `scripts/run-exact-test.sh <package> <target-or-dash> <full-test-name>` for every narrow CK Rust test; it MUST assert exactly one discovered test before executing it, so a zero-test Cargo success is a failed gate.
- Before cross-repository tasks, export `SEA_ROOT`, `SWE_SEED_ROOT`, and `GODSPEED_ROOT` to their checkout roots; gates use these variables and must not depend on one developer's `/home` layout.

### Dependency graph

```text
Task 0 baseline + retain existing harden corrections
  ├─ Task 1 correct SEA/SWE context-authority contract
  │    └─ Task 2 application-owned governed context retrieval
  │         └─ Task 3 canonical read-only learning-proposal adapter
  ├─ Task 4 stable identity + atomic canonical/FTS writes + repair
  │    └─ Task 5 split StoragePort at proven use-case boundaries
  │         └─ Task 7 runtime source-kind pipelines (also Tasks 3 and 6)
  │              └─ Task 8 scheduler becomes sole refresh owner
  │                   └─ Task 9 graceful lifecycle + live readiness
  │                        └─ Task 10 effective configuration + observability identity
  └─ Task 6 pin web requests to validated DNS addresses

Tasks 1-10 ── Task 11 audit conformance, docs, and persistent evidence
```

### Global verification gates (must stay green after EVERY task)

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo build --release -p ck-bin
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
| --- | --- |
| Audit scope and all 12 severity findings | `.agents/reports/neatcode-audit-2026-07-29.md:28-196` |
| SWE_SEED owns work contracts, execution, proof commands, traces, and evidence packaging | `/home/sprime01/projects/godspeed_agent/docs/explanations/SWE_SEED_to_godspeed_agent.md:27-62` |
| SEA-Forge owns allow/deny/escalate before protected side effects | `/home/sprime01/projects/godspeed_agent/docs/explanations/SEA_Forge_to_godspeed_agent.md:25-55`, `/home/sprime01/projects/godspeed_agent/docs/explanations/SEA_Forge_to_godspeed_agent.md:158-172` |
| GodSpeed-Agent owns settlement, capability, repetition, and learning proposals | `/home/sprime01/projects/godspeed_agent/docs/explanations/SWE_SEED_to_godspeed_agent.md:64-94` |
| Adapters translate but do not decide authority, proof success, or developmental meaning | `/home/sprime01/projects/godspeed_agent/docs/explanations/SEA_Forge_to_godspeed_agent.md:351-376` |
| Current SEA source incorrectly assigns `ContextAuthorized` to Context Kernel | `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea:48-55`, `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea:75-88` |
| Context Kernel duplicates generated event DTOs and decides authorization locally | `crates/ck-mcp/src/agentic_capability_loop.rs:36-162`, `crates/ck-mcp/src/agentic_capability_loop.rs:191-263` |
| SWE_SEED currently sends caller-owned privacy facts and expects CK authorization output | `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/context_client.rs:103-135` |
| Corpus visibility already exists in canonical CK storage, but the ACL path bypasses it | `crates/ck-core/src/entities/corpus.rs:3-11`, `crates/ck-core/src/ports/storage.rs:24-29`, `crates/ck-mcp/src/agentic_capability_loop.rs:266-323` |
| `ck-mcp` directly constructs filesystem retrieval and parses a GodSpeed-named side ledger | `crates/ck-mcp/Cargo.toml:7-18`, `crates/ck-mcp/src/agentic_capability_loop.rs:266-380` |
| GodSpeed canonical storage wraps persisted events under `record` with sequence/checksum metadata | `/home/sprime01/projects/godspeed_agent/godspeed_nav/storage.py:103-126` |
| Current CK learning-proposal adapter expects raw root-level `payload` rows | `crates/ck-adapter-filesystem/src/learning_proposals.rs:68-113` |
| Canonical and FTS writes still open independent SQLite connections | `crates/ck-app/src/lib.rs:161-268`, `crates/ck-app/src/lib.rs:378-434`, `crates/ck-search-fts/src/lib.rs:9-22` |
| `StoragePort` combines unrelated source, retrieval, jobs, retention, and artifact capabilities | `crates/ck-core/src/ports/storage.rs:6-88` |
| Web validation and connection still resolve DNS separately | `crates/ck-adapter-web/src/lib.rs:98-125`, `crates/ck-adapter-web/src/lib.rs:217-282` |
| Runtime composes only the filesystem adapter and bypasses the scheduler crate | `crates/ck-bin/src/main.rs:179-235`, `crates/ck-bin/src/main.rs:353-400`, `crates/ck-bin/Cargo.toml:11-28` |
| SSE and refresh handles are discarded; refresh sleep is not interruptible | `crates/ck-bin/src/main.rs:135-177`, `crates/ck-bin/src/main.rs:179-235`, `crates/ck-bin/src/main.rs:303-305` |
| Config fields still lack named runtime behavior | `crates/ck-config/src/lib.rs:40-98`, `crates/ck-bin/src/main.rs:100-121`, `crates/ck-bin/src/main.rs:238-301`, `crates/ck-transport-sse/src/lib.rs:49-54` |

---

## Task 0 — Preserve Hardened Fixes and Complete Their Evidence  (baseline · S1/S2/S3)

**Goal:** Every audit correction already present in the harden worktree has a direct runtime or persistence test, and the current 206-test baseline remains green before structural work starts.

**Why this shape:** The audit's reload, enum, observability-lifetime, DNS fail-closed, bind-startup, containment, de-duplication, compensation, and freshness findings must not regress while residual work proceeds. Dependency: no prior task. Blast radius: tests in SQLite, app, web, MCP, binary, and observability; production edits only where a missing test exposes an incomplete correction. Authority: independent crate-local evidence may be delegated; one in-context owner records the baseline and protects the dirty worktree.

### Steps

1. Record the current worktree and run the global gates before edits; do not reset the nine modified harden files — files: `.agents/work_log.md:1`, `.agents/evidence.md:77-91`.
2. Add `scripts/run-exact-test.sh`; it lists tests for a package/optional integration target, requires exactly one full-name match, then runs that test with `--exact` — file: `scripts/run-exact-test.sh:1` (new).
3. Extend the existing SQLite `tests` module from codec-only checks to persist-and-read every session transport, continuation mode/status, job trigger, and job status through real tables — file: `crates/ck-storage-sqlite/src/lib.rs:1829-1879`.
4. Extend the reload CLI test to prove corpus denial, context-service preservation, atomic service/auth generation replacement, and rejection of `require_auth=true` with no tokens — files: `crates/ck-bin/tests/cli.rs:662-765`, `crates/ck-bin/src/main.rs:247-295`.
5. Add an AppServices test only for the existing artifact-metadata failure compensation: assert the newly written payload and sidecar are removed when `upsert_artifact` fails; leave event/index failure to Task 4 — files: `crates/ck-app/src/lib.rs:172-188`, `crates/ck-app/src/lib.rs:215-232`, `crates/ck-artifacts/src/lib.rs:109-124`.
6. Add CLI JSON-log tests proving one successful and one failed tool call emit structured stderr and that log records emitted inside spawned SSE and refresh threads use the same subscriber; add a real HTTP `/readyz` test instead of the current stdio-only health assertion — files: `crates/ck-bin/tests/cli.rs:322-345`, `crates/ck-observability/src/lib.rs:18-46`, `crates/ck-transport-sse/src/lib.rs:62-72`.
7. Strengthen freshness tests with half-TTL, exact-TTL, future timestamp, and zero-TTL cases while retaining malformed/absent behavior — file: `crates/ck-app/src/lib.rs:1496-1523`.
8. Run each new evidence test against the audit parent or a temporary local mutation that removes the corrected branch, record the expected non-zero RED result, restore the current harden code, then run the gate GREEN — file: `.agents/work_log.md:1`.
9. Record each audit item as implemented, partial, or not implemented from these tests; do not promote residual items — files: `.agents/project_state.md:31-67`, `.agents/evidence.md:11-91`, `.agents/OBSERVED_DEBT.md:5-30`.

### Gate

```bash
./scripts/run-exact-test.sh ck-storage-sqlite - tests::persisted_enums_roundtrip_every_domain_variant
./scripts/run-exact-test.sh ck-bin cli sse_reload_preserves_tool_and_corpus_authorization
./scripts/run-exact-test.sh ck-app - tests::artifact_payload_is_compensated_on_metadata_failure
./scripts/run-exact-test.sh ck-bin cli json_logging_records_success_and_failure
./scripts/run-exact-test.sh ck-bin cli json_logging_covers_sse_and_refresh_threads
./scripts/run-exact-test.sh ck-bin cli sse_readyz_reflects_live_service
./scripts/run-exact-test.sh ck-app - tests::compute_freshness_table
./scripts/run-exact-test.sh ck-mcp - tests::context_required_corpus_id_uses_authorization_arm
./scripts/run-exact-test.sh ck-mcp - tests::context_required_rejects_path_traversal_outside_corpus_root
./scripts/run-exact-test.sh ck-app - tests::refresh_does_not_accumulate_duplicate_documents
./scripts/run-exact-test.sh ck-adapter-web - tests::validate_url_with_resolver_fails_closed_on_resolver_error
./scripts/run-exact-test.sh ck-bin cli sse_bind_failure_fails_startup
```

**Done when:** Each retained harden fix has a test that fails if its corrected branch is reverted, and all residual findings remain explicitly partial/open.

**Redesign trigger:** If a proposed evidence test requires changing public behavior rather than exposing the already-claimed fix, move it to the task that owns that behavior; do not smuggle residual implementation into the baseline task.

## Task 1 — Correct the SEA/SWE Context Authority Contract  (cross-system contract · prerequisite)

**Goal:** The canonical contract says SEA-Forge emits authority decisions, SWE_SEED preserves and submits their references, and Context Kernel emits only a correlated cited context packet after enforcing verified authority.

**Why this shape:** The audit says to validate scope through `AuthPort`, but the controlling documents prohibit Context Kernel from independently deciding authority. This contract correction must precede CK implementation or generated and handwritten DTOs will keep ratifying the wrong owner. Dependency: Task 0. Blast radius: SEA source/generated contracts, SWE_SEED client/tests, CK integration fixture/contracts; separate commits in each repository. Authority: SEA spec/generation work, SWE client work, and CK consumer work may be delegated to repository owners after the event shape is frozen; do not edit generated SEA output by hand.

### Steps

1. Add failing contract and consumer tests proving `ContextAuthorized` is absent; `AuthorityChecked` carries `key_id`, base64 `signed_payload`, and Ed25519 `signature`, while the signed payload contains issued/expiry times plus decision identity/actor/operation/resource/result/policy; SWE_SEED owns `EvidenceRecorded`; `ContextPacketCreated` carries original context/authority references; and the CK fixture neither fabricates authority nor citations — files: `/home/sprime01/projects/SEA/tests/unit/test_agentic_capability_loop_contract.py:1` (new), `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/tests/context_kernel_client.rs:1-81`, `integration/agentic_capability_loop/test_adapters.py:1` (new), `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea:41-97`, `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea:131-139`.
2. Run the new SEA test and the updated SWE/CK consumer tests before implementation; record non-zero RED results showing CK owns `ContextAuthorized`, authority references are absent, and callers still send privacy facts — files: `/home/sprime01/projects/SEA/tests/unit/test_agentic_capability_loop_contract.py:1`, `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/tests/context_kernel_client.rs:1-81`, `integration/agentic_capability_loop/test_adapters.py:1` (new), `.agents/work_log.md:1`.
3. Edit the SEA source contract/private-corpus policy and the domain's contract generator field map, not generated outputs, to remove `ContextAuthorized`, define the signed opaque `AuthorityChecked` envelope, make protected corpus retrieval require a prior signed allow, assign authority evidence to SEA, assign `EvidenceRecorded`/execution evidence to SWE_SEED, and preserve GodSpeed settlement/capability ownership — files: `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea:41-115`, `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea:131-139`, `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/tools/generate_contracts.py:17-45`.
4. Regenerate AST→IR→manifest with the SEA compiler commands, run the domain's actual `generate_contracts.py`, delete stale schemas not present in the regenerated manifest, and verify Rust/Python/TypeScript/JSON Schema contracts agree on IDs, hashes, references, and `max_results` type — files: `/home/sprime01/projects/SEA/generated/rust/agentic_capability_loop.rs:55-139`, `/home/sprime01/projects/SEA/generated/python/agentic_capability_loop/types.py:1`, `/home/sprime01/projects/SEA/generated/typescript/agentic_capability_loop.ts:1`, `/home/sprime01/projects/SEA/generated/schemas/agentic_capability_loop/ContextAuthorized.schema.json:1`, `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.ast.json:1`, `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.ir.json:1`, `/home/sprime01/projects/SEA/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json:1`.
5. Change SWE_SEED's context client to submit the original typed `ContextRequired` envelope plus SEA authority evidence/reference instead of caller-owned `is_private`/`scope_claim`; receive a complete `ContextPacketCreated` envelope — files: `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/src/federation/context_client.rs:103-135`, `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/tests/context_kernel_client.rs:1-81`.
6. Replace CK's Python integration fixture behavior and add its tests: no fallback domain hash, no synthetic citations, no local authority decision, and no caller-selected ledger path — files: `integration/agentic_capability_loop/adapters.py:1-267`, `integration/agentic_capability_loop/test_adapters.py:1` (new).
7. Add compatibility documentation naming the breaking request/response change and correlation fields before Task 2 changes production Rust — files: `docs/mcp-contracts.md:13-40`, `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/tests/context_kernel_client.rs:1-81`.

### Gate

```bash
test -n "$SEA_ROOT" && test -n "$SWE_SEED_ROOT" && test -n "$GODSPEED_ROOT"
just --justfile "$SEA_ROOT/justfile" sea-parse "$SEA_ROOT/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.sea" "$SEA_ROOT/docs/specs/domains/agentic_capability_loop"
just --justfile "$SEA_ROOT/justfile" ast-ir "$SEA_ROOT/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.ast.json" "$SEA_ROOT/docs/specs/domains/agentic_capability_loop"
just --justfile "$SEA_ROOT/justfile" ir-manifest "$SEA_ROOT/docs/specs/domains/agentic_capability_loop/agentic_capability_loop.ir.json" "$SEA_ROOT/docs/specs/domains/agentic_capability_loop"
python "$SEA_ROOT/docs/specs/domains/agentic_capability_loop/tools/generate_contracts.py"
python -m pytest "$SEA_ROOT/tests/unit/test_agentic_capability_loop_contract.py" -q
cargo test --manifest-path "$SWE_SEED_ROOT/Cargo.toml" -p swe-seed-core --test context_kernel_client -- --list | rg '^context_kernel_contract_preserves_authority_references: test$'
cargo test --manifest-path "$SWE_SEED_ROOT/Cargo.toml" -p swe-seed-core --test context_kernel_client context_kernel_contract_preserves_authority_references -- --exact
python -m pytest integration/agentic_capability_loop/test_adapters.py -q
bash -c 'rg "ContextAuthorized" "$SEA_ROOT/docs/specs/domains/agentic_capability_loop" "$SEA_ROOT/generated/rust/agentic_capability_loop.rs" "$SEA_ROOT/generated/python/agentic_capability_loop/types.py" "$SEA_ROOT/generated/typescript/agentic_capability_loop.ts" "$SEA_ROOT/generated/schemas/agentic_capability_loop"; status=$?; [ "$status" -eq 1 ]'
rg 'SWE_SEED.*EvidenceRecorded|EvidenceRecorded.*SWE_SEED' "$SEA_ROOT/tests/unit/test_agentic_capability_loop_contract.py"
```

**Done when:** Regeneration, SWE client tests, and CK fixture tests agree that CK cannot create authority, a missing/mismatched domain hash fails, and removing an authority reference makes the contract gate fail.

**Redesign trigger:** If a released consumer requires `ContextAuthorized`, retain it only as a deprecated projection of a verified SEA decision carrying the SEA decision ID; it must contain no independently computed CK result.

## Task 2 — Route Governed Context Retrieval Through ck-app  (architecture/security · S1)

**Goal:** Both `context_required` and protected ordinary reference reads derive corpus protection from canonical CK metadata, enforce a verified SEA decision before retrieval, preserve SWE request identity, and execute no filesystem/search read on deny, escalate, stale, malformed, or mismatched authority evidence.

**Why this shape:** This closes the audit's context bypass/path escape and architecture correction without building a SEA policy engine in CK. Transport token/tool/corpus allowlists remain defense in depth; `AuthorityEvidenceVerifierPort` verifies SEA evidence but does not decide policy. Dependency: Task 1. Blast radius: core authority/retrieval ports and DTOs, app use cases, MCP translation, filesystem composition, binary startup/reload, contracts/tests. Authority: verifier adapter and app retrieval tests may be delegated after DTOs freeze; MCP and binary convergence stay with one in-context owner.

### Steps

1. Add failing-first tests for no authority, deny, escalate, tampering, expiry, wrong actor/operation/corpus/work request/context requirement, unknown visibility, caller `is_private:false`, fake scope, absolute path, traversal, and symlink escape; every denial uses a counting resolver/search fake and asserts zero calls — files: `crates/ck-app/src/lib.rs:510-708`, `crates/ck-mcp/src/lib.rs:647-798`, `crates/ck-bin/tests/cli.rs:532-581`.
2. Run the exact protected-read tests before implementation and record non-zero RED results proving fake claims/unsigned allows are accepted, corpus metadata is bypassed, unscoped search can include protected corpora, and the MCP/filesystem dependency still exists — files: `.agents/work_log.md:1`, `crates/ck-app/src/lib.rs:510-708`, `crates/ck-mcp/src/agentic_capability_loop.rs:191-323`.
3. Define domain-only `AuthorityEvidence`, `ProtectedOperation`, `VerifiedAuthority`, and `AuthorityEvidenceVerifierPort`; the evidence DTO carries opaque signed bytes/key ID/signature, while parsed verified decisions are constructible only by the verifier — files: `crates/ck-core/src/ports/authority_evidence.rs:1` (new), `crates/ck-core/src/ports/mod.rs:1-25`.
4. Add an app-owned `retrieve_required_context` use case that loads `Corpus`, treats unknown visibility as protected, requires verified SEA allow for protected reads, searches canonical corpus rows, and returns citation provenance without an authorization decision — files: `crates/ck-app/src/lib.rs:510-708`, `crates/ck-core/src/entities/corpus.rs:3-11`.
5. Preserve existing corpus visibility during ingestion instead of replacing it with `local`; reject unknown visibility strings or replace the string with a typed enum and update SQLite codecs/tests — files: `crates/ck-app/src/lib.rs:365-374`, `crates/ck-storage-sqlite/src/lib.rs:404-443`, `crates/ck-storage-sqlite/src/lib.rs:1085-1092`.
6. Add optional authority evidence to the `query_reference` MCP/application request for an explicitly selected corpus. Make explicit protected-corpus queries use the same verified-authority gate; for omitted `canonical_id`, enumerate only explicitly public/local corpora before lexical/vector search and exclude protected/unknown corpora before any search/read — files: `crates/ck-mcp/src/lib.rs:307-434`, `crates/ck-app/src/lib.rs:510-597`, `crates/ck-core/src/ports/lexical_search.rs:1`.
7. Reduce MCP handling to deserialize/validate the original event, authenticate/admit, delegate to app, and serialize the complete packet envelope; remove local `ContextAuthorized`, caller authority facts, generated IDs for incoming events, and discarded envelopes — files: `crates/ck-mcp/src/lib.rs:105-174`, `crates/ck-mcp/src/lib.rs:435-477`, `crates/ck-mcp/src/agentic_capability_loop.rs:1-263`.
8. Move live filesystem reading/containment behind a narrow app port implemented in `ck-adapter-filesystem`; add workspace `ed25519-dalek` and implement `SeaAuthorityEvidenceVerifier` in `ck-auth` using base64 decode, exact signed-byte verification, key-ID lookup, expiry/skew checks, and post-verification payload parsing. Add `[authority] sea_public_keys` plus `max_clock_skew_secs`, compose verifier/reader in `ck-bin`, and remove `ck-adapter-filesystem` from `ck-mcp` — files: `crates/ck-core/src/ports/corpus_reader.rs:1` (new), `crates/ck-adapter-filesystem/src/lib.rs:1-206`, `crates/ck-auth/src/authority_evidence.rs:1` (new), `Cargo.toml:31-57`, `crates/ck-auth/Cargo.toml:7-15`, `crates/ck-config/src/lib.rs:7-28`, `crates/ck-mcp/Cargo.toml:7-18`, `crates/ck-bin/src/main.rs:109-133`, `crates/ck-bin/src/main.rs:247-295`, `crates/ck-bin/src/main.rs:403-423`.
9. Add stdio and SSE boundary tests proving transport choice/local trust cannot bypass SEA evidence for a protected read, while public registered corpora still use local allowlists and retrieval normally — file: `crates/ck-bin/tests/cli.rs:532-765`.

### Gate

```bash
./scripts/run-exact-test.sh ck-app - tests::protected_context_requires_verified_sea_allow_before_read
./scripts/run-exact-test.sh ck-app - tests::query_reference_uses_same_protected_read_gate
./scripts/run-exact-test.sh ck-app - tests::unscoped_query_excludes_protected_corpora_before_search
./scripts/run-exact-test.sh ck-mcp - tests::context_required_preserves_request_and_authority_references
./scripts/run-exact-test.sh ck-auth - tests::sea_authority_verifier_rejects_unsigned_tampered_unknown_and_expired_evidence
./scripts/run-exact-test.sh ck-bin cli protected_context_is_denied_before_read_over_stdio_and_sse
bash -c 'cargo tree -p ck-mcp --edges normal | rg "ck-adapter-filesystem"; status=$?; [ "$status" -eq 1 ]'
bash -c 'rg "ContextAuthorized|scope_claim|is_private" crates/ck-mcp; status=$?; [ "$status" -eq 1 ]'
```

**Done when:** Exact SEA allow evidence permits one protected retrieval, every invalid/non-allow case performs zero reads, public retrieval remains functional, and reintroducing caller-owned privacy or an MCP filesystem dependency fails a gate.

**Redesign trigger:** If SEA evidence cannot be verified locally, define a narrow online verifier adapter with fail-closed timeout behavior; do not substitute bearer claims, scope presence, or CK allowlists for SEA authority.

## Task 3 — Consume Learning Proposals as a Canonical Read-Only Projection  (cross-system storage · boundary)

**Goal:** GodSpeed persists learning proposals through its canonical `LedgerStore`, and a tested CK read-only adapter can consume only persisted proposal records; runtime composition remains disabled until Task 7 adds the ordinary source-kind pipeline.

**Why this shape:** The ownership documents require one developmental storage authority; moving the current raw JSONL reader behind a port without fixing its source would preserve the violation. Dependency: Task 2's app-owned retrieval boundary and Task 1's contract ownership; Task 7 depends on this adapter before composing it into the ordinary pipeline. Blast radius: GodSpeed proposal persistence/tests and CK filesystem source adapter/tests; binary composition is intentionally deferred to Task 7. Authority: GodSpeed migration/persistence must be performed by its repository owner; CK only consumes canonical envelopes.

### Steps

1. Add GodSpeed failing-first tests proving `LearningProposalCreated` is appended through `LedgerStore.append_event`, receives sequence/checksum/redaction/index/backup behavior, and legacy side-file migration is idempotent — files: `/home/sprime01/projects/godspeed_agent/tests/test_agentic_capability_loop.py:1` (new), `/home/sprime01/projects/godspeed_agent/agentic_capability_loop/learning_proposals.py:14-77`, `/home/sprime01/projects/godspeed_agent/godspeed_nav/storage.py:103-144`.
2. Run the new GodSpeed and CK adapter tests before implementation; record non-zero RED results showing the root-level writer and raw `payload` parser cannot satisfy canonical envelope assertions — files: `/home/sprime01/projects/godspeed_agent/tests/test_agentic_capability_loop.py:1`, `crates/ck-adapter-filesystem/tests/learning_proposals.rs:1-137`, `.agents/work_log.md:1`.
3. Add `migrate_legacy_learning_proposals(storage_root, ledger_store)` in GodSpeed: read the root side file, append unseen event IDs through `LedgerStore.append_event`, fsync canonical records, rename the side file to `.migrated`, and return zero on repeats; invoke it once during agentic-capability-loop adapter bootstrap before proposal reads/writes — files: `/home/sprime01/projects/godspeed_agent/agentic_capability_loop/learning_proposals.py:14-77`, `/home/sprime01/projects/godspeed_agent/agentic_capability_loop/adapters.py:343-368`.
4. Replace all new GodSpeed root-level `LearningProposalLedger` writes with canonical ledger persistence and a post-persistence read/event surface; keep settlement/capability/proposal-status decisions in GodSpeed — files: `/home/sprime01/projects/godspeed_agent/agentic_capability_loop/learning_proposals.py:14-77`, `/home/sprime01/projects/godspeed_agent/agentic_capability_loop/adapters.py:343-368`.
5. Change CK's `LearningProposalsAdapter` to accept canonical rows with `record.payload`, validate event type/schema/checksum metadata supplied by GodSpeed, preserve canonical event ID/timestamp as source provenance, and never write the ledger — files: `crates/ck-adapter-filesystem/src/learning_proposals.rs:1-113`, `crates/ck-adapter-filesystem/tests/learning_proposals.rs:1-137`.
6. Remove the duplicate learning-proposal parser and `CK_LEARNING_PROPOSALS_LEDGER` special route from MCP/binary; leave the canonical adapter uncomposed/fail-closed until Task 7 exposes it through source-kind routing — files: `crates/ck-mcp/src/agentic_capability_loop.rs:326-380`, `crates/ck-bin/src/main.rs:403-423`.
7. Add adapter exact tests proving canonical parsing/provenance, CK byte immutability, and rejection of raw legacy rows — file: `crates/ck-adapter-filesystem/tests/learning_proposals.rs:1-137`.

### Gate

```bash
python -m pytest "$GODSPEED_ROOT/tests/test_agentic_capability_loop.py" -q
./scripts/run-exact-test.sh ck-adapter-filesystem learning_proposals canonical_rows_preserve_event_provenance
./scripts/run-exact-test.sh ck-adapter-filesystem learning_proposals adapter_never_mutates_canonical_ledger
./scripts/run-exact-test.sh ck-adapter-filesystem learning_proposals raw_legacy_rows_are_rejected
bash -c 'rg "CK_LEARNING_PROPOSALS_LEDGER|resolve_from_learning_proposals" crates/ck-mcp crates/ck-bin; status=$?; [ "$status" -eq 1 ]'
```

**Done when:** Every adapter-visible proposal has a canonical GodSpeed event/sequence/timestamp, CK does not write the source ledger, the runtime special path is absent, and restoring the root-level side-file writer or MCP parser fails a gate.

**Redesign trigger:** If GodSpeed cannot publish a stable canonical read surface, leave the CK proposal source disabled and document it as unavailable; do not retain the alternate side ledger for convenience.

## Task 4 — Make Writes Atomic, Identities Stable, and FTS Repairable  (data integrity · S1)

**Goal:** Source refresh and session-context persistence either commit canonical rows plus FTS rows as one SQLite outcome or leave the prior state intact, filesystem payload failures are compensated, document identities are stable, and an explicit command rebuilds FTS from canonical rows.

**Why this shape:** This completes the audit's multi-store correction beyond current de-duplication and metadata compensation. Dependency: Task 0 only; independent of Tasks 1-3. It must precede Task 5 so the atomic write capability defines the port split, and Task 7 so remote adapters cannot erase a good snapshot on transient failure. Blast radius: core write port, app ingest/context orchestration, SQLite, FTS, filesystem identity, binary CLI/tests. Authority: transaction design and core port remain in-context; adapter-local tests may be delegated after the boundary freezes.

### Steps

1. Add failing-first tests injecting failure after payload, artifact metadata, event, document, chunk, first/middle/final FTS row, and before commit; assert complete rollback, prior searchable snapshot, or explicit repair state — files: `crates/ck-app/src/lib.rs:161-268`, `crates/ck-app/src/lib.rs:323-459`, `crates/ck-storage-sqlite/src/lib.rs:1460-1879`.
2. Run all new failure-injection/stable-ID/rebuild tests before implementation and record non-zero RED results proving separate commits, delete-before-fetch, random IDs, and missing repair command — file: `.agents/work_log.md:1`.
3. Centralize document identity in AppServices after normalization: derive each ID from source ID, `SourceUnit.external_id`, and a stable normalized subdocument key such as canonical source URI/path (never list ordinal), reject missing/duplicate keys, overwrite adapter-generated random IDs before chunking, and test unchanged IDs after discovery reorder and insertion before an existing unit for filesystem, GitHub, and web — files: `crates/ck-app/src/lib.rs:323-434`, `crates/ck-app/Cargo.toml:7-13`, `crates/ck-core/src/entities/source.rs:1`, `crates/ck-core/src/entities/document.rs:1`.
4. Discover, fetch, normalize, and chunk an entire source snapshot before mutating its previous rows; an adapter/fetch failure leaves the old snapshot untouched — file: `crates/ck-app/src/lib.rs:323-434`.
5. Define domain-only atomic source-snapshot, context-event/index, and index-rebuild operations; no `rusqlite` type crosses core/app — files: `crates/ck-core/src/ports/index_write.rs:1` (new), `crates/ck-core/src/ports/mod.rs:1-25`.
6. Implement source replacement and event/FTS persistence on one SQLite connection/transaction; use connection-scoped FTS helpers or move write SQL into the SQLite adapter while retaining FTS read/search ownership — files: `crates/ck-storage-sqlite/src/lib.rs:351-573`, `crates/ck-storage-sqlite/src/lib.rs:905-1021`, `crates/ck-search-fts/src/lib.rs:63-105`, `crates/ck-search-fts/src/lib.rs:176-190`.
7. Extend compensation through event/index failure so newly written artifact payload and metadata do not survive a failed context write — files: `crates/ck-app/src/lib.rs:172-268`, `crates/ck-artifacts/src/lib.rs:109-124`.
8. Add `rebuild-index` to clear/repopulate chunk, session-event, and alias FTS tables from canonical rows; make it explicit rather than automatic startup work — files: `crates/ck-bin/src/main.rs:30-60`, `crates/ck-storage-sqlite/src/lib.rs:351-1021`, `crates/ck-bin/tests/cli.rs:1-765`.
9. Record transaction ownership and schema compatibility; add no migration unless persisted schema actually changes — files: `.agents/decisions.md:1`, `.agents/project_state.md:49-52`, `.agents/evidence.md:36-43`.

### Gate

```bash
./scripts/run-exact-test.sh ck-storage-sqlite - tests::source_snapshot_rolls_back_canonical_and_fts
./scripts/run-exact-test.sh ck-storage-sqlite - tests::context_event_rolls_back_with_fts_failure
./scripts/run-exact-test.sh ck-app - tests::failed_refresh_preserves_previous_searchable_snapshot
./scripts/run-exact-test.sh ck-app - tests::document_identity_is_stable_across_refresh
./scripts/run-exact-test.sh ck-app - tests::document_identity_survives_discovery_reorder_and_insertion
./scripts/run-exact-test.sh ck-bin cli rebuild_index_repairs_search_from_canonical_rows
```

**Done when:** Failure after a partial attempted write leaves no partial new FTS/canonical set, successful canonical and FTS ID sets match, stable IDs survive refresh, and deleting FTS followed by `rebuild-index` restores search results.

**Redesign trigger:** If bundled FTS5 writes cannot roll back in the same transaction, prove it with a focused test and re-plan around a durable outbox/reconciliation state; do not call best-effort compensation atomicity.

## Task 5 — Split StoragePort Along Existing Use Cases  (architecture · S4)

**Goal:** Scheduler, retrieval, ingestion, retention, and artifact/context consumers receive only their used capabilities, and test doubles contain no unrelated no-ops or `unimplemented!()` methods.

**Why this shape:** The audit explicitly says to split on the next real port change, not perform a speculative rewrite; Task 4 provides that change and its atomic boundary. Dependency: Task 4. Blast radius: core storage ports/exports, SQLite implementations, app/scheduler constructors and fakes, binary composition. Authority: trait design/removal stays in-context; consumer migrations may be delegated by crate after interfaces freeze.

### Steps

1. Add an architecture gate for no monolithic `StoragePort` and compile-focused scheduler/app fakes that initially fail because unrelated methods are required — files: `crates/ck-core/src/ports/storage.rs:6-88`, `crates/ck-app/src/lib.rs:760-1073`, `crates/ck-scheduler/src/lib.rs:122-349`.
2. Run the narrow-fake compile tests and architecture checks before implementation; record non-zero RED results caused by `StoragePort` and unrelated `unimplemented!()` obligations — file: `.agents/work_log.md:1`.
3. Split only observed capabilities into source/corpus catalog, retrieval reads, Task 4 atomic index writes/maintenance, refresh-job persistence, retention persistence, and artifact/context metadata — files: `crates/ck-core/src/ports/source_catalog.rs:1` (new), `crates/ck-core/src/ports/retrieval_store.rs:1` (new), `crates/ck-core/src/ports/refresh_job_store.rs:1` (new), `crates/ck-core/src/ports/retention_store.rs:1` (new), `crates/ck-core/src/ports/artifact_metadata.rs:1` (new), `crates/ck-core/src/ports/mod.rs:1-25`.
4. Implement each focused trait on `SqliteStore` without changing SQL behavior — file: `crates/ck-storage-sqlite/src/lib.rs:351-1021`.
5. Change AppServices/use-case constructors to accept focused capabilities and replace broad fakes/unimplemented adjacent fakes with behavior-specific doubles — files: `crates/ck-app/src/lib.rs:15-139`, `crates/ck-app/src/lib.rs:760-1073`.
6. Change `RefreshScheduler` to receive only source-staleness and job persistence capabilities — files: `crates/ck-scheduler/src/lib.rs:9-119`, `crates/ck-scheduler/src/lib.rs:122-349`.
7. Update binary composition and delete `StoragePort` instead of retaining an aggregate compatibility trait — files: `crates/ck-bin/src/main.rs:13-19`, `crates/ck-bin/src/main.rs:353-400`, `crates/ck-core/src/ports/storage.rs:1-88`.

### Gate

```bash
cargo test -p ck-app -- --list | rg ': test$'
cargo test -p ck-app
cargo test -p ck-scheduler -- --list | rg ': test$'
cargo test -p ck-scheduler
cargo test -p ck-storage-sqlite -- --list | rg ': test$'
cargo test -p ck-storage-sqlite
bash -c 'rg "trait StoragePort|impl StoragePort|dyn StoragePort" crates --glob "*.rs"; status=$?; [ "$status" -eq 1 ]'
bash -c 'rg "unimplemented!\(\)" crates --glob "*.rs"; status=$?; [ "$status" -eq 1 ]'
```

**Done when:** Each test double implements only its subject's capabilities, no monolithic storage trait remains, and adding an unrelated method to a focused port breaks the relevant narrow fake/review gate.

**Redesign trigger:** If two focused traits duplicate one read method, extract one shared read capability; do not recreate the broad trait under a new aggregate name.

## Task 6 — Pin Web Requests to Validated Addresses  (security · S2)

**Goal:** Web ingestion resolves once, rejects any failed/empty/private answer set, and connects only to the validated addresses while preserving original Host/TLS authority and blocking unsafe redirects.

**Why this shape:** Current fail-closed validation fixes half of the audit finding; wiring web before address pinning would expose DNS rebinding in production. Dependency: Task 0 only; no dependency on storage or authority work. Blast radius: web adapter implementation/tests and possibly its existing HTTP-client configuration; no new dependency expected because installed `ureq` supports a custom resolver. Authority: safe to delegate entirely to a web-adapter worker.

### Steps

1. Add failing-first resolver/request tests where validation sees a public address but a second DNS answer is private; assert no private target receives a request — file: `crates/ck-adapter-web/src/lib.rs:523-878`.
2. Run the rebinding/pinned-address tests before implementation and record non-zero RED results proving `ureq` performs an unrestricted second resolution — file: `.agents/work_log.md:1`.
3. Resolve exactly once, reject mixed answer sets containing any blocked address, and build the request agent with a resolver restricted to validated socket addresses — file: `crates/ck-adapter-web/src/lib.rs:75-166`, `crates/ck-adapter-web/src/lib.rs:217-282`.
4. Preserve the original hostname for HTTP Host and TLS SNI/certificate checks; keep redirects disabled or revalidate each redirect target before following — file: `crates/ck-adapter-web/src/lib.rs:98-125`.
5. Add IPv4, IPv6, mixed-set, resolver-error, empty-set, redirect, Host-header, and rebinding tests using local deterministic servers only — file: `crates/ck-adapter-web/src/lib.rs:523-878`.

### Gate

```bash
./scripts/run-exact-test.sh ck-adapter-web - tests::resolver_failure_and_empty_set_fail_closed
./scripts/run-exact-test.sh ck-adapter-web - tests::request_uses_only_validated_socket_addresses
./scripts/run-exact-test.sh ck-adapter-web - tests::dns_rebinding_cannot_reach_private_target
./scripts/run-exact-test.sh ck-adapter-web - tests::validated_connection_preserves_original_host
```

**Done when:** No request path performs an unrestricted second resolution, a rebinding test cannot reach its private listener, and removing the pinned resolver makes the gate fail.

**Redesign trigger:** If `ureq` cannot preserve TLS verification while pinning addresses, use another already-approved HTTP path or request a dependency decision; never disable certificate/hostname verification.

## Task 7 — Compose Filesystem, GitHub, and Web Pipelines by Source Kind  (runtime wiring/docs · S2)

**Goal:** The shipped binary routes `filesystem_tree`, `github_repo`, `website`, and the read-only canonical learning-proposal source through matching pipelines for ingest and refresh, rejects unknown kinds before mutation, and documents only tested runtime support.

**Why this shape:** This closes the advertised-but-unreachable adapter finding without a speculative registry abstraction. Dependencies: Tasks 3, 4, and 5 provide the canonical proposal adapter, safe atomic writes, and final app constructors; Task 6 makes web safe to expose. Blast radius: app pipeline selection, GitHub test transport, binary dependencies/composition/tests, adapter no-change semantics, README/operations docs. Authority: adapter smoke tests may be delegated; shared app/bin integration remains in-context.

### Steps

1. Add failing-first recording-pipeline tests for kind selection, matching normalizer, unknown-kind rejection before mutation, and refresh using persisted kind — file: `crates/ck-app/src/lib.rs:680-1018`.
2. Run pipeline and CLI tests before implementation and record non-zero RED results showing all non-filesystem kinds reach filesystem parsing or cannot be composed — file: `.agents/work_log.md:1`.
3. Replace the single source adapter/normalizer pair with the smallest kind-keyed collection in `AppServices`; reject duplicates/unknowns and select before storage mutation — files: `crates/ck-app/src/lib.rs:15-139`, `crates/ck-app/src/lib.rs:323-434`.
4. Add a deterministic GitHub test constructor that accepts a local API base/HTTP client while production defaults remain `api.github.com`; use it to exercise discover, fetch, and normalize without public network — files: `crates/ck-adapter-github/src/lib.rs:35-64`, `crates/ck-adapter-github/src/lib.rs:155-213`, `crates/ck-adapter-github/src/lib.rs:365-638`.
5. Add existing workspace dependencies and compose direct mappings for filesystem, GitHub, web, and Task 3's canonical learning-proposal adapter; do not add a registry crate/factory — files: `crates/ck-bin/Cargo.toml:11-28`, `crates/ck-bin/src/main.rs:353-400`, `crates/ck-adapter-filesystem/src/learning_proposals.rs:1-113`.
6. Define web 304/no-change semantics so refresh preserves the prior Task 4 snapshot rather than replacing it with empty content — files: `crates/ck-adapter-web/src/lib.rs:127-135`, `crates/ck-adapter-web/src/lib.rs:478-514`, `crates/ck-app/src/lib.rs:323-434`.
7. Add deterministic CLI smoke tests backed by local GitHub/web fixtures and a canonical proposal ledger; assert real adapter discovery/fetch/normalization and no public network — file: `crates/ck-bin/tests/cli.rs:152-314`.
8. After tests pass, correct README and operations claims; if an adapter cannot pass runtime smoke, call it library-only rather than implemented — files: `README.md:40-47`, `README.md:105-115`, `docs/operations.md:56-78`.

### Gate

```bash
./scripts/run-exact-test.sh ck-app - tests::ingest_selects_adapter_and_normalizer_by_kind
./scripts/run-exact-test.sh ck-app - tests::unknown_source_kind_fails_before_mutation
./scripts/run-exact-test.sh ck-app - tests::refresh_uses_persisted_source_kind
./scripts/run-exact-test.sh ck-app - tests::web_not_modified_preserves_snapshot
./scripts/run-exact-test.sh ck-bin cli ingest_source_dispatches_github_repo_kind
./scripts/run-exact-test.sh ck-bin cli ingest_source_dispatches_website_kind
./scripts/run-exact-test.sh ck-bin cli ingest_source_dispatches_canonical_learning_proposals
```

**Done when:** Each source kind reaches only its matching production pipeline, refresh uses the same kind, unknown kinds leave storage unchanged, and removing either binary adapter dependency fails its CLI test.

**Redesign trigger:** If deterministic adapter smoke tests require network access, add injectable clients inside adapter tests; do not make CI depend on GitHub, DNS, or public websites.

## Task 8 — Make ck-scheduler the Sole Refresh Owner  (runtime architecture · S3)

**Goal:** Manual and periodic refresh use one scheduler-owned queued→running→completed/failed record with coalescing/recovery, and the binary contains no parallel direct refresh loop.

**Why this shape:** Scheduler tests currently prove dead production code; composing it before final source pipelines and atomic writes would schedule unsafe/wrong adapter behavior. Dependencies: Tasks 5 and 7. It must precede Task 9 because lifecycle should join the final worker owner. Blast radius: scheduler/core API, app job separation, MCP response path, SQLite claim/recovery, binary dependencies/runtime/tests. Authority: scheduler crate work may be delegated; app/MCP/bin convergence remains in-context.

### Steps

1. Add failing-first tests proving one refresh produces one job, manual/periodic triggers coalesce, exact returned jobs are claimed, and interrupted running jobs have explicit restart recovery — files: `crates/ck-scheduler/src/lib.rs:370-509`, `crates/ck-bin/tests/cli.rs:284-314`.
2. Run scheduler/app/CLI ownership tests before implementation and record non-zero RED results proving production bypasses `ck-scheduler` and AppServices creates a second job — file: `.agents/work_log.md:1`.
3. Add exact-job claim to `SchedulerPort`/job persistence so synchronous manual refresh cannot claim a different queued job; retain global oldest claim for the worker — files: `crates/ck-core/src/ports/scheduler.rs:6-25`, `crates/ck-storage-sqlite/src/lib.rs:819-845`.
4. Separate content refresh from job lifecycle in AppServices; manual MCP refresh enqueues/claims/executes/settles the same scheduler record and preserves response compatibility — files: `crates/ck-app/src/lib.rs:443-485`, `crates/ck-app/src/lib.rs:1636-1660`, `crates/ck-mcp/src/lib.rs:480-503`.
5. Make `RefreshScheduler` own enqueue, coalescing, status transitions, source health, and interrupted-job recovery — file: `crates/ck-scheduler/src/lib.rs:9-120`.
6. Persist `freshness_ttl_secs` from ingestion so runtime-created sources can become stale — files: `crates/ck-app/src/lib.rs:325-360`, `crates/ck-mcp/src/lib.rs:281-306`.
7. Add `ck-scheduler` to `ck-bin`, compose one worker, route periodic stale sources through queue/claim/execute, and delete the direct stale-source refresh owner — files: `crates/ck-bin/Cargo.toml:11-28`, `crates/ck-bin/src/main.rs:179-235`.

### Gate

```bash
./scripts/run-exact-test.sh ck-scheduler - tests::scheduled_refresh_uses_one_job_record
./scripts/run-exact-test.sh ck-scheduler - tests::interrupted_running_job_has_explicit_recovery
./scripts/run-exact-test.sh ck-app - tests::manual_refresh_uses_one_scheduler_job
./scripts/run-exact-test.sh ck-app - tests::ingested_source_persists_refresh_ttl
./scripts/run-exact-test.sh ck-bin cli manual_and_periodic_refresh_coalesce
bash -c 'rg "refreshing stale sources" crates/ck-bin/src/main.rs; status=$?; [ "$status" -eq 1 ]'
```

**Done when:** Production constructs `RefreshScheduler`, one execution has one history record, duplicate triggers coalesce, restart recovery is tested, and reintroducing direct binary refresh fails a gate.

**Redesign trigger:** If synchronous MCP compatibility conflicts with queued ownership, perform enqueue/claim/execute synchronously on the same job; do not create a second record or silently change the API to asynchronous.

## Task 9 — Gracefully Stop Workers and Make Readiness Live  (operations · S2)

**Goal:** Startup reports transport failure to the parent, `/readyz` probes live required dependencies, stdin EOF stops intake, drains controlled in-flight SSE work, interrupts a long scheduler wait, joins every worker, and propagates worker errors/panics.

**Why this shape:** Bind failure and a basic status probe exist, but detached handles and fixed sleeps leave the audit lifecycle correction partial. Dependency: Task 8's final scheduler owner. Blast radius: SSE serve/router/tests, binary worker ownership/reload/tests, scheduler wait, operations docs. Authority: transport-local graceful shutdown may be delegated; process ordering stays in-context.

### Steps

1. Add failing-first tests for actual `/readyz`, dependency failure, SSE shutdown return, controlled in-flight drain, long refresh interval interruption, EOF joins, and worker panic propagation — files: `crates/ck-transport-sse/src/lib.rs:45-181`, `crates/ck-bin/tests/cli.rs:322-345`, `crates/ck-bin/tests/cli.rs:633-660`.
2. Run lifecycle/readiness tests before implementation and record non-zero RED results proving detached handles, fixed sleep, constant/snapshot readiness, and missing drain/join — file: `.agents/work_log.md:1`.
3. Add a live app/storage health operation used by readiness; avoid returning a preconstructed status snapshot — files: `crates/ck-app/src/lib.rs:1666-1701`, `crates/ck-mcp/src/lib.rs:121-133`, `crates/ck-transport-sse/src/lib.rs:66-72`.
4. Change `serve_sse` to accept a shutdown future/channel and use Axum graceful shutdown; mark started only after the pre-bound listener is registered — file: `crates/ck-transport-sse/src/lib.rs:45-60`.
5. Replace scheduler `thread::sleep` with interruptible channel/timer waiting and define claimed-job state on shutdown from Task 8 recovery rules — files: `crates/ck-scheduler/src/lib.rs:9-120`, `crates/ck-bin/src/main.rs:179-235`.
6. Retain SSE/scheduler handles and shutdown senders; on stdin EOF stop intake, signal both, await drain, join both, and return errors/panics — file: `crates/ck-bin/src/main.rs:63-305`.
7. Keep blocking adapter I/O drain semantics explicit: cooperative idle waits exit promptly, active synchronous calls run to completion because safe timed thread join is unavailable — files: `docs/operations.md:56-78`, `crates/ck-bin/src/main.rs:135-305`.
8. Ensure reload cannot leave an old worker generation against an old database/config; restart through the supervisor or reject startup-only changes — file: `crates/ck-bin/src/main.rs:247-295`.

### Gate

```bash
./scripts/run-exact-test.sh ck-transport-sse - lifecycle_tests::serve_sse_returns_after_shutdown
./scripts/run-exact-test.sh ck-transport-sse - lifecycle_tests::shutdown_drains_in_flight_request
./scripts/run-exact-test.sh ck-bin cli readyz_changes_when_dependency_fails
./scripts/run-exact-test.sh ck-bin cli eof_joins_sse_and_scheduler_workers
./scripts/run-exact-test.sh ck-bin cli long_refresh_interval_is_interruptible
./scripts/run-exact-test.sh ck-bin cli sse_bind_failure_fails_startup
```

**Done when:** Real HTTP readiness changes with dependency state, hour-long idle waits stop promptly on EOF, controlled requests drain, every handle is joined, and removing a join or error propagation makes the gate fail.

**Redesign trigger:** If a hard deadline for active synchronous I/O is required, migrate that adapter to cancellable async I/O or a killable process; do not detach a timed-out thread and call it graceful.

## Task 10 — Make Every Retained Config Field Observable  (configuration/observability · S2/S3)

**Goal:** Every audited config field either causes one tested runtime behavior or is removed with fail-fast migration errors; service identity appears in status/logs, SSE-only mode remains alive until signal shutdown, and unsupported OTEL configuration is not silently accepted.

**Why this shape:** Lifecycle must exist before honoring `enable_stdio=false`, and authority contracts must exist before final auth semantics. Dependency: Task 9. Blast radius: config parser/tests, status types, observability, binary supervisor/reload, SSE route, examples/docs. Authority: parsing/route tests may be delegated; compatibility/removal choices and binary behavior stay in-context.

### Steps

1. Add failing-first behavior tests changing one field at a time: stdio switch, SSE base path, semantic switch/removal, trust-mode removal, service name, JSON logging, and OTEL endpoint removal — files: `crates/ck-config/src/lib.rs:306-520`, `crates/ck-bin/tests/cli.rs:1-765`, `crates/ck-transport-sse/src/lib.rs:160-181`.
2. Run every one-field behavior/removal test before implementation and record non-zero RED results proving each retained key is ignored or unsupported configuration is accepted — file: `.agents/work_log.md:1`.
3. Remove duplicate `retrieval.semantic_enabled`; retain `[embeddings].enabled` as the single semantic authority and reject the old key with `deny_unknown_fields` — files: `crates/ck-config/src/lib.rs:40-44`, `crates/ck-bin/src/main.rs:100-107`, `crates/ck-bin/src/main.rs:371-398`.
4. Remove `auth.trust_mode`; retain explicit local stdio trust plus SSE `require_auth`, keeping service admission separate from verified SEA authority — files: `crates/ck-config/src/lib.rs:88-98`, `crates/ck-bin/src/main.rs:109-121`, `crates/ck-core/src/ports/auth.rs:1-14`.
5. Honor `mcp.enable_stdio`; when false and SSE is enabled, wait on `tokio::signal::ctrl_c()` before using Task 9's shutdown/join path; reject both transports disabled — files: `crates/ck-config/src/lib.rs:46-85`, `crates/ck-bin/src/main.rs:135-305`.
6. Validate and pass `mcp.sse.base_path` to router construction; reject `/mcp` when another path is configured and classify route/bind changes as startup-only — files: `crates/ck-config/src/lib.rs:70-85`, `crates/ck-transport-sse/src/lib.rs:45-59`, `crates/ck-bin/src/main.rs:135-177`.
7. Pass configured `service.name` into live status and structured logs; remove the hardcoded status name — files: `crates/ck-core/src/types/status.rs:52-82`, `crates/ck-app/src/lib.rs:1666-1701`, `crates/ck-observability/src/lib.rs:6-46`.
8. Remove unsupported `observability.otel_endpoint` and fail fast on the old key, or implement a real exporter only if separately approved; do not retain warning-only ignored config — files: `crates/ck-config/src/lib.rs:208-227`, `crates/ck-observability/src/lib.rs:18-46`.
9. Update examples and migration docs only after behavior gates pass — files: `config/context-kernal.example.toml:1`, `config/context-kernal.local.example.toml:1`, `config/context-kernal.sse.example.toml:1`, `docs/configuration.md:21-78`, `docs/operations.md:56-78`.

### Gate

```bash
./scripts/run-exact-test.sh ck-config - tests::removed_duplicate_fields_fail_fast
./scripts/run-exact-test.sh ck-config - tests::rejects_all_transports_disabled
./scripts/run-exact-test.sh ck-transport-sse - lifecycle_tests::configured_base_path_is_only_mcp_route
./scripts/run-exact-test.sh ck-bin cli disabled_stdio_does_not_answer_stdin
./scripts/run-exact-test.sh ck-bin cli sse_only_mode_waits_for_signal_and_joins
./scripts/run-exact-test.sh ck-bin cli configured_service_name_appears_in_status_and_logs
./scripts/run-exact-test.sh ck-bin cli json_logging_records_success_and_failure
rg -F '`retrieval.semantic_enabled` | removed | use `[embeddings].enabled`' docs/configuration.md
rg -F '`auth.trust_mode` | removed | use local stdio trust or `mcp.sse.require_auth`' docs/configuration.md
rg -F '`observability.otel_endpoint` | removed | no exporter is implemented' docs/configuration.md
rg -F 'mcp.enable_stdio' docs/configuration.md
rg -F 'mcp.sse.base_path' docs/configuration.md
```

**Done when:** No audited field is parsed without a dedicated behavior/removal test, an SSE-only process does not exit on stdin EOF, old removed keys produce actionable errors, and restoring a parsed-but-unused field fails its test.

**Redesign trigger:** If released configs require `semantic_enabled`, `trust_mode`, or `otel_endpoint`, retain only with explicit precedence/behavior and compatibility tests; do not silently ignore them.

## Task 11 — Prove Audit and Cross-System Conformance  (documentation/evidence · final)

**Goal:** All audit corrections and controlling ownership invariants are demonstrably implemented, documentation matches the shipped binary, project memory records exact evidence, and no generated/source boundary or alternate truth ledger remains.

**Why this shape:** Documentation cannot prove behavior and must follow Tasks 1-10. Dependency: all prior tasks. Blast radius: README, contracts, security/config/operations/schema docs, examples, CI, `.agents` memory, and cross-repository integration gates; no new runtime behavior. Authority: documentation/evidence can be delegated after final code freezes; one reviewer runs the complete conformance matrix.

### Steps

1. Add a cross-system allow integration test: SEA allow → CK cited packet with authority reference → SWE proof bundle preserving reference → GodSpeed persisted settlement; each system asserts only its own evidence class — files: `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/tests/context_kernel_client.rs:1-81`, `/home/sprime01/projects/godspeed_agent/tests/e2e/test_governed_capability_loop.py:1` (new), `crates/ck-bin/tests/cli.rs:1-765`.
2. Add a denial integration test: SEA deny/escalate → zero CK read → no SWE proof-failure classification → no GodSpeed capability decrement — files: `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/tests/context_kernel_client.rs:1-81`, `/home/sprime01/projects/godspeed_agent/tests/e2e/test_governed_capability_loop.py:1`, `crates/ck-bin/tests/cli.rs:1-765`.
3. Run the allow/deny tests before final integration and then mutation-test them by removing one authority reference, allowing one denied CK read, and mapping deny to SWE proof failure; record each expected non-zero RED result before restoring and running GREEN — files: `/home/sprime01/projects/SWE_SEED/crates/swe-seed-core/tests/context_kernel_client.rs:1-81`, `/home/sprime01/projects/godspeed_agent/tests/e2e/test_governed_capability_loop.py:1`, `crates/ck-bin/tests/cli.rs:1-765`, `.agents/work_log.md:1`.
4. Add `scripts/cross-system-conformance.sh <sea-root> <swe-seed-root> <godspeed-root>`; it validates all roots, asserts exact SWE test discovery, forbids skip output, runs SEA generation/contracts, CK Python fixtures, SWE integration, and GodSpeed unit/e2e tests — file: `scripts/cross-system-conformance.sh:1` (new).
5. Add an owning cross-system CI workflow that checks out SEA, SWE_SEED, and GodSpeed-Agent at immutable post-change commit SHAs into workflow-local paths, builds CK, and invokes the script with those paths; record/update the pinned revisions with each contract change — files: `.github/workflows/cross-system-conformance.yml:1` (new), `.ci/cross-system-revisions.env:1` (new).
6. Add architecture checks for no MCP filesystem dependency/local authority, no CK developmental lifecycle code, no broad StoragePort, no synthetic citation, and no root-level proposal ledger; strengthen primary CI Clippy to `cargo clippy --workspace --all-targets -- -D warnings` — files: `.github/workflows/ci.yml:7-19`, `.github/workflows/cross-system-conformance.yml:1`, `integration/agentic_capability_loop/adapters.py:1-267`.
7. Update README adapter claims and architecture boundaries; document authority evidence, request correlation, citation provenance, config migrations, scheduler/lifecycle behavior, FTS repair, and canonical learning-proposal projection — files: `README.md:35-115`, `README.md:298-327`, `docs/mcp-contracts.md:1-61`, `docs/security.md:1`, `docs/configuration.md:1-121`, `docs/operations.md:1-78`, `docs/schema.md:1`.
8. Update every required memory file with actual code/test evidence and remove questions only when gates answer them — files: `.agents/project_state.md:1-68`, `.agents/current_plan.md:1-24`, `.agents/work_log.md:1`, `.agents/decisions.md:1-41`, `.agents/open_questions.md:1-20`, `.agents/evidence.md:1-91`, `.agents/OBSERVED_DEBT.md:1-30`.
9. Run the complete cross-repository and Context Kernel gates from clean task-scoped commits; inspect changed/generated artifacts before status claims — files: `.github/workflows/ci.yml:7-19`, `.github/workflows/cross-system-conformance.yml:1`, `/home/sprime01/projects/SEA/generated/rust/agentic_capability_loop.rs:55-139`.

### Gate

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo build --release -p ck-bin
test -n "$SEA_ROOT" && test -n "$SWE_SEED_ROOT" && test -n "$GODSPEED_ROOT"
test -x target/release/context-kernal
./scripts/cross-system-conformance.sh "$SEA_ROOT" "$SWE_SEED_ROOT" "$GODSPEED_ROOT"
bash -c 'cargo tree -p ck-mcp --edges normal | rg "ck-adapter-filesystem"; status=$?; [ "$status" -eq 1 ]'
bash -c 'rg "ContextAuthorized|scope_claim|is_private" crates/ck-mcp; status=$?; [ "$status" -eq 1 ]'
bash -c 'rg "settlement_count|metabolized|CapabilityUpdated|classify.*settlement" crates; status=$?; [ "$status" -eq 1 ]'
bash -c 'rg "trait StoragePort|impl StoragePort|dyn StoragePort" crates --glob "*.rs"; status=$?; [ "$status" -eq 1 ]'
bash -c 'rg "class LearningProposalLedger" "$GODSPEED_ROOT/agentic_capability_loop"; status=$?; [ "$status" -eq 1 ]'
bash -c 'rg "CK_LEARNING_PROPOSALS_LEDGER|resolve_from_learning_proposals" crates/ck-mcp crates/ck-bin; status=$?; [ "$status" -eq 1 ]'
rg -F 'SEA-Forge authorizes' docs/security.md
rg -F 'SWE_SEED' docs/mcp-contracts.md
rg -F 'authority decision reference' docs/mcp-contracts.md
rg -F 'rebuild-index' docs/operations.md
rg -F 'canonical learning-proposal' docs/operations.md
rg -F 'mcp.enable_stdio' docs/configuration.md
rg -F 'mcp.sse.base_path' docs/configuration.md
rg -F 'GitHub repositories' README.md
rg -F 'websites' README.md
rg -F 'FTS rebuild' docs/schema.md
rg -F 'cross-system-conformance' .github/workflows/cross-system-conformance.yml
rg -F 'cargo clippy --workspace --all-targets -- -D warnings' .github/workflows/ci.yml
```

**Done when:** Every audit correction maps to a passing teeth test, both cross-system flows preserve authority/execution/developmental distinctions, documentation matches runtime, and deleting any required integration from the built distribution or CI makes a gate fail.

**Redesign trigger:** If a cross-repository prerequisite is not released, do not claim the dependent CK feature complete; keep it feature-disabled/fail-closed and record the exact external blocker.

---

## Final acceptance checklist (whole plan)

- [ ] Existing reload, enum, observability, containment, compensation, bind, DNS fail-closed, and freshness fixes have direct regression evidence. *(Task 0)*
- [ ] SEA owns authority events, SWE_SEED preserves authority/proof evidence, and CK emits only correlated context packets. *(Task 1)*
- [ ] Protected retrieval derives corpus policy server-side, verifies SEA evidence before reads, and has no MCP filesystem dependency. *(Task 2)*
- [ ] Learning proposals come only from canonical GodSpeed persistence and CK remains a read-only rebuildable projection. *(Task 3)*
- [ ] Canonical/FTS writes are atomic, document IDs stable, payload failures compensated, and FTS rebuild is proven. *(Task 4)*
- [ ] Broad `StoragePort` and unrelated/unimplemented test fakes are gone. *(Task 5)*
- [ ] Web requests use only validated DNS addresses and cannot rebind to private targets. *(Task 6)*
- [ ] Filesystem, GitHub, web, and canonical learning-proposal source kinds are runtime-wired through matching pipelines and docs match tests. *(Task 7)*
- [ ] `ck-scheduler` is the sole refresh owner with one job lifecycle and restart recovery. *(Task 8)*
- [ ] SSE/scheduler workers drain/join on EOF and readiness reflects live dependencies. *(Task 9)*
- [ ] Every retained config field changes runtime behavior or is fail-fast removed; service/log identity is configured. *(Task 10)*
- [ ] Cross-system allow/deny flows, architecture checks, docs, and `.agents` evidence all pass. *(Task 11)*
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [ ] `README.md`, `docs/mcp-contracts.md`, `docs/security.md`, `docs/configuration.md`, `docs/operations.md`, and `docs/schema.md` contain no status ahead of passing runtime tests.
- [ ] All global gates green; SEA generated artifacts were regenerated from source and inspected.

## Guardrails (do not violate)

- Do not build a SEA policy engine, SWE_SEED proof runner, GodSpeed settlement/capability lifecycle, or developmental ledger in Context Kernel.
- Do not accept caller `is_private`, `scope_claim`, bearer admission, or CK corpus allowlists as SEA authority evidence.
- Do not hand-edit SEA generated outputs; change the `.sea` source and regenerate.
- Do not fabricate citations, timestamps, event IDs, domain hashes, proof results, or developmental records.
- Keep SQL/HTTP/filesystem/transport types out of `ck-core`; keep canonical GodSpeed records read-only and CK indexes rebuildable.
- Preserve all existing uncommitted harden work; use task-scoped commits in each repository and never reset unrelated changes.
