# Explanation: Known Limits & Technical Debt

In accordance with Context Kernel's core operating principles (*codebase evidence over documentation claims*), this document provides an honest, repository-grounded inventory of active technical debt, architectural gaps, and known operational limits.

---

## 1. Summary Status Matrix

| Area | Status | Evidence Location | Impact & Reality |
| :--- | :--- | :--- | :--- |
| **GitHub & Web Adapters** | Open Debt | [`crates/ck-bin/src/main.rs:L376-L395`](../../crates/ck-bin/src/main.rs#L376-L395) | Multi-kind routing exists, but `ck-bin` registers only the filesystem and learning-proposals adapters. The GitHub and Web crates compile and pass tests yet are never registered. |
| **FTS ↔ DB Write Atomicity** | Mitigated Residual | [`crates/ck-app/src/lib.rs:L348`](../../crates/ck-app/src/lib.rs#L348) | Relational inserts and FTS indexing use separate SQLite connections; a crash mid-ingest can cause FTS drift (recoverable via `rebuild-index`). |
| **Scheduler Ownership** | Open Debt | [`crates/ck-bin/src/main.rs:L190`](../../crates/ck-bin/src/main.rs#L190) | `ck-scheduler` crate exists and passes tests, but `main.rs` runs a parallel refresh loop instead of composing it. |
| **OpenTelemetry Export** | Unimplemented | [`crates/ck-observability/src/lib.rs:L21`](../../crates/ck-observability/src/lib.rs#L21) | Configuration accepts `otel_endpoint`, but no OTLP collector exporter or metrics backend is wired. |
| **Dead Configuration Fields**| Open Debt | [`crates/ck-config/src/lib.rs`](../../crates/ck-config/src/lib.rs) | `retrieval.semantic_enabled`, `mcp.enable_stdio`, `mcp.sse.base_path`, and `auth.trust_mode` are parsed but ignored. |
| **Broad StoragePort** | Open Debt | [`crates/ck-core/src/ports/storage.rs`](../../crates/ck-core/src/ports/storage.rs) | `StoragePort` defines 26 methods, forcing large mock implementations in tests. |
| **SSE Shutdown / Worker Drain**| Mitigated Residual | [`crates/ck-bin/src/main.rs:L160`](../../crates/ck-bin/src/main.rs#L160) | Axum server runs in a detached thread; on stdin EOF, worker threads die with the process without draining in-flight requests. |

---

## 2. In-Depth Analysis of Architectural Debt

### 1. Unwired Ingestion Adapters (GitHub & Web)
* **What Exists**: `ck-adapter-github` and `ck-adapter-web` are implemented crates with unit coverage (tree truncation detection, base64 decoding, SSRF IP blocking, DNS resolution checks). Kind-based routing also exists: `AppServices` holds `kind_adapters` and resolves them through `adapter_for_kind`, and `ck-bin` registers `learning_proposals_jsonl` that way.
* **What Is Missing**: Registration, not routing. `crates/ck-bin/src/main.rs` still passes `FilesystemSourceAdapter` as the default and registers no GitHub or Web adapter. `ck-bin/Cargo.toml` does not declare dependencies on either crate.
* **Consequence**: Calling `ingest_source` with `kind = "github"` or `kind = "website"` falls back to the default filesystem adapter, which fails because it only understands local paths. The fix is now additive: wire the crates and register each kind.
* **Next Move**: Add the two dependencies and register `GitHubSourceAdapter` and `WebSourceAdapter` through `with_source_adapter_kind` in `wire_app`.

### 2. Non-Transactional FTS ↔ Canonical DB Writes
* **What Exists**: `ck-storage-sqlite` writes to canonical tables (`documents`, `chunks`), while `ck-search-fts` opens its own connection to write to `chunks_fts`.
* **What Is Missing**: Because these are two separate connection instances, document inserts and FTS index updates do not occur within a single atomic SQLite transaction.
* **Consequence**: If the process is killed (SIGKILL / power loss) precisely between document insertion and FTS indexing, the SQLite tables will have canonical documents that are missing from search results.
* **Mitigation**: The operator can run `context-kernal rebuild-index --database <path>`, which executes pure SQL to repopulate all FTS tables directly from canonical rows.

### 3. Scheduler Crate Bypass
* **What Exists**: `ck-scheduler::RefreshScheduler` implements `SchedulerPort` with job coalescing, status tracking, and queue management.
* **What Is Missing**: `main.rs` spawns its own raw thread:
  ```rust
  std::thread::spawn(move || {
      while refresh_running.load(Ordering::Relaxed) {
          std::thread::sleep(Duration::from_secs(interval_secs));
          for source in refresh_store.list_stale_sources()? {
              refresh_app.refresh_source(...);
          }
      }
  });
  ```
* **Consequence**: The background thread bypasses `SchedulerPort` job queueing and status transitions, performing direct synchronous refreshes.

### 4. Parsed-but-Ignored Configuration Options
* Several configuration knobs exist in `ck-config` for forward compatibility but have no effect in the runtime:
  - `retrieval.semantic_enabled`: Overridden by whether `--embeddings` is active and provider is configured.
  - `mcp.enable_stdio`: Stdio is always served; cannot be disabled.
  - `mcp.sse.base_path`: Axum server binds `/mcp` directly; base path is not dynamically routed.
  - `auth.trust_mode`: Stdio is always local trust mode; SSE always checks tokens if `require_auth = true`.

---

## 3. Verified vs. Inferred Design Claims

| Claim | Evidence Status | Evidence Source |
| :--- | :--- | :--- |
| **All SQLite enum codecs round-trip safely** | Verified Fact | Tested in `crates/ck-storage-sqlite/src/lib.rs:L1300` |
| **Constant-time token equality** | Verified Fact | `subtle::ConstantTimeEq` used in `ck-auth/src/lib.rs:L61` |
| **SSRF blocks private IPv4/IPv6** | Verified Fact | Tested in `ck-adapter-web/src/lib.rs:L300` |
| **FTS rebuild recovers index state** | Verified Fact | Implemented in `SqliteStore::rebuild_fts_index` and verified in CLI tests |
| **OpenTelemetry exporter is functional** | Refuted / False Claim | `ck-observability/src/lib.rs:L21` logs warning and ignores endpoint |
| **GitHub adapter usable from CLI** | Refuted / False Claim | Not registered in `main.rs`; kind routing exists |
