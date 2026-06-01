# Evidence Log

## Claim
- Claim: No Rust implementation existed before Milestone 0 work started.
- Status: not implemented
- Evidence type: code
- Files: repository root listing
- Lines: not applicable
- Notes: Static inspection showed only `AGENTS.md`, `Context_Kernel_Service_Plan.md`, and `Context_Kernel_Service_Specification.md`.
- Confidence: high

## Claim
- Claim: Cargo workspace exists for Milestone 0 crates.
- Status: implemented
- Evidence type: code
- Files: `Cargo.toml`
- Lines: 1-9
- Notes: Workspace includes `ck-core`, `ck-config`, `ck-storage-sqlite`, `ck-mcp`, `ck-transport-stdio`, and `ck-bin`.
- Confidence: high

## Claim
- Claim: SQLite migration framework creates schema version 1 and FTS5 tables.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Lines: 22-24, 47-248, 257-281
- Notes: `SqliteStore::migrate` executes schema v1; test verifies `schema_migrations` and `chunks_fts`.
- Confidence: high

## Claim
- Claim: `get_status` is exposed through the MCP service.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`
- Lines: 20-24, 33-42
- Notes: `McpService::call_tool` returns serialized `KernelStatus` for `get_status`; test verifies contract fields.
- Confidence: high

## Claim
- Claim: stdio `serve` can answer JSON-RPC `tools/call` for `get_status`.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-bin/src/main.rs`, `crates/ck-bin/tests/cli.rs`
- Lines: `crates/ck-bin/src/main.rs` 54-72; `crates/ck-bin/tests/cli.rs` 19-35
- Notes: CLI test sends a JSON-RPC tool call and asserts `context-kernal` plus `fts5_enabled: true`; manual smoke command also returned valid JSON.
- Confidence: high

## Claim
- Claim: Retrieval, context persistence tools, source ingestion, scheduler, SSE, and auth are not implemented.
- Status: not implemented
- Evidence type: code
- Files: `crates/ck-mcp/src/lib.rs`, `Cargo.toml`
- Lines: `crates/ck-mcp/src/lib.rs` 20-24; `Cargo.toml` 1-9
- Notes: MCP dispatch only recognizes `get_status`; no crates for app, memory, adapters, scheduler, auth, or SSE exist yet.
- Confidence: high

## Claim
- Claim: `.gitignore` excludes local build/runtime artifacts without hiding project source, docs, lockfile, or memory files.
- Status: implemented
- Evidence type: config
- Files: `.gitignore`
- Lines: 1-41
- Notes: Verified with `git check-ignore -v` for ignored paths (`target/`, SQLite files, local config, `.env.*`, `data/`, `logs/`) and non-ignored paths (`Cargo.lock`, `.agent/project_state.md`, `context-kernal.toml.example`, docs, source).
- Confidence: high
