# Project State

## Devin onboarding (2026-10-13)
- Repository configuration: low-effort DeepWiki plan and pinned Rust environment blueprint.
- Existing code and behavior are unchanged; no cargo test execution is claimed here.
- Corrected instructions to existing .agents/ memory directory; external Devin generation and snapshot remain unverified.


## Last Updated
- Date: 2026-09-02
- By: Antigravity (Repository Documentation Architecture Knowledge System)

## Current Repository Status

### Crates
- `ck-core`: implemented domain entities, enums, and ports; no SQL/HTTP/transport dependencies observed.
- `ck-config`: implemented TOML/env expansion and configuration types; observability/service.name now wired; `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, `auth.trust_mode` remain parsed-but-ignored (debt).
- `ck-app`: implemented context storage/recall, ingestion/refresh, retrieval, status, retention. Refresh now replaces a source's prior documents (no accumulation); artifact payload compensated on metadata-write failure; freshness score now age-vs-TTL. FTS↔DB write still non-transactional (debt).
- `ck-storage-sqlite`: implemented schema and port adapters; **all enum codecs now round-trip** (sse, pinned, compacted, dependency_change, cancelled decoders added).
- `ck-search-fts`: implemented SQLite FTS indexing/search; indexing still coordinated non-atomically with canonical storage (debt).
- `ck-ranker`: implemented lexical/semantic ranking and result de-duplication.
- `ck-memory`: implemented event retention classification; runtime storage uses `SessionMemoryPort` directly.
- `ck-artifacts`: implemented filesystem artifact storage.
- `ck-mcp`: implemented canonical tools, compatibility aliases, auth delegation, ACL context tool; `context_required` corpus allowlist arm and filesystem path containment now enforced.
- `ck-transport-stdio`: implemented JSON-RPC line handling in local trust mode.
- `ck-transport-sse`: implemented HTTP MCP endpoint and reloadable state; `/readyz` now probes `get_status`; accepts a pre-bound listener; graceful shutdown not implemented (debt).
- `ck-auth`: implemented bearer-token and allowlist policy with constant-time token comparison.
- `ck-observability`: tracing subscriber installed process-wide via `set_global_default`; config-driven via `init_logging`; OTEL exporter still not implemented.
- `ck-scheduler`: implemented and unit-tested refresh queue orchestration; still not composed into the binary (debt).
- `ck-bin`: CLI, composition, stdio/SSE startup. SSE listener bound in main thread (bind failure fails startup); reload preserves auth policy + ACL context agent; observability wired from config.
- `ck-embeddings`: implemented no-op and OpenAI-compatible providers.
- `ck-vector`: implemented SQLite brute-force vector search.
- `ck-adapter-filesystem`: implemented filesystem and learning-proposal adapters.
- `ck-adapter-github`: implemented and tested but not composed into the binary (debt).
- `ck-adapter-web`: implemented and tested but not composed into the binary; SSRF now fails closed on DNS errors/empty resolution (DNS rebinding pinning still residual).

### Implemented Features
- Single Rust CLI binary with migrate, doctor, schema export, and serve commands.
- SQLite schema, FTS5 retrieval, filesystem artifacts, session memory, retention, ranking, embeddings/vector search, stdio MCP, and SSE MCP.
- Bearer-token validation and tool/corpus allowlist authorization (preserved across config reload).
- Filesystem source ingestion in the shipped runtime; refresh replaces rather than accumulates.
- Complete 9-layer technical knowledge system in `docs/` combining Diátaxis, DeepWiki architecture guides, and Google Code Wiki source traceability.

### Partially Implemented Features
- Ports-and-adapters architecture: primary path conforms; ACL context retrieval bypasses application ports (debt).
- GitHub/web ingestion: crates exist and pass tests but are not runtime-wired (debt).
- Scheduler: crate exists and passes tests but runtime uses a parallel loop (debt).
- SSE lifecycle: bind-fails-startup + readiness probe done; graceful worker shutdown/join not done (debt).
- FTS↔DB write atomicity: document accumulation + payload compensation done; single-transaction coverage not done (debt).

### Not Yet Implemented
- OpenTelemetry exporter and metrics backend.
- PDF and bundled external dataset adapters.
- `context_required` server-side `is_private`/`scope_claim` validation (currently mitigated by corpus allowlist + path containment).

### Database / Migrations
- One embedded idempotent `SCHEMA_V1` creates all tables and records schema version 1.
- No ordered migration-file mechanism observed.
- Enum codecs now round-trip all domain variants.

### MCP Tools
- `get_status`, `store_context`, `recall_context`, `ingest_source`, `refresh_source`, `query_reference`, `resolve_reference`, and `context_required` are dispatched.
- Aliases: `resolve-library-id`, `get-library-docs`, and `query-docs`.

### Source Adapters
- Runtime-wired: filesystem only.
- Present but unwired: GitHub and web (debt).
- ACL context path directly constructs filesystem/learning-proposal readers inside `ck-mcp` (debt).

### Verification Summary
- `cargo test --workspace`: **215 passed, 0 failed** (was 206 baseline; +7 Task 0, +3 Task 4, -1 net from Task 1 contract changes = 215).
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- SWE_SEED compiles clean with updated context client API.
- Task 0 (baseline evidence) COMPLETE: all 12 gate exact-tests pass.
- Task 1 (SEA/SWE context authority contract) COMPLETE: `ContextAuthorized` removed from all generated contracts; CK no longer creates authority decisions; SWE_SEED submits authority references.
- Task 4 (data integrity) PARTIAL: fetch-before-delete + stable IDs + FTS rebuild done; full transactional atomicity (single-connection canonical+FTS) NOT done.
- Tasks 2, 3, 5-11 remain not started.
