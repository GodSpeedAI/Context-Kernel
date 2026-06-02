# Decisions

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
