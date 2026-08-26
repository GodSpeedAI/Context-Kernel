# Context Kernel

### A single-binary MCP service for durable context, documentation retrieval, and governed reference search

[![CI](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/ci.yml)
[![Release](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/release.yml/badge.svg)](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-stdio%20%2B%20SSE-blue.svg)](docs/mcp-contracts.md)

Context Kernel gives AI agents a memory and reference layer that can survive past the current chat window, fetch the right documentation, and keep private context inside your own deployment.

It stores session context, indexes documentation, resolves library identities, reads public and private references, and serves the result through MCP tools. It does this as one Rust binary backed by SQLite, FTS5, filesystem artifacts, optional embeddings, and explicit authorization boundaries.

AI work fails when context is folklore. The agent remembers half a decision, guesses at an API, cites stale docs, and then writes code with tremendous confidence. Context Kernel is the infrastructure layer that makes context inspectable before the agent acts.

---

## The Problem

Most AI coding setups treat context as a prompt stuffing exercise.

That breaks down quickly:

- documentation gets fetched repeatedly and inconsistently
- session decisions disappear between runs
- private and public references blur together
- large artifacts do not fit cleanly in conversation
- agents cite what they remember, not what the repository proves

The answer is not a bigger prompt. It is a local service with storage, retrieval, source adapters, and policy.

Context Kernel turns context into an operational system: current enough to trust, small enough to fit, durable enough to resume, and scoped enough to govern.

---

## What Context Kernel Does

Context Kernel unifies three jobs that agents need every day:

### Documentation Retrieval

- ingest local docs, GitHub repositories, and websites
- index chunks with SQLite FTS5
- resolve a library name into a stable library or corpus identity
- retrieve library/API documentation by version, corpus, or free text
- return compact excerpts with citations
- support common documentation MCP aliases: `resolve-library-id`, `get-library-docs`, and `query-docs`

This handles the boring but expensive failure mode: an agent writing against APIs that moved since its training data.

### Context Continuity

- store session events across tool runs
- recall decisions, requirements, constraints, and notes
- keep large payloads in artifact storage instead of stuffing them inline
- classify session memory through the `ck-memory` crate
- preserve pinned events during retention cleanup

This handles the other half of the context problem: after compaction or a new run, the agent needs the relevant memory, not a transcript landfill.

### Token-Efficient Reference Reading

- search first, then read the few resources that matter
- return short excerpts instead of whole pages
- use query intent and session context in ranking
- collapse duplicate document results
- store artifacts and raw context outside the prompt

This keeps documentation useful without turning the context window into a junk drawer with syntax highlighting.

### Governed Access

- serve MCP over stdio for local workflows
- serve MCP over SSE for remote access
- validate bearer tokens for SSE
- authorize by MCP tool and corpus
- restrict web ingestion with SSRF protection and DNS resolution checks

The point is simple: agents should not have to improvise their memory or their sources.

---

## Capabilities

### MCP Tools

Canonical tools:

- `get_status`
- `store_context`
- `recall_context`
- `ingest_source`
- `refresh_source`
- `query_reference`
- `resolve_reference`

Compatibility aliases:

- `resolve-library-id` → `resolve_reference`
- `get-library-docs` → `query_reference`
- `query-docs` → `query_reference`

See [MCP contracts](docs/mcp-contracts.md).

### Source Adapters

Context Kernel currently supports:

- local filesystem trees
- GitHub repositories
- websites

The GitHub adapter detects truncated tree API responses instead of silently indexing partial data. The web adapter blocks private IP ranges, non-HTTP schemes, localhost-style hosts, and DNS names that resolve to private addresses.

PDF ingestion and a bundled external documentation dataset adapter are not part of v1. Compatibility with common documentation MCP clients is provided through tool aliases and response contracts.

### Retrieval

Retrieval uses:

- SQLite FTS5 and BM25 for lexical search
- intent-aware ranking
- trust and freshness score signals
- document-level result de-duplication
- optional OpenAI-compatible embeddings
- brute-force cosine vector search for small corpora

The vector store records corpus, library, and document metadata so semantic search can respect retrieval scope.

The retrieval shape is deliberate:

```text
resolve identity → search focused chunks → read compact excerpts → cite source
```

That beats dumping a full documentation page into the model and hoping attention survives the trip.

### Memory And Artifacts

Session events are stored in SQLite. Large payloads are stored in the filesystem artifact store and referenced from session events.

Retention GC can remove:

- unpinned old session events
- inactive sessions
- finished refresh jobs
- artifact metadata and payload files
- disabled old sources

Pinned events survive cleanup.

### Security

SSE mode can require bearer tokens:

```toml
[mcp.sse]
require_auth = true

[auth]
tokens = ["${CONTEXT_KERNEL_AUTH_TOKEN}"]
```

Tool and corpus allowlists are configured separately:

```toml
[authorization]
allow_tools = ["get_status", "query_reference", "resolve_reference"]
allow_corpora = ["public_docs"]
```

Stdio mode is local trust mode. Use it for local editor and agent integrations where the process boundary is the trust boundary.

See [security](docs/security.md).

---

## Quick Start

Build the binary:

```bash
cargo build --release -p ck-bin
```

Initialize storage:

```bash
target/release/context-kernal migrate --database ./data/kernel.db
```

Check health:

```bash
target/release/context-kernal doctor --database ./data/kernel.db
```

Run in local stdio mode:

```bash
target/release/context-kernal serve \
  --database ./data/kernel.db \
  --data-dir ./data
```

Run with SSE:

```bash
export CONTEXT_KERNEL_AUTH_TOKEN="$(openssl rand -hex 32)"

target/release/context-kernal serve \
  --config config/context-kernal.sse.example.toml \
  --sse 127.0.0.1:8765
```

Print the embedded SQLite schema:

```bash
target/release/context-kernal export-schema
```

See [install](docs/install.md), [configuration](docs/configuration.md), and [operations](docs/operations.md).

---

## Core Workflows

```bash
# Format, lint, and test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Build release binary
cargo build --release -p ck-bin

# Initialize a database
context-kernal migrate --database ./data/kernel.db

# Check database health
context-kernal doctor --database ./data/kernel.db

# Serve MCP locally over stdio
context-kernal serve --database ./data/kernel.db --data-dir ./data

# Serve MCP remotely over SSE
context-kernal serve --config config/context-kernal.sse.example.toml

# Export schema SQL
context-kernal export-schema
```

---

## Configuration

Production-oriented templates live in [config](config/):

- [context-kernal.example.toml](config/context-kernal.example.toml)
- [context-kernal.local.example.toml](config/context-kernal.local.example.toml)
- [context-kernal.sse.example.toml](config/context-kernal.sse.example.toml)

Important sections:

| Section | Purpose |
| --- | --- |
| `[storage]` | SQLite database path |
| `[mcp]` | stdio and SSE transport settings |
| `[auth]` | bearer token configuration |
| `[authorization]` | tool and corpus allowlists |
| `[scheduler]` | periodic source refresh |
| `[embeddings]` | optional semantic retrieval |
| `[reload]` | config reload behavior |
| `[retention]` | GC windows |
| `[observability]` | text or JSON tracing |

Config reload updates the application service and SSE auth/service state. It does not restart the listening socket; bind address changes require a restart.

---

## Observability

Context Kernel emits structured tracing to stderr.

Use JSON logs for machines:

```toml
[observability]
log_format = "json"
```

The config accepts `otel_endpoint`, and the `ck-observability` crate detects it, but OpenTelemetry exporter and metrics backend wiring are not implemented yet. That is the remaining observability gap.

Evidence matters. The README says this plainly because a fake telemetry claim is worse than no telemetry claim.

---

## Architecture

Context Kernel follows ports and adapters.

| Layer | Crates | Purpose |
| --- | --- | --- |
| Domain | `ck-core` | entities, enums, and port traits |
| Application | `ck-app`, `ck-memory` | use cases, memory policy, orchestration |
| Storage | `ck-storage-sqlite`, `ck-artifacts`, `ck-vector` | SQLite persistence, artifact files, vector storage |
| Retrieval | `ck-search-fts`, `ck-ranker`, `ck-embeddings` | FTS search, ranking, optional embeddings |
| Adapters | `ck-adapter-filesystem`, `ck-adapter-github`, `ck-adapter-web` | source discovery, fetching, normalization |
| MCP | `ck-mcp` | tool contracts, aliases, error mapping |
| Transports | `ck-transport-stdio`, `ck-transport-sse` | local and remote serving |
| Runtime | `ck-bin`, `ck-config`, `ck-auth`, `ck-observability`, `ck-scheduler` | CLI, config, auth, tracing, refresh jobs |

The rule is strict: no SQL, HTTP, or transport behavior belongs in `ck-core`.

---

## Status

Implemented:

- single Rust binary
- SQLite schema and migrations
- stdio MCP serving
- SSE MCP serving
- bearer auth and tool/corpus authorization
- local filesystem, GitHub, and web source adapters
- FTS5 retrieval
- optional OpenAI-compatible embeddings
- vector search with scope metadata
- session memory and artifact storage
- retention GC
- config templates
- release workflow
- deployment smoke tests

Known limits:

- OpenTelemetry exporter and metrics backend wiring are not implemented
- PDF adapter is future work
- bundled external documentation dataset adapter is future work
- vector search is brute-force and intended for small corpora
- SSE config reload does not rebind the HTTP listener

This is infrastructure, not theater. If a feature is not wired and tested, it is listed as a limit.

---

## Lineage

Context Kernel is built around three proven MCP patterns:

- current, version-aware documentation for coding agents
- token-efficient reference search and URL reading
- persistent session memory with indexed recall

The implementation is its own Rust service. It keeps the useful parts in one local-first binary and adds deployment concerns the separate tools do not all share: SQLite migrations, source adapters, retention, artifact storage, corpus/tool authorization, SSE serving, release packaging, and repository-grounded status evidence.

---

## Development

Read these first:

- [AGENTS.md](AGENTS.md)
- [.github/copilot-instructions.md](.github/copilot-instructions.md)
- [.agents/project_state.md](.agents/project_state.md)
- [.agents/evidence.md](.agents/evidence.md)

Development rules:

- trust code and tests over documentation intent
- preserve ports-and-adapters boundaries
- use focused tests for behavior changes
- update `.agents/` memory files after meaningful work
- do not report implementation status without repository evidence

Run the full gate before release:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p ck-bin
```

---

## Release

The release workflow builds the Linux binary and publishes a SHA-256 checksum on version tags.

```bash
git tag v0.1.0
git push origin v0.1.0
```

See [release process](docs/release.md).

---

## What Context Kernel Is Not

- It is not a hosted SaaS memory product.
- It is not a prompt template library.
- It is not a PDF ingestion suite.
- It is not a vector database.
- It is not an excuse for agents to skip evidence.

Context Kernel is a local-first context and retrieval service. It gives agents a place to look before they guess.

---

## License

Context Kernel is licensed under the Apache License 2.0. See [LICENSE](LICENSE).
