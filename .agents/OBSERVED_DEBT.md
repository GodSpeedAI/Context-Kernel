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
- [OPEN] GitHub/web adapters implemented + tested but not runtime-wired (`AppServices` takes a single `source_adapter`).
- [OPEN] Scheduler crate tested but bypassed by a separate runtime loop.
- [OPEN] ACL context retrieval introduces a concrete filesystem-adapter dependency into `ck-mcp`.
- [OPEN] `StoragePort` is broad enough to force large inert test implementations.

## Product Contract
- [OPEN] `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, `auth.trust_mode` parsed but ignored (observability fields now wired).
- [RESOLVED] Freshness score ignores timestamp age and TTL.
- [OPEN] README overstates runtime source-adapter support (GitHub/web).

## Reports
- Audit: `.agents/reports/neatcode-audit-2026-07-29.md`.
- Harden: `.agents/reports/neatcode-harden-2026-07-29.md`.

