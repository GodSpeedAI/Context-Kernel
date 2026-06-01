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
