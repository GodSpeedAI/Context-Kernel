# Decisions

## Decision: Technical Knowledge System Information Architecture
- Date: 2026-09-02
- Context: The repository required comprehensive documentation serving both newcomers (high-level mental models) and experienced engineers (subsystem depth, invariants, code traceability) without compromising either audience.
- Decision: Adopt a unified information architecture combining Diátaxis (Tutorials, How-To, Reference, Explanation), DeepWiki (architecture-first, subsystem guides, execution traces, Mermaid diagrams), and Google Code Wiki (symbol-level source traceability in `docs/source-map.md`). Structure documentation across 9 progressive disclosure layers (Layers 0 to 8).
- Alternatives considered:
  - Single monolithic README or wiki page (rejected: overwhelms newcomers and lacks searchable precision for experts).
  - Pure Diátaxis 4-folder dump (rejected: fails to capture architectural decomposition, subsystem boundaries, and execution traces).
- Evidence in codebase: `docs/README.md`, `docs/documentation-map.md`, `docs/architecture.md`, `docs/source-map.md`, `docs/subsystems/*`, `docs/workflows/*`, `docs/explanation/*`, `docs/tutorials/*`, `docs/how-to/*`, `docs/reference/*`.
- Consequences: All architectural claims are grounded in repository code; verified behavior is explicitly separated from unwired library crates (GitHub/Web adapters) and tracked debt; documentation is navigable by intent.

## Decision: Adopt Workflow-and-Permissions governance in AGENTS.md
- Date: 2026-08-24
- Context: The prior six-step Agent Workflow prescribed generic inspect-plan-implement-verify-update steps without permission gates, handoff discipline, or build-resource rules.
- Decision: Replace Agent Workflow with an explicit Workflow and Permissions section plus a Handoff section including build-lock and artifact discipline. Keep evidence-priority ordering under Implementation Verification Rules. Extend required memory files with `.agents/LESSONS.md` and `.agents/current_status.yml`; retire the empty CURRENT_STATUS.md template.
- Alternatives considered: Appending the new sections alongside the old workflow (rejected: leaves two competing workflows); verbatim adoption of OPEN_QUESTIONS.md naming from the source approach (rejected: diverges from the established lowercase `open_questions.md` used throughout this repository).
- Evidence in codebase: AGENTS.md sections "Workflow and Permissions" and "Handoff", required-files list, templates; `.agents/current_status.yml`; `.agents/LESSONS.md`.
- Consequences: Agents must gate risky changes behind user approval, maintain resumable handoff state in current_status.yml, respect the single-writer Cargo build lock; `just context-check` is mandated before a justfile recipe for it exists.

## Decision: Audit depth and focus
- Date: 2026-07-29
- Context: Repository-wide audit covers 19 crates, remote auth, persistent state, concurrency, and architectural claims.
- Decision: Use Deep depth; trace SSE reload authorization, ACL corpus retrieval, and storage/FTS writes in detail.
- Alternatives considered: shallow line-by-line scan; rejected (lower semantic tracing quality).
- Evidence: `Cargo.toml:1-23`; depth triggers in `.agents/skills/neatcode/SKILL.md`.
- Consequences: Report is explicit about examined and unexamined runtime properties.

## Decision: Subagent-driven implementation on disjoint crates
- Date: 2026-07-29
- Context: Audit corrections span isolated crates plus interdependent ck-bin/ck-transport-sse lifecycle code.
- Decision: Dispatch 5 implementer subagents in parallel ONLY on disjoint files (no shared-file conflicts); do the auth-reload + observability + SSE-bind work directly where full context and lowest risk are required. Skip per-task reviewer dispatch as disproportionate to surgical, fully-specified fixes; consolidate verification centrally.
- Alternatives considered: full SDD ceremony (task-brief scripts, per-task review packages) — excessive overhead for surgical fixes; sequential manual edits — slower, more controller-context.
- Evidence: git diff shows 9 disjoint files; each subagent ran its crate's tests green.
- Consequences: faster, lower controller-context use; correctness hinges on consolidated verification (done: 206 tests, clippy, fmt green).

## Decision: Defer lower-consequence / higher-risk findings
- Date: 2026-07-29
- Context: Several audit findings are feature-wiring or refactor work (GitHub/web adapter wiring, scheduler ownership, broad StoragePort split, full FTS transactional atomicity, graceful shutdown) with integration risk disproportionate to their severity.
- Decision: Implement all S1 security/data-integrity fixes and the operational S2 fixes; DEFER the above as documented debt (named in OBSERVED_DEBT + harden report) rather than destabilize the working ingestion/runtime paths.
- Alternatives considered: wire GitHub/web via kind-based adapter selection — requires `AppServices` to take multiple adapters (design change); rejected this pass.
- Evidence: harden report "Open / deferred" section; `AppServices::new` signature takes a single `source_adapter`.
- Consequences: README still overstates runtime adapter support; FTS can be partially inconsistent after a crash; these are tracked, not silent.

## Decision: SSE bind in main thread; non-blocking + from_std in worker
- Date: 2026-07-29
- Context: Making SSE bind failure fail startup requires binding in the main thread, but `tokio::net::TcpListener::from_std` needs (a) a non-blocking socket and (b) a live tokio reactor.
- Decision: bind `std::net::TcpListener` + `set_nonblocking(true)` in main; move it into the spawned runtime thread and call `from_std` there (reactor present). Convert `serve_sse` to accept a pre-bound listener.
- Alternatives considered: bind inside `serve_sse` async (original — swallowable failure); bind+from_std both in main (panics: no reactor).
- Evidence: repro showed `from_std` panics on blocking socket and on missing reactor; both resolved by split.
- Consequences: bind collision now fails the process (non-zero); tested by `sse_bind_failure_fails_startup`.

## Decision: Dependency order for deferred harden debt
- Date: 2026-07-29
- Context: The report listed eight items by risk, but current code shows shared trait, transaction, and composition boundaries.
- Decision: Extract the ACL corpus port before server policy; define atomic index writes before splitting `StoragePort`; wire source pipelines before scheduler ownership; establish scheduler ownership before worker shutdown; honor/remove transport config only after shutdown/join exists.
- Alternatives considered: preserve report order; rejected because it would implement validation on the adapter bypass, rewrite storage traits twice, and join a refresh loop that scheduler composition later replaces.
- Evidence in codebase: `crates/ck-mcp/src/agentic_capability_loop.rs:191-380`, `crates/ck-app/src/lib.rs:338-434`, `crates/ck-core/src/ports/storage.rs:6-88`, `crates/ck-bin/src/main.rs:135-305`.
- Consequences: Tasks 1 and 3 can begin independently, but core exports and binary composition are serialized; no deferred item is considered resolved until its failing-first gate passes.

## Decision: Context Kernel enforces but does not create action authority
- Date: 2026-07-29
- Context: The audit proposed validating caller scope through `AuthPort`, while the controlling relationship documents assign protected-action authority exclusively to SEA-Forge.
- Decision: Context Kernel keeps bearer/tool/corpus admission as defense in depth, derives corpus protection from canonical metadata, and permits protected reads only after cryptographically verifying a signed SEA `AuthorityChecked` envelope. It emits retrieval evidence, not an independent authority decision.
- Alternatives considered: local scope-token policy or caller claims; rejected because both make Context Kernel a competing authority owner.
- Evidence in codebase: current violation at `crates/ck-mcp/src/agentic_capability_loop.rs:191-263`; controlling boundary at `/home/sprime01/projects/godspeed_agent/docs/explanations/SEA_Forge_to_godspeed_agent.md:25-55`.
- Consequences: SEA/SWE generated contracts must change before CK implementation; protected unscoped search excludes protected/unknown corpora before search.

## Decision: Implement Milestone 10 as hardening primitives without schema version bump
- Date: 2026-06-02 10:05 EDT
- Context: Milestone 10 requires config reload, retention GC, authz by corpus/tool, telemetry, release docs, install docs, and CI examples. Existing schema already has enough timestamp/status columns for retention deletion.
- Decision: Add typed config, storage retention methods, app GC orchestration, MCP/SSE authorization checks, and CLI schema export without adding a new SQLite migration.
- Alternatives considered: Add a retention_jobs table or policy tables; rejected because configured retention can operate over existing tables. Add full OTEL exporter crates; deferred because current workspace uses tracing only and M10 can expose structured tracing without changing runtime dependency shape.
- Evidence in codebase: `crates/ck-config/src/lib.rs`, `crates/ck-core/src/ports/storage.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-mcp/src/lib.rs`, `crates/ck-transport-sse/src/lib.rs`, `crates/ck-bin/src/main.rs`.
- Consequences: Retention is explicit and test-covered; artifact files are not deleted by GC yet. OTEL endpoint config exists but exporter wiring remains a follow-up.

## Decision: Use allowlist authz policy on BearerAuth
- Date: 2026-06-02 10:05 EDT
- Context: Milestone 10 requires authz by corpus/tool and existing AuthPort only supported token validation.
- Decision: Extend AuthPort with `authorize_tool` and `authorize_corpus`; implement allowlists in BearerAuth, defaulting to `*`.
- Alternatives considered: New separate AuthorizationPort; rejected for now because token validation and policy are configured together in ck-bin. Per-token policy maps; deferred until multi-tenant auth is needed.
- Evidence in codebase: `crates/ck-core/src/ports/auth.rs`, `crates/ck-auth/src/lib.rs`, `crates/ck-mcp/src/lib.rs`, `crates/ck-transport-sse/src/lib.rs`.
- Consequences: SSE requests enforce token validation and tool/corpus allowlists before tool dispatch. Plain stdio remains a local/internal path.

## Decision: Use Milestone 0 as Initial Implementation Slice
- Date: 2026-06-01
- Context: User asked to follow the plan and implement the spec. The plan orders work by milestones and starts with a compilable skeleton.
- Decision: Implement Milestone 0 first.
- Alternatives considered: Attempting multiple milestones at once; rejected because the plan emphasizes small, verifiable phases.
- Evidence in codebase: `Context_Kernel_Service_Plan.md` section 21 defines Milestone 0 deliverables and acceptance.
- Consequences: Initial work focuses on workspace, config, migrations, stdio MCP, and `get_status`; retrieval and memory tools remain not implemented.

## Decision: Use `rusqlite` for Initial SQLite Adapter
- Date: 2026-06-01
- Context: Milestone 0 needs local SQLite migration bootstrap and FTS5 table creation with minimal async/runtime overhead.
- Decision: Use `rusqlite` with bundled SQLite for `ck-storage-sqlite`.
- Alternatives considered: `sqlx`; deferred because Milestone 0 does not need async DB access or query macros.
- Evidence in codebase: `Cargo.toml` workspace dependency `rusqlite`; `crates/ck-storage-sqlite/src/lib.rs` uses `rusqlite::Connection`.
- Consequences: SQLite access is synchronous for now; async orchestration can wrap or replace the port in later milestones if needed.

## Decision: Implement Milestones 1-3 Together
- Date: 2026-06-01
- Context: User requested implementation of milestones 1 through 3. They share significant infrastructure (domain types, storage, search).
- Decision: Implement all three milestones in a single session, building up from the foundation.
- Alternatives considered: Implementing one milestone at a time with separate verification; rejected because the shared infrastructure makes it more efficient to build together.
- Evidence in codebase: All three milestones' acceptance criteria are met with 37 passing tests.
- Consequences: Larger change set but more efficient; all three milestones verified together.

## Decision: Use Parallel Subagent Delegation for Independent Crates
- Date: 2026-06-01
- Context: Five new crates (ck-storage-sqlite expansion, ck-search-fts, ck-artifacts, ck-ranker, ck-adapter-filesystem) all depend only on ck-core and are independent of each other.
- Decision: Dispatch 5 parallel subagents to implement these crates simultaneously.
- Alternatives considered: Sequential implementation; rejected for efficiency.
- Evidence in codebase: All 5 crates compile and pass their tests.
- Consequences: Faster implementation; required careful integration afterward to fix cross-crate issues (e.g., FTS query joining through documents table for library_id).

## Decision: Store library_id on documents, not chunks
- Date: 2026-06-01
- Context: The schema has library_id on the documents table, not on chunks. FTS search needs to filter by library_id.
- Decision: FTS search for chunks with library_id filter joins through the documents table.
- Alternatives considered: Adding library_id to chunks table; rejected to match the plan's schema.
- Evidence in codebase: `crates/ck-search-fts/src/lib.rs` search_chunks method joins chunks→documents for library_id filtering.
- Consequences: Slightly more complex queries but matches the normalized schema design.

## Decision: Implement Milestones 4-6 Together
- Date: 2026-06-01
- Context: User requested implementation of milestones 4 through 6. They share infrastructure (enhanced storage, scheduler, auth) and build on each other.
- Decision: Implement all three milestones in a single session.
- Alternatives considered: One milestone at a time; rejected because M5 (scheduler) depends on M4 (source health/stale detection) and M6 (SSE) depends on M5 (auth).
- Evidence in codebase: All three milestones' acceptance criteria are met with 57 passing tests.
- Consequences: Larger change set but efficient; all three milestones verified together.

## Decision: Synchronous core with tokio only in transport layer
- Date: 2026-06-01
- Context: M6 adds SSE transport requiring async HTTP (axum + tokio), but the rest of the codebase is synchronous (rusqlite).
- Decision: Keep tokio confined to ck-transport-sse and ck-bin's SSE thread. SSE server runs in a background std::thread with its own tokio runtime.
- Alternatives considered: Making the entire codebase async; rejected because it would require rewriting all storage/search code and adding tokio to ck-core.
- Evidence in codebase: `ck-transport-sse/Cargo.toml` depends on tokio/axum; `ck-bin/src/main.rs` creates a tokio runtime inside a std::thread for SSE.
- Consequences: Clean separation between sync domain logic and async transport; no tokio dependency in ck-core.

## Decision: Separate AppServices instance for periodic refresh thread
- Date: 2026-06-01
- Context: Periodic refresh thread needs its own AppServices because AppServices is not Clone.
- Decision: Create a second AppServices via wire_app() for the refresh thread.
- Alternatives considered: Wrapping AppServices in Arc; rejected because AppServices owns Box<dyn trait> ports that are not thread-safe.
- Evidence in codebase: `crates/ck-bin/src/main.rs` lines 129-130 create `refresh_store` and `refresh_app`.
- Consequences: Two SQLite connections for refresh thread; acceptable for a local service.

## Decision: Hybrid scoring weights (FTS 0.7, trust 0.2, freshness 0.1)
- Date: 2026-06-01
- Context: M4 requires enhanced ranking with trust and freshness signals.
- Decision: Use fixed weights: FTS 0.7, trust 0.2, freshness 0.1. Trust is normalized to [0,1] by dividing by 100. Freshness is computed from last_refresh_success_at and freshness_ttl_secs.
- Alternatives considered: Configurable weights; deferred as a future enhancement.
- Evidence in codebase: `crates/ck-ranker/src/lib.rs` combine_scores method.
- Consequences: Simple but effective ranking; can be made configurable later.

## Decision: Add StoragePort primitives for refresh job atomicity
- Date: 2026-06-02 08:06 EDT
- Context: Review found scheduler enqueue and claim used read-modify-write sequences that could race and scanned job histories.
- Decision: Add `insert_active_refresh_job` and `claim_next_refresh_job_for_source` to `StoragePort`; SQLite uses a partial unique index for active jobs and `UPDATE ... RETURNING` for claim.
- Alternatives considered: Relax scheduler documentation to best-effort; rejected because SQLite can enforce the needed atomic primitives. Keeping direct scheduler read/modify/write; rejected because it preserves the race.
- Evidence in codebase: `crates/ck-core/src/ports/storage.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-scheduler/src/lib.rs`; tests `insert_active_refresh_job_returns_existing_active_job` and `claim_next_refresh_job_for_source_is_single_use`.
- Consequences: StoragePort implementors must provide refresh-job atomic primitives; current repository implementations were updated.

## Decision: Persist refresh corpus identity through corpus_sources
- Date: 2026-06-02 08:06 EDT
- Context: Review asked refresh to use `source.corpus_id`, but `Source` has no `corpus_id` field. Schema already has `corpus_sources`.
- Decision: Add `link_source_to_corpus` and `find_corpus_id_for_source` to `StoragePort`; ingest records the link and refresh reads it.
- Alternatives considered: Add `corpus_id` to `Source`; rejected because it would duplicate existing schema intent and require broader entity/schema changes. Parse corpus from adapter config; rejected because config is not a stable storage relation.
- Evidence in codebase: `crates/ck-core/src/ports/storage.rs`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-app/src/lib.rs`; test `refresh_source_reuses_persisted_corpus_id`.
- Consequences: Refresh is idempotent against the persisted corpus relation; StoragePort interface expanded.

## Decision: Use SourceHealth::Degraded for ingest failures
- Date: 2026-06-02 08:06 EDT
- Context: Review asked for `SourceHealth::Failing`, but current enum variants are `Unknown`, `Healthy`, and `Degraded`.
- Decision: On ingest failure after source creation, call `update_source_health(..., SourceHealth::Degraded, Some(error))` and return the original error.
- Alternatives considered: Add a new `Failing` enum variant; rejected as unnecessary schema/API expansion for a narrow bug fix and inconsistent with scheduler failure handling.
- Evidence in codebase: `crates/ck-core/src/types/mod.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-scheduler/src/lib.rs`; test `ingest_source_marks_source_degraded_on_failure`.
- Consequences: Failed ingest now updates health consistently with existing failure semantics.

## Decision: Fail fast on SSE auth misconfiguration
- Date: 2026-06-02 08:06 EDT
- Context: Review found `require_auth = true` with empty tokens silently created no-auth mode.
- Decision: `ck-bin` returns an error before starting SSE when auth is required and token list is empty.
- Alternatives considered: Warn and continue with SSE disabled; rejected because failing fast is clearer and avoids running a surprising partial service.
- Evidence in codebase: `crates/ck-bin/src/main.rs`; workspace integration tests still pass.
- Consequences: Misconfigured deployments receive an immediate startup error instead of an unauthenticated SSE server.

## Decision: Implement Milestones 7-9 Together
- Date: 2026-06-02 10:30 EDT
- Context: User requested implementation of milestones 7 through 9. They share infrastructure (enhanced ranking, embedding ports, source adapters) and build on each other.
- Decision: Implement all three milestones in a single session using parallel subagent delegation for independent crates.
- Alternatives considered: One milestone at a time; rejected because the independent crates (ck-ranker, ck-embeddings, ck-vector, ck-adapter-github, ck-adapter-web) can be built in parallel.
- Evidence in codebase: All three milestones' acceptance criteria are met with 153 passing tests.
- Consequences: Larger change set but efficient; all three milestones verified together.

## Decision: Use ureq for synchronous HTTP in adapters and embeddings
- Date: 2026-06-02 10:30 EDT
- Context: M8 (embeddings) and M9 (GitHub/web adapters) need HTTP client capabilities. The codebase is synchronous (rusqlite) with async only in transport layer.
- Decision: Use ureq (synchronous HTTP client) for all HTTP needs in adapters and embeddings.
- Alternatives considered: reqwest (async, would require tokio in domain crates); rejected to preserve sync architecture. std::process::Command + curl; rejected as fragile.
- Evidence in codebase: `crates/ck-embeddings/Cargo.toml`, `crates/ck-adapter-github/Cargo.toml`, `crates/ck-adapter-web/Cargo.toml` all depend on ureq.
- Consequences: Consistent sync HTTP across all adapters; no tokio dependency in domain/adapter crates.

## Decision: Brute-force cosine similarity for vector search v1
- Date: 2026-06-02 10:30 EDT
- Context: M8 requires a vector search adapter. The spec says brute-force cosine over small corpora is acceptable for v1.
- Decision: SqliteVectorStore loads all vectors for a model and computes cosine similarity in-memory. No corpus/library/session filtering at the vector layer for v1.
- Alternatives considered: sqlite-vec; deferred as an optimization. In-memory HNSW; rejected as over-engineering for v1.
- Evidence in codebase: `crates/ck-vector/src/lib.rs` search_chunks loads all vectors for model_id and computes similarity.
- Consequences: Simple and correct; performance acceptable for small corpora; can be swapped for sqlite-vec or external vector DB later without domain changes.

## Decision: SSRF protection via IP parsing without DNS resolution
- Date: 2026-06-02 10:30 EDT
- Context: M9 web adapter needs SSRF protections. DNS rebinding attacks are a known SSRF vector.
- Decision: Block private IP ranges by parsing IP literals in URLs. Do not perform DNS resolution for v1 (accepts the DNS rebinding risk for now).
- Alternatives considered: Full DNS resolution with IP checking; deferred as it adds complexity and may break legitimate CDN-hosted docs.
- Evidence in codebase: `crates/ck-adapter-web/src/lib.rs` validate_url function.
- Consequences: Protects against basic SSRF via IP literals; DNS rebinding mitigation can be added in a future milestone.

## Decision: RankerPort updated to accept RankContext instead of bare parameters
- Date: 2026-06-02 10:30 EDT
- Context: M7 requires query intent, session affinity, and debug mode in ranking. The existing RankerPort only accepted query and items.
- Decision: Added RankContext struct with intent, session_id, and debug fields. Updated all RankerPort methods to accept RankContext.
- Alternatives considered: Adding separate parameters per method; rejected as it would bloat the trait. Using a builder pattern; rejected as over-engineering.
- Evidence in codebase: `crates/ck-core/src/ports/ranker.rs`, `crates/ck-ranker/src/lib.rs`, `crates/ck-app/src/lib.rs`.
- Consequences: All ranker implementations and callers updated; clean extension point for future ranking signals.

## Decision: ck-observability provides text/JSON tracing init only; OTEL exporter deferred
- Date: 2026-06-02
- Context: Deployment readiness task 1 required observability. Full OTEL exporter would add significant dependency weight.
- Decision: ck-observability crate provides ObservabilityRuntimeConfig, init_observability (text or JSON tracing-subscriber), and ObservabilityGuard. OTEL endpoint config exists in ck-config but no exporter crate is wired.
- Alternatives considered: Adding opentelemetry-otlp crate; deferred to avoid dependency complexity for v1. Using tracing-subscriber only without a crate; rejected because ck-observability provides a clean abstraction boundary.
- Evidence in codebase: `crates/ck-observability/src/lib.rs`, `crates/ck-bin/src/main.rs`.
- Consequences: Structured logging works; OTEL metrics/traces can be added later without changing ck-bin wiring.

## Decision: ReloadableSseState uses RwLock<Arc<SseState>> for SSE config reload
- Date: 2026-06-02
- Context: SSE transport needed to support config reload without restart. SseState was previously a plain Arc.
- Decision: Introduced ReloadableSseState wrapping RwLock<Arc<SseState>>. SSE handlers read through the RwLock. ck-bin updates the reloadable state when config changes.
- Alternatives considered: tokio watch channel; rejected because SSE handlers are sync axum handlers. Rebuilding the entire axum server; rejected as too heavyweight.
- Evidence in codebase: `crates/ck-transport-sse/src/lib.rs`, `crates/ck-bin/src/main.rs`.
- Consequences: SSE config reload is lightweight; handlers always see the latest config.

## Decision: Artifact payload GC orchestrated at app layer
- Date: 2026-06-02
- Context: Retention GC previously only deleted SQLite metadata rows, not artifact payload files on disk.
- Decision: Added delete_artifact_payload to ArtifactPort, list_artifacts_older_than to StoragePort. AppServices::run_retention_gc orchestrates both metadata and payload deletion. RetentionGcReport includes artifact_payload_errors count.
- Alternatives considered: Having ck-storage-sqlite directly delete files; rejected because storage layer should not know about filesystem paths. Having ck-artifacts scan for orphaned files; rejected because it would need to know about retention policy.
- Evidence in codebase: `crates/ck-core/src/ports/artifact.rs`, `crates/ck-core/src/ports/storage.rs`, `crates/ck-app/src/lib.rs`.
- Consequences: Clean separation: storage lists old artifacts, artifacts deletes payloads, app orchestrates.

## Decision: SSRF DNS resolution via DnsResolver trait
- Date: 2026-06-02
- Context: Original SSRF protection only blocked IP literals, leaving DNS rebinding as a known gap.
- Decision: Added DnsResolver trait with SystemDnsResolver (production) and StaticResolver (tests). validate_url_with_resolver resolves hostname and checks resulting IPs against private ranges.
- Alternatives considered: Using hickory-resolver; rejected as over-engineering for v1. Keeping IP-literal-only protection; rejected as insufficient for production.
- Evidence in codebase: `crates/ck-adapter-web/src/lib.rs`.
- Consequences: DNS rebinding attacks mitigated; testable with StaticResolver for deterministic assertions.

## Decision: Context7 dataset adapter and PDF adapter removed from v1 scope
- Date: 2026-06-02
- Context: Deployment readiness plan included Context7 dataset adapter (task 12) and PDF adapter (task 13). Both would add significant dependency weight and complexity.
- Decision: Both removed from v1 scope. Context7 compatibility is provided through tool-contract aliases (resolve-library-id, get-library-docs, query-docs). PDF support can be added as a future adapter behind SourceAdapter port.
- Alternatives considered: Implementing minimal versions; rejected as they would be incomplete and untested. Marking as stubbed; rejected as misleading.
- Evidence in codebase: No ck-adapter-context7 or ck-adapter-pdf crates exist. Compatibility aliases in `crates/ck-mcp/src/lib.rs`.
- Consequences: v1 focuses on filesystem, GitHub, and web adapters. Future adapters can be added without architecture changes.

## Decision: ck-memory crate provides retention classification only
- Date: 2026-06-02
- Context: Deployment readiness task 11 required ck-memory crate. Full session memory management is already in ck-app and ck-storage-sqlite.
- Decision: ck-memory provides EventRetentionClass, classify_event_kind, and SessionRetentionPolicy. ck-app re-exports these types. No duplication of existing session memory logic.
- Alternatives considered: Moving all session memory logic to ck-memory; rejected as it would require significant refactoring of ck-app and ck-storage-sqlite. Not creating the crate; rejected as it was a deployment readiness requirement.
- Evidence in codebase: `crates/ck-memory/src/lib.rs`, `crates/ck-app/src/lib.rs`.
- Consequences: Clean classification layer; can be expanded to own more memory logic in future.

## Decision: Vector filtering columns added to chunk_embeddings
- Date: 2026-06-02
- Context: Vector search had no corpus/library/document filtering, returning results across all data.
- Decision: Added corpus_id, library_id, document_id columns to chunk_embeddings table. search_chunks filters by these fields. upsert_chunk_embedding_with_metadata stores filter columns.
- Alternatives considered: Joining through documents table at query time; rejected as slower for vector search. Separate index table; rejected as redundant.
- Evidence in codebase: `crates/ck-vector/src/lib.rs`.
- Consequences: Vector search respects data boundaries; remove_chunk_embeddings_by_document enables clean document deletion.

## Decision: Context packets pin world_ref and state retrieval completeness
- Date: 2026-10-05
- Context: CEP-0008 world_ref migration Stage 9. Packets carried only a bare `domain_model_hash`, and a packet cut off at `max_results` looked identical to a complete one.
- Decision: `ContextRequired` ingress requires a well-formed `world_ref` (syntax only; no digest recomputation). The packet echoes it unchanged and carries `retrieval_completeness` (complete/partial/none) plus machine-readable `omissions`. Hitting the cap with a further match, or truncating citation content, is `partial`.
- Alternatives considered: deriving `world_ref` from `domain_model_hash` (rejected: different identity scheme, DomainForge owns it); making `world_ref` optional (rejected: fail-closed on consequential paths).
- Evidence in codebase: `crates/ck-mcp/src/agentic_capability_loop.rs`, `crates/ck-mcp/tests/acl_context_agent.rs`, governing spec `.agents/specs/cep-world-ref-migration.spec.md` in the sea-rs migration worktree.
- Consequences: SWE_SEED must send `world_ref` (Stage 9, same migration). Legacy requests without it are refused.
