# Open Questions

## Blocking
- None.

## Non-Blocking
- `.github/copilot-instructions.md` is required by `AGENTS.md` but is missing.
- Review text referenced `SourceHealth::Failing` and `source.corpus_id`, but current code has `SourceHealth::Degraded` and a `corpus_sources` relation instead.
- Whether `export-schema` should emit live SQLite schema text or docs/schema.md content.
- Whether `refresh_source` should be a separate tool or if `ingest_source` with upsert mode is sufficient for M1-M3.
- The ranker currently passes through FTS scores without enriching with document/chunk metadata; full enrichment requires storage lookups in the ranker or app layer.
- Session memory helpers (retention policy, event typing, pinned facts) are not yet in a separate ck-memory crate; they're embedded in ck-app and ck-storage-sqlite.

## Active Assumptions
- The service binary name is `context-kernal`, matching the plan.
- Synchronous rusqlite access is acceptable for M1-M3; async can be added later.
- The `data_dir` CLI flag is used for artifact storage; the database path is separate via `--database`.
- Compatibility aliases (resolve-library-id, get-library-docs, query-docs) map to the canonical tools as specified in the plan.
- `SourceHealth::Degraded` is the intended failing health state unless the domain model is explicitly expanded later.
