# Current Plan

## Objective
- Implement Milestones 1 through 3 from `Context_Kernel_Service_Plan.md`:
  - M1: Storage + memory foundation (sessions, events, artifacts, store_context, recall_context)
  - M2: Local document ingestion (filesystem adapter, normalizer, chunker, ingest_source, query_reference)
  - M3: Unified identity layer (library_identities, aliases, resolve_reference, compatibility aliases)

## Scope
- Expand `ck-core` with all domain entities and port traits
- Create `ck-app` crate for use case orchestration
- Expand `ck-storage-sqlite` with full CRUD for all entities
- Create `ck-search-fts` for FTS5/BM25 lexical search
- Create `ck-artifacts` for filesystem artifact storage
- Create `ck-ranker` for lexical ranking and score breakdowns
- Create `ck-adapter-filesystem` for filesystem source adapter, normalizer, chunker
- Expand `ck-mcp` with all new MCP tools and compatibility aliases
- Update `ck-bin` to wire all new crates
- Add integration tests for all new tools

## Constraints
- Follow ports-and-adapters boundaries from `AGENTS.md`
- `ck-core` must have no dependency on other internal crates
- `ck-app` depends on `ck-core` only
- Adapters depend on `ck-core`, not on `ck-app`
- `ck-bin` depends on everything, wires concrete implementations
- Use `rusqlite` for SQLite access (synchronous)
- Use `ulid` for ID generation
- Use `sha2` for checksums

## Files / Crates Expected To Change
- `Cargo.toml` (add new workspace members and dependencies)
- `crates/ck-core/src/lib.rs` (expand with entities, ports, types)
- `crates/ck-storage-sqlite/src/lib.rs` (expand with CRUD operations)
- `crates/ck-mcp/src/lib.rs` (expand with all new tools)
- `crates/ck-transport-stdio/src/lib.rs` (update for new tools)
- `crates/ck-bin/src/main.rs` (wire new crates)
- `crates/ck-bin/tests/cli.rs` (add integration tests)

## New Crates
- `crates/ck-app/` (use cases)
- `crates/ck-search-fts/` (FTS5 lexical search)
- `crates/ck-artifacts/` (filesystem artifact storage)
- `crates/ck-ranker/` (ranking logic)
- `crates/ck-adapter-filesystem/` (filesystem source adapter)

## Steps
1. Expand `ck-core` with all domain entities, enums, and port traits
2. Expand `ck-storage-sqlite` with CRUD for sessions, events, artifacts, documents, chunks, sources, libraries
3. Create `ck-search-fts` implementing LexicalSearchPort
4. Create `ck-artifacts` implementing ArtifactPort
5. Create `ck-ranker` with lexical ranker and score breakdowns
6. Create `ck-adapter-filesystem` with source adapter, normalizer, chunker
7. Create `ck-app` with use cases
8. Expand `ck-mcp` with all new tools and compatibility aliases
9. Update `ck-bin` to wire everything
10. Add integration tests
11. Run cargo test, cargo fmt, cargo clippy

## Validation Plan
- `cargo test --workspace` passes
- `cargo fmt --check` passes
- `cargo clippy --workspace` passes
- Integration tests verify each MCP tool works end-to-end
