# Current Plan

## Objective
- Implement every correction in `.agents/reports/neatcode-audit-2026-07-29.md`, constrained by the SWE_SEED/SEA-Forge/GodSpeed-Agent ownership boundaries. Plan: `.agents/plans/2026-07-29_audit_corrections.md`.

## Status
- Task 0 (baseline evidence) COMPLETE.
- Task 1 (SEA/SWE context authority contract) COMPLETE: cross-repo contract corrected, regenerated, all consumers updated.
- Task 4 (data integrity) PARTIAL: fetch-before-delete + stable IDs + FTS rebuild done; full transactional atomicity pending.
- Tasks 2, 3, 5-11 not started.

## Completed this session
- Task 0: `scripts/run-exact-test.sh`; +7 evidence tests (persisted enums, artifact compensation, freshness table, reload corpus+context+no-tokens, /readyz HTTP, JSON log success/failure, JSON log SSE+refresh threads). RED→GREEN proven for enum decoder + artifact compensation.

## Next (when resumed)
- Task 1 (SEA/SWE context-authority contract) — cross-repo: SEA spec + codegen, SWE client, CK fixtures. Requires `SEA_ROOT`/`SWE_SEED_ROOT`/`GODSPEED_ROOT`.
- Task 4 (atomic writes + stable IDs + FTS rebuild) — CK-only, no cross-repo dep; can proceed independently of Task 1.
- Task 6 (DNS address pinning) — CK-only.
- Then Tasks 2, 3, 5, 7, 8, 9, 10, 11 per the dependency graph in the plan.

## Authority boundaries (unchanged)
- SEA owns action authority, SWE_SEED owns execution/proof evidence, GodSpeed-Agent owns developmental truth, and Context Kernel owns retrieval only.
