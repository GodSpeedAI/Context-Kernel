# Current Plan

## Objective
- Verify and address still-valid review findings from `tmp/cr.md`, then strike each completed item in that file.

## Status
- Completed 2026-06-02 08:06 EDT.
- `tmp/cr.md` review text struck with completion notes added.
- Validation passed: `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --workspace`.

## Scope
- Scheduler/storage atomic refresh job enqueue and claim.
- Ranker FTS score normalization.
- Config MCP stdio defaults.
- App refresh/ingest health and excerpt behavior.
- Auth fail-closed token validation and SSE auth misconfiguration handling.
- Evidence metadata requested by review.
- Project memory updates.

## Constraints
- `.github/copilot-instructions.md` is required by AGENTS.md but missing; proceed with source code and `.agent` memory as evidence.
- Preserve ports-and-adapters boundaries; storage primitives belong in `StoragePort` and adapter implementations.
- Use the existing `SourceHealth::Degraded` variant for failed ingest state because `SourceHealth::Failing` does not exist.
- Keep changes minimal and add focused regression tests before production changes.

## Files / Crates Expected To Change
- `tmp/cr.md`
- `.agent/current_plan.md`
- `.agent/project_state.md`
- `.agent/work_log.md`
- `.agent/evidence.md`
- `.agent/open_questions.md`
- `.agent/decisions.md`
- `crates/ck-core/src/ports/storage.rs`
- `crates/ck-core/src/ports/scheduler.rs`
- `crates/ck-storage-sqlite/src/lib.rs`
- `crates/ck-scheduler/src/lib.rs`
- `crates/ck-ranker/src/lib.rs`
- `crates/ck-config/src/lib.rs`
- `crates/ck-app/src/lib.rs`
- `crates/ck-auth/Cargo.toml`
- `crates/ck-auth/src/lib.rs`
- `crates/ck-bin/src/main.rs`

## Steps
1. Completed — Add focused regression tests for still-valid review findings.
2. Completed — Run targeted tests and confirm expected failures.
3. Completed — Implement minimal production changes through existing ports and adapters.
4. Completed — Run targeted tests, then workspace format/test/clippy validation.
5. Completed — Strike completed items in `tmp/cr.md`.
6. Completed — Update project memory files with evidence and any decisions/open questions.

## Validation Plan
- Passed: `cargo test -p ck-config`
- Passed: `cargo test -p ck-auth`
- Passed: `cargo test -p ck-ranker`
- Passed: `cargo test -p ck-app`
- Passed: `cargo test -p ck-storage-sqlite`
- Passed: `cargo test -p ck-scheduler`
- Passed: `cargo test --workspace`
- Passed: `cargo fmt --check`
- Passed: `cargo clippy --workspace`
