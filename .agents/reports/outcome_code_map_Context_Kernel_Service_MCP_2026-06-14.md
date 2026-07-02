# Outcome-to-Code Map: Context Kernel Service MCP

## 1. Executive Summary

| Field | Value |
|-------|-------|
| Repo name | Context_Kernel_Service_MCP (product: "Context Kernel", binary `context-kernal`) |
| Audit date | 2026-06-14 |
| Languages/frameworks | Rust 2021 edition; Cargo workspace (20 crates); rusqlite (bundled SQLite + FTS5); axum/hyper (SSE); tokio; ureq; clap; tracing/tracing-subscriber |
| Main purpose | A single-binary, local-first MCP service providing (a) up-to-date documentation retrieval, (b) durable session context/continuity, and (c) governed reference search — unified behind one MCP tool surface over stdio + SSE, backed by SQLite/FTS5, optional embeddings, and explicit authz boundaries. |
| Outcomes found | 26 |
| Implemented | 23 |
| Partially implemented | 2 (OUT-013 GitHub ingest, OUT-014 Web ingest — adapters exist but are not wired into runtime) |
| Declared only / config only | 1 (OUT-024 OTEL export — spec-REQUIRED, config-only stub) |
| Contradicted (doc-vs-code) | 1 documentation-drift case under OUT-011 (docs/schema.md line 28 vs code; outcome itself is implemented) |
| Top 5 risks | (1) GitHub + Web source adapters are not wired into runtime ingestion despite being claimed as supported; (2) OTEL export is spec-REQUIRED but unwired (config-only); (3) `docs/schema.md` contradicts code on artifact-payload GC; (4) no end-to-end test exercises github/web ingestion, embeddings, or vector search through the binary; (5) single global source adapter/normalizer/chunker makes the "multi-source" architecture unreachable at runtime. |

The README and `.agents/` memory are unusually candid about limits (OTEL, PDF, Context7 dataset, brute-force vectors, SSE rebind). The most material gap — that the GitHub and Web adapters, though implemented and unit-tested as crates, are **not reachable from the running binary's `ingest_source` path** — is not disclosed in the README and is the single highest-risk finding.

## 2. Method

Files/directories inspected directly:
- `README.md`, `AGENTS.md`, `.github/copilot-instructions.md`
- `Context_Kernel_Service_Specification.md` (1496 lines, full)
- `docs/mcp-contracts.md`, `docs/security.md`, `docs/schema.md`, `docs/operations.md`, `docs/configuration.md`, `docs/release.md`, `docs/install.md`, `docs/ci.md`
- `.agents/project_state.md`, `.agents/evidence.md`, `.agents/open_questions.md`
- `Cargo.toml` (workspace), all 21 crate `Cargo.toml`s (via glob)
- `crates/ck-bin/src/main.rs` (full bootstrap wiring, 356 lines)
- `crates/ck-app/src/lib.rs` (AppServices, ingest_source, retention GC, lines 1-130, 320-439)
- `crates/ck-mcp/src/lib.rs` (tool dispatch, authz) via grep
- `crates/ck-core/src/ports/*` (12 port traits via grep)
- `crates/ck-observability/src/lib.rs` (OTEL gap) via grep
- `crates/ck-storage-sqlite/src/lib.rs` (SCHEMA_V1, migrations) via grep
- `crates/ck-artifacts/src/lib.rs` (validate_id path-traversal) via grep
- `crates/ck-config/src/lib.rs` (env interpolation) via grep
- `crates/ck-transport-sse/src/lib.rs` (routes, authz) via grep
- `crates/ck-bin/tests/cli.rs` (full, 530 lines — all 16 integration tests)
- `.github/workflows/ci.yml`, `.github/workflows/release.yml`
- `config/*.toml`, `context-kernal.toml.example`

Commands/tools used: `ls`, `git log`, `find`, glob, grep (content mode), Read (file + offset), ast-grep available but not required (direct reads sufficed). Four parallel `explore` background agents were launched for broad mapping but were cancelled after 8+ minutes without returning; all findings below are grounded in direct reads/greps performed by the auditor.

Limitations:
- Test counts and pass/fail are taken from `.agents/evidence.md` claims (e.g. "186 tests pass"); tests were not re-executed during this audit. The audit verifies *that tests exist and what they assert*, not that they currently pass.
- The `agentic_capability_loop` integration (latest commit, `crates/ck-mcp/src/agentic_capability_loop.rs` + `integration/agentic_capability_loop/adapters.py`) references an external `SEA` domain framework not present in this repo; it is out of scope for the core Context Kernel outcomes and noted only as an ownership-boundary concern.

## 3. Outcome Inventory

| ID | Outcome | Actor/User | Status | Confidence | Primary Evidence |
| -- | ------- | ---------- | ------ | ---------- | ---------------- |
| OUT-001 | Ship one Rust binary exposing the MCP service with `serve`/`migrate`/`doctor`/`export-schema` | Operator / CI | implemented | high | `crates/ck-bin/src/main.rs::main`, `Commands` enum |
| OUT-002 | Initialize a versioned SQLite schema (12 tables + 3 FTS5 virtual tables) via migration | Operator | implemented | high | `crates/ck-storage-sqlite/src/lib.rs::SCHEMA_V1`, `migrate` |
| OUT-003 | Serve MCP over stdio JSON-RPC in local-trust mode | Local agent / IDE | implemented | high | `crates/ck-transport-stdio`, `main.rs` stdin loop, `cli.rs::serve_stdio_answers_get_status_tool_call` |
| OUT-004 | Serve MCP over SSE HTTP with `/healthz`,`/readyz`,`/api/v1/status` | Remote client / operator | implemented | high | `crates/ck-transport-sse/src/lib.rs::serve_sse`, `cli.rs::sse_healthz_endpoint_works` |
| OUT-005 | Require + validate bearer tokens for SSE (fail-closed when none configured) | Operator / security | implemented | high | `crates/ck-auth/src/lib.rs::BearerAuth`, `main.rs:115-118` fail-fast |
| OUT-006 | Authorize SSE tool calls by tool-name and corpus allowlists | Operator / security | implemented | high | `call_tool_authorized`, `BearerAuth::with_policy` |
| OUT-007 | Index and search document chunks, session events, and library aliases with FTS5/BM25 | Agent / retrieval | implemented | high | `crates/ck-search-fts/src/lib.rs::FtsSearch`, `chunks_fts`/`session_events_fts`/`library_aliases_fts` |
| OUT-008 | Hybrid-rank results combining lexical + freshness + trust + intent + session-affinity + dedup + optional semantic, with score breakdown | Agent / retrieval | implemented | high | `crates/ck-ranker/src/lib.rs::LexicalRanker` (intent profiles, affinity, dedup, debug) |
| OUT-009 | Persist and recall session context across tool runs (store_context / recall_context) | Agent / continuity | implemented | high | `SessionMemoryPort`, `handle_store_context`/`handle_recall_context`, `cli.rs` store/recall tests |
| OUT-010 | Store large payloads as out-of-prompt artifacts (filesystem + SQLite metadata, SHA-256) | Agent / continuity | implemented | high | `crates/ck-artifacts/src/lib.rs::FileArtifactStore` |
| OUT-011 | Run retention GC over events/sessions/jobs/artifacts(metadata+payload)/disabled-sources; pinned events survive | Operator | implemented | high | `crates/ck-app/src/lib.rs::run_retention_gc`, `delete_artifact_payload`, `cli.rs::retention_deletes_old_artifact_payload` |
| OUT-012 | Ingest documentation from the local filesystem tree (markdown normalizer + heading chunker) | Operator / agent | implemented | high | `wire_app` injects `FilesystemSourceAdapter`, `cli.rs::ingest_source_indexes_local_docs` |
| OUT-013 | Ingest documentation from GitHub repositories (tree API + static file list, truncated-tree detection) | Operator / agent | **partially_implemented** | high | `crates/ck-adapter-github/src/lib.rs::GitHubSourceAdapter` (crate + 18 tests) — **NOT wired into `wire_app`/`ingest_source` dispatch; unreachable from binary** |
| OUT-014 | Ingest documentation from websites (HTTP fetch, conditional headers, HTML→text) | Operator / agent | **partially_implemented** | high | `crates/ck-adapter-web/src/lib.rs::WebSourceAdapter` (crate + 34 tests) — **NOT wired into `wire_app`/`ingest_source`; unreachable from binary** |
| OUT-015 | Block SSRF on web fetches (private IPs, DNS resolution, scheme + allowed_domains enforcement) | Security | implemented (library-level) | high | `validate_url_with_resolver`, `DnsResolver`/`SystemDnsResolver`/`StaticResolver`, 12 SSRF tests — reachable only through the unwired web adapter |
| OUT-016 | Detect and reject truncated GitHub tree responses instead of silently indexing partial data | Operator / data-integrity | implemented (library-level) | high | `units_from_tree_response` errors on `truncated:true` — reachable only through the unwired github adapter |
| OUT-017 | Optionally enable OpenAI-compatible embeddings with graceful Noop fallback | Operator | implemented | high | `crates/ck-embeddings` `OpenAIEmbeddingProvider`/`NoopEmbeddingProvider`, `wire_app` conditional wiring |
| OUT-018 | Provide vector similarity search scoped by corpus/library/document metadata | Agent / retrieval | implemented (small-corpus brute-force) | high | `crates/ck-vector/src/lib.rs::SqliteVectorStore` (cosine, filtering) |
| OUT-019 | Support one-time / manual / scheduled source refresh | Operator / agent | implemented | high | `ingest_source`, `refresh_source`, periodic-refresh thread `main.rs:160-215` |
| OUT-020 | Resolve a library/package/repo name into canonical IDs (resolve_reference + resolve-library-id alias) | Agent / retrieval | implemented | high | `handle_resolve_reference`, `cli.rs::resolve_library_id_alias_works` |
| OUT-021 | Retrieve ranked documentation excerpts with citations (query_reference + get-library-docs/query-docs aliases) | Agent / retrieval | implemented | high | `handle_query_reference`, `cli.rs::query_reference_includes_citations_after_ingest` |
| OUT-022 | Report service/storage/sources/sessions/jobs/retrieval health via `get_status` | Operator / client | implemented | high | `handle_get_status`, `cli.rs::serve_stdio_answers_get_status_tool_call` |
| OUT-023 | Hot-reload config (sources/retrieval/auth/retention/observability) without restart, except the SSE bind socket | Operator | implemented | high | `main.rs` reload loop, `ReloadableSseState`; bind limitation documented |
| OUT-024 | Export OpenTelemetry-compatible observability (traces/metrics/spans) | Operator / SRE | **declared_only (config_only)** | high | Spec §26.1/§20.2 REQUIRES it; `ck-observability/src/lib.rs:23-26` logs "not yet supported; ignoring"; `otel_endpoint` accepted but unused |
| OUT-025 | Preserve ports-and-adapters isolation (no SQL/HTTP/transport in the domain core) | Architect / maintainer | implemented | high | 12 port traits in `ck-core/src/ports/*`; `ck-core` has no rusqlite/axum/ureq deps (minor: Config/MCP-Transport/Observability ports are not abstract traits) |
| OUT-026 | Prevent artifact path-traversal and keep secrets out of logs; support `$VAR`/`${VAR}` indirection | Security | implemented | high | `validate_id` rejects `../`, `/`, `\`; `expand_env_vars`; constant-time token compare |

## 4. Detailed Outcome Maps

### OUT-001 — Single-binary MCP service

**Outcome statement:** Building `ck-bin` yields one `context-kernal` executable that runs the entire MCP service (serve, migrate, doctor, export-schema) with no external daemons beyond SQLite files and optional embedding HTTP.

**Actor/User:** Operator / CI pipeline.

**Settlement / success criterion:** `cargo build --release -p ck-bin` produces a binary that responds to all four subcommands and serves MCP over stdio.

**Status:** implemented — **Confidence:** high

**Declared evidence:**
- `README.md::Quick Start` — documents the four commands.
- `Context_Kernel_Service_Specification.md::§26.1` — "single Rust binary" REQUIRED.

**Implementation evidence:**
- `crates/ck-bin/src/main.rs::main` (lines 50-68) — dispatches `Serve|Migrate|Doctor|ExportSchema`.
- `Cargo.toml` workspace lists `ck-bin`; binary name `context-kernal` (`ck-bin/Cargo.toml`).

**Test evidence:**
- `crates/ck-bin/tests/cli.rs::migrate_creates_sqlite_database`, `export_schema_prints_sqlite_schema`, `serve_stdio_answers_get_status_tool_call`.

**Runtime path:** `main()` → `init_observability` → `Cli::parse` → `serve(args)` → `open_store` → `store.migrate()` → `wire_app` → stdin loop / SSE thread.

**Missing links / gaps:** None material. Naming inconsistency: config/service name uses "context-**kernel**" while binary uses "context-**kernal**" (cosmetic; tests assert the config name).

**Reviewer notes:** Confirm `cargo build --release -p ck-bin` succeeds in CI; the release workflow runs it (`.github/workflows/release.yml:42`).

---

### OUT-002 — Versioned SQLite schema + migrations

**Outcome statement:** The service creates all required metadata, document/chunk, session-event, artifact, refresh-job, and FTS5 tables on startup via a versioned migration, and can emit the embedded schema SQL.

**Actor/User:** Operator.

**Settlement / success criterion:** `context-kernal migrate` produces a DB with `schema_migrations`, `sources`, `corpora`, ..., `chunks_fts`; `export-schema` prints the same SQL.

**Status:** implemented — **Confidence:** high

**Declared evidence:** `docs/schema.md`; `README.md::Status`.

**Implementation evidence:**
- `crates/ck-storage-sqlite/src/lib.rs::SCHEMA_V1` (lines 1053-1255) — 12 `CREATE TABLE` + 3 `CREATE VIRTUAL TABLE ... USING fts5`.
- `SqliteStore::migrate` (line 44) runs `execute_batch(SCHEMA_V1)`.

**Test evidence:** `cli.rs::export_schema_prints_sqlite_schema`, `cli.rs::schema_export_can_initialize_database`, `cli.rs::migrate_then_doctor`.

**Runtime path:** `serve()`/`migrate()` → `SqliteStore::open` → `store.migrate()`.

**Missing links / gaps:** None. (Schema is "version 1"; no v2 migration path exists yet, but none is required today.)

---

### OUT-003 — stdio MCP serving

**Outcome statement:** A local agent/IDE can drive the service over JSON-RPC on stdin/stdout without authentication (process boundary = trust boundary).

**Actor/User:** Local agent / IDE integration.

**Settlement / success criterion:** A `tools/call` JSON-RPC line on stdin returns a well-formed result on stdout.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-transport-stdio/src/lib.rs::handle_line`; `main.rs:268` `println!(... handle_line(...))`.

**Test evidence:** `cli.rs::serve_stdio_answers_get_status_tool_call` (asserts `"name":"context-kernel"` and `fts5_enabled:true`).

**Missing links / gaps:** None.

---

### OUT-004 — SSE MCP serving

**Outcome statement:** A remote MCP client can reach the service over HTTP (`POST /mcp`) and observe liveness/readiness.

**Actor/User:** Remote client / operator.

**Settlement / success criterion:** With `--sse 127.0.0.1:0` the binary starts an HTTP server exposing `/healthz`, `/readyz`, and serves stdio concurrently.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-transport-sse/src/lib.rs::serve_sse` (axum router, lines 45-63); `main.rs:135-156` spawns the SSE thread.

**Test evidence:** `cli.rs::sse_healthz_endpoint_works`.

**Missing links / gaps:** Config reload does not rebind the listener (documented limit in `README.md::Status` and `.agents/open_questions.md`). Acceptable for v1.

---

### OUT-005 — SSE bearer-token auth (fail-closed)

**Outcome statement:** When SSE auth is required, requests without a valid constant-time-equal bearer token are rejected; if auth is required but no tokens are configured, the service refuses to start.

**Actor/User:** Operator / security.

**Settlement / success criterion:** Missing/empty token config with `require_auth=true` aborts startup; valid token passes; invalid token is denied.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-auth/src/lib.rs::BearerAuth` (constant-time compare via `subtle`); `main.rs:115-118` `anyhow::bail!` on empty tokens.

**Test evidence:** `ck-auth` tests `empty_tokens_fail_closed_without_local_trust_mode` (per `.agents/evidence.md`).

**Missing links / gaps:** None.

---

### OUT-006 — Tool + corpus authorization (SSE)

**Outcome statement:** SSE tool calls are authorized against configured tool-name and corpus allowlists before dispatch.

**Actor/User:** Operator / security.

**Settlement / success criterion:** A tool/corpus not in the allowlist returns an authz error even with a valid bearer token.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-mcp/src/lib.rs::call_tool_authorized`; `BearerAuth::with_policy(allow_tools, allow_corpora)`; SSE handler `crates/ck-transport-sse/src/lib.rs:137` calls `call_tool_authorized`.

**Test evidence:** `ck-mcp` test at `lib.rs:493` (`token == "allowed" && matches!(tool_name, "get_status"|"query_reference")`).

**Missing links / gaps:** Stdio path uses direct `call_tool` (local-trust bypass) — this is by design per `docs/security.md` and `docs/mcp-contracts.md:38`, not a defect, but worth noting that authz is SSE-only.

---

### OUT-007 — FTS5/BM25 lexical retrieval

**Outcome statement:** Indexed chunks, session events, and library aliases are searchable with BM25-style ranking.

**Actor/User:** Agent / retrieval.

**Settlement / success criterion:** After ingest, a `query_reference`/`recall_context` query returns BM25-ranked hits from the FTS5 tables.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-search-fts/src/lib.rs::FtsSearch`; `chunks_fts`, `session_events_fts`, `library_aliases_fts` in `SCHEMA_V1`.

**Test evidence:** `ck-search-fts` 5 tests incl. `bm25_score_positivity` (per evidence log); `cli.rs::query_reference_returns_results_after_ingest`.

**Missing links / gaps:** None.

---

### OUT-008 — Hybrid ranking with transparency

**Outcome statement:** Retrieved candidates are ranked by a weighted combination of lexical score, freshness, trust, intent-specific profiles, session affinity, document-level dedup, and (when configured) semantic score, with an optional score breakdown for debugging.

**Actor/User:** Agent / retrieval / evaluator.

**Settlement / success criterion:** Different intents reorder results; same-document chunks collapse; debug mode emits a breakdown.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-ranker/src/lib.rs::LexicalRanker` (intent weight profiles, session affinity, dedup-by-document_id, debug breakdown, semantic integration).

**Test evidence:** `ck-ranker` tests `intent_weighting_changes_ranking_order`, `session_affinity_boosts_matching_session_items`, `deduplication_collapses_same_document_chunks`, `debug_mode_includes_detailed_breakdown`, `semantic_score_included_in_weighted_combination` (per evidence log).

**Missing links / gaps:** None.

---

### OUT-009 — Session memory persistence + recall

**Outcome statement:** An agent can store decisions/notes/tool-results as session events and later recall the relevant ones by query, surviving across tool calls (and, with retention, across restarts).

**Actor/User:** Agent / continuity.

**Settlement / success criterion:** `store_context` returns an `event_id`; a subsequent `recall_context` for related text returns that event ranked.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `SessionMemoryPort` (`ck-core`); `SqliteStore` impl; `handle_store_context`/`handle_recall_context` (`ck-mcp`).

**Test evidence:** `cli.rs::store_context_persists_event`, `cli.rs::recall_context_returns_stored_events`.

**Missing links / gaps:** None. (Per `.agents/open_questions.md`, sync rusqlite is an accepted v1 choice.)

---

### OUT-010 — Out-of-prompt artifact storage

**Outcome statement:** Large payloads are written to the filesystem artifact store and referenced from session events rather than inlined into the model context.

**Actor/User:** Agent / continuity / token-budget.

**Settlement / success criterion:** Storing a base64 `artifact_ref` yields a filesystem file + metadata row; the event references the artifact id.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-artifacts/src/lib.rs::FileArtifactStore` (SHA-256 checksums, `data_dir/artifacts/`).

**Test evidence:** `ck-artifacts` 3 tests (store+read, checksum verify, missing→None).

**Missing links / gaps:** None.

---

### OUT-011 — Retention GC (incl. artifact payload files)

**Outcome statement:** On startup (when `[retention].enabled=true`), the service deletes unpinned old session events, inactive sessions, finished refresh jobs, artifact metadata **and payload files**, and long-disabled sources; pinned events always survive.

**Actor/User:** Operator.

**Settlement / success criterion:** After GC with `artifact_days=0`, the artifact payload file is removed and the event metadata is gone; a pinned event remains.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `crates/ck-app/src/lib.rs::run_retention_gc` (lines 50-95) calls `storage.list_artifacts_older_than` then `artifact_store.delete_artifact_payload` per id, plus metadata deletes; `main.rs:88-104` invokes it.

**Test evidence:** `cli.rs::retention_deletes_old_artifact_payload`; `ck-app` tests preserve pinned events.

**Missing links / gaps:**
- **contradicted (documentation drift):** `docs/schema.md:28` states "Retention GC does not delete artifact payload files yet; it removes SQLite metadata only." The code (`run_retention_gc` + `delete_artifact_payload` + the `retention_deletes_old_artifact_payload` test) contradicts this. `docs/operations.md:51` correctly states payload files ARE deleted. **`docs/schema.md` is stale and should be updated.**

**Reviewer notes:** Update `docs/schema.md` line 28 to match `docs/operations.md` and the code.

---

### OUT-012 — Filesystem source ingestion

**Outcome statement:** An operator can point `ingest_source` at a local directory tree and have its markdown files normalized, heading-chunked, and indexed into FTS5.

**Actor/User:** Operator / agent.

**Settlement / success criterion:** `ingest_source` with `kind:"filesystem_tree"` and a `path` returns a completed job; subsequent queries hit the indexed docs.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `wire_app` (`main.rs:321`) injects `FilesystemSourceAdapter`, `MarkdownNormalizer`, `HeadingChunker` into `AppServices`; `ingest_source` (`ck-app/src/lib.rs:329`) discovers/fetches/chunks/indexes.

**Test evidence:** `cli.rs::ingest_source_indexes_local_docs`, `query_reference_returns_results_after_ingest`, `refresh_source_tool_works`.

**Missing links / gaps:** None for filesystem.

---

### OUT-013 — GitHub source ingestion  ⚠ HIGHEST-RISK GAP

**Outcome statement:** An operator can ingest documentation from a GitHub repository (via tree API or a static file list, with token auth) and have it indexed like any other source.

**Actor/User:** Operator / agent.

**Settlement / success criterion:** `ingest_source` with `kind:"github_repo"` discovers units via the GitHub adapter and indexes them through the running binary.

**Status:** **partially_implemented** — **Confidence:** high

**Declared evidence:**
- `README.md::Source Adapters` — "Context Kernel currently supports: ... GitHub repositories".
- `README.md::Status` — "local filesystem, GitHub, and web source adapters" listed under Implemented.
- `.agents/project_state.md` — `ck-adapter-github: implemented`.

**Implementation evidence (library only):**
- `crates/ck-adapter-github/src/lib.rs::GitHubSourceAdapter` implements `SourceAdapter` (`kind()=="github_repo"`), 18 unit tests, tree API + static file list, base64 decode, `$ENV` token syntax, truncated-tree detection.

**Integration wiring evidence (MISSING):**
- `crates/ck-bin/src/main.rs::wire_app` (lines 308-356) injects **only** `FilesystemSourceAdapter` as `AppServices.source_adapter`. There is no source registry, no `match source_kind` dispatch.
- `crates/ck-app/src/lib.rs::ingest_source` (lines 369-381) always calls `self.source_adapter.discover_units` / `fetch_content` — the single injected adapter — regardless of `req.source_kind`. `source_kind` is stored on the `Source` row but never selects an adapter.
- All 4 `ingest_source` integration tests (`cli.rs`) use `"kind":"filesystem_tree"`. **No end-to-end test exercises `github_repo`.**

**Runtime path:** `ingest_source({kind:"github_repo"})` → `self.source_adapter` (which is `FilesystemSourceAdapter`) → filesystem discovery on a config that isn't a filesystem path → failure or wrong behavior. The GitHub adapter is unreachable.

**Missing links / gaps:**
- `missing integration wiring`: no source-kind → adapter dispatch in `wire_app`/`AppServices`.
- `overclaimed maturity`: README + project_state list GitHub as supported/implemented without disclosing the wiring gap.
- `missing test coverage`: no binary-level test ingests a GitHub source.

**Reviewer notes:** This is the single most important finding. A frontier reviewer should confirm by (a) reading `wire_app` and the `AppServices` struct (single `source_adapter` field), and (b) attempting `ingest_source` with `kind:"github_repo"` against the built binary. Either a source registry/dispatch layer must be added, or the README/project_state must downgrade GitHub to "library-implemented, not yet wired".

---

### OUT-014 — Website source ingestion  ⚠ HIGH-RISK GAP

**Outcome statement:** An operator can ingest documentation from a list of website URLs and have the HTML normalized to text and indexed.

**Actor/User:** Operator / agent.

**Settlement / success criterion:** `ingest_source` with `kind:"website"` fetches, converts HTML→text, and indexes through the running binary.

**Status:** **partially_implemented** — **Confidence:** high

**Declared evidence:** `README.md::Source Adapters` ("websites"); `README.md::Status`.

**Implementation evidence (library only):** `crates/ck-adapter-web/src/lib.rs::WebSourceAdapter` (`kind()=="website"`), 34 unit tests incl. HTML→text, conditional request headers, SSRF.

**Integration wiring evidence (MISSING):** Same as OUT-013 — `wire_app` injects only `FilesystemSourceAdapter`; `WebSourceAdapter`/`WebNormalizer` are not wired; no end-to-end web-ingest test.

**Missing links / gaps:** identical structural gap to OUT-013.

**Reviewer notes:** Same remediation as OUT-013. Note the SSRF protections (OUT-015) are only reachable once this adapter is wired.

---

### OUT-015 — SSRF protection on web fetches

**Outcome statement:** Web ingestion blocks private IP ranges, non-HTTP schemes, localhost-style hosts, and DNS names resolving to private addresses, and honors `allowed_domains`.

**Actor/User:** Security.

**Settlement / success criterion:** A fetch to `http://127.0.0.1`, `http://169.254.x.x`, `file://`, or a public host resolving to a private IP is rejected.

**Status:** implemented (library-level) — **Confidence:** high

**Implementation evidence:** `crates/ck-adapter-web/src/lib.rs::validate_url_with_resolver`, `DnsResolver`/`SystemDnsResolver`/`StaticResolver`.

**Test evidence:** 12 SSRF-specific tests (private IPv4, IPv6, scheme, localhost, allowed_domains) per evidence log.

**Missing links / gaps:** Reachable only through the unwired web adapter (OUT-014). Until wiring lands, this protection has no production code path.

---

### OUT-016 — GitHub truncated-tree detection

**Outcome statement:** When GitHub's tree API returns a truncated response, the adapter errors rather than silently indexing partial data.

**Actor/User:** Data-integrity.

**Settlement / success criterion:** A `truncated:true` tree response yields an ingest error, not a partial index.

**Status:** implemented (library-level) — **Confidence:** high

**Implementation evidence:** `crates/ck-adapter-github/src/lib.rs::units_from_tree_response` returns `Err` on `truncated:true`; test `errors_on_truncated_tree`.

**Missing links / gaps:** Reachable only through the unwired github adapter (OUT-013).

---

### OUT-017 — Optional OpenAI-compatible embeddings

**Outcome statement:** When `[embeddings]` is configured with an OpenAI-compatible provider, the service generates embeddings and merges semantic results; otherwise it degrades cleanly to lexical/hybrid-without-semantic.

**Actor/User:** Operator / retrieval.

**Settlement / success criterion:** With embeddings off, `get_status` reports `semantic_enabled:false` and queries still succeed; with embeddings on, vector results merge into ranking.

**Status:** implemented — **Confidence:** high (library-level wiring present; no binary-level embedding E2E test)

**Implementation evidence:** `ck-embeddings` `OpenAIEmbeddingProvider` (ureq sync HTTP) + `NoopEmbeddingProvider`; `wire_app` (`main.rs:327-353`) conditionally wires embedding + `SqliteVectorStore`.

**Test evidence:** `ck-embeddings` 5 tests, `ck-vector` 14 tests (per evidence log). No `cli.rs` test enables embeddings end-to-end.

**Missing links / gaps:** `missing test coverage`: no integration test exercises the real OpenAI HTTP path or the hybrid merge through the binary (reasonable, since it needs network/credentials).

---

### OUT-018 — Vector search with scope metadata

**Outcome statement:** Vector similarity search can be filtered by corpus/library/document so semantic retrieval respects retrieval scope.

**Actor/User:** Retrieval.

**Settlement / success criterion:** `search_chunks` with a corpus filter returns only vectors in that corpus.

**Status:** implemented (brute-force cosine; documented small-corpus limit) — **Confidence:** high

**Implementation evidence:** `crates/ck-vector/src/lib.rs::SqliteVectorStore` — `chunk_embeddings(corpus_id, library_id, document_id)`, `search_chunks`, `remove_chunk_embeddings_by_document`.

**Test evidence:** `ck-vector` 14 tests incl. `upsert_and_search`, model filtering, limit enforcement.

**Missing links / gaps:** Brute-force scan (documented in `README.md::Status` and `.agents/open_questions.md`); acceptable for v1.

---

### OUT-019 — Refresh (one-time / manual / scheduled)

**Outcome statement:** Sources can be indexed once, refreshed on demand, or refreshed periodically by a background thread that re-ingests stale sources.

**Actor/User:** Operator / agent.

**Settlement / success criterion:** `refresh_source` after ingest returns a completed/queued job; with `enable_periodic_refresh=true` the background thread enumerates stale sources.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `ingest_source`/`refresh_source` (`ck-app`); periodic thread `main.rs:160-215` calls `list_stale_sources` → `refresh_app.refresh_source`.

**Test evidence:** `cli.rs::refresh_source_tool_works`; `ck-scheduler` 9 tests.

**Missing links / gaps:** `local-confidence` proof: the periodic thread path compiles and is wired but no test runs a full periodic cycle (per `.agents/evidence.md` note).

---

### OUT-020 — Library/reference identity resolution

**Outcome statement:** A library/package/repo name resolves to one or more canonical IDs (and the `resolve-library-id` alias works for Context7-familiar clients).

**Actor/User:** Agent / retrieval.

**Settlement / success criterion:** `resolve_reference({query})` / `resolve-library-id` returns ranked candidates with canonical IDs.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `handle_resolve_reference` (`ck-mcp`); alias mapping at `lib.rs:117` (`"resolve_reference"|"resolve-library-id"`).

**Test evidence:** `cli.rs::resolve_library_id_alias_works`.

**Missing links / gaps:** None.

---

### OUT-021 — Ranked documentation retrieval with citations

**Outcome statement:** A query returns compact, ranked excerpts with citations (and `get-library-docs`/`query-docs` aliases work).

**Actor/User:** Agent / retrieval.

**Settlement / success criterion:** `query_reference` returns `query_id` + ranked `results` with citation metadata.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `handle_query_reference`; alias mapping at `lib.rs:114` (`"query_reference"|"get-library-docs"|"query-docs"`).

**Test evidence:** `cli.rs::query_reference_returns_results_after_ingest`, `query_reference_includes_citations_after_ingest`, `get_library_docs_alias_works`.

**Missing links / gaps:** None.

---

### OUT-022 — Service health/status surface

**Outcome statement:** `get_status` reports service identity, storage kind/path/fts5, source/session/job counts, and retrieval mode/semantic availability.

**Actor/User:** Operator / client.

**Settlement / success criterion:** `get_status` returns the JSON shape in `docs/mcp-contracts.md`.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `handle_get_status`; `main.rs:108-113` builds status with `semantic_enabled`.

**Test evidence:** `cli.rs::serve_stdio_answers_get_status_tool_call`.

**Missing links / gaps:** None.

---

### OUT-023 — Config hot-reload

**Outcome statement:** With `[reload].enabled=true` and `--config`, the stdio loop rebuilds storage/app/MCP/SSE-auth state when the file changes; SSE bind changes require restart.

**Actor/User:** Operator.

**Settlement / success criterion:** Editing the config file between stdio requests swaps in new auth/retention/retrieval settings without restarting the process.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** `main.rs:226-266` reload loop; `ReloadableSseState` (`ck-transport-sse`) for live SSE auth/service swap.

**Test evidence:** reload path is covered by static compilation + config parsing tests; no binary-level reload E2E in `cli.rs` (documented behavior).

**Missing links / gaps:** Bind-socket reload intentionally unsupported (documented).

---

### OUT-024 — OpenTelemetry-compatible observability export  ⚠ SPEC-REQUIRED, NOT IMPLEMENTED

**Outcome statement:** The service exports traces/metrics/spans/events to an OpenTelemetry collector when `otel_endpoint` is configured.

**Actor/User:** Operator / SRE.

**Settlement / success criterion:** With `otel_endpoint` set, spans/metrics are emitted to the collector; OTEL exporter failure does not crash the service.

**Status:** **declared_only (config_only)** — **Confidence:** high

**Declared evidence:**
- `Context_Kernel_Service_Specification.md::§20.2` — "An OpenTelemetry-compatible export path is REQUIRED."
- `Context_Kernel_Service_Specification.md::§26.1` — "OTEL-compatible observability export" listed under REQUIRED for conformance.
- `docs/configuration.md` — documents `otel_endpoint` (marked "(reserved)").

**Implementation evidence (config-only):**
- `crates/ck-config` accepts `otel_endpoint`.
- `crates/ck-observability/src/lib.rs::ObservabilityRuntimeConfig::otel_enabled` detects the field, but `init_observability` (lines 23-26) explicitly logs: **"OTEL endpoint configured but not yet supported; ignoring"**. No exporter, no metrics, no spans are wired.

**Test evidence:** `ck-observability` tests only assert detection of the endpoint (`detects_otel_endpoint`) and text-config-without-otel — they do not assert any export.

**Runtime path:** `otel_endpoint` set → detected → logged-and-ignored. No OTEL traffic ever leaves the process.

**Missing links / gaps:**
- `missing runtime enforcement` / `overclaimed maturity` (against the spec's REQUIRED language).
- Mitigated by candid README disclosure (`README.md::Observability` and `::Status` both state it is not implemented). Against the *spec* this is a conformance gap; against the *README* it is honestly disclosed.

**Reviewer notes:** Decide whether v1 conformance requires OTEL (spec says yes). If not, amend the spec to RECOMMENDED; if yes, implement the exporter.

---

### OUT-025 — Ports-and-adapters isolation

**Outcome statement:** Domain logic in `ck-core` depends on no SQL, HTTP, or transport detail; adapters are substitutable behind port traits.

**Actor/User:** Architect / maintainer.

**Settlement / success criterion:** `ck-core` has no `rusqlite`/`axum`/`hyper`/`ureq` dependencies; 12 port traits are defined; adapter substitution is tested.

**Status:** implemented — **Confidence:** high

**Implementation evidence:** 12 `pub trait` definitions in `crates/ck-core/src/ports/*` (`StoragePort`, `SessionMemoryPort`, `ArtifactPort`, `LexicalSearchPort`, `RankerPort`, `SourceAdapter`, `Normalizer`, `Chunker`, `EmbeddingPort`, `VectorSearchPort`, `AuthPort`, `SchedulerPort`). `ck-core/Cargo.toml` carries no infra crates.

**Test evidence:** `ck-app` uses a `TestSourceAdapter` mock to test domain logic without real adapters (`ck-app/src/lib.rs:1150`).

**Missing links / gaps:** Minor: the spec's §9 also lists Config Port (§9.1), MCP Transport Port (§9.12), and Observability Port (§9.14) as conceptual interfaces; these are **not** abstracted as `ck-core` traits (config is a concrete struct, transport is concrete in `ck-transport-*`, observability is a function module). Acceptable given the spec says "conceptual interfaces", but a strict reader would flag it.

---

### OUT-026 — Secret handling + path-traversal safety

**Outcome statement:** Secrets use `$VAR`/`${VAR}` indirection and are compared in constant time without being logged; artifact ids are validated to prevent path traversal.

**Actor/User:** Security.

**Settlement / success criterion:** `${CONTEXT_KERNEL_AUTH_TOKEN}` resolves from env; `../etc/passwd` as an artifact id is rejected; tokens are not printed.

**Status:** implemented — **Confidence:** high (secret-redaction-in-logs is structural, not assertion-tested)

**Implementation evidence:**
- `crates/ck-config/src/lib.rs::expand_env_vars` (line 287) — `${...}` interpolation.
- `crates/ck-artifacts/src/lib.rs::validate_id` (line 34) — rejects non-`[A-Za-z0-9_-]`.
- `crates/ck-auth` — `subtle::ConstantTimeEq` for tokens.

**Test evidence:** `ck-artifacts::validate_id_rejects_traversal` (tests `../etc/passwd`, `foo/bar`, `foo\bar`, `a..b`); `ck-config` env-expansion test.

**Missing links / gaps:** `missing test coverage`: no explicit test asserts "a configured token never appears in any tracing output" (secret redaction is assumed by absence, not asserted).

---

## 5. Cross-Cutting Architecture Map

- **Entrypoints:** `crates/ck-bin/src/main.rs::main` → `serve`/`migrate`/`doctor`/`export-schema`.
- **Core domain:** `crates/ck-core` — entities (`source`, `corpus`, `library`, `document`, `chunk`, `session`, `artifact`, `job`, `retrieval`) + 12 port traits + status types. No infra deps.
- **Application/orchestration:** `crates/ck-app/src/lib.rs::AppServices` — single struct holding one `StoragePort`, one `SessionMemoryPort`, one `ArtifactPort`, one `LexicalSearchPort`, one `RankerPort`, **one** `SourceAdapter`, **one** `Normalizer`, **one** `Chunker`, optional `EmbeddingPort`/`VectorSearchPort`. Use cases: `ingest_source`, `refresh_source`, `query_reference`, `resolve_reference`, `store_context`, `recall_context`, `run_retention_gc`, `get_status`.
- **Persistence/ledger:** `crates/ck-storage-sqlite` (`SCHEMA_V1`, CRUD, retention deletes), `crates/ck-artifacts` (filesystem payloads), `crates/ck-vector` (brute-force cosine).
- **Retrieval:** `crates/ck-search-fts` (FTS5/BM25), `crates/ck-ranker` (hybrid), `crates/ck-embeddings` (OpenAI/noop).
- **Source adapters:** `ck-adapter-filesystem` (wired), `ck-adapter-github` (NOT wired), `ck-adapter-web` (NOT wired).
- **MCP surface:** `crates/ck-mcp/src/lib.rs` — 7 canonical tools + 3 aliases; `call_tool` (stdio/local) + `call_tool_authorized` (SSE); `ToolTelemetry` DTO. Plus an out-of-band `agentic_capability_loop` stub module (external SEA domain integration, not part of the core tool dispatch).
- **Transports:** `ck-transport-stdio` (JSON-RPC line handler), `ck-transport-sse` (axum: `/mcp`, `/healthz`, `/readyz`, `/api/v1/status`; `ReloadableSseState`).
- **Runtime/security:** `ck-auth` (BearerAuth, allowlists), `ck-config` (TOML + `${}` expansion), `ck-observability` (text/json tracing; OTEL stub), `ck-scheduler` (RefreshScheduler).
- **Memory policy:** `ck-memory` (`EventRetentionClass`, `classify_event_kind`, `SessionRetentionPolicy`).
- **Tests/proofs:** `crates/ck-bin/tests/cli.rs` (16 binary-level tests, all `filesystem_tree`); per-crate unit tests (186 total per evidence log).
- **Build/release:** `Cargo.toml` workspace (20 crates), `.github/workflows/ci.yml`, `.github/workflows/release.yml` (Linux binary + SHA-256 on tag).

## 6. Contradictions and Overclaims

| Claim / Outcome | Evidence for Claim | Evidence Against / Missing | Risk |
| --------------- | ------------------ | -------------------------- | ---- |
| README: "supports ... GitHub repositories" / "GitHub ... source adapters" implemented (OUT-013) | `ck-adapter-github` crate with 18 tests | `wire_app` injects only `FilesystemSourceAdapter`; `ingest_source` has no kind-dispatch; no E2E github test | **High** — users will expect working github ingestion; it fails at runtime |
| README: "supports ... websites" (OUT-014) | `ck-adapter-web` crate with 34 tests | Same wiring gap as above | **High** — same failure mode |
| `.agents/project_state.md`: "ck-adapter-github: implemented", "ck-adapter-web: implemented" | Crate-level implementation | Not reachable from the binary; "implemented" overstates runtime maturity | **High** — status memory is misleading |
| Spec §26.1: "OTEL-compatible observability export" REQUIRED (OUT-024) | `otel_endpoint` config field | `ck-observability` logs "not yet supported; ignoring"; no exporter | **Medium** — spec conformance gap (honestly disclosed in README) |
| `docs/schema.md:28`: "Retention GC does not delete artifact payload files yet" (OUT-011) | (stale doc sentence) | `run_retention_gc` calls `delete_artifact_payload`; `cli.rs::retention_deletes_old_artifact_payload` proves it; `docs/operations.md:51` says the opposite | **Low** — code is correct; doc is stale |
| `.agents/evidence.md` "37 tests"/"65 tests"/"153 tests"/"186 tests" pass | Evidence-log entries | Test counts grew over sessions; not re-run in this audit | **Low** — verify with `cargo test --workspace` |
| Binary name `context-kernal` vs service/config name `context-kernel` | `ck-bin` binary; cli.rs asserts `"name":"context-kernel"` | Cosmetic spelling mismatch | **Low** — confusing but functional |

## 7. Missing Proofs

| Outcome ID | Missing Proof | Suggested Minimal Test | Why It Matters |
| ---------- | ------------- | ---------------------- | -------------- |
| OUT-013 | GitHub ingest through the binary | `cli.rs`: `ingest_source` with `kind:"github_repo"` against a fixture (or mocked HTTP) asserting indexed chunks | Proves the headline "ingest GitHub repos" claim |
| OUT-014 | Web ingest through the binary | `cli.rs`: `ingest_source` with `kind:"website"` against a local HTTP fixture asserting indexed text | Proves the headline "ingest websites" claim |
| OUT-015 | SSRF block via the binary | A wired web-adapter E2E asserting a `127.0.0.1` URL is rejected end-to-end | Security guarantee currently only proven at unit level |
| OUT-024 | OTEL export | If claiming conformance: a test asserting spans reach a collector (or drop the spec's REQUIRED) | Spec compliance / operator observability |
| OUT-017/018 | Hybrid retrieval through the binary | `cli.rs`: enable `[embeddings]` with a stubbed embedding provider, assert `semantic_enabled:true` and vector merge | The semantic path is library-tested only |
| OUT-026 | Secret never logged | A test asserting no tracing output contains a configured token string | Defense-in-depth for secret handling |

## 8. Frontier Model Audit Brief

### Most Important Outcomes
1. OUT-013 GitHub ingestion (partially_implemented — unwired)
2. OUT-014 Website ingestion (partially_implemented — unwired)
3. OUT-024 OTEL export (declared_only — spec REQUIRED, unwired)
4. OUT-011 Retention GC incl. artifact payloads (implemented but contradicted by `docs/schema.md`)
5. OUT-008 Hybrid ranking (implemented)
6. OUT-009/010 Session memory + artifacts (implemented)
7. OUT-003/004 stdio + SSE serving (implemented)
8. OUT-005/006 SSE auth + authz (implemented, SSE-only)
9. OUT-025 ports-and-adapters isolation (implemented; 3 spec ports not abstracted)
10. OUT-019 refresh scheduling (implemented; periodic path not E2E-tested)

### Highest-Risk Gaps
- **Source-kind dispatch is absent.** `AppServices` holds a single `source_adapter`/`normalizer`/`chunker`; `wire_app` injects only filesystem. GitHub + Web adapters are dead code from the binary's perspective. This silently breaks the two most externally-visible "supports" claims.
- **OTEL is spec-REQUIRED but stubbed** — conformance vs. honest-disclosure tension.
- **No binary-level proof** for github/web ingest, embeddings/vector hybrid merge, or periodic refresh — all are library-tested only.
- **`docs/schema.md` contradicts code** on artifact-payload GC (low severity, but a trust-eroding doc drift).

### First Files to Inspect
1. `crates/ck-bin/src/main.rs::wire_app` (lines 308-356) — the single source of truth for what the binary can actually do.
2. `crates/ck-app/src/lib.rs` — `AppServices` struct (lines 15-26) + `ingest_source` (lines 329-439); confirm no kind dispatch.
3. `crates/ck-mcp/src/lib.rs::call_tool` (lines 109-117) — tool/alias surface + `call_tool_authorized`.
4. `crates/ck-observability/src/lib.rs` (lines 13-26) — OTEL stub.
5. `crates/ck-bin/tests/cli.rs` — the entire binary-level proof surface (all `filesystem_tree`).
6. `crates/ck-core/src/ports/*` — port-trait inventory for architecture conformance.

### Open Architectural Questions
1. Is a **source registry / kind-dispatch layer** intended for v1, or are multi-source claims meant to be deferred? (Determines whether OUT-013/014 are bugs or overclaims.)
2. Should `AppServices` hold a *collection* of adapters/normalizers/chunkers keyed by `source_kind`, or should each ingest job carry its own adapter instance?
3. Is OTEL **REQUIRED** (spec §26.1) or **RECOMMENDED** for v1 conformance? The spec and README disagree in severity.
4. Are the Config/MCP-Transport/Observability "ports" (spec §9.1/§9.12/§9.14) meant to be `ck-core` traits, or is the current concrete-implementation acceptable?
5. What is the ownership boundary of `crates/ck-mcp/src/agentic_capability_loop.rs` and `integration/agentic_capability_loop/` relative to the core Context Kernel product?

### Recommended Next Prompt
> "Using the outcome-to-code map at `.agents/reports/outcome_code_map_Context_Kernel_Service_MCP_2026-06-14.md`, perform an adversarial audit of the Context Kernel binary's *runtime reachability*. For each outcome marked `implemented`, trace a concrete path from `crates/ck-bin/src/main.rs::serve` to the outcome's effect and either (a) confirm a binary-level test exercises it, or (b) downgrade it to `partially_implemented` with the exact missing wiring. Pay special attention to OUT-013/014 (GitHub/Web adapters): read `wire_app` and `AppServices::ingest_source`, then attempt an `ingest_source` call with `kind:"github_repo"` against the release binary and report the exact failure. Separately, decide whether OUT-024 (OTEL) must be implemented for v1 conformance per spec §26.1, and propose either an exporter implementation or a spec amendment to RECOMMENDED."

---

*Report generated 2026-06-14. All file/symbol references were verified by direct inspection of repository source. Test pass/fail counts are cited from `.agents/evidence.md` and were not re-executed during this audit; run `cargo test --workspace` to reproduce.*
