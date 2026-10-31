# Observed Debt

Status legend: [OPEN] unresolved · [MITIGATED] addressed but residual · [RESOLVED] fixed.

## Critical
- [RESOLVED] `context_required` corpus authorization bypass + filesystem path-traversal escape.
- [RESOLVED] Config reload drops MCP tool/corpus authorization policy (+ context agent).
- [RESOLVED] SQLite enum codecs do not round-trip reachable persisted variants.
- [MITIGATED] Context/ingestion write consistency: refresh no longer accumulates; artifact payload compensated on metadata-write failure. RESIDUAL: FTS indexing and document/chunk writes use separate connections — a mid-ingest crash can leave a partial FTS index. Full single-transaction coverage needs the storage connection threaded into `FtsSearch` (restructure).
- [RESOLVED] `context_required` still trusts caller-supplied `is_private`/`scope_claim`. Mitigated by SSE corpus allowlist (`corpus_ids_from_arguments`) + path containment; server-side privacy registry not implemented. (2026-10-07: RESOLVED — registered `corpora.visibility` is now authoritative in `resolve_context`; private corpora require explicit `scope_claim` (POL-ACL-002) and return `requires_scope` otherwise; re-ingestion preserves registered visibility; a caller-declared-private unregistered corpus is never auto-ingested.)

## Operational
- [RESOLVED] Observability guard dropped immediately; config ignored.
- [MITIGATED] SSE lifecycle: bind-fails-startup + readiness probe fixed. RESIDUAL: graceful SSE/refresh worker shutdown + join on EOF (workers die with the process).
- [MITIGATED] Web SSRF fails closed on DNS errors/empty resolution. RESIDUAL: DNS rebinding — validation and connection use separate resolutions (pinning validated IP into `ureq` not done; adapter still not wired into the binary).

## Architectural
- [OPEN] EGM contract READMEs (48 files, generated 2026-10-30) have no automated drift gate: nothing fails CI when a contract falls behind its code. One-time drift check on 2026-10-30 found and fixed 2 missing public types in `crates/ck-mcp/src/README.md` (evidence: `.agents/reports/2026-10-30_EGM_DRIFT_REPORT.md`). Next move: extend a docs CI link/section checker with a pub-symbol coverage scan (script exists in session history; pattern documented in the drift report), or run `egm-drift-report` after contract-relevant changes.
- [OPEN] GitHub/web adapters implemented + tested but not registered. Multi-kind routing now exists (`AppServices::kind_adapters`, `adapter_for_kind`); `ck-bin` registers only `learning_proposals_jsonl`, and `ck-bin/Cargo.toml` declares no dependency on either crate. Next move: add deps and call `with_source_adapter_kind` in `wire_app` (evidence: `crates/ck-bin/src/main.rs:L376-L395`, `crates/ck-app/src/lib.rs:L34,L176`).
- [OPEN] Scheduler crate tested but bypassed by a separate runtime loop.
- [MITIGATED] ACL context retrieval no longer constructs filesystem/ledger readers: it calls `AppServices::ingest_source` with `source_kind` and `resolve_context`. RESIDUAL: `ck-mcp/Cargo.toml` still declares `ck-adapter-filesystem` as a normal dependency, used only by test helpers (`crates/ck-mcp/src/lib.rs:L871`, inside `#[cfg(test)]`); move it to `[dev-dependencies]`.
- [OPEN] `StoragePort` is broad enough to force large inert test implementations.
- [OPEN] The settlement-backed `ContextEffectStore` (append-only JSONL, per-world) is implemented inside `ck-app` (`crates/ck-app/src/context_effect.rs`), so application logic owns a file-format detail with no port. Next move: extract a `ContextEffectPort` in `ck-core` and move JSONL IO to an adapter crate (evidence: `crates/ck-app/src/context_effect.rs:L201`, wired at `crates/ck-bin/src/main.rs:L430`).
- [OPEN] Documentation had broken `file:///crates/...` links in 9 pages (they resolve to filesystem root `/crates`, not the repo). Fixed on 2026-10-24 by rewriting to depth-correct relative paths; a link-check should be part of any docs CI (evidence: `rg 'file:///' docs` now returns nothing).
- [OPEN] Repo `AGENTS.md` mandates `just context-check` before handoff, but there is no justfile at the repo root, so the gate cannot run (evidence: `ls justfile Justfile` both absent).

## Product Contract
- [OPEN] `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, `auth.trust_mode` parsed but ignored (observability fields now wired).
- [RESOLVED] Freshness score ignores timestamp age and TTL.
- [MITIGATED] Root README tool list and SWE_SEED section updated on 2026-10-24 to the bundle-first surface. RESIDUAL: the README does not state that GitHub/web adapters are unwired; consider an explicit status note (evidence: `README.md` core tool sections).

## Reports
- Audit: `.agents/reports/neatcode-audit-2026-07-29.md`.
- Harden: `.agents/reports/neatcode-harden-2026-07-29.md`.


- [OPEN] Config reload is triggered only by stdin requests (`crates/ck-bin/src/main.rs` stdin loop). A standalone SSE service with closed stdin cannot reload config. Next move: add a reload trigger (e.g. SIGHUP) with the same authorization-preservation tests, or document as unsupported.
- [OPEN] Unauthorized tool refusal over SSE returns HTTP 200 with JSON-RPC `error.code = "internal"` (`src/main.rs` via `crates/ck-mcp`), not an authorization code. Next move: classify as an authorization error and add a test asserting the code.
- [OPEN] `doctor` opens and migrates the database; it is not read-only (`crates/ck-bin/src/main.rs` `doctor`). Next move: add a read-only path or document (documented in `docs/install.md`).
- [OPEN] `config/context-kernal.example.toml` uses a relative `database_path = "./data/kernel.db"`, which resolves against the working directory. Next move: annotate or ship an absolute installed example.
- [OPEN] Recall scope/`kind` for `RecalledEvent` was hard-coded placeholder before 2026-10-31 (fixed by hydration; no regression test for the Current/Recent scope with hydration yet). Next move: add a scope test.
- [OPEN] Windows and macOS packages are unexecuted in this environment (no pwsh, no hdiutil/pkgbuild). Next move: verify via `ci.yml` package jobs; fix any failures.
- [OPEN] `actionlint` is unavailable here (mise shim has no version); workflows checked only for YAML syntax. Next move: run actionlint in CI or install a pinned version.
