# Implementation Plan — Resolve NeatCode Deferred Debt

**Created:** 2026-07-29
**Source of truth:** `.agents/reports/neatcode-harden-2026-07-29.md`, especially "Open / deferred (named debt, with rationale)"
**Originating context:** The 2026-07-29 NeatCode harden pass fixed the immediate S1-S3 findings but deliberately retained eight residual or deferred items. This plan was derived from the current post-harden code and its passing 206-test baseline, not from report text alone.
**Status of the work today:** Corpus allowlisting/path containment, refresh replacement/compensation, SSE bind failure, readiness, and adapter unit implementations exist. Server-derived corpus privacy, DB/FTS atomicity, graceful joins, four config semantics, scheduler composition, narrow storage ports, the MCP adapter boundary, and GitHub/web runtime composition remain partial or not implemented.

---

## 0. How to use this plan (agent operating instructions)

- Execute tasks in the order given. Dependencies: Task 1→2; Task 3→4; Tasks 3 and 4→5; Task 5→6; Task 6→7; Task 7→8. Tasks 1 and 3 have no dependency on each other and may start in parallel, but their `ck-bin` convergence must be serialized.
- **Every task ends with a verification gate.** Do not mark a task done until its gate command exits 0.
- **A deferred label changes to resolved only after a failing-first test reproduces the report's residual behavior on the pre-change code and passes after the change.** Mitigations, compile-only proofs, and documentation claims do not satisfy this rule.
- Match surrounding code style; locate the relevant idioms here: `crates/ck-core/src/ports/`, `crates/ck-storage-sqlite/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/src/main.rs`, and crate-local `#[cfg(test)]` modules.
- Delegate by authority, not model name: give a worker one independently gated crate/file set only after prerequisite contracts are frozen; keep shared trait definitions, transaction boundaries, and all `ck-bin/src/main.rs` convergence with the in-context owner.

### Dependency graph

```text
Task 1 ACL context resolver port (item 7)
  └─ Task 2 server-derived context_required policy (item 1)

Task 3 atomic canonical + FTS writes (item 2)
  └─ Task 4 split StoragePort around proven transaction boundary (item 6)
       └─ Task 5 GitHub/web runtime source routing + docs (item 8; also Task 3)
            └─ Task 6 scheduler owns refresh jobs/execution (item 5)
                 └─ Task 7 graceful SSE/refresh shutdown + join (item 3)
                      └─ Task 8 resolve ignored config fields (item 4)

Tasks 1 and 3 may run in parallel in disjoint crates until either reaches ck-bin composition.
Tasks 2 and 4 may run in parallel after their respective prerequisites, but not against shared
ck-core port exports or ck-bin composition without an in-context integration owner.
```

### Global verification gates (must stay green after EVERY task)
```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

---

## Key facts already discovered (do not re-derive)

| Thing | Location |
| --- | --- |
| The harden report still labels all eight plan items residual or deferred | `.agents/reports/neatcode-harden-2026-07-29.md:34-42` |
| `context_required` copies caller-controlled `is_private` and `scope_claim` into the event | `crates/ck-mcp/src/lib.rs:435-477` |
| `AclContextAgent` treats any non-empty caller claim as sufficient and directly resolves files | `crates/ck-mcp/src/agentic_capability_loop.rs:191-220`, `crates/ck-mcp/src/agentic_capability_loop.rs:266-380` |
| `ck-mcp` directly depends on the filesystem adapter; `ck-bin` is already the composition root for both | `crates/ck-mcp/Cargo.toml:7-18`, `crates/ck-bin/Cargo.toml:11-28` |
| `AuthPort` has token/tool/corpus checks but no authenticated scope check | `crates/ck-core/src/ports/auth.rs:1-14` |
| `SqliteStore` and `FtsSearch` each own a path and open independent connections | `crates/ck-storage-sqlite/src/lib.rs:17-42`, `crates/ck-search-fts/src/lib.rs:9-22` |
| Ingest deletes old data, writes canonical documents/chunks, then indexes each FTS chunk through separate calls | `crates/ck-app/src/lib.rs:338-434` |
| Session events and their FTS rows are also persisted through separate ports/connections | `crates/ck-app/src/lib.rs:162-268`, `crates/ck-search-fts/src/lib.rs:176-190` |
| `StoragePort` combines source, corpus, document, chunk, library, refresh, retention, and artifact operations | `crates/ck-core/src/ports/storage.rs:6-88` |
| Scheduler tests implement the entire storage trait although the scheduler uses only source/job operations | `crates/ck-scheduler/src/lib.rs:9-119`, `crates/ck-scheduler/src/lib.rs:147-349` |
| `AppServices` owns exactly one source adapter and normalizer and does not dispatch by source kind | `crates/ck-app/src/lib.rs:15-26`, `crates/ck-app/src/lib.rs:98-122`, `crates/ck-app/src/lib.rs:386-403` |
| The binary composes only filesystem ingestion; GitHub/web are absent from its manifest | `crates/ck-bin/src/main.rs:353-400`, `crates/ck-bin/Cargo.toml:11-28` |
| README claims local, GitHub, and website ingestion in the shipped product | `README.md:40-47`, `README.md:105-115` |
| Runtime bypasses `ck-scheduler`, directly scans stale sources, and invokes refresh | `crates/ck-bin/src/main.rs:179-235` |
| Scheduler and application can both create refresh job records | `crates/ck-scheduler/src/lib.rs:19-101`, `crates/ck-app/src/lib.rs:443-485`, `crates/ck-app/src/lib.rs:1636-1660` |
| SSE and refresh thread handles are discarded; refresh uses an uninterruptible long sleep | `crates/ck-bin/src/main.rs:135-177`, `crates/ck-bin/src/main.rs:179-235`, `crates/ck-bin/src/main.rs:303-305` |
| Axum is served without a graceful-shutdown future | `crates/ck-transport-sse/src/lib.rs:45-59` |
| Four fields are parsed but ignored: semantic switch, stdio switch, SSE base path, and trust mode | `crates/ck-config/src/lib.rs:40-98`, `crates/ck-bin/src/main.rs:100-121`, `crates/ck-bin/src/main.rs:135-138`, `crates/ck-bin/src/main.rs:238-301` |

---

## Task 1 — Move ACL Corpus Resolution Behind a Core Port  (architecture · prerequisite)

**Goal:** `ck-mcp` has no direct or transitive normal dependency on `ck-adapter-filesystem`, while existing filesystem, learning-proposal, and path-containment behavior remains executable through an injected core port.

**Why this shape:** This task resolves deferred item 7 separately and creates the trusted corpus lookup boundary required by Task 2. It must not implement caller-scope authorization yet; mixing that security behavior into the architecture move would make the failing-first proof ambiguous. Dependency: no prior stage; it is ordered first because Task 2 depends on its server-owned registry, not by risk. Blast radius: `ck-core`, `ck-adapter-filesystem`, `ck-mcp`, and `ck-bin` composition; no SQLite transaction code. Authority: an adapter worker may implement `ck-adapter-filesystem` after the core port signature is frozen; the in-context owner must define the port and integrate `ck-mcp`/`ck-bin`.

### Steps

1. Add failing architecture and behavior tests: assert `cargo tree -p ck-mcp --edges normal` contains no filesystem adapter after the fix; add fake-resolver MCP tests; add a `context_registry_tests` module with exact tests for filesystem corpus, learning ledger, parent traversal, absolute path, and symlink escape — files: `crates/ck-mcp/tests/acl_context_agent.rs:1-180`, `crates/ck-adapter-filesystem/src/lib.rs:1-206`.
2. Run `cargo test -p ck-mcp --test acl_context_agent` and the dependency assertion before editing; record that behavior tests pass but the dependency assertion fails on `ck-adapter-filesystem`, proving the deferred architecture defect — file: `.agents/work_log.md:1`.
3. Define narrow domain DTOs and a `ContextCorpusPort` that returns server-owned corpus registration/policy plus resolved citation material without filesystem or MCP types — files: `crates/ck-core/src/ports/context_corpus.rs:1` (new), `crates/ck-core/src/ports/mod.rs:1`.
4. Implement `ContextCorpusPort` in the filesystem adapter, moving canonical containment, filesystem discovery/fetch, and learning-proposal ledger reading out of MCP without weakening fail-closed behavior — file: `crates/ck-adapter-filesystem/src/lib.rs:1-206`.
5. Change `AclContextAgent` and `McpService` to hold an injected trait object rather than paths or a concrete adapter; preserve current event output and current caller-driven authorization temporarily for Task 2 — files: `crates/ck-mcp/src/agentic_capability_loop.rs:144-380`, `crates/ck-mcp/src/lib.rs:57-103`.
6. Compose the filesystem registry in the binary for startup and reload, then remove `ck-adapter-filesystem` from MCP dependencies — files: `crates/ck-bin/src/main.rs:123-133`, `crates/ck-bin/src/main.rs:247-295`, `crates/ck-bin/src/main.rs:403-423`, `crates/ck-mcp/Cargo.toml:7-18`.
7. Update `.agents/evidence.md` from architecture partial to implemented only after both the behavior and dependency gates pass; keep server-side privacy validation explicitly residual — files: `.agents/evidence.md:3-9`, `.agents/project_state.md:37-47`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-adapter-filesystem context_registry_tests::resolves_filesystem_corpus -- --exact
cargo test -p ck-adapter-filesystem context_registry_tests::resolves_learning_proposals -- --exact
cargo test -p ck-adapter-filesystem context_registry_tests::rejects_parent_traversal -- --exact
cargo test -p ck-adapter-filesystem context_registry_tests::rejects_absolute_path -- --exact
cargo test -p ck-adapter-filesystem context_registry_tests::rejects_symlink_escape -- --exact
cargo test -p ck-mcp --test acl_context_agent
test -z "$(cargo tree -p ck-mcp --edges normal | rg 'ck-adapter-filesystem')"
cargo test -p ck-bin --test cli context_required_served_over_stdio_returns_cited_packet -- --exact
```

**Done when:** Filesystem and learning-proposal resolution still pass through the injected port, traversal tests fail if containment is removed, and the scripted dependency check fails if `ck-adapter-filesystem` is reintroduced anywhere in `ck-mcp`'s normal dependency tree.

**Redesign trigger:** If the resolver needs SQLite corpus records rather than the env-backed corpus root/ledger used today, stop and define one core registry contract implemented by composition; do not inject `SqliteStore` or filesystem types into MCP.

## Task 2 — Derive context_required Privacy and Scope Server-Side  (security · highest)

**Goal:** A caller cannot gain or alter `context_required` access by lying about `is_private` or supplying an arbitrary non-empty `scope_claim`; registered policy and authenticated server-side entitlement decide access before citation resolution.

**Why this shape:** This is the full fix for deferred item 1, not another mitigation. Dependency: Task 1, because server-side validation needs the injected corpus registry and must not preserve the direct filesystem bypass. It does not depend on storage/FTS work. Blast radius: auth/config/core policy, MCP dispatch, filesystem registry policy, binary startup/reload, and MCP contract docs; no storage schema. Authority: core auth/config and filesystem policy implementation can be delegated after the policy DTO is frozen; MCP access-context propagation and binary auth/reload composition must remain with one in-context owner.

### Steps

1. Add failing-first tests in the existing `tests` module where the registry says private but the caller sends `is_private:false`, where a non-empty claim differs from the required scope, where a matching claim is not entitled for the bearer token, where an unknown corpus is requested, and where denied requests make zero resolver calls — files: `crates/ck-mcp/src/lib.rs:647-798`, `crates/ck-mcp/tests/acl_context_agent.rs:1-180`.
2. Run the named MCP tests before implementation and record that at least the false-public and fabricated-claim cases are allowed by current code, reproducing the harden residual — file: `.agents/work_log.md:1`.
3. Add `authorization.context_corpora: Vec<ContextCorpusPolicyConfig>` entries with exact `corpus_id` and optional `required_scope`; a registered entry with no scope is public, one with a non-empty scope is private, duplicate/empty IDs and empty scope strings are invalid, and an absent corpus fails closed. Caller `is_private` remains compatibility input only and never an authorization fact — files: `crates/ck-core/src/ports/context_corpus.rs:1`, `crates/ck-config/src/lib.rs:183-206`, `config/context-kernal.example.toml:1`, `config/context-kernal.local.example.toml:1`, `config/context-kernal.sse.example.toml:1`.
4. Add `authorization.scope_tokens: BTreeMap<String, Vec<String>>`, mapping each required scope to the bearer tokens entitled to it; extend `AuthPort` with scope entitlement and implement constant-time token checks in `BearerAuth`. Explicit local stdio trust may access any registered corpus without a bearer token or caller claim, but registry privacy remains authoritative in emitted policy events; SSE bearer access requires both claim equality and token entitlement — files: `crates/ck-config/src/lib.rs:183-206`, `crates/ck-core/src/ports/auth.rs:1-14`, `crates/ck-auth/src/lib.rs:4-87`, `crates/ck-mcp/src/lib.rs:105-174`.
5. Pass access context through authorized MCP dispatch; in `context_required`, load server policy, require claim equality and bearer entitlement for private corpora, fail closed for unknown corpora, and call the resolver only after allow — files: `crates/ck-mcp/src/lib.rs:105-174`, `crates/ck-mcp/src/lib.rs:435-477`, `crates/ck-mcp/src/agentic_capability_loop.rs:191-220`.
6. Build identical policy/auth state on startup and reload; add SSE boundary coverage proving a client-public lie and fabricated claim are denied while an entitled claim succeeds — files: `crates/ck-bin/src/main.rs:109-133`, `crates/ck-bin/src/main.rs:247-295`, `crates/ck-bin/tests/cli.rs:662-765`.
7. Document `is_private` as non-authoritative compatibility input and `scope_claim` as checked against server registration and bearer entitlement — file: `docs/mcp-contracts.md:26-61`.
8. Mark the residual resolved only after the pre-change failure evidence and all positive/negative tests are recorded — files: `.agents/evidence.md:27-34`, `.agents/project_state.md:37-47`, `.agents/open_questions.md:3-20`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-auth tests::scope_authorization -- --nocapture
cargo test -p ck-mcp tests::context_required_uses_server_private_policy_not_client_is_private -- --exact
cargo test -p ck-mcp tests::context_required_rejects_fabricated_nonempty_scope_claim -- --exact
cargo test -p ck-mcp tests::context_required_rejects_scope_not_authorized_for_token -- --exact
cargo test -p ck-mcp tests::context_required_rejects_unregistered_corpus -- --exact
cargo test -p ck-mcp tests::context_required_allows_authorized_required_scope -- --exact
cargo test -p ck-bin --test cli context_required_scope_policy_survives_reload -- --exact
```

**Done when:** Tests prove caller privacy flags and arbitrary claims cannot authorize access, denied requests never resolve citations, registered public access still works, correctly entitled private access works, and deleting the server policy lookup makes the gate fail.

**Redesign trigger:** If bearer tokens cannot carry per-scope entitlement without changing the public auth contract, introduce a narrow `ScopeAuthorizationPort`; do not accept claim presence or corpus allowlisting as a substitute for entitlement.

## Task 3 — Make Canonical and FTS Writes Transactionally Atomic  (data integrity · highest)

**Goal:** Source snapshot replacement and session-event persistence commit canonical SQLite rows and corresponding FTS rows together, or roll back together, with tests proving no partial index survives an injected mid-write failure.

**Why this shape:** This resolves deferred item 2 beyond the current deletion/compensation mitigation. The transaction must be owned by the SQLite adapter; a borrowed `rusqlite::Connection`/`Transaction` may be passed to connection-scoped `FtsSearch` write helpers inside adapter crates, but no SQLite type may enter `ck-core` or `ck-app`. Dependency: no dependency on Tasks 1-2, ordered by risk after the security chain; Task 4 depends on the narrow atomic write capability established here. Blast radius: `ck-core` write port, `ck-app` ingestion/context writes, SQLite storage, FTS writes, and binary construction; no schema migration expected. Authority: one in-context owner defines the transaction and core write contract; storage and FTS test implementation may be delegated only after that contract is frozen.

### Steps

1. Add failing-first SQLite tests in the existing `tests` module that seed an old searchable source snapshot, inject failure on a later new chunk, and assert old canonical/FTS rows remain with no partial new rows; add the matching successful-set test — file: `crates/ck-storage-sqlite/src/lib.rs:1460-1640`.
2. Add a failing application test proving discovery/fetch/normalization failure preserves the prior source snapshot, and a session-event test proving an FTS failure does not commit the canonical event — files: `crates/ck-app/src/lib.rs:680-1018`, `crates/ck-storage-sqlite/src/lib.rs:1460-1640`.
3. Run those tests before implementation and record the current delete-before-fetch or partial-write failure, satisfying the harden report's residual proof requirement — file: `.agents/work_log.md:1`.
4. Define `IndexWritePort` in a new file with one source-snapshot operation and one atomic session-event/index operation; keep its DTOs domain-only — files: `crates/ck-core/src/ports/index_write.rs:1` (new), `crates/ck-core/src/ports/mod.rs:1`.
5. Change ingestion to discover, fetch, normalize, and chunk all units before mutating the previous snapshot; invoke the atomic writer once instead of alternating storage and lexical-search calls — file: `crates/ck-app/src/lib.rs:323-434`.
6. Implement each operation with one SQLite connection and transaction covering old FTS deletion, canonical replacement, new canonical rows, new FTS rows, and commit; expose connection-scoped FTS write helpers from `FtsSearch` rather than opening another connection — files: `crates/ck-storage-sqlite/src/lib.rs:351-573`, `crates/ck-search-fts/src/lib.rs:9-105`, `crates/ck-storage-sqlite/Cargo.toml:7`, `crates/ck-search-fts/Cargo.toml:7`.
7. Keep `FtsSearch`'s independent connection behavior for read-only search; remove application-level FTS mutation calls once all persisted write paths use the atomic adapter — files: `crates/ck-search-fts/src/lib.rs:107-190`, `crates/ck-app/src/lib.rs:162-268`, `crates/ck-app/src/lib.rs:378-434`, `crates/ck-bin/src/main.rs:353-400`.
8. Record no schema migration if tables remain unchanged; mark atomicity resolved only when trigger-induced rollback tests and canonical-vs-FTS set equality pass — files: `.agents/evidence.md:36-43`, `.agents/project_state.md:37-52`, `.agents/decisions.md:19-25`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-storage-sqlite tests::replace_source_index_rolls_back_canonical_and_fts -- --exact
cargo test -p ck-storage-sqlite tests::replace_source_index_commits_matching_chunk_and_fts_sets -- --exact
cargo test -p ck-storage-sqlite tests::add_event_rolls_back_when_fts_write_fails -- --exact
cargo test -p ck-app tests::ingest_fetch_failure_preserves_previous_snapshot -- --exact
cargo test -p ck-app tests::refresh_does_not_accumulate_duplicate_documents -- --exact
```

**Done when:** Injected failure after at least one attempted new FTS row leaves the complete prior snapshot and no new canonical/FTS prefix; successful writes yield identical canonical and FTS chunk-ID sets; deliberately moving one FTS insert outside the transaction makes the rollback gate fail.

**Redesign trigger:** If FTS5 virtual-table operations cannot participate in the same SQLite transaction under the bundled SQLite build, stop and prove that limitation with a minimal test before designing an outbox/reconciliation protocol; do not call compensation equivalent to atomicity.

## Task 4 — Split StoragePort by Consumer Authority  (architecture · structural)

**Goal:** No consumer implements or receives the monolithic `StoragePort`; application, scheduler, retention, retrieval, index writing, and artifact metadata depend only on the smallest port sets they use.

**Why this shape:** This resolves deferred item 6 after Task 3 defines the correct atomic index-write boundary. Splitting first would either preserve non-atomic methods or force a second trait rewrite. Dependency: Task 3. It is independent of Tasks 1-2 after shared `ck-core/src/ports/mod.rs` edits are serialized. Blast radius: all `StoragePort` declarations, implementations, consumers, constructors, and test fakes across core, app, SQLite, scheduler, and binary crates. Authority: individual consumer migrations can be delegated by crate after the new traits are frozen; trait design, blanket implementations, and removal of `StoragePort` remain with the in-context owner.

### Steps

1. Add a failing architecture assertion that `pub trait StoragePort` no longer exists and add a scheduler test fake implementing only source-refresh/job operations; capture the current failure because the scheduler fake must implement the full trait — files: `crates/ck-core/src/ports/storage.rs:6-88`, `crates/ck-scheduler/src/lib.rs:122-349`.
2. Partition existing methods around observed consumers and the Task 3 transaction boundary into `source_catalog.rs`, `retrieval_store.rs`, `refresh_job_store.rs`, `retention_store.rs`, and `artifact_metadata.rs`, retaining Task 3's `index_write.rs` — files: `crates/ck-core/src/ports/source_catalog.rs:1` (new), `crates/ck-core/src/ports/retrieval_store.rs:1` (new), `crates/ck-core/src/ports/refresh_job_store.rs:1` (new), `crates/ck-core/src/ports/retention_store.rs:1` (new), `crates/ck-core/src/ports/artifact_metadata.rs:1` (new), `crates/ck-core/src/ports/index_write.rs:1`, `crates/ck-core/src/ports/mod.rs:1`.
3. Implement the focused traits on `SqliteStore` without changing SQL behavior or schema — file: `crates/ck-storage-sqlite/src/lib.rs:351-1021`.
4. Change `AppServices` constructor/fields to accept only its focused capabilities and reduce its test fake(s) to the methods exercised by each use-case test module — file: `crates/ck-app/src/lib.rs:13-139`, `crates/ck-app/src/lib.rs:680-1018`.
5. Change `RefreshScheduler` to accept only source-catalog and refresh-job-store capabilities; replace its large partial fake with a narrow fake — file: `crates/ck-scheduler/src/lib.rs:1-119`, `crates/ck-scheduler/src/lib.rs:122-349`.
6. Update binary composition and all remaining implementations found at the current call sites; delete `storage.rs` rather than retaining a compatibility facade — files: `crates/ck-bin/src/main.rs:13-19`, `crates/ck-bin/src/main.rs:353-400`, `crates/ck-core/src/ports/storage.rs:1-88`, `crates/ck-app/src/lib.rs:760-1018`, `crates/ck-scheduler/src/lib.rs:147-349`.
7. Record the port interface decision and exact consumers; do not claim runtime scheduler ownership changed in this task — files: `.agents/decisions.md:1`, `.agents/evidence.md:3-9`, `.agents/project_state.md:9-29`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-scheduler
cargo test -p ck-app
cargo test -p ck-storage-sqlite
test -z "$(rg 'trait StoragePort|impl StoragePort|dyn StoragePort' crates --glob '*.rs')"
```

**Done when:** Scheduler and application tests compile with focused fakes, no monolithic trait definition/use remains, and adding an unrelated retention method to the refresh-job port makes the narrow scheduler fake fail review rather than silently broadening it.

**Redesign trigger:** If more than one focused trait has the same method solely to satisfy constructor ergonomics, consolidate the shared read capability once; do not recreate `StoragePort` under a new aggregate name.

## Task 5 — Wire GitHub and Web Source Pipelines into Runtime  (feature wiring/docs · correctness)

**Goal:** The shipped binary selects matching adapter and normalizer pipelines for `filesystem_tree`, `github_repo`, and `website`, rejects unknown kinds before mutation, refreshes each stored kind through the same route, and README claims exactly that tested runtime capability.

**Why this shape:** This resolves deferred item 8 as runtime wiring plus a docs fix, not a docs-only downgrade. Adapter-only routing is insufficient because GitHub/web normalizers preserve source-specific metadata. Dependencies: Task 3, because remote failures must not erase the old snapshot, and Task 4, because this changes the `AppServices` constructor/composition point after its port split. It precedes Task 6 because the scheduler must execute the final kind-routed refresh path. Blast radius: app pipeline selection, GitHub/web security and no-change semantics, binary dependencies/composition/tests, README, operations docs, and examples. Authority: adapter-specific tests may be delegated by crate; `AppServices` pipeline selection, `ck-bin` composition, and docs claims require one in-context integration owner.

### Steps

1. Add failing-first application tests using two recording adapter/normalizer pairs to prove source-kind selection, unknown-kind rejection before storage mutation, and refresh routing from persisted kind; add a web-adapter test proving the validated IP is the IP used for the request so DNS rebinding cannot switch to a private address — files: `crates/ck-app/src/lib.rs:680-1018`, `crates/ck-adapter-web/src/lib.rs:75-166`.
2. Add binary smoke tests for `github_repo` and `website` using deterministic empty/local adapter inputs so no external network is required; record the current `FilesystemSourceAdapter` "missing path" failure — file: `crates/ck-bin/tests/cli.rs:188-314`.
3. Introduce the smallest pipeline registry accepted by `AppServices`, keyed by the existing `SourceAdapter::kind()` values and pairing each adapter with its normalizer; reject duplicate and unknown kinds — files: `crates/ck-app/src/lib.rs:13-139`, `crates/ck-core/src/ports/source_adapter.rs:1-15` only if a small pipeline DTO is necessary.
4. Select a pipeline before any source/corpus/index mutation, then use that same pipeline for discover, fetch, normalize, initial ingest, and refresh — file: `crates/ck-app/src/lib.rs:323-434`, `crates/ck-app/src/lib.rs:1636-1652`.
5. Add GitHub/web dependencies and compose all three pipelines in the shared `wire_app` used by startup, refresh, and reload — files: `crates/ck-bin/Cargo.toml:11-28`, `crates/ck-bin/src/main.rs:79-80`, `crates/ck-bin/src/main.rs:181-185`, `crates/ck-bin/src/main.rs:255-289`, `crates/ck-bin/src/main.rs:353-400`.
6. Before exposing web ingestion in `ck-bin`, pin the outbound request to an address returned by the validated resolution while preserving the original Host/TLS authority, then add regression coverage for 304/no-change so it preserves the prior atomic snapshot rather than replacing it with empty content — files: `crates/ck-adapter-web/src/lib.rs:75-166`, `crates/ck-adapter-web/src/lib.rs:478-514`, `crates/ck-app/src/lib.rs:680-1018`.
7. After runtime tests pass, update README adapter statements and operations/config examples to distinguish tested binary support from crate availability; if any adapter cannot pass its runtime smoke test, document it as library-only instead of claiming completion — files: `README.md:40-47`, `README.md:105-115`, `docs/operations.md:56-67`, `config/context-kernal.example.toml:1`, `config/context-kernal.local.example.toml:1`, `config/context-kernal.sse.example.toml:1`.
8. Record exact wiring evidence and close the README overstatement only after tests pass — files: `.agents/evidence.md:1`, `.agents/project_state.md:37-61`, `.agents/open_questions.md:6-20`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-app tests::ingest_source_selects_adapter_and_normalizer_by_kind -- --exact
cargo test -p ck-app tests::ingest_source_rejects_unknown_kind_before_storage_mutation -- --exact
cargo test -p ck-app tests::refresh_source_uses_persisted_source_kind -- --exact
cargo test -p ck-app tests::not_modified_web_refresh_preserves_existing_document -- --exact
cargo test -p ck-adapter-web tests::request_uses_validated_resolved_address -- --exact
cargo test -p ck-bin --test cli ingest_source_dispatches_github_repo_kind -- --exact
cargo test -p ck-bin --test cli ingest_source_dispatches_website_kind -- --exact
```

**Done when:** Each runtime kind reaches only its matching adapter and normalizer, unknown kinds leave storage unchanged, refresh preserves kind routing, and changing `wire_app` back to filesystem-only makes both binary adapter smoke tests fail; README then matches those results.

**Redesign trigger:** If adapter configs cannot express deterministic no-network smoke cases, add injected HTTP clients only inside the adapter tests; do not make workspace verification depend on GitHub or public DNS.

## Task 6 — Make ck-scheduler the Single Refresh Owner  (runtime architecture · structural)

**Goal:** Manual and periodic refreshes use `ck-scheduler` for coalescing and one queued→running→completed/failed job lifecycle, while `ck-bin` contains no parallel inline stale-source execution loop.

**Why this shape:** This resolves deferred item 5. Dependencies: Task 4 supplies narrow scheduler storage traits; Task 5 supplies the final kind-routed refresh executor. It must precede Task 7 because graceful shutdown must stop and join the real worker owner, not the inline loop being removed. Blast radius: scheduler/core job APIs, SQLite claiming, app refresh orchestration, MCP-visible response behavior, binary dependencies/composition, and lifecycle tests. Authority: scheduler execution API and tests may be delegated within `ck-scheduler`; the application job boundary and binary convergence must stay with one in-context owner.

### Steps

1. Add failing-first tests proving one scheduled refresh creates exactly one job record, concurrent manual/periodic requests coalesce per source, and success/failure transitions update that same record — files: `crates/ck-scheduler/src/lib.rs:122-510`, `crates/ck-bin/tests/cli.rs:284-314`.
2. Run the tests before implementation and record the duplicate-job/bypass behavior: scheduler creates queue records while `AppServices::refresh_source` creates its own completed manual record — file: `.agents/work_log.md:1`.
3. Inject `Arc<dyn SchedulerPort>` into `AppServices`; keep the existing MCP handler calling `AppServices::refresh_source`, but make that method enqueue and atomically claim that exact returned job ID (or return the already-running coalesced job), run a private content-refresh operation, and settle the same scheduler-owned job while preserving the current response shape — files: `crates/ck-app/src/lib.rs:13-139`, `crates/ck-app/src/lib.rs:443-485`, `crates/ck-app/src/lib.rs:1636-1660`, `crates/ck-mcp/src/lib.rs:480-493`.
4. Add `SchedulerPort::claim_job(job_id)` backed by an atomic queued→running update for that ID; retain global `claim_next_job` only for the background worker. Make `RefreshScheduler` own all status transitions and source-health updates exactly once — files: `crates/ck-core/src/ports/scheduler.rs:1-26`, `crates/ck-scheduler/src/lib.rs:9-119`, `crates/ck-storage-sqlite/src/lib.rs:824-845`.
5. Persist/accept `freshness_ttl_secs` from ingestion input so scheduler-created sources can actually become stale; add a focused failing-first test for non-`None` TTL — files: `crates/ck-app/src/lib.rs:325-360`, `crates/ck-mcp/src/lib.rs:281-306`.
6. Add `ck-scheduler` to binary dependencies, compose one scheduler/executor, route manual MCP refresh and periodic stale-source enqueue/claim through it, and delete the direct stale-source refresh loop — files: `crates/ck-bin/Cargo.toml:11-28`, `crates/ck-bin/src/main.rs:179-235`.
7. Define and test startup recovery for jobs left `running` by a prior process; use an explicit failed/requeued policy rather than leaving them permanently active — files: `crates/ck-scheduler/src/lib.rs:53-101`, `crates/ck-storage-sqlite/src/lib.rs:690-845`.
8. Record scheduler ownership as implemented but leave worker shutdown partial until Task 7 — files: `.agents/decisions.md:1`, `.agents/evidence.md:1`, `.agents/project_state.md:37-47`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-scheduler tests::scheduled_refresh_uses_one_job_record -- --exact
cargo test -p ck-scheduler tests::scheduler_recovers_interrupted_running_job -- --exact
cargo test -p ck-app tests::ingested_source_persists_refresh_ttl -- --exact
cargo test -p ck-bin --test cli manual_and_periodic_refresh_coalesce_per_source -- --exact
```

**Done when:** One source has at most one active job, one execution produces one history record, restart recovery is explicit, and reintroducing direct binary refresh execution makes either the coalescing test or the source assertion fail.

**Redesign trigger:** If preserving synchronous `refresh_source` responses cannot coexist with scheduler ownership, keep the call synchronous but enqueue, claim, execute, and settle the same job inline; do not create a second job or silently change the MCP response to asynchronous.

## Task 7 — Gracefully Stop and Join SSE and Refresh Workers  (operations · lifecycle)

**Goal:** Stdio EOF stops new work, drains controlled in-flight HTTP work, interrupts a long refresh wait promptly, settles/cancels worker state according to the Task 6 policy, and joins every retained SSE/refresh worker handle without detaching it.

**Why this shape:** This resolves deferred item 3 after scheduler ownership is known. A naive join is forbidden because the current refresh sleep can last 1,800 seconds. Dependency: Task 6. Blast radius: SSE serve signature/tests, scheduler wait behavior, binary worker ownership/reload, and operations docs; no MCP contract change. It can delegate transport-local graceful shutdown to a `ck-transport-sse` worker, but the in-context owner must retain shutdown channels/handles and define process-wide ordering in `ck-bin`.

### Steps

1. Add failing-first tests in a new `lifecycle_tests` module for an interruptible long refresh interval, SSE return on an explicit shutdown channel, controlled in-flight request drain, and process EOF joining all workers; prove current code lacks a channel/handle or exits without drain — files: `crates/ck-transport-sse/src/lib.rs:160-181`, `crates/ck-bin/src/main.rs:135-305`, `crates/ck-bin/tests/cli.rs:1-765`.
2. Change `serve_sse` to accept a shutdown future/channel and use Axum graceful shutdown; retain behavior for health, readiness, auth, and MCP routes — file: `crates/ck-transport-sse/src/lib.rs:45-59`.
3. Replace scheduler/refresh `thread::sleep` with an interruptible channel wait or runtime timer, and make claimed-job shutdown behavior follow Task 6's explicit recovery/settlement rule — files: `crates/ck-scheduler/src/lib.rs:9-119`, `crates/ck-bin/src/main.rs:179-235`.
4. Retain the existing blocking stdio loop on the main thread, but retain SSE and scheduler `JoinHandle`s plus shutdown senders. On stdin EOF, notify SSE graceful shutdown, notify the scheduler, join both threads, and propagate thread panics and ordinary worker errors through `serve` — file: `crates/ck-bin/src/main.rs:63-305`.
5. Keep the prompt-exit bound limited to cooperative idle-wait interruption and controlled test requests; real synchronous adapter calls are drained to completion because standard `JoinHandle` has no safe timed join. Do not add a cancellability claim for blocking stdin or adapter I/O — file: `crates/ck-bin/src/main.rs:135-305`.
6. Ensure config reload cannot leave an old scheduler/application generation running against the prior database; either restart the worker through the supervisor or classify those fields startup-only and reject live change — files: `crates/ck-bin/src/main.rs:247-295`, `docs/operations.md:56-67`.
7. Record shutdown/join as resolved only after a test uses an interval much longer than its prompt-exit assertion and still joins promptly; separately document that active synchronous adapter I/O is drained without a fixed deadline — files: `.agents/evidence.md:60-67`, `.agents/project_state.md:37-47`, `.agents/decisions.md:1`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-transport-sse lifecycle_tests::serve_sse_returns_after_shutdown_signal -- --exact
cargo test -p ck-transport-sse lifecycle_tests::shutdown_waits_for_in_flight_http_request -- --exact
cargo test -p ck-bin lifecycle_tests::refresh_worker_shutdown_interrupts_long_interval -- --exact
cargo test -p ck-bin --test cli eof_gracefully_joins_all_workers -- --exact
```

**Done when:** A test configured with an hour-long refresh interval joins promptly after stdin EOF, a controlled in-flight request completes before SSE returns, worker panic reaches the process result, and removing either join causes the process-level gate to fail.

**Redesign trigger:** If product requirements demand a hard shutdown deadline for synchronous adapter I/O, replace those calls with cancellable async I/O or isolate them in killable worker processes; do not fake timed `std::thread` joins or detach live workers.

## Task 8 — Remove or Honor Every Parsed Config Field  (configuration · compatibility)

**Goal:** `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, and `auth.trust_mode` each have one tested runtime meaning or are removed with fail-fast unknown-field validation and migration documentation; none is silently parsed and ignored.

**Why this shape:** This resolves deferred item 4 after Task 7 provides worker shutdown/join; honoring `enable_stdio=false` before that would let `serve` return immediately with detached workers. Dependency: Task 7. No dependency exists among the four fields beyond their shared config/composition files, so they are combined only because the source report names one dead-config item. Blast radius: config structs/parser/tests, binary transport lifecycle, SSE routing, examples, and configuration/operations docs. Authority: config parsing tests and transport route tests may be delegated in disjoint crates; final semantics, compatibility choice, and binary composition remain in-context.

### Steps

1. Add failing-first tests proving `enable_stdio=false` does not read/respond on stdin while SSE stays alive until explicit shutdown, configured `base_path` is the only MCP route, and invalid/removed semantic/trust settings fail config load rather than disappear — files: `crates/ck-bin/tests/cli.rs:1-765`, `crates/ck-transport-sse/src/lib.rs:160-181`, `crates/ck-config/src/lib.rs:300-520`.
2. Resolve `mcp.enable_stdio` through the Task 7 supervisor: run the existing blocking stdin loop only when true; when false and SSE is enabled, wait for `tokio::signal::ctrl_c()` in a small runtime before invoking the same shutdown-and-join path; reject configurations with both stdio and SSE disabled — files: `crates/ck-bin/src/main.rs:135-305`, `crates/ck-config/src/lib.rs:46-85`.
3. Pass normalized `mcp.sse.base_path` into router construction; validate that it is an absolute single route path and classify it startup-only unless reload can atomically replace the listener/router — files: `crates/ck-config/src/lib.rs:70-85`, `crates/ck-transport-sse/src/lib.rs:45-59`, `crates/ck-bin/src/main.rs:135-177`.
4. Remove `retrieval.semantic_enabled` as the duplicate switch and make `[embeddings].enabled` the single semantic authority already used by composition; reject the removed key rather than silently accepting it — files: `crates/ck-config/src/lib.rs:40-44`, `crates/ck-bin/src/main.rs:100-107`, `crates/ck-bin/src/main.rs:371-398`, `config/context-kernal.example.toml:1`, `config/context-kernal.local.example.toml:1`, `config/context-kernal.sse.example.toml:1`.
5. Remove `auth.trust_mode` as the duplicate/unsafe remote trust switch and retain `mcp.sse.require_auth` plus explicit local stdio trust as the transport authorities; reject the removed key rather than silently accepting it — files: `crates/ck-config/src/lib.rs:88-98`, `crates/ck-bin/src/main.rs:109-121`, `config/context-kernal.example.toml:1`, `config/context-kernal.local.example.toml:1`, `config/context-kernal.sse.example.toml:1`.
6. Add `#[serde(deny_unknown_fields)]` to `RetrievalConfig` and `AuthConfig`, with migration tests proving the two removed keys produce actionable load errors — file: `crates/ck-config/src/lib.rs:40-44`, `crates/ck-config/src/lib.rs:88-98`, `crates/ck-config/src/lib.rs:300-520`.
7. Update the configuration reference and all three example files only after the behavior tests pass; state restart requirements for bind/base-path/scheduler changes — files: `docs/configuration.md:33-60`, `docs/operations.md:56-67`, `config/context-kernal.example.toml:1`, `config/context-kernal.local.example.toml:1`, `config/context-kernal.sse.example.toml:1`.
8. Record all four fields individually as honored or removed, with their exact tests; do not use one passing parser test as evidence for runtime behavior — files: `.agents/evidence.md:1`, `.agents/project_state.md:9-29`, `.agents/open_questions.md:3-20`, `.agents/work_log.md:1`.

### Gate

```bash
cargo test -p ck-bin --test cli disabled_stdio_does_not_read_or_answer_stdin -- --exact
cargo test -p ck-bin --test cli sse_only_mode_survives_stdin_eof_until_shutdown -- --exact
cargo test -p ck-transport-sse lifecycle_tests::configured_base_path_routes_mcp -- --exact
cargo test -p ck-config tests::rejects_configuration_with_all_transports_disabled -- --exact
cargo test -p ck-config tests::removed_semantic_enabled_is_not_silently_ignored -- --exact
cargo test -p ck-config tests::removed_trust_mode_is_not_silently_ignored -- --exact
```

**Done when:** Every named field has a runtime assertion or a fail-fast removal assertion, an SSE-only process remains alive without stdin, old removed keys produce an actionable load error, and restoring any field as parsed-but-unused makes its dedicated test fail.

**Redesign trigger:** If released configuration compatibility is confirmed for `semantic_enabled` or `trust_mode`, retain and honor it through one validated authority with an explicit precedence test; do not keep two switches whose conflict behavior is undocumented.

---

## Final acceptance checklist (whole plan)
- [ ] `ck-mcp` resolves context through an injected port and its dependency tree contains no filesystem adapter; containment teeth tests pass. *(Task 1)*
- [ ] Caller `is_private` and arbitrary non-empty claims cannot authorize a private corpus; denied requests perform no resolution. *(Task 2)*
- [ ] Injected mid-ingest/session-index failures roll back canonical and FTS rows as one unit, and canonical/FTS sets match after commit. *(Task 3)*
- [ ] No `StoragePort` definition/use remains and scheduler/application tests use focused port fakes. *(Task 4)*
- [ ] Filesystem, GitHub, and website kinds select matching runtime pipelines, unknown kinds do not mutate storage, and README matches tested wiring. *(Task 5)*
- [ ] One scheduler-owned job represents each manual/periodic refresh and direct binary refresh execution is absent. *(Task 6)*
- [ ] SSE and refresh workers receive shutdown, drain according to policy, interrupt long waits, and are joined by process-level tests. *(Task 7)*
- [ ] All four formerly ignored config fields are honored or fail-fast removed with migration docs and dedicated behavior tests. *(Task 8)*
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` exits 0.
- [ ] `README.md`, `docs/configuration.md`, `docs/operations.md`, and `docs/mcp-contracts.md` reflect runtime reality; no status is flipped ahead of a passing test.
- [ ] All global gates green; no generated artifact rebuild is required.

## Guardrails (do not violate)
- Do not call the current allowlist/path containment or arbitrary non-empty claim check a full server-side privacy fix; the source report explicitly labels it residual.
- Keep SQL, `rusqlite::Connection`, filesystem paths, HTTP clients, and MCP protocol types out of `ck-core`; composition belongs in `ck-bin`.
- Preserve one atomic source snapshot: discover/fetch/normalize before mutation, then commit canonical and FTS rows together.
- Do not retain a compatibility aggregate that recreates broad `StoragePort`, duplicate refresh-job ownership, or detached worker ownership under a new name.
- Keep each task in its own reviewable commit; serialize edits to shared port exports and `crates/ck-bin/src/main.rs` even when crate-local workers run in parallel.
