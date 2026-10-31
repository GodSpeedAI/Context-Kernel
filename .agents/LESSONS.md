# Lessons

Verified, reusable project lessons only. Each entry needs evidence and scope.
Remove or rewrite an entry when superseded; update entries instead of
duplicating them.

<!-- Entry format:
## Lesson: <short statement>
- Observed: YYYY-MM-DD
- Evidence: file, test, command, or incident
- Scope: where this lesson applies
- Confidence: high | medium | low
-->

## Lesson: Cross-check worker partition assignments against the planned directory list before root compilation
- Observed: 2026-10-30
- Evidence: During the EGM bootstrap run, regrouping crates into worker partitions silently dropped `crates/ck-bin` (44 of 45 expected worker files). The gap surfaced only at root compilation when compiling `crates/README.md` against worker payloads; a quick file-count reconciliation (`wc -l` / `ls` over expected paths) caught it and a follow-up worker (W8) produced the missing contracts.
- Scope: any multi-worker generation run in this repo (EGM bootstrap, parallel refactors): reconcile the emitted artifact list against the discovery map before compiling parent-level rollups.
- Confidence: high

## Lesson: Hashing raw JSON floats breaks across parse round-trips; normalize numbers in canonical hash forms
- Observed: 2026-10-07
- Evidence: serde_json's fast float parser renders `0.44999999999999996` as `0.45` after a parse round-trip; the CEP `context_bundle` integrity hash declared by CK did not recompute in SWE_SEED on a file round-trip (`e2e_world_loop` hop 2 failed with `integrity mismatch`). Fixed by normalizing floats to 6 decimals (and `-0.0` to `0.0`) inside `ck-mcp::cep_bundle::canonical_json` and `swe_seed_core::context_bundle::bundle_content_hash`.
- Scope: every cross-boundary content/integrity hash over JSON payloads.
- Confidence: high

## Lesson: `INSERT OR REPLACE` on a parent row cascade-deletes its children under `ON DELETE CASCADE`
- Observed: 2026-10-07
- Evidence: `SqliteStore::upsert_source`/`upsert_corpus` used `INSERT OR REPLACE`; updating a source's freshness timestamp (or a corpus's visibility) silently deleted every document and chunk of that source/corpus. Retrieval returned 0 results after any metadata upsert. Fixed with `ON CONFLICT ... DO UPDATE`, and re-ingestion now preserves registered corpus visibility.
- Scope: `ck-storage-sqlite` upsert paths and any future FK-bearing schema.
- Confidence: high

## Lesson: The GodSpeed integration path must route through the real app retrieval pipeline
- Observed: 2026-10-07
- Evidence: `AclContextAgent` previously did ad hoc substring matching over `corpus_root` plus a JSONL ledger reader, bypassing FTS, ranking, trust/freshness, scope enforcement, and budgets. Replaced with `AppServices::resolve_context` (auto-ingest + `query_reference`) and a kind-selected `LearningProposalsAdapter` ingestion path; `crates/ck-mcp/tests/context_pipeline.rs` covers the behaviors only the real pipeline proves.
- Scope: any boundary adapter in `ck-mcp` or downstream integration crates.
- Confidence: high
## Lesson: Documentation of a wire contract must be derived from the handler's response assembly, not the legacy schema
- Observed: 2026-10-24
- Evidence: `docs/workflows/agentic-capability-loop.md` and `docs/README.md` described `ContextPacketCreated` as the output while `ck-mcp/src/lib.rs:L562-L569` returned `context_bundle` first with the packet deprecated. Reading the handler's JSON construction exposed the drift; the schema docs alone did not.
- Scope: any doc describing MCP tool responses or cross-system envelopes.
- Confidence: high

## Lesson: `file:///` links in repository docs resolve to the host filesystem root, not the repo
- Observed: 2026-10-24
- Evidence: 33 links across 9 pages used `file:///crates/...`; a link checker resolved them to `/crates/...` and failed. Rewriting to depth-correct relative paths (`../crates/` from `docs/`, `../../crates/` from `docs/<dir>/`) made every link resolve.
- Scope: markdown documentation in any repository.
- Confidence: high

## Lesson: Recall must be verified by content after a process restart (2026-10-31)
- Verified: the ranker returned event IDs with empty text and kind `unknown`; the earlier test only asserted the keys `query_id`/`results` existed.
- Evidence: probe over stdio on the debug binary (2026-10-31); `crates/ck-bin/tests/installed_package.rs::memory_survives_process_restart_and_is_recalled_with_content`.
- Scope: any MCP tool assertion should parse JSON and check the payload value, not only key presence.

## Lesson: Blocking lifecycle changes need the test suite run with a timeout (2026-10-31)
- Verified: making `--sse` keep running after stdin EOF made `sse_healthz_endpoint_works` hang (it waited on process exit).
- Evidence: `crates/ck-bin/tests/cli.rs::sse_healthz_endpoint_works` rewritten to read the response line and stop the child.
- Scope: `crates/ck-bin/tests/`; run tests with `timeout` or in background.

## Lesson: Linux ARM64 artifacts must be built on the target's glibc, not the newest runner (2026-10-31)
- Verified: `ubuntu-24.04-arm` would produce binaries requiring glibc newer than the Jetson's 2.35.
- Evidence: `.github/workflows/package.yml` (`ubuntu-22.04-arm`), `scripts/package/linux/build-linux.sh` (`CK_MAX_GLIBC`).
- Scope: all Linux packaging.

## Lesson: Persisted identifiers must not come from `std::hash::DefaultHasher`
- Observed: 2026-10-24
- Evidence: `ck-app::stable_document_id` used `DefaultHasher`; its algorithm is not guaranteed across Rust releases, so stored document IDs could change on a toolchain upgrade. Replaced with length-prefixed SHA-256 (`ck_core::entities::stable_document_id`) and a v2 storage migration.
- Scope: any ID or hash written to disk or sent across a boundary.
- Confidence: high
