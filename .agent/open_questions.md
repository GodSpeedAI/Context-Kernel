# Open Questions

## Blocking
- None.

## Non-Blocking
- `.github/copilot-instructions.md` is required by `AGENTS.md` but is missing.
- Whether `export-schema` should emit live SQLite schema text or docs/schema.md content.
- Session memory helpers (retention policy, event typing, pinned facts) are not yet in a separate ck-memory crate; they're embedded in ck-app and ck-storage-sqlite.
- Vector search does not filter by corpus_id/library_id/session_id at the storage layer for v1; filtering happens at the app layer or is deferred.
- remove_chunk_embeddings_by_document is a no-op in ck-vector for v1; the app layer should delete by chunk IDs.
- SSRF protection does not perform DNS resolution; DNS rebinding attacks are not mitigated for v1.
- GitHub adapter uses tree API which may not work for very large repos; pagination support may be needed.
- Web adapter does not implement ETag/Last-Modified caching for incremental refresh in v1.

## Active Assumptions
- The service binary name is `context-kernal`, matching the plan.
- Synchronous rusqlite access is acceptable; async can be added later.
- The `data_dir` CLI flag is used for artifact storage; the database path is separate via `--database`.
- Compatibility aliases (resolve-library-id, get-library-docs, query-docs) map to the canonical tools as specified in the plan.
- `SourceHealth::Degraded` is the intended failing health state unless the domain model is explicitly expanded later.
- Brute-force cosine similarity is acceptable for v1 vector search; can be swapped for sqlite-vec or external vector DB later.
- OpenAI-compatible embedding API is the primary semantic provider; other providers can be added behind EmbeddingPort.
