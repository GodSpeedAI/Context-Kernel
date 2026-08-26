# NeatCode Repository Audit

**NeatCode · audit** · repository (50 Rust files, 12,720 lines) · depth: deep · 2026-07-29

**Summary** · The workspace is a real, tested Rust service with mostly sound crate-level dependency direction, but it is not safe to expose remotely with config reload or `context_required` enabled. Two authorization paths can be bypassed, persisted enum values do not round-trip, and multi-store writes can leave durable state inconsistent after ordinary failures. The architecture is **partially conformant**: the primary MCP-to-application-to-port path holds, while the ACL context path bypasses it and several advertised adapters are not in production composition.

## Scope

- Examined all 50 Rust files structurally, workspace manifests, CI, runtime composition, configuration, and MCP contracts.
- Traced three consequential paths in detail: SSE authorization and reload, `context_required` corpus retrieval, and ingestion/storage/FTS writes.
- Read storage enum codecs, scheduler wiring, source-adapter wiring, observability initialization, and representative tests.
- Did not perform load testing, dependency vulnerability scanning, fuzzing, or a live remote penetration test.

## Architecture Conformance

**Claim** · Context Kernel follows ports and adapters; `ck-core` owns entities and port traits, `ck-app` owns orchestration, and SQL/HTTP/transport behavior must stay out of `ck-core` (`README.md:298-313`, `AGENTS.md:119-136`).

**Observed** · `ck-core` depends only on general-purpose data/error crates (`crates/ck-core/Cargo.toml:7-11`); `ck-app` depends inward on `ck-core` and `ck-memory` (`crates/ck-app/Cargo.toml:7-13`); SQLite implements core ports (`crates/ck-storage-sqlite/src/lib.rs:346`, `crates/ck-storage-sqlite/src/lib.rs:933`); and `ck-bin` performs primary composition (`crates/ck-bin/src/main.rs:317-364`). However, `ck-mcp` depends directly on `ck-adapter-filesystem` (`crates/ck-mcp/Cargo.toml:11-14`) and constructs it inside protocol handling (`crates/ck-mcp/src/agentic_capability_loop.rs:266-305`). The binary also composes only `FilesystemSourceAdapter` (`crates/ck-bin/src/main.rs:324-333`) while GitHub and web adapters exist only in their own crates and tests.

**Verdict** · **Partially conformant.** The main service path expresses the claimed dependency rule, but the ACL context feature puts filesystem retrieval and authorization decisions in the MCP layer, and runtime composition does not support the full advertised adapter set.

**Consequence** · Security and retrieval policy now have a parallel owner outside `ck-app`; runtime behavior differs from the documented source-adapter capability set.

**Correction** · Move ACL corpus lookup behind an application-owned port and compose its adapter in `ck-bin`. Select source adapters from `source.kind` in the composition/application path, or stop advertising unsupported adapters until they are reachable.

**Confidence** · confirmed by manifest, imports, call-path, and symbol-use inspection.

## Findings

### S1: `context_required` permits corpus authorization bypass and filesystem escape

**Confidence** · confirmed

**Location** · `crates/ck-mcp/src/lib.rs:150-174`, `crates/ck-mcp/src/lib.rs:435-477`, `crates/ck-mcp/src/lib.rs:506-532`, `crates/ck-mcp/src/agentic_capability_loop.rs:194-218`, `crates/ck-mcp/src/agentic_capability_loop.rs:266-305`

**Evidence** · SSE authorization asks `corpus_ids_from_arguments` for corpus IDs, but that function has no `context_required` arm. The handler accepts `is_private` and `scope_claim` from the requester. The agent treats any non-empty claim as authorization and treats `is_private: false` as public. It then joins unvalidated `corpus_id` onto `corpus_root`; absolute paths or `..` segments can select directories outside the configured root before recursive filesystem discovery reads supported files.

**Consequence** · A bearer token allowed to call `context_required` can bypass the configured corpus allowlist, label a private corpus public, fabricate a scope claim, or read matching `.md`, `.txt`, `.toml`, `.rs`, and `.json` files outside the corpus root.

**Correction** · Derive corpus privacy server-side, include `context_required` in corpus authorization, validate claims through `AuthPort`, canonicalize the selected corpus directory, and reject it unless it remains beneath the canonical corpus root. Do not accept authorization facts from MCP arguments.

**Verification** · Add SSE-level tests using an allowed tool but denied corpus, `is_private: false` against a private corpus, a fake non-empty claim, `../` traversal, and an absolute path; assert zero file reads and an authorization error.

### S1: Config reload drops tool and corpus authorization policy

**Confidence** · confirmed

**Location** · `crates/ck-bin/src/main.rs:235-271`, `crates/ck-mcp/src/lib.rs:150-174`, `crates/ck-transport-sse/src/lib.rs:75-149`

**Evidence** · Initial startup uses `McpService::with_app_and_auth` (`crates/ck-bin/src/main.rs:134-138`). Reload reconstructs the service with `McpService::with_app`, which sets its internal auth field to `None` (`crates/ck-bin/src/main.rs:255`, `crates/ck-mcp/src/lib.rs:76-82`). The transport still validates that a token is valid, but `call_tool_authorized` reaches `authorize`, sees no policy object, and returns success before checking tool or corpus allowlists.

**Consequence** · After the first effective config reload, any valid SSE token can invoke tools and access corpora excluded by the configured allowlists.

**Correction** · Rebuild the reloaded `McpService` with the same newly constructed auth policy and preserve the context agent. Prefer one constructor that requires auth for remotely served services so a reload cannot omit it.

**Verification** · Start SSE with reload enabled, a token, one allowed tool, and one allowed corpus; mutate the config, trigger reload, then assert a disallowed tool and corpus remain denied.

### S1: SQLite enum codecs do not round-trip reachable persisted values

**Confidence** · confirmed

**Location** · `crates/ck-storage-sqlite/src/lib.rs:73-157`, `crates/ck-storage-sqlite/src/lib.rs:288-330`, `crates/ck-core/src/types/mod.rs:20-67`

**Evidence** · Writers serialize `SessionTransport::Sse`, `ContinuationMode::Pinned`, `JobTrigger::DependencyChange`, and `JobStatus::Cancelled`, while corresponding readers omit those strings. `SessionStatus::Compacted` is also omitted by the reader, although current `SessionMemoryPort` methods do not write it. SSE/pinned values are reachable through the storage API; dependency-change currently comes from the unwired scheduler, and no shipped cancellation transition was found.

**Consequence** · Valid SSE/pinned sessions written through the storage port become unreadable. If the scheduler is composed or cancellation is added using existing domain variants, those job rows will also be unreadable.

**Correction** · Make every decoder exhaustive over its domain enum and add one shared round-trip table per enum so writer and reader mappings cannot drift.

**Verification** · Persist and retrieve every writable variant against real SQLite, and unit-test every decoder against every domain variant.

### S1: Application write paths can leave database, files, and FTS inconsistent

**Confidence** · confirmed

**Location** · `crates/ck-app/src/lib.rs:161-264`, `crates/ck-app/src/lib.rs:328-459`, `crates/ck-search-fts/src/lib.rs:63-104`, `crates/ck-storage-sqlite/src/lib.rs:463-568`

**Evidence** · `store_context` writes an artifact payload, artifact metadata, session event, and FTS row as separate effects with no compensation. Ingestion upserts a document, deletes old FTS rows, replaces database chunks transactionally, then inserts FTS rows one at a time on separate connections. Any late error returns after earlier effects commit. Separately, filesystem normalization assigns a new document ULID on every refresh (`crates/ck-adapter-filesystem/src/lib.rs:169-206`), and refresh simply re-ingests (`crates/ck-app/src/lib.rs:1533-1547`); FTS removal targets only the new document ID, so successful refreshes retain prior documents and FTS rows.

**Consequence** · Disk failures, SQLite errors, or process termination can create orphan payloads, unsearchable stored events, or documents with missing/partial FTS entries. Even without failure, repeated refresh accumulates stale duplicate documents and searchable FTS rows.

**Correction** · Give documents a stable identity derived from source plus external ID, replace/remove prior source documents on refresh, and put database-owned document/chunk/FTS changes in one adapter transaction. For filesystem payloads, define compensation that removes a newly written payload when metadata/event persistence fails, and make indexing repairable from canonical database rows.

**Verification** · Inject failure after each effect and assert either the complete outcome or a clean rollback/recoverable state; restart and rebuild FTS from canonical rows.

### S2: Observability is initialized and immediately disabled

**Confidence** · confirmed by code inspection; runtime log capture was not performed

**Location** · `crates/ck-bin/src/main.rs:50-56`, `crates/ck-observability/src/lib.rs:18-46`, `crates/ck-bin/src/main.rs:96-103`, `crates/ck-bin/src/main.rs:177-223`

**Evidence** · `init_observability` returns a guard whose lifetime keeps the scoped subscriber installed. `main` converts the result to `Option` and drops it at the end of the statement. It also hard-codes text/no-OTEL before loading configuration, so `[observability]` is never applied.

**Consequence** · Tool, retention, refresh, and SSE failure logs can be absent even though the code emits them. Operators lose the signals needed to diagnose the other findings.

**Correction** · Load configuration before observability initialization and bind the returned guard for the lifetime of `main`.

**Verification** · Run the binary with JSON logging, trigger one successful and one failed tool call, and assert structured stderr records are emitted.

### S2: Advertised GitHub and web ingestion are not reachable in the binary

**Confidence** · confirmed by dependency and symbol-use search

**Location** · `README.md:105-115`, `crates/ck-bin/Cargo.toml:11-28`, `crates/ck-bin/src/main.rs:317-364`, `crates/ck-app/src/lib.rs:328-385`

**Evidence** · `ck-bin` has no dependencies on `ck-adapter-github` or `ck-adapter-web`, always injects `FilesystemSourceAdapter` and `MarkdownNormalizer`, and stores but never branches on `source_kind`. Repository-wide symbol search finds GitHub/web adapter construction only in their own tests.

**Consequence** · Requests declaring `github_repo` or `website` execute filesystem parsing instead and fail; the README's implemented capability claim is false for the shipped binary.

**Correction** · Add direct kind-based adapter selection in composition with the three existing implementations, without adding a registry abstraction, or mark GitHub/web adapters as not implemented in runtime.

**Verification** · Add one CLI test per source kind through `context-kernal serve`; each must reach its real adapter.

### S2: Web SSRF validation fails open and is vulnerable to DNS rebinding when wired

**Confidence** · probable; adapter path is tested but not currently composed into `ck-bin`

**Location** · `crates/ck-adapter-web/src/lib.rs:217-282`, `crates/ck-adapter-web/src/lib.rs:98-125`

**Evidence** · Resolver errors emit a warning and validation succeeds. Validation resolves once, then `ureq` performs a separate DNS resolution for the actual connection. The existing `validate_url_with_resolver_allows_unresolvable_host` test covers an empty successful resolution, not the resolver-error branch, but it also records a permissive unresolved-host policy (`crates/ck-adapter-web/src/lib.rs:826-841`).

**Consequence** · Once web ingestion is wired, transient DNS failure bypasses the safety decision, and attacker-controlled DNS can return a public address during validation then a private address during connection.

**Correction** · Fail closed on DNS resolution errors and connect to a validated resolved address while preserving the original host for HTTP/TLS, or use an HTTP client resolver that enforces the validated address set.

**Verification** · Add resolver-failure and rebinding tests that prove no request reaches a private target.

### S2: SSE lifecycle and readiness do not represent service health

**Confidence** · confirmed by code inspection

**Location** · `crates/ck-bin/src/main.rs:140-165`, `crates/ck-bin/src/main.rs:167-224`, `crates/ck-bin/src/main.rs:226-282`, `crates/ck-transport-sse/src/lib.rs:45-65`

**Evidence** · SSE starts in a detached OS thread; bind/runtime failure is only logged and cannot fail the parent process. The binary logs "started" before bind success is known. When reachable, `/readyz` always returns true without checking dependencies. On stdin EOF, no SSE shutdown signal or thread join occurs; the refresh thread can be sleeping for the full interval and is also not joined.

**Consequence** · The parent process can remain alive after SSE bind failure, while readiness provides no dependency-health signal once bound. Shutdown can terminate in-flight work without a drain boundary.

**Correction** · Make listener bind part of startup success, have readiness reflect bound/running state, and coordinate cancellation/join for SSE and refresh workers.

**Verification** · Occupy the configured port and assert startup fails; then terminate a running service and assert workers stop and in-flight requests complete or cancel predictably.

### S3: Configuration exposes settings that runtime ignores

**Confidence** · confirmed by field-use search

**Location** · `crates/ck-config/src/lib.rs:40-98`, `crates/ck-config/src/lib.rs:208-227`, `crates/ck-bin/src/main.rs:50-56`, `crates/ck-bin/src/main.rs:140-142`, `crates/ck-bin/src/main.rs:226-278`

**Evidence** · `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, `auth.trust_mode`, `service.name`, and configured observability values are parsed but do not control their named runtime behavior. Stdio is always served; the SSE route is fixed at `/mcp`.

**Consequence** · Operators can make valid configuration changes that silently do nothing, undermining the entire configuration contract.

**Correction** · Wire settings that are part of the supported contract and remove the rest until needed. Add behavior tests rather than parse-only tests.

**Verification** · For each retained setting, change only that value and assert an observable runtime difference.

### S3: Scheduler behavior has two owners, one of them dead in production

**Confidence** · confirmed by symbol-use search

**Location** · `crates/ck-scheduler/src/lib.rs:9-120`, `crates/ck-bin/src/main.rs:167-224`, `crates/ck-bin/Cargo.toml:11-28`

**Evidence** · `ck-scheduler::RefreshScheduler` implements queued, atomic job claiming and completion, but `ck-bin` does not depend on `ck-scheduler`. The runtime instead contains a separate sleep/list/refresh loop that bypasses `SchedulerPort` and the queue state machine.

**Consequence** · Scheduler tests prove code that production never invokes, while runtime refresh has different coalescing, status, and lifecycle behavior.

**Correction** · Either compose `RefreshScheduler` in `ck-bin` and delete the parallel loop in a dedicated change, or remove the unused scheduler crate/port and test the actual runtime loop. Keep one owner.

**Verification** · A production-wiring test must fail if `RefreshScheduler` is disconnected.

### S3: Freshness scoring is a fixed placeholder presented as a signal

**Confidence** · confirmed

**Location** · `crates/ck-app/src/lib.rs:649-657`, `crates/ck-app/src/lib.rs:694-708`, `README.md:117-128`

**Evidence** · `compute_freshness` ignores `_ttl_secs` and returns `0.5` for every non-empty refresh timestamp. The README says retrieval uses freshness score signals.

**Consequence** · Ranking cannot distinguish newly refreshed content from expired content, while score breakdowns imply that it can.

**Correction** · Compute age relative to TTL using parsed timestamps, or remove freshness from ranking/output until implemented.

**Verification** · Table-test fresh, half-TTL, expired, malformed, and absent timestamps.

### S4: Broad storage port forces large partial test implementations

**Confidence** · confirmed

**Location** · `crates/ck-core/src/ports/storage.rs:6-88`, `crates/ck-app/src/lib.rs:711-1268`, `crates/ck-scheduler/src/lib.rs:122-349`

**Evidence** · One `StoragePort` owns sources, corpora, documents, chunks, identities, jobs, retention, and artifacts. App and scheduler tests must implement dozens of unrelated methods, many as inert stubs or `unimplemented!()`.

**Consequence** · Port changes create broad test churn, and inert methods make it easy for a new code path to panic rather than provide meaningful evidence.

**Correction** · When the next port change requires touching these fakes, split only along existing use-case boundaries such as retrieval/source storage and job scheduling; do not perform a speculative full rewrite.

**Verification** · Each test double implements only the capability consumed by its subject, with no unreachable `unimplemented!()` methods.

## What Is Working

- The primary crate dependency direction is disciplined: core has no SQL, HTTP, or transport dependencies; application orchestration depends on core ports; adapter SQL is parameterized.
- SQLite enables foreign keys on every storage connection and uses a partial unique index plus atomic `UPDATE ... RETURNING` to prevent multiple active refresh jobs per source (`crates/ck-storage-sqlite/src/lib.rs:37-42`, `crates/ck-storage-sqlite/src/lib.rs:685-713`, `crates/ck-storage-sqlite/src/lib.rs:819-839`, `crates/ck-storage-sqlite/src/lib.rs:1225-1227`).
- Initial SSE startup fails closed when authentication is required but no token is configured (`crates/ck-bin/src/main.rs:115-127`), and token comparison is constant-time (`crates/ck-auth/src/lib.rs:53-74`).
- Tests include real SQLite behavior and CLI-level stdio paths rather than only mocks; 198 tests passed in the audited workspace.
- CI runs formatting, Clippy, tests, and a release build (`.github/workflows/ci.yml:7-19`).

## Priority Corrections

1. Close `context_required` authorization and path-containment defects, then add SSE boundary tests.
2. Preserve auth policy and context-agent wiring across config reload.
3. Repair all SQLite enum round trips and add exhaustive variant tests.
4. Define atomicity/compensation for context and ingestion writes.
5. Restore durable observability before relying on background refresh or remote serving.

## Evidence

- `cargo test --workspace` · exit 0 · 198 tests passed; proves the existing suite is green, not that uncovered audit paths are safe.
- `cargo clippy --workspace --all-targets -- -D warnings` · exit 0 · strict lint pass.
- `cargo fmt --check` · non-zero · committed formatting differences in `crates/ck-bin/src/main.rs` and `crates/ck-mcp/src/lib.rs`.
- `git status --short` before report creation · exit 0 with no output · worktree was clean.
- Morphology · 50 Rust files, 12,720 lines; largest modules are `ck-storage-sqlite` (1,838), `ck-app` (1,598), and `ck-adapter-web` (885).
- Caller discovery used repository-wide textual search. Dynamic consumers and external crates remain outside this audit's visibility.

**Counts** · 4 S1 · 4 S2 · 3 S3 · 1 S4 · 0 S5
