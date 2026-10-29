# Project State

## Devin onboarding (2026-10-13)
- Repository configuration: low-effort DeepWiki plan and pinned Rust environment blueprint.
- Existing code and behavior are unchanged; no cargo test execution is claimed here.
- Corrected instructions to existing .agents/ memory directory; external Devin generation and snapshot remain unverified.


## Last Updated
- Date: 2026-10-24
- By: Documentation reconciliation pass (context pipeline, 2026-10)

## Current Repository Status

### Crates
- `ck-core`: implemented domain entities, enums, and ports; no SQL/HTTP/transport dependencies observed.
- `ck-config`: implemented TOML/env expansion and configuration types; observability/service.name now wired; `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, `auth.trust_mode` remain parsed-but-ignored (debt).
- `ck-app`: implemented context storage/recall, ingestion/refresh, retrieval, status, retention, plus the context pipeline: `context.rs` (`ContextRequirement`, `ContextAccess`, `resolve_context` 8 stages) and `context_effect.rs` (settlement-backed `ContextEffectStore`). Kind-based adapter routing via `kind_adapters`/`adapter_for_kind`. Refresh replaces a source's prior documents; artifact payload compensated on metadata-write failure; FTS↔DB write still non-transactional (debt). The JSONL effect store is implemented in `ck-app` rather than an adapter crate (layering debt).
- `ck-storage-sqlite`: implemented schema and port adapters; **all enum codecs now round-trip** (sse, pinned, compacted, dependency_change, cancelled decoders added).
- `ck-search-fts`: implemented SQLite FTS indexing/search; indexing still coordinated non-atomically with canonical storage (debt).
- `ck-ranker`: implemented lexical/semantic ranking and result de-duplication.
- `ck-memory`: implemented event retention classification; runtime storage uses `SessionMemoryPort` directly.
- `ck-artifacts`: implemented filesystem artifact storage.
- `ck-mcp`: implemented canonical tools, compatibility aliases, auth delegation, ACL context agent; `context_required` corpus allowlist arm and filesystem path containment enforced. Egress is now bundle-first: `AclContextAgent` delegates to `AppServices::resolve_context` and `cep_bundle::project_context_bundle` emits the sealed `godspeed.context_bundle`, with the legacy `ContextPacketCreated` envelope kept deprecated for dual-read. `ingest_context_effects` is dispatched.
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
- Bounded context pipeline: typed requirement -> POL-ACL-002 data-access gate -> retrieval/freshness/budget/concept stages -> settlement calibration -> sealed `godspeed.context_bundle` with verifiable integrity hash and explicit governed-absence reasons.
- Settlement-backed effect ingestion (`ingest_context_effects`) with per-world, append-only JSONL calibration capped at 0.50 per source.
- SQLite schema, FTS5 retrieval, filesystem artifacts, session memory, retention, ranking, embeddings/vector search, stdio MCP, and SSE MCP.
- Bearer-token validation and tool/corpus allowlist authorization (preserved across config reload).
- Filesystem source ingestion in the shipped runtime; refresh replaces rather than accumulates.
- Complete 9-layer technical knowledge system in `docs/` combining Diátaxis, DeepWiki architecture guides, and Google Code Wiki source traceability.

### Partially Implemented Features
- Ports-and-adapters architecture: primary path conforms. ACL context retrieval now routes through `AppServices::resolve_context` and `ingest_source`, so `ck-mcp` no longer constructs storage adapters in production code. Residual: the JSONL effect store is concrete infrastructure living in `ck-app`.
- GitHub/web ingestion: multi-kind routing exists (`kind_adapters`), but only `filesystem` (default) and `learning_proposals_jsonl` are registered in `ck-bin`; the GitHub and Web crates remain unwired (debt).
- Scheduler: crate exists and passes tests but runtime uses a parallel loop (debt).
- SSE lifecycle: bind-fails-startup + readiness probe done; graceful worker shutdown/join not done (debt).
- FTS↔DB write atomicity: document accumulation + payload compensation done; single-transaction coverage not done (debt).

### Not Yet Implemented
- OpenTelemetry exporter and metrics backend.
- PDF and bundled external dataset adapters.
- Effect-ledger repair/rotation tooling (unreadable ledgers currently fail ingest closed and degrade retrieval to uncalibrated).

### Database / Migrations
- One embedded idempotent `SCHEMA_V1` creates all tables and records schema version 1.
- No ordered migration-file mechanism observed.
- Enum codecs now round-trip all domain variants.

### MCP Tools
- `get_status`, `store_context`, `recall_context`, `ingest_source`, `refresh_source`, `query_reference`, `resolve_reference`, `context_required`, and `ingest_context_effects` are dispatched.
- Aliases: `resolve-library-id`, `get-library-docs`, and `query-docs`.

### Source Adapters
- Runtime-wired: `filesystem` (default) and `learning_proposals_jsonl` (kind-registered).
- Present but unwired: GitHub and web (registration missing, routing present; debt).
- The context agent's corpus bootstrap goes through `AppServices::ingest_source` with explicit `source_kind` values, not ad hoc readers.

### Verification Summary
- `cargo test --workspace`: **265 passed, 0 failed** (recorded 2026-10-07 in `.agents/current_status.yml`; not re-run during the 2026-10-24 documentation pass).
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo fmt --check`: passed.
- SWE_SEED compiles clean with updated context client API.
- Task 0 (baseline evidence) COMPLETE: all 12 gate exact-tests pass.
- Task 1 (SEA/SWE context authority contract) COMPLETE: `ContextAuthorized` removed from all generated contracts; CK no longer creates authority decisions; SWE_SEED submits authority references.
- Task 4 (data integrity) PARTIAL: fetch-before-delete + stable IDs + FTS rebuild done; full transactional atomicity (single-connection canonical+FTS) NOT done.
- Tasks 2, 3, 5-11 remain not started.
### Documentation System (2026-10-24)
- Reconciled the 9-layer docs with the context pipeline: canonical bundle-first wire shape, eight no-context reasons, four omission codes, three completeness statuses, POL-ACL-002 access semantics, and settlement calibration.
- Added `docs/workflows/context-requirement-resolution.md` (execution trace) and `docs/explainers/context-requirement-explorer.html` (offline single-file interactive model with in-page SHA-256 verification).
- Replaced broken `file:///crates/` links with correct repo-relative paths across `docs/`; all relative doc links verified to resolve.
