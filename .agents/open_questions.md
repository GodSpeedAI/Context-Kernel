# Open Questions

## Blocking
- None.

## Non-Blocking (require a maintainer decision)
- Is `context_required` intended for SSE exposure, or trusted stdio only? Server-side `is_private`/`scope_claim` validation depends on the answer.
- Should GitHub/web adapters ship in the binary now (kind-based selection), or should the README mark them library-only/incomplete until wired?
- Should empty authorization allowlists mean allow-all (current) or deny-all?
- Is full FTS↔DB transactional consistency worth threading the storage connection into `FtsSearch` (a restructure)?
- Should graceful SSE/refresh shutdown be added (drain in-flight requests on EOF), accepting the refresh-thread long-sleep rework it requires?

## Non-Blocking
- Private-repository indexing and Low/free DeepWiki availability depend on the connected Devin account; verify in the Devin UI after merge. The Git configuration alone does not establish entitlement or generated wiki success.
- Observability is structured tracing plus config for an OTEL endpoint; no OpenTelemetry exporter/metrics backend is wired yet (ck-observability crate exists with text/JSON tracing init). This is a future enhancement, not a v1 blocker.
- Context7 dataset adapter is not part of v1 deployment. Context7 compatibility is provided through tool-contract aliases (resolve-library-id, get-library-docs, query-docs) only. Can be added as a future adapter behind SourceAdapter port.
- PDF source adapter is not part of v1 deployment. Filesystem, GitHub, and web adapters cover the primary documentation use case. Can be added as a future adapter behind SourceAdapter port.
- Config reload for SSE updates the ReloadableSseState but does not restart the axum server (port/listen changes require restart). This is acceptable for v1.
- Vector search uses brute-force cosine similarity; sqlite-vec or external vector DB can be swapped in later without domain changes.
- AGENTS.md mandates `just context-check` before handoff, but the repository has no justfile defining that recipe; add the justfile or relax the requirement.

## Active Assumptions
- MCP arguments received over SSE are untrusted.
- Corpus roots can contain sensitive files and must enforce containment independently of caller claims.
- The README's implemented-feature list describes the shipped binary.
- Deferred findings (GitHub/web wiring, scheduler ownership, broad StoragePort, remaining dead config fields) are acceptable to leave as tracked debt for now.
- The service binary name is `context-kernal`, matching the plan.
- Synchronous rusqlite access is acceptable; async can be added later.
- The `data_dir` CLI flag is used for artifact storage; the database path is separate via `--database`.
- Compatibility aliases (resolve-library-id, get-library-docs, query-docs) map to the canonical tools as specified in the plan.
- `SourceHealth::Degraded` is the intended failing health state unless the domain model is explicitly expanded later.
- OpenAI-compatible embedding API is the primary semantic provider; other providers can be added behind EmbeddingPort.
- `export-schema` emits the live embedded SQLite migration SQL from `ck-storage-sqlite`.
- Release workflow builds for x86_64-unknown-linux-gnu only; cross-compilation for other targets can be added to `.github/workflows/release.yml`.

## Deferred Debt (tracked in OBSERVED_DEBT.md)
- FTS↔DB transactional atomicity; `is_private` server-validation; GitHub/web wiring; graceful shutdown; `semantic_enabled`/`enable_stdio`/`base_path`/`trust_mode` config fields; scheduler ownership; broad `StoragePort`; ACL→filesystem-adapter coupling in `ck-mcp`.
