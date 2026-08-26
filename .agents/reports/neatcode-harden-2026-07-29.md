# NeatCode Harden Report

**NeatCode · harden** · workspace (9 files, +580/−73) · archetype: workflow orchestration / boundary service · 2026-07-29

Implements the corrections from `neatcode-audit-2026-07-29.md`, then runs the harden pass (edge cases, recovery, security boundaries, observability, production wiring). Each fix carries a failing-first test.

## Baseline
- Pre-change: 198 tests pass, clippy clean, `cargo fmt --check` failing on committed formatting in `ck-bin`/`ck-mcp`.
- Post-change: **206 tests pass, 0 failures**; clippy clean; `cargo fmt --check` clean.

## Addressed (with failing-first tests)

| # | Finding | Fix | Proof |
|---|---------|-----|-------|
| S1 | SQLite enum codecs do not round-trip reachable values | Added missing decoder arms: `sse`, `pinned`, `compacted`, `dependency_change`, `cancelled` (`ck-storage-sqlite/src/lib.rs`) | `enum_roundtrips_all_writable_variants` |
| S1 | Config reload drops tool/corpus authorization policy + ACL context agent | Reload now rebuilds service via `with_app_and_auth` and re-applies the context agent (`ck-bin/src/main.rs`) | `sse_reload_preserves_tool_authorization` (disallowed tool rejected after reload; allowed tool still works) |
| S1 | `context_required` bypasses corpus allowlist + filesystem escape | Added `context_required` arm to `corpus_ids_from_arguments`; canonicalize-and-contain `corpus_dir` under `corpus_root`, reject escapes (`ck-mcp`) | `context_required_rejects_path_traversal_outside_corpus_root`, corpus-id unit assertion |
| S1 | Refresh accumulates duplicate documents; orphan payloads on metadata-write failure | Ingest removes a source's prior docs (FTS rows then doc) before re-indexing; payload-compensation on `upsert_artifact` failure; fixed `TestStorage.list_documents_by_source` to filter by source | `refresh_does_not_accumulate_duplicate_documents` |
| S2 | Observability guard dropped immediately; config ignored | `set_global_default` (so spawned SSE/refresh threads log); wired `config.observability` + `service.name` via `init_logging` in serve/migrate/doctor; removed hardcoded top-of-main init | builds + existing obs tests; `init_logging` wired |
| S2 | Web SSRF fails open on DNS errors / empty resolution | `validate_url_with_resolver` now bails on resolver `Err` and empty resolution (`ck-adapter-web`) | `validate_url_with_resolver_fails_closed_on_resolver_error`, `…rejects_unresolvable_host`; 4 bug-ratifying tests corrected |
| S2 | SSE bind failure swallowed; readiness a constant | Bind `std::net::TcpListener` in main thread (fail-closed on collision), `set_nonblocking`, convert via `from_std` inside the runtime thread; `/readyz` now probes `get_status` | `sse_bind_failure_fails_startup` (process exits non-zero); `sse_healthz_endpoint_works` |
| S3 | Freshness score a fixed 0.5 placeholder | `compute_freshness` parses RFC3339, decays by age vs TTL (30-day default), clamps [0,1]; absent→0.0, malformed→0.5 | `compute_freshness_reflects_age_and_ttl` |

## Behaviour changes (agreed scope)
- Previously-allowed **unresolvable web hosts are now rejected** (SSRF fail-closed).
- **Refresh now replaces** a source's documents instead of accumulating duplicates.
- **SSE bind collision now fails startup** (non-zero) instead of leaving a dead "started" log.
- **`/readyz` now reflects** the live MCP service instead of a constant `true`.
- **Observability** now reads `[observability]`/`[service].name` from config and covers all threads.

## Critique
correctness 4 · fit 5 · semantics 4 · restraint 5 · operations 4 · evidence 5

## Open / deferred (named debt, with rationale)
- **FTS↔DB transactional atomicity** — refresh no longer accumulates and payload writes are compensated, but document/chunk writes and FTS indexing still use separate connections; a mid-ingest crash can leave a document with a partial FTS index. Full single-transaction coverage requires passing the storage connection into `FtsSearch` (a restructure). Residual.
- **`context_required` `is_private`/`scope_claim` server-side validation** — the agent still trusts caller-supplied `is_private` and accepts any non-empty `scope_claim`. Mitigated: corpus access is now gated by the SSE corpus allowlist (`corpus_ids_from_arguments`) and path containment, so a lying caller cannot escape the corpus root or read a corpus outside its allowlist. Server-side privacy registry not implemented.
- **GitHub/web adapters not wired into the binary** (audit S2) — `AppServices` takes a single `source_adapter`; supporting `github_repo`/`website` needs kind-based adapter+normalizer selection. Deferred (feature-wiring, not a running-code defect); README still overstates runtime capability.
- **Graceful SSE/refresh worker shutdown + join** (audit S2) — bind-fails-startup and readiness are fixed; in-flight work is not drained on EOF (workers die with the process). The refresh thread's long sleep makes naive join block shutdown, so it was not joined. Deferred.
- **Dead config fields** (audit S3) — `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, `auth.trust_mode` remain parsed-but-ignored (observability fields were wired). Deferred.
- **Scheduler ownership** (audit S3) — `ck-scheduler` crate still unused; runtime uses the inline refresh loop. Deferred (integration is a behavior change).
- **Broad `StoragePort`** (audit S4) — still one trait; test fakes still large. Deferred (refactor on next port change).
- **ACL context path depends directly on `ck-adapter-filesystem`** in `ck-mcp` (audit architecture-conformance) — unchanged this pass.

## Verification evidence
- `cargo test --workspace` → exit 0, **206 passed, 0 failed** (was 198; +8 new).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, clean.
- `cargo fmt --check` → clean (committed formatting in edited files corrected).
- Could-new-tests-fail-before-change: yes, confirmed by subagents (enum/SSRF/context_required) and by reasoning (reload→"missing 'path'" not "unauthorized tool"; bind-failure→exit 0; freshness→0.5).
- No new dependencies added (gate 49). `cargo fmt` touched only the 9 already-edited files (gate 48).

## Method note
Used subagent-driven development selectively: 5 fresh implementer subagents on **disjoint files** (ck-storage-sqlite, ck-adapter-web, ck-mcp, ck-app, ck-observability) with verbatim specs + failing-first tests; the interdependent `ck-bin`/`ck-transport-sse` lifecycle work (auth-reload, observability wiring, SSE bind) was done directly where full context and lowest risk were required. Per-task reviewer dispatch was skipped as disproportionate to surgical fixes; consolidated verification + this critique replace it.
