# Deployment Readiness Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Finish the remaining partial and not-yet-implemented production features so Context Kernel can be deployed by other users as a fully documented, observable, secure, single-binary MCP service.

**Architecture:** Keep `ck-core` as domain ports and types only. Put runtime behavior in `ck-app`, concrete storage in `ck-storage-sqlite`, observability setup in a new `ck-observability` crate, memory policy in a new `ck-memory` crate, and composition in `ck-bin`. Preserve the current synchronous core and only use async inside transports.

**Tech Stack:** Rust 2021, Cargo workspace, rusqlite, axum/tokio for SSE, tracing/tracing-subscriber, OpenTelemetry crates for OTEL export, GitHub Actions, TDD with existing unit and CLI integration tests.

## Release Standard

The project is deployable only when all items below are true:
- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets` passes with no warnings.
- `cargo test --workspace` passes.
- `cargo build --release -p ck-bin` produces `target/release/context-kernal`.
- The binary can run `migrate`, `doctor`, `export-schema`, and `serve`.
- SSE requests enforce bearer auth and tool/corpus authorization.
- Config reload updates both stdio and SSE behavior.
- Retention GC removes database metadata and artifact payload files.
- Source scope filtering protects private corpora in resolve/query flows.
- OTEL traces and metrics can be exported to a local collector.
- Install, config, operations, and release docs match code behavior.
- `.agent/project_state.md` and `.agent/evidence.md` contain code/test evidence for every completed claim.

## Task 1: Add Project Instructions File

**Files:**
- Create: `.github/copilot-instructions.md`
- Modify: `.agent/open_questions.md`
- Test: static inspection

**Step 1: Write the file**

Create `.github/copilot-instructions.md` with repository rules copied from `AGENTS.md` in concise form:

```markdown
# Context Kernel Agent Instructions

- Trust repository code and tests over docs.
- Read `.agent/` memory files before work.
- Preserve ports-and-adapters boundaries.
- Keep `ck-core` free of SQL, HTTP, and transport details.
- Use test-first changes for behavior.
- Update `.agent/project_state.md`, `.agent/work_log.md`, and `.agent/evidence.md` after meaningful work.
```

**Step 2: Verify**

Run:

```bash
test -f .github/copilot-instructions.md
```

Expected: exit 0.

**Step 3: Update memory**

Remove the missing-copilot-instructions entry from `.agent/open_questions.md`.

## Task 2: Create `ck-observability` Crate

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ck-observability/Cargo.toml`
- Create: `crates/ck-observability/src/lib.rs`
- Modify: `crates/ck-bin/Cargo.toml`
- Modify: `crates/ck-bin/src/main.rs`
- Test: `crates/ck-observability/src/lib.rs`

**Step 1: Write failing tests**

Add tests in `crates/ck-observability/src/lib.rs`:

```rust
#[test]
fn builds_text_tracing_config_without_otel() {
    let config = ObservabilityRuntimeConfig {
        log_format: "text".to_string(),
        otel_endpoint: None,
        service_name: "context-kernal".to_string(),
    };

    assert_eq!(config.log_format, "text");
    assert!(!config.otel_enabled());
}

#[test]
fn detects_otel_endpoint() {
    let config = ObservabilityRuntimeConfig {
        log_format: "json".to_string(),
        otel_endpoint: Some("http://127.0.0.1:4317".to_string()),
        service_name: "context-kernal".to_string(),
    };

    assert!(config.otel_enabled());
}
```

Run:

```bash
cargo test -p ck-observability
```

Expected: fail because the crate and types do not exist.

**Step 2: Implement crate**

Add workspace member `crates/ck-observability`.

Implement:
- `ObservabilityRuntimeConfig`
- `otel_enabled()`
- `init_observability(config) -> anyhow::Result<ObservabilityGuard>`
- text and JSON tracing setup
- OTEL trace export when `otel_endpoint` is set
- metrics meter setup or explicit no-op guard when metrics are unavailable

Keep all OpenTelemetry dependency usage out of `ck-core`.

**Step 3: Wire binary**

Replace direct `tracing_subscriber::fmt()` setup in `crates/ck-bin/src/main.rs` with `ck_observability::init_observability`.

**Step 4: Verify**

Run:

```bash
cargo test -p ck-observability
cargo test -p ck-bin
```

Expected: all pass.

## Task 3: Add Operational Telemetry Events And Metrics

**Files:**
- Modify: `crates/ck-mcp/src/lib.rs`
- Modify: `crates/ck-app/src/lib.rs`
- Modify: `crates/ck-transport-sse/src/lib.rs`
- Modify: `crates/ck-bin/src/main.rs`
- Test: `crates/ck-mcp/src/lib.rs`, `crates/ck-transport-sse/src/lib.rs`

**Step 1: Write failing tests**

Add tests that call MCP tools and assert spans/fields are emitted through a test subscriber, or expose a small `ToolTelemetry` DTO from `ck-mcp` and test its field construction:

```rust
#[test]
fn tool_telemetry_records_name_and_outcome() {
    let event = ToolTelemetry::success("get_status", 12);

    assert_eq!(event.tool_name, "get_status");
    assert_eq!(event.outcome, "success");
    assert_eq!(event.duration_ms, 12);
}
```

Run:

```bash
cargo test -p ck-mcp tool_telemetry_records_name_and_outcome
```

Expected: fail because telemetry DTO/helper does not exist.

**Step 2: Implement instrumentation**

Add structured events for:
- MCP tool call start/end/error
- SSE request auth success/failure
- retention GC counts
- config reload success/failure
- ingestion document/chunk counts
- refresh job lifecycle

Use stable field names:
- `tool_name`
- `request_id`
- `source_id`
- `corpus_id`
- `job_id`
- `outcome`
- `duration_ms`
- `error`

**Step 3: Verify**

Run:

```bash
cargo test -p ck-mcp
cargo test -p ck-transport-sse
```

Expected: all pass.

## Task 4: Make Config Reload Update SSE Runtime

**Files:**
- Modify: `crates/ck-transport-sse/src/lib.rs`
- Modify: `crates/ck-bin/src/main.rs`
- Test: `crates/ck-transport-sse/src/lib.rs` or `crates/ck-bin/tests/cli.rs`

**Step 1: Write failing test**

Create a test proving an SSE state holder can swap MCP service and auth policy:

```rust
#[test]
fn reloadable_sse_state_swaps_service_and_auth() {
    let first = Arc::new(McpService::new(KernelStatus::placeholder()));
    let second = Arc::new(McpService::new(KernelStatus::placeholder()));
    let state = ReloadableSseState::new(first.clone(), Arc::new(BearerAuth::no_auth()));

    state.replace_service(second.clone());

    assert!(Arc::ptr_eq(&state.mcp(), &second));
}
```

Run:

```bash
cargo test -p ck-transport-sse reloadable_sse_state_swaps_service_and_auth
```

Expected: fail because reloadable SSE state does not exist.

**Step 2: Implement reloadable state**

Replace immutable `SseState { mcp, auth }` internals with a reloadable holder:
- `Arc<RwLock<Arc<McpService>>>`
- `Arc<RwLock<Arc<dyn AuthPort>>>`

Keep handler code simple:
- read current MCP service
- read current auth policy
- validate token
- call `call_tool_authorized`

**Step 3: Wire binary reload**

In `crates/ck-bin/src/main.rs`, when config reload rebuilds app/auth:
- update stdio `service`
- update SSE shared state if SSE is enabled
- log reload outcome

**Step 4: Verify**

Run:

```bash
cargo test -p ck-transport-sse
cargo test -p ck-bin
```

Expected: all pass.

## Task 5: Implement Artifact Payload File GC

**Files:**
- Modify: `crates/ck-core/src/ports/artifact.rs`
- Modify: `crates/ck-artifacts/src/lib.rs`
- Modify: `crates/ck-storage-sqlite/src/lib.rs`
- Modify: `crates/ck-app/src/lib.rs`
- Test: `crates/ck-artifacts/src/lib.rs`, `crates/ck-app/src/lib.rs`

**Step 1: Write failing artifact-store test**

Add:

```rust
#[test]
fn delete_artifact_removes_payload_file() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileArtifactStore::new(dir.path());
    store.store_artifact("art_1", Some("sess"), "note", Some("text/plain"), b"hello").unwrap();

    assert!(store.read_artifact_content("art_1").unwrap().is_some());

    store.delete_artifact_payload("art_1").unwrap();

    assert!(store.read_artifact_content("art_1").unwrap().is_none());
}
```

Run:

```bash
cargo test -p ck-artifacts delete_artifact_removes_payload_file
```

Expected: fail because delete API does not exist.

**Step 2: Return artifact IDs from storage GC**

Change storage retention API from deleting artifact rows directly to:
- `list_artifacts_older_than(cutoff) -> Vec<Artifact>`
- `delete_artifacts_by_id(ids) -> u64`

This lets the app delete payload files before metadata rows.

**Step 3: Implement app-level order**

In `AppServices::run_retention_gc`:
1. list expired artifacts
2. delete each payload through `ArtifactPort`
3. delete metadata rows
4. include payload deletion failures in the report

Do not delete metadata when payload deletion fails unless policy explicitly says to force cleanup.

**Step 4: Verify**

Run:

```bash
cargo test -p ck-artifacts
cargo test -p ck-storage-sqlite
cargo test -p ck-app
```

Expected: all pass.

## Task 6: Enforce Source Scope And Corpus Visibility

**Files:**
- Modify: `crates/ck-core/src/ports/auth.rs`
- Modify: `crates/ck-app/src/lib.rs`
- Modify: `crates/ck-mcp/src/lib.rs`
- Modify: `crates/ck-auth/src/lib.rs`
- Test: `crates/ck-app/src/lib.rs`, `crates/ck-mcp/src/lib.rs`

**Step 1: Write failing app test**

Add a test where `query_reference` requests a private corpus not in the allowed scope and returns no results or an authorization error, depending on API decision:

```rust
#[test]
fn query_reference_rejects_private_corpus_outside_source_scope() {
    let app = test_app_with_private_corpus("private_docs");

    let err = app.query_reference(QueryReferenceRequest {
        canonical_id: Some("private_docs".to_string()),
        query: "secret".to_string(),
        mode: None,
        top_k: 5,
        version: None,
        filters: serde_json::json!({"source_scope":["public_docs"]}),
    }).unwrap_err();

    assert!(err.to_string().contains("unauthorized corpus"));
}
```

Run:

```bash
cargo test -p ck-app query_reference_rejects_private_corpus_outside_source_scope
```

Expected: fail because app does not enforce source scope.

**Step 2: Implement policy**

Define source scope behavior:
- Missing `source_scope` means use authorized corpora for authenticated remote calls, and all local corpora for stdio.
- Explicit `source_scope` limits query/resolve results to those corpora.
- Corpus visibility `public` can be read without corpus allowlist only if the deployment config allows public reads.
- Corpus visibility `private` requires explicit allowlist.

**Step 3: Add storage helpers**

Add methods if needed:
- `get_corpus(corpus_id)`
- existing method is already present; use it before result expansion.

**Step 4: Verify**

Run:

```bash
cargo test -p ck-app
cargo test -p ck-mcp
```

Expected: all pass.

## Task 7: Fix Vector Filtering And Embedding Cleanup

**Files:**
- Modify: `crates/ck-core/src/ports/vector_search.rs`
- Modify: `crates/ck-vector/src/lib.rs`
- Modify: `crates/ck-app/src/lib.rs`
- Test: `crates/ck-vector/src/lib.rs`, `crates/ck-app/src/lib.rs`

**Step 1: Write failing vector tests**

Add tests:

```rust
#[test]
fn search_chunks_filters_by_corpus_id() {
    // insert two embeddings with different corpus IDs
    // query should return only the requested corpus
}

#[test]
fn remove_chunk_embeddings_by_document_deletes_rows() {
    // insert two chunk embeddings for one document
    // remove by document
    // search returns none for that document
}
```

Run:

```bash
cargo test -p ck-vector search_chunks_filters_by_corpus_id remove_chunk_embeddings_by_document_deletes_rows
```

Expected: fail because filtering and document cleanup are partial/no-op.

**Step 2: Add metadata columns or joins**

Choose the simpler implementation:
- Add `corpus_id`, `library_id`, and `document_id` columns to vector tables.
- Add migration or idempotent table upgrade for existing databases.
- Populate metadata when app writes embeddings.

**Step 3: Implement cleanup**

Implement:
- `remove_chunk_embeddings_by_document(document_id)`
- corpus/library/session filters in vector search

**Step 4: Verify**

Run:

```bash
cargo test -p ck-vector
cargo test -p ck-app
```

Expected: all pass.

## Task 8: Harden Web Adapter SSRF With DNS Resolution

**Files:**
- Modify: `crates/ck-adapter-web/src/lib.rs`
- Test: `crates/ck-adapter-web/src/lib.rs`

**Step 1: Write failing SSRF tests**

Add tests using a resolver trait:

```rust
#[test]
fn validate_url_blocks_hostname_that_resolves_to_private_ip() {
    let resolver = StaticResolver::new("docs.example.com", "127.0.0.1");

    let err = validate_url_with_resolver("https://docs.example.com", &[], &resolver).unwrap_err();

    assert!(err.to_string().contains("private ip"));
}
```

Run:

```bash
cargo test -p ck-adapter-web validate_url_blocks_hostname_that_resolves_to_private_ip
```

Expected: fail because validation does not resolve DNS.

**Step 2: Implement resolver port inside adapter**

Add:
- `DnsResolver` trait
- default resolver using `ToSocketAddrs`
- test resolver

Reject hostnames that resolve to:
- loopback
- private IPv4
- link-local
- unspecified
- IPv6 loopback/link-local/unique-local

**Step 3: Verify**

Run:

```bash
cargo test -p ck-adapter-web
```

Expected: all pass.

## Task 9: Improve GitHub Adapter For Large Repositories

**Files:**
- Modify: `crates/ck-adapter-github/src/lib.rs`
- Test: `crates/ck-adapter-github/src/lib.rs`

**Step 1: Write failing tests**

Add tests for paginated/static fallback behavior:

```rust
#[test]
fn discover_units_reports_truncated_tree_error() {
    let response = GithubTreeResponse { truncated: true, tree: vec![] };

    let err = units_from_tree_response(response, &config()).unwrap_err();

    assert!(err.to_string().contains("truncated"));
}
```

Run:

```bash
cargo test -p ck-adapter-github discover_units_reports_truncated_tree_error
```

Expected: fail if truncated responses are silently accepted.

**Step 2: Implement behavior**

For large repos:
- detect GitHub tree API `truncated = true`
- return a clear error that tells users to configure `files`
- keep static file list support as the reliable production path

Do not add a complex crawler unless tests prove it is needed.

**Step 3: Verify**

Run:

```bash
cargo test -p ck-adapter-github
```

Expected: all pass.

## Task 10: Add Web Incremental Refresh Hints

**Files:**
- Modify: `crates/ck-adapter-web/src/lib.rs`
- Modify: `crates/ck-app/src/lib.rs`
- Modify: `crates/ck-storage-sqlite/src/lib.rs`
- Test: `crates/ck-adapter-web/src/lib.rs`, `crates/ck-app/src/lib.rs`

**Step 1: Write failing tests**

Add tests that parse/store ETag and Last-Modified:

```rust
#[test]
fn fetch_content_records_etag_and_last_modified() {
    let raw = raw_content_from_response(headers, body);

    assert_eq!(raw.metadata_json["etag"], "\"abc\"");
    assert_eq!(raw.metadata_json["last_modified"], "Tue, 02 Jun 2026 10:00:00 GMT");
}
```

Expected: fail if metadata is not captured.

**Step 2: Implement metadata capture**

Store:
- `etag`
- `last_modified`
- `fetched_at`

Use existing `metadata_json` fields. Avoid schema changes unless conditional fetch requires them.

**Step 3: Add conditional request support**

On refresh, pass previous document metadata into adapter config or source metadata so web fetches can send:
- `If-None-Match`
- `If-Modified-Since`

Represent unchanged content clearly and do not re-index unchanged documents.

**Step 4: Verify**

Run:

```bash
cargo test -p ck-adapter-web
cargo test -p ck-app
```

Expected: all pass.

## Task 11: Create `ck-memory` Crate For Session Policy

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/ck-memory/Cargo.toml`
- Create: `crates/ck-memory/src/lib.rs`
- Modify: `crates/ck-app/src/lib.rs`
- Test: `crates/ck-memory/src/lib.rs`

**Step 1: Write failing tests**

Add tests:

```rust
#[test]
fn retention_policy_preserves_pinned_events() {
    let policy = SessionRetentionPolicy::default();

    assert!(policy.should_preserve_event(true, "2026-01-01T00:00:00Z"));
}

#[test]
fn event_kind_classifies_decision_as_pinnable() {
    assert_eq!(classify_event_kind("decision"), EventRetentionClass::Pinnable);
}
```

Run:

```bash
cargo test -p ck-memory
```

Expected: fail because crate does not exist.

**Step 2: Implement memory policy**

Move session policy helpers out of `ck-app`:
- event kind classification
- pinned event preservation
- retention class
- recall scope helpers

Do not move SQLite persistence into `ck-memory`.

**Step 3: Wire app**

Use `ck-memory` helpers in:
- `store_context`
- `recall_context`
- retention GC policy construction

**Step 4: Verify**

Run:

```bash
cargo test -p ck-memory
cargo test -p ck-app
```

Expected: all pass.

## Task 12: Decide And Implement Context7 Dataset Adapter Or Remove From Release Scope

**Files:**
- Option A Create: `crates/ck-adapter-context7-dataset/Cargo.toml`
- Option A Create: `crates/ck-adapter-context7-dataset/src/lib.rs`
- Option A Modify: `Cargo.toml`, `crates/ck-bin/src/main.rs`
- Option B Modify: `.agent/project_state.md`, docs
- Test: adapter unit tests or docs evidence

**Step 1: Make release decision**

Pick one:
- Implement a local static dataset adapter if compatibility requires bundled Context7-style docs.
- Remove it from deployment readiness and document Context7 compatibility as alias/tool-contract compatibility only.

Recommended: remove from release scope unless a real dataset source exists in the repo.

**Step 2A: If implementing**

Write tests for local JSON/JSONL dataset loading, normalization, and source adapter behavior.

**Step 2B: If removing**

Update docs and `.agent/open_questions.md` to say Context7 dataset adapter is not part of v1 deployment.

**Step 3: Verify**

Run:

```bash
cargo test --workspace
```

Expected: all pass.

## Task 13: Decide And Implement PDF Adapter Or Remove From Release Scope

**Files:**
- Option A Create: `crates/ck-adapter-pdf/Cargo.toml`
- Option A Create: `crates/ck-adapter-pdf/src/lib.rs`
- Option B Modify: docs and `.agent/project_state.md`
- Test: adapter tests or docs evidence

**Step 1: Make release decision**

Recommended: remove PDF adapter from v1 deployment scope unless the release explicitly needs PDF ingestion. Filesystem, GitHub, and web adapters already cover the primary documentation use case.

**Step 2A: If implementing**

Use a proven PDF extraction crate. Tests must cover:
- text extraction
- title fallback
- unsupported/encrypted PDF error
- chunking path through app ingestion

**Step 2B: If removing**

Update docs and `.agent/open_questions.md` to list PDF as future work, not a release blocker.

**Step 3: Verify**

Run:

```bash
cargo test --workspace
```

Expected: all pass.

## Task 14: Add Production Config Templates

**Files:**
- Create: `config/context-kernal.example.toml`
- Create: `config/context-kernal.sse.example.toml`
- Create: `config/context-kernal.local.example.toml`
- Modify: `docs/install.md`
- Test: `crates/ck-config/src/lib.rs`

**Step 1: Write failing tests**

Add tests that load every config template:

```rust
#[test]
fn example_configs_parse() {
    for path in [
        "../../config/context-kernal.example.toml",
        "../../config/context-kernal.sse.example.toml",
        "../../config/context-kernal.local.example.toml",
    ] {
        load_config(path).unwrap();
    }
}
```

Run:

```bash
cargo test -p ck-config example_configs_parse
```

Expected: fail because templates do not exist.

**Step 2: Add templates**

Templates must include:
- storage path
- data dir guidance in comments
- retrieval mode
- SSE bind/auth
- auth tokens through env vars
- authorization allowlists
- scheduler
- embeddings disabled by default
- observability endpoint
- retention enabled with sane defaults
- reload enabled only where safe

**Step 3: Verify**

Run:

```bash
cargo test -p ck-config
```

Expected: all pass.

## Task 15: Add End-To-End Deployment Smoke Tests

**Files:**
- Modify: `crates/ck-bin/tests/cli.rs`
- Test: `crates/ck-bin/tests/cli.rs`

**Step 1: Write failing tests**

Add tests:
- release-like migration then doctor
- serve with config template
- retention deletes old artifact payload and metadata
- SSE rejects unauthorized tool/corpus
- `export-schema` output can initialize an empty SQLite database

Example:

```rust
#[test]
fn schema_export_can_initialize_database() {
    let schema = Command::cargo_bin("context-kernal").unwrap()
        .arg("export-schema")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert!(String::from_utf8(schema).unwrap().contains("schema_migrations"));
}
```

**Step 2: Implement missing behavior**

Patch only the behavior needed for failing smoke tests.

**Step 3: Verify**

Run:

```bash
cargo test -p ck-bin
```

Expected: all pass.

## Task 16: Update Operations Documentation

**Files:**
- Create: `docs/operations.md`
- Create: `docs/configuration.md`
- Create: `docs/security.md`
- Modify: `README.md` if present
- Modify: `docs/install.md`
- Modify: `docs/mcp-contracts.md`

**Step 1: Draft docs**

Document:
- install
- first run
- config templates
- SSE auth
- tool/corpus authorization
- retention behavior
- artifact storage layout
- telemetry setup with OTEL collector
- backup/restore for SQLite and artifacts
- upgrade procedure
- known limitations

**Step 2: Verify docs against code**

For each command in docs, run it or cite why it cannot be run locally.

Minimum commands:

```bash
cargo build --release -p ck-bin
target/release/context-kernal export-schema
target/release/context-kernal migrate --database /tmp/context-kernal-smoke.db
target/release/context-kernal doctor --database /tmp/context-kernal-smoke.db
```

Expected: all pass.

## Task 17: Add Release Packaging Workflow

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`
- Create: `docs/release.md`

**Step 1: Write workflow**

Create a release workflow that:
- runs fmt/clippy/tests
- builds Linux release binary
- optionally builds macOS and Windows binaries
- uploads artifacts
- publishes checksums

**Step 2: Add checksum script or shell step**

Use:

```bash
sha256sum target/release/context-kernal > context-kernal-linux-x86_64.sha256
```

**Step 3: Verify locally**

Run:

```bash
cargo build --release -p ck-bin
sha256sum target/release/context-kernal
```

Expected: binary builds and checksum prints.

## Task 18: Final Release Gate And Memory Update

**Files:**
- Modify: `.agent/project_state.md`
- Modify: `.agent/work_log.md`
- Modify: `.agent/evidence.md`
- Modify: `.agent/open_questions.md`
- Modify: `.agent/decisions.md` if architecture changed

**Step 1: Run final verification**

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo build --release -p ck-bin
target/release/context-kernal export-schema
```

Expected: every command exits 0.

**Step 2: Run manual smoke**

Run:

```bash
tmpdir="$(mktemp -d)"
target/release/context-kernal migrate --database "$tmpdir/kernel.db"
target/release/context-kernal doctor --database "$tmpdir/kernel.db"
```

Expected: doctor prints SQLite status with FTS5 enabled.

**Step 3: Update memory**

Record:
- implemented features
- removed-from-v1 features
- test commands and results
- remaining known limitations
- deployment evidence

**Step 4: Completion rule**

Do not call the project deployment-ready unless `.agent/evidence.md` contains code/test/config evidence for every release standard at the top of this plan.
