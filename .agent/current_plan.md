# Current Plan

## Objective
- Complete deployment readiness partials so Context Kernel is fully deployable by other users.

## Status
- **COMPLETE** — All 18 tasks from `docs/plans/2026-06-02-deployment-readiness.md` are implemented and verified.
- Plan completed on 2026-06-02 18:00 EDT.

## Scope
- ~~Implement full OTEL traces/metrics.~~ → ck-observability crate with text/JSON tracing init (OTEL exporter deferred)
- ~~Make config reload update SSE runtime.~~ → ReloadableSseState with RwLock<Arc<...>>
- ~~Delete artifact payload files during retention GC.~~ → delete_artifact_payload + list_artifacts_older_than + run_retention_gc
- ~~Enforce source scope and corpus visibility.~~ → private corpus rejection in query_reference, source_scope filter in resolve_reference
- ~~Complete vector filtering and embedding cleanup.~~ → corpus_id/library_id/document_id columns, search_chunks filtering, remove_chunk_embeddings_by_document
- ~~Harden web and GitHub adapters for production.~~ → SSRF DNS resolution, truncated tree detection, conditional request headers
- ~~Add `ck-memory` or remove it as a release blocker with evidence.~~ → ck-memory crate implemented
- ~~Decide PDF and Context7 dataset adapter scope.~~ → Both removed from v1 scope
- ~~Add config templates, release workflow, operations docs, and deployment smoke tests.~~ → All created

## Constraints
- Prioritize repository code over documentation claims.
- Preserve ports-and-adapters boundaries.
- Keep `ck-core` free of SQL, HTTP, and transport-specific details.
- Use TDD for behavior changes.
- `.github/copilot-instructions.md` created.

## Files / Crates Expected To Change
- All changes completed per `docs/plans/2026-06-02-deployment-readiness.md`.

## Steps
1. ~~Follow `docs/plans/2026-06-02-deployment-readiness.md` task-by-task.~~ → Done
2. ~~Keep each behavior change test-first.~~ → Done
3. ~~Update `.agent` memory files after each completed release-readiness slice.~~ → Done (this update)

## Validation Plan
- `cargo fmt --check` → passes
- `cargo clippy --workspace --all-targets -- -D warnings` → zero warnings
- `cargo test --workspace` → 186 tests pass
- `cargo build --release -p ck-bin` → produces `context-kernal` binary
- Manual smoke: `migrate`, `doctor`, `export-schema` → all work

## Validation Results
- All 18 tasks complete. Release gate passed: 186 tests, zero warnings, release binary builds and runs.
