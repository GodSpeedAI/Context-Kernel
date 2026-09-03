# Current Plan

## Objective
- Construct a comprehensive, self-contained technical knowledge system for Context Kernel, satisfying both newcomers and experienced engineers via progressive disclosure, Diátaxis separation, DeepWiki-style architectural depth, and Google Code Wiki-style source traceability.

## Deliverables & Information Architecture
1. **Orientation & Map**:
   - `docs/README.md` (Documentation Landing Page & Navigation by Intent)
   - `docs/documentation-map.md` (Complete documentation catalog, audience need, Diátaxis class, prerequisites)
   - `docs/architecture.md` (Canonical system architecture: logical, runtime, dependency, data, control flow, security)
   - `docs/source-map.md` (Comprehensive mapping of concepts, ports, adapters, and tools to source symbols)
2. **Foundations (Layers 0 & 1)**:
   - `docs/mental-model.md` (Conceptual boundaries, core responsibilities, state ownership, invariants)
   - `docs/concepts.md` (Domain vocabulary: corpora, sources, documents, chunks, libraries, sessions, events, artifacts)
3. **Subsystems (Layer 3)**:
   - `docs/subsystems/core-and-domain.md` (`ck-core`, `ck-memory`)
   - `docs/subsystems/storage-and-persistence.md` (`ck-storage-sqlite`, `ck-artifacts`, `ck-vector`)
   - `docs/subsystems/retrieval-and-ranking.md` (`ck-search-fts`, `ck-ranker`, `ck-embeddings`)
   - `docs/subsystems/source-adapters.md` (`ck-adapter-filesystem`, `ck-adapter-github`, `ck-adapter-web`)
   - `docs/subsystems/application-orchestration.md` (`ck-app`, `AppServices`)
   - `docs/subsystems/mcp-and-protocols.md` (`ck-mcp`, `ck-transport-stdio`, `ck-transport-sse`, `ck-auth`)
   - `docs/subsystems/runtime-and-cli.md` (`ck-bin`, `ck-config`, `ck-observability`, `ck-scheduler`)
4. **Execution Workflows (Layer 4)**:
   - `docs/workflows/service-lifecycle-and-startup.md` (CLI parsing, migration, GC, transports, reload)
   - `docs/workflows/documentation-ingestion-and-refresh.md` (Discovery, normalization, chunking, indexing, refresh coalescing)
   - `docs/workflows/reference-query-and-resolution.md` (Library resolution, FTS5 lookup, hybrid scoring, citation building)
   - `docs/workflows/session-context-continuity.md` (Event storage, artifact offloading, recall, retention)
   - `docs/workflows/agentic-capability-loop.md` (E2/E3 boundary, `context_required` tool, SWE_SEED & SEA-Forge contract)
5. **Explanations & Design Rationale (Layer 5)**:
   - `docs/explanation/architectural-decisions.md` (Ports-and-adapters, single-binary, SQLite+FTS5, filesystem artifacts)
   - `docs/explanation/retrieval-and-ranking-design.md` (Token-efficient excerpts, intent-weighted hybrid ranking, BM25 scoring)
   - `docs/explanation/retention-and-memory-model.md` (Tiered event classification, pinned vs ephemeral, data integrity)
   - `docs/explanation/known-limits-and-debt.md` (Wired vs unwired adapters, FTS atomicity residual, scheduler thread bypass)
6. **Tutorials (Layer 6)**:
   - `docs/tutorials/01-first-steps.md` (Local build, migration, doctor check, stdio MCP context store/recall)
   - `docs/tutorials/02-local-documentation-indexing.md` (Indexing a markdown documentation tree, querying references)
7. **How-To Guides (Layer 7)**:
   - `docs/how-to/configure-sse-with-auth.md` (Configuring SSE with bearer auth and tool/corpus allowlists)
   - `docs/how-to/manage-database-and-migrations.md` (Migrations, doctor, schema export, and FTS index rebuild)
   - `docs/how-to/configure-semantic-retrieval.md` (OpenAI embedding provider and vector search setup)
   - `docs/how-to/implement-a-source-adapter.md` (Adding a new source adapter behind SourceAdapter port)
8. **Reference & Operations (Layer 8)**:
   - `docs/reference/cli.md` (Complete CLI flag and command reference for `context-kernal`)
   - `docs/reference/configuration.md` (TOML configuration reference, env vars, defaults)
   - `docs/reference/mcp-tools.md` (MCP tool contracts, JSON-RPC schemas, aliases, errors)
   - `docs/reference/database-schema.md` (Complete SQLite schema, virtual tables, indexes, constraints)
   - `docs/troubleshooting.md` (Diagnostic guide, error codes, common failure modes, recovery paths)

## Validation & Verification Steps
1. Trace every documented assertion, port trait, entity struct, configuration key, CLI flag, and SQLite table directly to source code and tests.
2. Clearly distinguish verified facts from inferences or known architectural debt.
3. Validate all markdown links and source references.
4. Update project memory files: `.agents/work_log.md`, `.agents/decisions.md`, `.agents/evidence.md`, `.agents/project_state.md`, `.agents/current_status.yml`.
