# Evidence Log

## Claim
- Claim: Cargo workspace exists for Milestone 0 crates.
- Status: implemented
- Evidence type: code
- Files: `Cargo.toml`
- Lines: 1-9
- Notes: Workspace includes ck-core, ck-config, ck-storage-sqlite, ck-search-fts, ck-artifacts, ck-ranker, ck-adapter-filesystem, ck-app, ck-mcp, ck-transport-stdio, ck-bin.
- Confidence: high

## Claim
- Claim: SQLite migration framework creates schema version 1 and FTS5 tables.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Lines: SCHEMA_V1 constant, migrate(), tests
- Notes: `SqliteStore::migrate` executes schema v1; test verifies `schema_migrations` and `chunks_fts`.
- Confidence: high

## Claim
- Claim: `get_status` is exposed through the MCP service.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`
- Notes: `McpService::call_tool` returns serialized `KernelStatus` for `get_status`; test verifies contract fields.
- Confidence: high

## Claim
- Claim: stdio `serve` can answer JSON-RPC `tools/call` for `get_status`.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-bin/src/main.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: CLI test sends a JSON-RPC tool call and asserts `context-kernal` plus `fts5_enabled: true`.
- Confidence: high

## Claim
- Claim: StoragePort is fully implemented with CRUD for sources, corpora, documents, chunks, library identities, aliases, and refresh jobs.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Notes: 6 unit tests cover session CRUD, event CRUD, source upsert, document+chunk CRUD, library identity+alias CRUD, refresh job CRUD.
- Confidence: high

## Claim
- Claim: SessionMemoryPort is implemented with session creation, event persistence, and active session counting.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-storage-sqlite/src/lib.rs`
- Notes: Tests verify session creation, event addition, listing, and count.
- Confidence: high

## Claim
- Claim: LexicalSearchPort is implemented with FTS5/BM25 for chunks, session events, and library aliases.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-search-fts/src/lib.rs`
- Notes: 5 tests cover chunk indexing/search, removal, session event indexing/search, library alias indexing/search, BM25 score positivity.
- Confidence: high

## Claim
- Claim: ArtifactPort is implemented with filesystem storage and SHA-256 checksums.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-artifacts/src/lib.rs`
- Notes: 3 tests cover store+read, checksum verification, non-existent returns None.
- Confidence: high

## Claim
- Claim: RankerPort is implemented with lexical ranking and score breakdowns.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-ranker/src/lib.rs`
- Notes: 3 tests cover resolution candidates, retrieval results, recall results.
- Confidence: high

## Claim
- Claim: Filesystem source adapter, markdown normalizer, and heading-aware chunker are implemented.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-adapter-filesystem/src/lib.rs`
- Notes: 6 tests cover file discovery, content fetching, normalization, heading-based chunking, text without headings.
- Confidence: high

## Claim
- Claim: `store_context` MCP tool persists session events and handles large payloads as artifacts.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test verifies store_context returns event_id and stored_as fields.
- Confidence: high

## Claim
- Claim: `recall_context` MCP tool returns BM25-ranked session events.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test verifies recall returns query_id and results.
- Confidence: high

## Claim
- Claim: `ingest_source` MCP tool indexes local filesystem documents.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test creates a temp docs directory, ingests it, and verifies job_id and completed status.
- Confidence: high

## Claim
- Claim: `query_reference` MCP tool returns lexical search results over indexed chunks.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-app/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration test ingests docs then queries, verifying query_id and results.
- Confidence: high

## Claim
- Claim: `resolve_reference` MCP tool and compatibility aliases work.
- Status: implemented
- Evidence type: code, test
- Files: `crates/ck-mcp/src/lib.rs`, `crates/ck-bin/tests/cli.rs`
- Notes: Integration tests verify resolve-library-id and get-library-docs aliases return candidates/query_id.
- Confidence: high

## Claim
- Claim: All 37 workspace tests pass, cargo fmt is clean, cargo clippy has zero warnings.
- Status: implemented
- Evidence type: test
- Files: workspace
- Notes: `cargo test --workspace` = 37 passed, `cargo fmt --check` = clean, `cargo clippy --workspace` = zero warnings.
- Confidence: high
