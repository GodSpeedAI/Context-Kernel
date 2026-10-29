# Troubleshooting & Failure Diagnostics

This operational guide details known failure modes, observable symptoms, underlying root causes, diagnostic evidence, and recovery procedures across Context Kernel.

---

## 1. Quick Diagnostic Checklist

When experiencing issues with Context Kernel:
1. **Run `doctor`**:
   ```bash
   context-kernal doctor --database <path-to-kernel.db>
   ```
   Verifies SQLite file accessibility and tests whether `fts5_enabled=true`.
2. **Inspect Structured Logs**:
   Set `RUST_LOG=debug` or configure `log_format = "text"` in `[observability]` to view span durations and errors on `stderr`.
3. **Rebuild Indexes**:
   If queries return zero results for ingested documents, run:
   ```bash
   context-kernal rebuild-index --database <path-to-kernel.db>
   ```

---

## 2. Failure Modes & Recovery Matrix

### 1. SSE Startup Port Conflict (Bind Failure)
* **Observable Symptom**: Process terminates immediately on launch with non-zero exit code.
* **Log Message**: `ERROR: failed to bind SSE listener on 127.0.0.1:8765: Address already in use (os error 98)`.
* **Subsystem**: `ck-bin` / `ck-transport-sse`.
* **Root Cause**: Another process (or a zombie instance of Context Kernel) is already listening on the configured port.
* **Source Location**: [`crates/ck-bin/src/main.rs:L146`](../crates/ck-bin/src/main.rs#L146).
* **Recovery Path**:
  1. Identify process occupying the port: `lsof -i :8765` or `netstat -ano | findstr :8765`.
  2. Terminate the conflicting process or change `mcp.sse.bind` in your TOML config (or pass `--sse 127.0.0.1:8766`).

---

### 2. Missing Authentication Tokens When Auth Required
* **Observable Symptom**: Service fails to start; exits with error message.
* **Log Message**: `ERROR: SSE auth is required but no auth tokens are configured`.
* **Subsystem**: `ck-bin` / `ck-auth`.
* **Root Cause**: `mcp.sse.require_auth = true` is set in configuration, but the `auth.tokens` list is empty, or the referenced environment variable (e.g. `${CONTEXT_KERNEL_AUTH_TOKEN}`) was not exported in the shell.
* **Source Location**: [`crates/ck-bin/src/main.rs:L113`](../crates/ck-bin/src/main.rs#L113).
* **Recovery Path**:
  Export the token before launching:
  ```bash
  export CONTEXT_KERNEL_AUTH_TOKEN="$(openssl rand -hex 32)"
  ```

---

### 3. FTS5 Search Returns Zero Hits for Ingested Documents
* **Observable Symptom**: Ingestion reports success, but `query_reference` or `recall_context` returns `total_hits: 0`.
* **Subsystem**: `ck-search-fts` / `ck-storage-sqlite`.
* **Root Cause**:
  1. An ungraceful process termination occurred between relational document insertion and FTS index updates (non-transactional write debt).
  2. The query uses characters that were stripped, or is scoped to a corpus that doesn't contain the document.
* **Diagnostic Check**:
  Query the database directly using `sqlite3`:
  ```bash
  sqlite3 kernel.db "SELECT count(*) FROM chunks;"
  sqlite3 kernel.db "SELECT count(*) FROM chunks_fts;"
  ```
  If `chunks` has rows but `chunks_fts` has 0 rows, the index is desynchronized.
* **Recovery Path**:
  Run the index rebuild command:
  ```bash
  context-kernal rebuild-index --database ./data/kernel.db
  ```

---

### 4. Web SSRF Request Blocked (Fail-Closed)
* **Observable Symptom**: `ingest_source` fails with an invalid argument or network error.
* **Log Message**: `ERROR: target IP ... is in a blocked private/loopback range` or `DNS resolution failed for host`.
* **Subsystem**: `ck-adapter-web`.
* **Root Cause**: The requested URL targets an internal loopback (`127.0.0.1`), a private LAN IP (`10.0.0.0/8`, `192.168.0.0/16`, `172.16.0.0/12`), a non-HTTP scheme, or a hostname whose DNS resolves to a private IP.
* **Source Location**: [`crates/ck-adapter-web/src/lib.rs:L160`](../crates/ck-adapter-web/src/lib.rs#L160).
* **Recovery Path**:
  Context Kernel intentionally blocks private IP ranges to prevent SSRF attacks against internal network infrastructure. Ingest only publicly routable HTTP/HTTPS URLs.

---

### 5. `context_required` Domain Identity Rejected
* **Observable Symptom**: Calling the `context_required` tool returns MCP error `invalid_arguments`.
* **Log Message**: `domain_model_hash is not a 64-char lowercase sha256 digest` or `fallback/placeholder pseudo-hash rejected as domain identity`.
* **Subsystem**: `ck-mcp::agentic_capability_loop`.
* **Root Cause**: The caller supplied an invalid hash or attempted to pass the standalone pseudo-hash (`FALLBACK_PSEUDO_HASH = "dc144cbd71a483431301ca1bf32e95c014af4edba8dbcc525f505310e30a107e"`) or an all-zero placeholder. The same ingress gate also rejects a `world_ref` that is not `world:<name>@sha256:<64 lowercase hex>`; a mutable alias such as `world:acme` is refused before retrieval.
* **Source Location**: [`crates/ck-mcp/src/agentic_capability_loop.rs:L51`](../crates/ck-mcp/src/agentic_capability_loop.rs#L51).
* **Recovery Path**: Ensure the initiating agent passes a genuine SHA-256 digest of the canonical domain model.

---

### 6. Artifact Path Traversal Attempt Blocked
* **Observable Symptom**: `store_context` or reading an artifact fails with an error.
* **Log Message**: `artifact_id contains invalid character: '/'` or `artifact_id must not be empty`.
* **Subsystem**: `ck-artifacts`.
* **Root Cause**: The caller supplied an `artifact_id` containing dots (`..`), slashes, or special characters.
* **Source Location**: [`crates/ck-artifacts/src/lib.rs:L34`](../crates/ck-artifacts/src/lib.rs#L34).
* **Recovery Path**: Artifact IDs must strictly contain only ASCII alphanumeric characters, hyphens (`-`), or underscores (`_`). Use generated ULIDs or sanitized identifiers.

---

### 7. SQLite Database Locked (`SQLITE_BUSY`)
* **Observable Symptom**: Intermittent query timeout or failure under high concurrency.
* **Log Message**: `database is locked (error code 5)`.
* **Subsystem**: `ck-storage-sqlite`.
* **Root Cause**: Another external process is holding an exclusive write lock on `kernel.db` (e.g., an external sqlite3 shell running an uncommitted transaction).
* **Recovery Path**:
  1. Ensure WAL mode is active (`PRAGMA journal_mode = WAL`).
  2. Close any open interactive `sqlite3` CLI sessions that hold write transactions against the database file.

---

### 8. `context_required` Returns `corpus_scope_unresolved`

* **Observable Symptom**: `outcome` is `no_context_governed` (or `empty_optional`), `reason` is `corpus_scope_unresolved`, and `access` is either `requires_scope` or `inaccessible` with reason `out_of_scope`.
* **Subsystem**: `ck-app::resolve_context` (data-access gate), surfaced by `ck-mcp`.
* **Root Cause**: The requested corpus is private and the payload carried no `scope_claim` (`requires_scope`), or the claim named only other corpora (`out_of_scope`). The gate runs before retrieval, so zero references is the correct result.
* **Source Location**: [`crates/ck-app/src/context.rs:L253-L296`](../crates/ck-app/src/context.rs#L253-L296).
* **Recovery Path**: Send `scope_claim` including the corpus id, for example `"scope_claim": "core_docs"`. Confirm the transport allowlist also permits the corpus. Do not retry unchanged: a governed no-context result is not a transient failure.

---

### 9. `ingest_context_effects` Rejects an Effect

* **Observable Symptom**: The tool returns `invalid_arguments` with `effects rejected: ...`, or the report shows a non-zero `rejected` count.
* **Subsystem**: `ck-app::context_effect` via `ck-mcp`.
* **Root Cause**: One of the required identity or evidence fields is missing or malformed: no `evidence_ref` (a bare self-report), an `evidence_ref` not in `settlement|evidence|execution|observation|proof`, a `world_ref` that is not a pinned revision, or a `bundle_content_hash` that is not `sha256:<64 lowercase hex>`.
* **Source Location**: [`crates/ck-app/src/context_effect.rs`](../crates/ck-app/src/context_effect.rs).
* **Recovery Path**: Cite the settlement artifact that justifies the effect and copy the exact `envelope_id` and `integrity.content_hash` from the historical bundle. Re-ingesting the same `effect_id` is skipped by design, so correct the record rather than replaying it.

---

### 10. Effect Ledger Unavailable

* **Observable Symptom**: `ingest_context_effects` returns an internal error, and retrieval logs a calibration warning.
* **Subsystem**: `ContextEffectStore` (JSONL ledger).
* **Root Cause**: `CK_CONTEXT_EFFECT_LEDGER` points at an unwritable path, or an existing ledger line is unreadable. Ingest fails closed; retrieval degrades to uncalibrated ranking rather than blocking context.
* **Recovery Path**: Check the path and permissions, repair or rotate the JSONL file, and restart. Calibration resumes on the next request; no stored chunk or historical bundle needs repair.
