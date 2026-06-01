# Current Plan

## Objective
- Implement Milestone 0 from `Context_Kernel_Service_Plan.md`: compile a Rust skeleton with stdio MCP `get_status` and SQLite migration bootstrap.

## Scope
- Create the initial Cargo workspace and core crates required for Milestone 0.
- Add domain/status types, TOML config loading, SQLite migration framework, MCP tool dispatch, stdio request handling, and `context-kernal` CLI commands.

## Constraints
- Follow ports-and-adapters boundaries from `AGENTS.md`.
- Treat code/tests as implementation evidence; docs alone are intent.
- Use TDD for behavior changes.
- `.github/copilot-instructions.md` is missing; no project-specific instructions could be read from that file.

## Files / Crates Expected To Change
- `Cargo.toml`
- `crates/ck-core`
- `crates/ck-config`
- `crates/ck-storage-sqlite`
- `crates/ck-mcp`
- `crates/ck-transport-stdio`
- `crates/ck-bin`
- `context-kernal.toml.example`
- `docs/schema.md`
- `docs/mcp-contracts.md`
- `.agent/*`

## Steps
1. Completed: Add failing tests for config loading, SQLite migration/status, MCP `get_status`, and CLI smoke behavior.
2. Completed: Scaffold the workspace and minimal crate APIs needed by those tests.
3. Completed: Implement the smallest Milestone 0 code path that passes the tests.
4. Completed: Run formatting and relevant tests.
5. Completed: Update project memory with repository-grounded evidence.

## Validation Plan
- Completed: `cargo test --workspace`
- Completed: `cargo fmt --check`
- Completed: manual stdio `get_status` smoke check with `cargo run -q -p ck-bin --bin context-kernal -- serve --database /tmp/context-kernal-smoke.db`
