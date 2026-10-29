# Documentation Map

This document catalogues every page in the Context Kernel technical knowledge system. It specifies the purpose of each page, its primary audience need, its Diátaxis classification, its prerequisites, and related reading paths.

---

## 1. Map Overview

The documentation system is organized into distinct layers to support progressive disclosure:

* **Layer 0 & 1 — Orientation & Foundations**: High-level mental models and essential vocabulary for newcomers.
* **Layer 2 — Architecture**: The canonical architectural model and structural boundaries.
* **Layer 3 — Subsystems**: Deep technical analysis of each crate group and its ports/adapters.
* **Layer 4 — Workflows & Traces**: Step-by-step execution traces from ingress to persistence.
* **Layer 5 — Explanation**: Design rationale, trade-offs, constraints, and verified vs inferred decisions.
* **Layer 6 — Tutorials**: Guided learning experiences starting from a clean environment.
* **Layer 7 — How-To Guides**: Practical, goal-oriented recipes for common operator and developer tasks.
* **Layer 8 — Reference**: Authoritative technical contracts, CLI flags, configuration schemas, and database DDL.

---

## 2. Master Page Directory

| Page Path | Diátaxis Class | Primary Audience Need | Prerequisites | Related Pages |
| :--- | :--- | :--- | :--- | :--- |
| [`docs/README.md`](README.md) | Orientation | Fast orientation, high-level mental model, navigation by intent | None | [`mental-model.md`](mental-model.md), [`documentation-map.md`](documentation-map.md) |
| [`docs/documentation-map.md`](documentation-map.md) | Meta-Guide | Understand documentation structure, discover relevant topics | None | [`README.md`](README.md), [`source-map.md`](source-map.md) |
| [`docs/mental-model.md`](mental-model.md) | Explanation | Understand what the system owns, its boundaries, and invariants | [`README.md`](README.md) | [`concepts.md`](concepts.md), [`architecture.md`](architecture.md) |
| [`docs/concepts.md`](concepts.md) | Reference | Learn domain terminology without deciphering source code | [`README.md`](README.md) | [`mental-model.md`](mental-model.md), [`source-map.md`](source-map.md) |
| [`docs/architecture.md`](architecture.md) | Explanation / Architecture | Deep architectural understanding for modifying or extending the system | [`mental-model.md`](mental-model.md) | Subsystem guides, [`source-map.md`](source-map.md) |
| [`docs/source-map.md`](source-map.md) | Reference / Traceability | Find exact source files, types, traits, and functions for any concept | [`architecture.md`](architecture.md) | Subsystem guides |
| [`docs/troubleshooting.md`](troubleshooting.md) | How-To / Reference | Diagnose runtime errors, startup failures, and corrupted state | [`README.md`](README.md) | [`reference/cli.md`](reference/cli.md), [`how-to/manage-database-and-migrations.md`](how-to/manage-database-and-migrations.md) |

### Subsystems (Layer 3)

| Page Path | Diátaxis Class | Primary Audience Need | Prerequisites | Related Pages |
| :--- | :--- | :--- | :--- | :--- |
| [`docs/subsystems/core-and-domain.md`](subsystems/core-and-domain.md) | Explanation / Reference | Understand domain entities, port definitions, and memory classification | [`architecture.md`](architecture.md) | [`subsystems/application-orchestration.md`](subsystems/application-orchestration.md) |
| [`docs/subsystems/storage-and-persistence.md`](subsystems/storage-and-persistence.md) | Explanation / Reference | Understand SQLite relational model, WAL mode, FTS5 virtual tables, and artifact files | [`architecture.md`](architecture.md) | [`reference/database-schema.md`](reference/database-schema.md), [`workflows/session-context-continuity.md`](workflows/session-context-continuity.md) |
| [`docs/subsystems/retrieval-and-ranking.md`](subsystems/retrieval-and-ranking.md) | Explanation / Reference | Understand BM25 indexing, intent weighting, score normalization, and vector search | [`architecture.md`](architecture.md) | [`explanation/retrieval-and-ranking-design.md`](explanation/retrieval-and-ranking-design.md), [`workflows/reference-query-and-resolution.md`](workflows/reference-query-and-resolution.md) |
| [`docs/subsystems/source-adapters.md`](subsystems/source-adapters.md) | Explanation / Reference | Understand filesystem, GitHub, and web ingestion logic and SSRF protection | [`architecture.md`](architecture.md) | [`how-to/implement-a-source-adapter.md`](how-to/implement-a-source-adapter.md), [`explanation/known-limits-and-debt.md`](explanation/known-limits-and-debt.md) |
| [`docs/subsystems/application-orchestration.md`](subsystems/application-orchestration.md) | Explanation / Reference | Understand how `AppServices` coordinates ports to fulfill use cases | [`architecture.md`](architecture.md), [`subsystems/core-and-domain.md`](subsystems/core-and-domain.md) | All workflow pages |
| [`docs/subsystems/mcp-and-protocols.md`](subsystems/mcp-and-protocols.md) | Explanation / Reference | Understand `McpService`, JSON-RPC stdio dispatch, axum SSE HTTP transport, and auth policy | [`architecture.md`](architecture.md) | [`reference/mcp-tools.md`](reference/mcp-tools.md), [`how-to/configure-sse-with-auth.md`](how-to/configure-sse-with-auth.md) |
| [`docs/subsystems/runtime-and-cli.md`](subsystems/runtime-and-cli.md) | Explanation / Reference | Understand the composition root, CLI commands, configuration loading, tracing, and scheduler | [`architecture.md`](architecture.md) | [`reference/cli.md`](reference/cli.md), [`reference/configuration.md`](reference/configuration.md) |

### Workflows (Layer 4)

| Page Path | Diátaxis Class | Primary Audience Need | Prerequisites | Related Pages |
| :--- | :--- | :--- | :--- | :--- |
| [`docs/workflows/service-lifecycle-and-startup.md`](workflows/service-lifecycle-and-startup.md) | Explanation / Workflow | Trace binary bootstrap, argument parsing, migration, GC, and hot config reload | [`architecture.md`](architecture.md) | [`subsystems/runtime-and-cli.md`](subsystems/runtime-and-cli.md) |
| [`docs/workflows/documentation-ingestion-and-refresh.md`](workflows/documentation-ingestion-and-refresh.md) | Explanation / Workflow | Trace discovery, normalization, chunking, indexing, and refresh job coalescing | [`subsystems/source-adapters.md`](subsystems/source-adapters.md) | [`subsystems/storage-and-persistence.md`](subsystems/storage-and-persistence.md) |
| [`docs/workflows/reference-query-and-resolution.md`](workflows/reference-query-and-resolution.md) | Explanation / Workflow | Trace library identity resolution, FTS5 chunk query, hybrid scoring, and citation building | [`subsystems/retrieval-and-ranking.md`](subsystems/retrieval-and-ranking.md) | [`reference/mcp-tools.md`](reference/mcp-tools.md) |
| [`docs/workflows/session-context-continuity.md`](workflows/session-context-continuity.md) | Explanation / Workflow | Trace event persistence, payload offloading to artifacts, recall, and retention GC | [`subsystems/core-and-domain.md`](subsystems/core-and-domain.md) | [`explanation/retention-and-memory-model.md`](explanation/retention-and-memory-model.md) |
| [`docs/workflows/agentic-capability-loop.md`](workflows/agentic-capability-loop.md) | Explanation / Workflow | Trace the `context_required` tool call across the frozen E2/E3 convergence boundary | [`concepts.md`](concepts.md) | [`workflows/context-requirement-resolution.md`](workflows/context-requirement-resolution.md), [`docs/agentic_loop_boundaries.md`](../docs/agentic_loop_boundaries.md) |
| [`docs/workflows/context-requirement-resolution.md`](workflows/context-requirement-resolution.md) | Explanation / Workflow | Trace the eight resolution stages from access gate to sealed bundle, including reasons, omissions, and calibration | [`concepts.md`](concepts.md) | [`reference/mcp-tools.md`](reference/mcp-tools.md), [`explainers/context-requirement-explorer.html`](explainers/context-requirement-explorer.html) |
| [`docs/explainers/context-requirement-explorer.html`](explainers/context-requirement-explorer.html) | Tutorial / Interactive | Experiment with request payloads, stage outcomes, omissions, integrity hashes, and tamper detection offline | [`workflows/context-requirement-resolution.md`](workflows/context-requirement-resolution.md) | [`concepts.md`](concepts.md) |

### Explanation (Layer 5)

| Page Path | Diátaxis Class | Primary Audience Need | Prerequisites | Related Pages |
| :--- | :--- | :--- | :--- | :--- |
| [`docs/explanation/architectural-decisions.md`](explanation/architectural-decisions.md) | Explanation | Understand why ports-and-adapters, single-binary, SQLite+FTS5, and filesystem artifacts were chosen | [`architecture.md`](architecture.md) | [`explanation/known-limits-and-debt.md`](explanation/known-limits-and-debt.md) |
| [`docs/explanation/retrieval-and-ranking-design.md`](explanation/retrieval-and-ranking-design.md) | Explanation | Understand why token-efficient excerpts, intent-weighted hybrid ranking, and BM25 inversion exist | [`subsystems/retrieval-and-ranking.md`](subsystems/retrieval-and-ranking.md) | [`workflows/reference-query-and-resolution.md`](workflows/reference-query-and-resolution.md) |
| [`docs/explanation/retention-and-memory-model.md`](explanation/retention-and-memory-model.md) | Explanation | Understand tiered event classification, pinning guarantees, and unpinned GC trade-offs | [`subsystems/core-and-domain.md`](subsystems/core-and-domain.md) | [`workflows/session-context-continuity.md`](workflows/session-context-continuity.md) |
| [`docs/explanation/known-limits-and-debt.md`](explanation/known-limits-and-debt.md) | Explanation | Understand verified vs unwired adapters, FTS write atomicity debt, and scheduler thread bypass | [`architecture.md`](architecture.md) | [`.agents/OBSERVED_DEBT.md`](../.agents/OBSERVED_DEBT.md) |

### Tutorials (Layer 6)

| Page Path | Diátaxis Class | Primary Audience Need | Prerequisites | Related Pages |
| :--- | :--- | :--- | :--- | :--- |
| [`docs/tutorials/01-first-steps.md`](tutorials/01-first-steps.md) | Tutorial | Build the binary, run migrations and doctor check, and execute first stdio MCP store/recall | Rust 2021 toolchain | [`tutorials/02-local-documentation-indexing.md`](tutorials/02-local-documentation-indexing.md) |
| [`docs/tutorials/02-local-documentation-indexing.md`](tutorials/02-local-documentation-indexing.md) | Tutorial | Configure a filesystem corpus, ingest markdown files, and query references with citations | [`tutorials/01-first-steps.md`](tutorials/01-first-steps.md) | [`how-to/configure-sse-with-auth.md`](how-to/configure-sse-with-auth.md) |

### How-To Guides (Layer 7)

| Page Path | Diátaxis Class | Primary Audience Need | Prerequisites | Related Pages |
| :--- | :--- | :--- | :--- | :--- |
| [`docs/how-to/configure-sse-with-auth.md`](how-to/configure-sse-with-auth.md) | How-To Guide | Configure SSE HTTP transport with bearer tokens, tool allowlists, and corpus allowlists | Working binary | [`reference/configuration.md`](reference/configuration.md) |
| [`docs/how-to/manage-database-and-migrations.md`](how-to/manage-database-and-migrations.md) | How-To Guide | Initialize database, verify storage health, export DDL, and rebuild corrupted FTS indexes | Working binary | [`reference/cli.md`](reference/cli.md), [`reference/database-schema.md`](reference/database-schema.md) |
| [`docs/how-to/configure-semantic-retrieval.md`](how-to/configure-semantic-retrieval.md) | How-To Guide | Configure OpenAI-compatible embedding provider and SQLite brute-force vector search | Working binary | [`reference/configuration.md`](reference/configuration.md) |
| [`docs/how-to/implement-a-source-adapter.md`](how-to/implement-a-source-adapter.md) | How-To Guide | Implement the `SourceAdapter` port trait and register a new documentation source kind | Rust knowledge | [`subsystems/source-adapters.md`](subsystems/source-adapters.md) |

### Reference (Layer 8)

| Page Path | Diátaxis Class | Primary Audience Need | Prerequisites | Related Pages |
| :--- | :--- | :--- | :--- | :--- |
| [`docs/reference/cli.md`](reference/cli.md) | Reference | Authoritative syntax, subcommands, flags, and exit behavior for `context-kernal` | None | [`how-to/manage-database-and-migrations.md`](how-to/manage-database-and-migrations.md) |
| [`docs/reference/configuration.md`](reference/configuration.md) | Reference | Complete TOML table reference, environment variable expansion syntax, and defaults | None | [`how-to/configure-sse-with-auth.md`](how-to/configure-sse-with-auth.md) |
| [`docs/reference/mcp-tools.md`](reference/mcp-tools.md) | Reference | Exact JSON-RPC schemas, parameter types, compatibility aliases, and error objects | None | [`subsystems/mcp-and-protocols.md`](subsystems/mcp-and-protocols.md) |
| [`docs/reference/database-schema.md`](reference/database-schema.md) | Reference | Complete SQLite DDL, virtual tables, triggers, indexes, and enum codec conversions | None | [`subsystems/storage-and-persistence.md`](subsystems/storage-and-persistence.md) |
