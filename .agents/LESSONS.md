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

## Lesson: Persisted identifiers must not come from `std::hash::DefaultHasher`
- Observed: 2026-10-24
- Evidence: `ck-app::stable_document_id` used `DefaultHasher`; its algorithm is not guaranteed across Rust releases, so stored document IDs could change on a toolchain upgrade. Replaced with length-prefixed SHA-256 (`ck_core::entities::stable_document_id`) and a v2 storage migration.
- Scope: any ID or hash written to disk or sent across a boundary.
- Confidence: high
