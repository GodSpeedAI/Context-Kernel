# crates

## Overview
The Cargo workspace crate set of Context Kernel, a Rust ports-and-adapters MCP service. `ck-core` holds domain entities, port traits, and vocabulary types only; `ck-app` implements use-case orchestration against injected ports; the `ck-adapter-*`, `ck-artifacts`, `ck-auth`, `ck-embeddings`, `ck-search-fts`, `ck-storage-sqlite`, `ck-transport-*`, and `ck-vector` crates are infrastructure adapters implementing those ports; `ck-bin` is the composition/bootstrap binary; `ck-mcp` is the MCP boundary adapter. Every crate directory below links its child contract; each crate's `src/` carries its own component contract.

## Subdirectories & Boundaries
- ck-adapter-filesystem: [./ck-adapter-filesystem/README.md](./ck-adapter-filesystem/README.md) - filesystem and learning-proposals JSONL source adapters plus markdown normalizer and heading chunker; composed into the binary.
- ck-adapter-github: [./ck-adapter-github/README.md](./ck-adapter-github/README.md) - GitHub REST API source adapter and normalizer; implemented and tested but not composed into `ck-bin`.
- ck-adapter-web: [./ck-adapter-web/README.md](./ck-adapter-web/README.md) - web source adapter with SSRF guardrails (scheme/IP/DNS/allowlist validation, redirects disabled); implemented and tested but not composed into `ck-bin`.
- ck-app: [./ck-app/README.md](./ck-app/README.md) - application layer: `AppServices` use-case orchestrator, the typed context resolution pipeline, and the settlement-backed context-effect JSONL ledger.
- ck-artifacts: [./ck-artifacts/README.md](./ck-artifacts/README.md) - filesystem artifact payload store implementing `ArtifactPort` with artifact-ID containment.
- ck-auth: [./ck-auth/README.md](./ck-auth/README.md) - bearer-token `AuthPort` adapter with constant-time comparison and tool/corpus allowlists.
- ck-bin: [./ck-bin/README.md](./ck-bin/README.md) - composition/bootstrap crate for the `context-kernal` binary: CLI (serve, migrate, doctor, export-schema, rebuild-index) and adapter wiring.
- ck-config: [./ck-config/README.md](./ck-config/README.md) - typed TOML configuration tree, canonical defaults, and the loader with `${VAR}` environment expansion.
- ck-core: [./ck-core/README.md](./ck-core/README.md) - innermost domain crate: entities, port traits, and vocabulary types only (no SQL, transport, or HTTP).
- ck-embeddings: [./ck-embeddings/README.md](./ck-embeddings/README.md) - no-op and OpenAI-compatible `EmbeddingPort` providers.
- ck-memory: [./ck-memory/README.md](./ck-memory/README.md) - session-memory retention vocabulary (event classes, retention policy, recall scope); pure domain logic, no I/O.
- ck-mcp: [./ck-mcp/README.md](./ck-mcp/README.md) - MCP tool dispatcher, ACL context agent for the agentic-capability-loop boundary, and CEP `godspeed.context_bundle` projection with canonical-JSON integrity.
- ck-observability: [./ck-observability/README.md](./ck-observability/README.md) - process-wide tracing subscriber installation (JSON/text, env-filtered); OTEL endpoint configured-for but not implemented.
- ck-ranker: [./ck-ranker/README.md](./ck-ranker/README.md) - intent-weighted ranking adapter (`RankerPort`) with signal combination and per-document de-duplication.
- ck-scheduler: [./ck-scheduler/README.md](./ck-scheduler/README.md) - refresh-queue orchestration (`SchedulerPort` over `StoragePort`); tested but not composed into `ck-bin`.
- ck-search-fts: [./ck-search-fts/README.md](./ck-search-fts/README.md) - FTS5 lexical search adapter (`LexicalSearchPort`) with bm25 ranking; owns no schema.
- ck-storage-sqlite: [./ck-storage-sqlite/README.md](./ck-storage-sqlite/README.md) - SQLite persistence adapter (`StoragePort` + `SessionMemoryPort`); owns the idempotent `SCHEMA_V1` DDL and FTS5 tables.
- ck-transport-sse: [./ck-transport-sse/README.md](./ck-transport-sse/README.md) - axum HTTP MCP endpoint with reloadable service/auth state, live `/readyz` probe, and bearer-token gate.
- ck-transport-stdio: [./ck-transport-stdio/README.md](./ck-transport-stdio/README.md) - JSON-RPC line handler mapping one request line to one response line onto `McpService`.
- ck-vector: [./ck-vector/README.md](./ck-vector/README.md) - SQLite brute-force cosine vector search (`VectorSearchPort`), scoped by embedding `model_id`.

## Verification
- Run: `cargo test --workspace`
