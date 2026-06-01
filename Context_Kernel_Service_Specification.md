# Context Kernel Service Specification

Status: Draft v1 (language-specific target: Rust; architecture-agnostic where possible)

Purpose: Define a single-binary Rust service that unifies:
- Context7-style up-to-date library/documentation retrieval
- Context Mode-style context persistence, continuity, and context-window minimization
- Ref-style public/private reference retrieval

The system is named **Context Kernel** in this specification.

Context Kernel MUST support:
- local MCP over stdio
- remote MCP over SSE for self-hosted internal deployments
- plugin-based ingestion and retrieval
- ports-and-adapters architecture throughout so components can be swapped later
- SQLite as the required baseline store
- FTS5 as the required lexical retrieval engine
- hybrid ranking with optional embeddings
- single-binary delivery and `.toml` configuration

## Normative Language

The key words `MUST`, `MUST NOT`, `REQUIRED`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, and
`OPTIONAL` in this document are to be interpreted as described in RFC 2119.

`Implementation-defined` means the behavior is part of the implementation contract, but this
specification does not prescribe one universal policy. Implementations MUST document the selected
behavior.

## 1. Problem Statement

Context Kernel is a local-first, self-hostable documentation and context infrastructure service for
LLM tools, coding agents, editors, and CI pipelines.

It solves five operational problems:

- It provides up-to-date, version-aware documentation retrieval so agents do not rely only on stale
  model training data.
- It reduces context-window waste by storing large raw outputs, references, and session artifacts
  outside the prompt and retrieving only relevant slices.
- It preserves session continuity across compaction, restarts, or resumed conversations by storing
  structured events and retrieving relevant prior state on demand.
- It unifies overlapping capabilities from multiple MCP servers into one coherent MCP surface,
  reducing tool redundancy and conflicting behaviors.
- It enforces a modular ports-and-adapters architecture so ingestion sources, storage engines,
  embedding providers, transports, and ranking strategies can be replaced later.

Important boundary:

- Context Kernel is a retrieval, continuity, and indexing service.
- It is not a general workflow engine, browser automation platform, or agent orchestrator.
- It is not required to execute arbitrary external tools on behalf of the model except where an
  explicitly configured adapter/plugin contract requires it.
- It is not required to be byte-for-byte or API-for-API compatible with the upstream TypeScript
  implementations; matching outcomes is sufficient.

## 2. Goals and Non-Goals

### 2.1 Goals

- Provide a single MCP server that serves documentation, references, and continuity/context-recall
  needs.
- Support MCP over stdio locally and MCP over SSE remotely.
- Expose a small, non-redundant, implementation-stable MCP tool surface.
- Support current-source retrieval from public and private inputs through plugins/adapters.
- Persist session memory and context artifacts locally using SQLite.
- Provide lexical retrieval with SQLite FTS5.
- Provide hybrid ranking that can combine lexical, metadata, and optional embedding signals.
- Support one-time indexing and scheduled/on-demand refresh.
- Preserve portability through ports-and-adapters architecture.
- Ship as a single Rust binary with `.toml` configuration.
- Support structured logs and OpenTelemetry-compatible observability.
- Be safe by default for local and trusted internal remote deployments.
- Support Context7-compatible datasets and identifiers when meaningful.

### 2.2 Non-Goals

- Multi-tenant SaaS control plane.
- Billing, quotas by customer, or user-facing account systems.
- Rich web UI as a core requirement.
- Distributed indexing cluster or distributed scheduler.
- Mandatory GPU dependency.
- Mandatory external vector database.
- Public-internet-first hardening profile.
- Browser extension/product-specific editor UI.
- Requiring exact naming or exact request/response parity with upstream TypeScript projects.
- Shipping overlapping duplicate tools when one unified tool can represent the same outcome.

## 3. Architectural Principles

### 3.1 Ports-and-Adapters Requirement

Ports-and-adapters is REQUIRED throughout the implementation.

The implementation MUST isolate stable domain contracts from infrastructure choices. The system
MUST define explicit ports for at least:

- transport
- MCP protocol serving
- configuration loading
- source ingestion
- source refresh scheduling
- document normalization
- chunking
- storage
- lexical search
- embedding generation
- vector similarity search
- hybrid ranking
- session event persistence
- context artifact persistence
- authorization/authentication
- observability export

Adapters MAY be swapped without changing domain-level behavior except where the behavior is
explicitly implementation-defined.

### 3.2 Composition Model

The preferred composition model is:

1. `Domain/Core`
   - stable entities and use cases
2. `Application Services`
   - orchestration of indexing, retrieval, continuity, and MCP handlers
3. `Ports`
   - traits/interfaces for external dependencies
4. `Adapters`
   - SQLite, HTTP, SSE, stdio, GitHub, filesystem, embedding providers, etc.

### 3.3 Single-Authority Domain Rules

The domain layer MUST own:
- identifier normalization
- retrieval semantics
- ranking composition
- context-budget policies
- session continuity semantics
- compatibility mapping for unified MCP tools

Infrastructure adapters MUST NOT redefine these rules independently.

## 4. System Overview

### 4.1 Main Components

1. `Config Loader`
   - Reads `context-kernel.toml` or explicit path.
   - Resolves defaults and environment indirection.
   - Produces typed config.

2. `Kernel Runtime`
   - Owns startup, shutdown, reload, and top-level wiring.
   - Starts transports and background jobs.
   - Coordinates indexing, refresh, and serving.

3. `MCP Service`
   - Exposes tools/resources/prompts over stdio and/or SSE.
   - Maps unified MCP requests to domain use cases.

4. `Source Registry`
   - Registers ingestion adapters/plugins.
   - Routes source definitions to the correct adapter.

5. `Indexer`
   - Fetches source content.
   - Normalizes documents.
   - Chunks content.
   - Writes documents, chunks, metadata, and search indexes.

6. `Retrieval Engine`
   - Resolves library/reference identities.
   - Executes lexical, metadata, and optional semantic retrieval.
   - Applies hybrid ranking.
   - Produces token-efficient result bundles.

7. `Continuity Engine`
   - Persists session events, decisions, summaries, tool artifacts, references, and context anchors.
   - Retrieves prior context using BM25 and optional semantic re-ranking.
   - Supports compaction-safe continuation.

8. `Artifact Store`
   - Stores large raw outputs and structured blobs outside the prompt context.
   - Returns references, excerpts, and summaries.

9. `Refresh Scheduler`
   - Executes one-time, periodic, and on-demand refresh workflows.
   - Tracks source freshness and refresh outcomes.

10. `Observability`
    - Structured logs.
    - Metrics/spans/events with OTEL export support.
    - Health/status surfaces.

### 4.2 External Dependencies

- Local filesystem.
- SQLite with FTS5 enabled. SQLite FTS5 is part of the required baseline storage/search model in
  this specification. ([github.com](https://github.com/mksglu/context-mode/blob/main/README.md?utm_source=openai))
- OPTIONAL embedding provider(s), local or remote.
- OPTIONAL source APIs (GitHub, docs endpoints, package registries, websites, private repos, PDFs).
- OPTIONAL SSE-capable HTTP serving stack.
- OPTIONAL compatible public datasets and source snapshots.

## 5. Unified Product Semantics

Context Kernel combines outcomes of three capability families:

### 5.1 Documentation Retrieval

Context7 publicly describes two key MCP outcomes:
- resolving a library name to a Context7-compatible library ID
- querying docs for a library ID and question/task. ([github.com](https://github.com/mcp/upstash/context7?utm_source=openai))

Context Kernel MUST preserve equivalent outcomes, though tool names MAY be unified.

### 5.2 Context Saving and Continuity

Context Mode publicly describes:
- keeping raw data out of the live context window
- storing session continuity state in SQLite
- indexing session events in FTS5/BM25 for relevant recall. ([github.com](https://github.com/mksglu/context-mode/blob/main/README.md?utm_source=openai))

Context Kernel MUST preserve these outcomes as first-class capabilities.

### 5.3 Public/Private Reference Retrieval

Ref MCP publicly describes:
- token-efficient documentation access
- retrieval from public web/GitHub and private resources
- one-stop documentation/reference access. ([github.com](https://github.com/ref-tools/ref-tools-mcp?utm_source=openai))

Context Kernel MUST preserve these outcomes using a unified retrieval model.

## 6. Core Domain Model

### 6.1 Entities

#### 6.1.1 Source

Represents a configured content source.

Fields:

- `source_id` (string)
  - Stable unique ID within the local installation.
- `kind` (string)
  - Examples: `context7_dataset`, `github_repo`, `filesystem_tree`, `website`, `pdf_bundle`,
    `package_registry`, `session_events`.
- `display_name` (string)
- `enabled` (boolean)
- `refresh_policy` (object)
- `auth_ref` (string or null)
- `adapter_config` (map)

#### 6.1.2 Corpus

Logical grouping of indexed content.

Fields:

- `corpus_id` (string)
- `name` (string)
- `kind` (enum/string)
  - Examples: `library_docs`, `private_docs`, `session_memory`, `reference_archive`
- `visibility` (enum/string)
  - `local`, `shared_internal`
- `source_ids` (list of strings)

#### 6.1.3 Library Identity

Represents a library/document target comparable to Context7 library IDs.

Fields:

- `library_id` (string)
  - Canonical ID, MAY use Context7-compatible slash format when meaningful.
- `aliases` (list of strings)
- `name` (string)
- `provider` (string)
- `version` (string or null)
- `namespace` (string or null)
- `language` (string or null)
- `summary` (string or null)
- `trust_score` (float or null)

#### 6.1.4 Document

Normalized retrievable unit before chunking.

Fields:

- `document_id` (string)
- `corpus_id` (string)
- `source_id` (string)
- `external_id` (string or null)
- `library_id` (string or null)
- `title` (string)
- `uri` (string or null)
- `version` (string or null)
- `mime_type` (string or null)
- `content_text` (string)
- `metadata` (map)
- `checksum` (string)
- `created_at` (timestamp)
- `updated_at` (timestamp)

#### 6.1.5 Chunk

Atomic retrieval unit.

Fields:

- `chunk_id` (string)
- `document_id` (string)
- `corpus_id` (string)
- `ordinal` (integer)
- `text` (string)
- `token_count_estimate` (integer)
- `section_path` (list of strings)
- `heading` (string or null)
- `anchor` (string or null)
- `metadata` (map)
- `checksum` (string)

#### 6.1.6 Retrieval Query

Logical retrieval request.

Fields:

- `query_id` (string)
- `query_text` (string)
- `intent` (enum/string)
  - `resolve_library`, `query_docs`, `search_refs`, `recall_context`, `general`
- `filters` (map)
- `budget` (object or null)
- `session_id` (string or null)

#### 6.1.7 Retrieval Result

Fields:

- `result_id` (string)
- `chunk_id` (string or null)
- `document_id` (string or null)
- `library_id` (string or null)
- `score` (float)
- `score_breakdown` (map)
- `excerpt` (string)
- `citation` (object)
- `metadata` (map)

#### 6.1.8 Session

Fields:

- `session_id` (string)
- `client_id` (string or null)
- `transport` (enum/string)
  - `stdio`, `sse`
- `created_at` (timestamp)
- `updated_at` (timestamp)
- `continuation_mode` (enum/string)
  - `ephemeral`, `resumeable`, `pinned`
- `status` (enum/string)

#### 6.1.9 Session Event

Fields:

- `event_id` (string)
- `session_id` (string)
- `kind` (string)
  - Examples: `tool_call`, `tool_result_ref`, `decision`, `summary`, `error`, `edit`, `git_op`,
    `reference_opened`, `artifact_stored`, `user_preference`
- `text` (string)
- `artifact_id` (string or null)
- `metadata` (map)
- `created_at` (timestamp)

#### 6.1.10 Artifact

Large payload stored out-of-band.

Fields:

- `artifact_id` (string)
- `session_id` (string or null)
- `kind` (string)
- `mime_type` (string or null)
- `storage_uri` (string)
- `size_bytes` (integer)
- `summary` (string or null)
- `checksum` (string)
- `created_at` (timestamp)

#### 6.1.11 Refresh Job

Fields:

- `job_id` (string)
- `source_id` (string)
- `trigger` (enum/string)
  - `startup`, `manual`, `scheduled`, `webhook`, `dependency_change`
- `status` (enum/string)
  - `queued`, `running`, `completed`, `failed`, `cancelled`
- `started_at` (timestamp or null)
- `finished_at` (timestamp or null)
- `error` (string or null)
- `stats` (map)

### 6.2 Stable Identifiers and Normalization Rules

- IDs MUST be stable within one installation.
- `library_id` SHOULD preserve Context7-compatible slash syntax where available because Context7
  explicitly documents slash-form library IDs in its user model. ([github.com](https://github.com/mcp/upstash/context7?utm_source=openai))
- URI normalization MUST strip fragment/query components only when doing so does not destroy
  identity semantics.
- Document checksums MUST be computed on normalized content.
- Session event kinds MUST be normalized strings suitable for filtering and FTS indexing.

## 7. Configuration Specification

### 7.1 File Discovery

Configuration file path precedence:

1. Explicit CLI `--config <path>`
2. `CONTEXT_KERNEL_CONFIG`
3. `./context-kernel.toml`
4. `~/.config/context-kernel/config.toml`

If the file cannot be read, return `missing_config_file` unless all configuration is supplied
through CLI/env and the implementation explicitly supports that mode.

### 7.2 File Format

Config file format is TOML.

Root sections:

- `[server]`
- `[mcp]`
- `[storage]`
- `[indexing]`
- `[retrieval]`
- `[embeddings]`
- `[observability]`
- `[security]`
- `[sources]`
- `[plugins]`
- `[auth]`

Unknown top-level keys SHOULD be ignored for forward compatibility.

### 7.3 Environment Variable Indirection

For designated fields, `$VAR_NAME` indirection MUST be supported.

Environment variables do not globally override TOML values. They apply only when a field explicitly
contains `$VAR_NAME`.

### 7.4 Required Baseline Fields

- `storage.kind = "sqlite"`
- `storage.sqlite.path`
- `mcp.enable_stdio` or `mcp.enable_sse` MUST be true
- `retrieval.default_mode`
- at least one source or one enabled session-memory corpus

### 7.5 Example Logical Schema

```toml
[server]
instance_name = "context-kernel"
data_dir = "~/.local/share/context-kernel"

[mcp]
enable_stdio = true
enable_sse = true

[mcp.sse]
bind = "127.0.0.1:8765"
base_path = "/mcp"
require_auth = true

[storage]
kind = "sqlite"

[storage.sqlite]
path = "~/.local/share/context-kernel/kernel.db"
busy_timeout_ms = 5000

[indexing]
default_chunk_tokens = 800
default_chunk_overlap_tokens = 120
enable_periodic_refresh = true
refresh_interval_secs = 1800

[retrieval]
default_mode = "hybrid"
default_top_k = 8
max_context_tokens = 12000
enable_bm25 = true
enable_semantic = false

[embeddings]
enabled = false
provider = "none"

[observability]
log_level = "info"
enable_otel = true
otel_endpoint = "http://127.0.0.1:4318"

[security]
trust_mode = "internal"
allow_remote_write_tools = false

[sources.context7_public]
kind = "context7_dataset"
enabled = true

[sources.local_repo_docs]
kind = "filesystem_tree"
enabled = true
path = "./docs"
```

## 8. Dynamic Reload Semantics

Dynamic reload is REQUIRED for:
- source definitions
- retrieval settings
- observability settings
- auth references where safe
- scheduling intervals

Dynamic reload is NOT REQUIRED for:
- SQLite schema changes
- HTTP bind address changes
- plugin binary loading strategy
- transport enablement changes

Invalid reloads MUST NOT crash the service. The last known good effective configuration MUST remain
active and an operator-visible error MUST be emitted.

## 9. Ports Specification

The following ports are REQUIRED as conceptual interfaces.

### 9.1 Config Port

Responsibilities:
- load config
- validate config
- watch config for changes

### 9.2 Source Adapter Port

Responsibilities:
- enumerate source units
- fetch source content
- report freshness metadata
- support incremental refresh where possible

### 9.3 Normalizer Port

Responsibilities:
- convert raw source payloads into normalized documents
- preserve source provenance
- extract metadata, headings, anchors, version hints, and identifiers

### 9.4 Chunker Port

Responsibilities:
- split normalized documents into retrieval chunks
- preserve section boundaries when possible
- support deterministic chunk IDs/checksums

### 9.5 Storage Port

Responsibilities:
- persist documents, chunks, metadata, sessions, events, artifacts, jobs
- support transactional writes
- support read paths for retrieval and continuity

### 9.6 Lexical Search Port

Responsibilities:
- index and query chunk/document/session text
- support BM25-like ranking or documented equivalent
- baseline adapter MUST use SQLite FTS5

### 9.7 Embedding Port

Responsibilities:
- generate embeddings for text units
- provide model identity and dimensionality
- support batch generation

### 9.8 Vector Search Port

Responsibilities:
- store/retrieve vector representations
- perform nearest-neighbor queries
- MAY be in SQLite or sidecar store

### 9.9 Ranker Port

Responsibilities:
- combine lexical, semantic, metadata, freshness, and trust signals
- return scored results with score breakdowns

### 9.10 Session Memory Port

Responsibilities:
- persist session events
- query prior context
- store and retrieve continuity summaries and anchors

### 9.11 Artifact Port

Responsibilities:
- persist large raw payloads
- return references, summaries, and excerpts
- enforce retention policies

### 9.12 MCP Transport Port

Responsibilities:
- expose the MCP server over stdio and/or SSE
- translate transport events into MCP request/response handling

### 9.13 Auth Port

Responsibilities:
- authenticate remote clients
- authorize access to corpora/tools
- support local-trust mode

### 9.14 Observability Port

Responsibilities:
- logs
- metrics
- traces/spans/events
- health/status exports

## 10. Adapter Requirements

### 10.1 Required Baseline Adapters

A conforming v1 implementation MUST provide:

- TOML config adapter
- local filesystem adapter
- SQLite storage adapter
- SQLite FTS5 lexical search adapter
- stdio MCP transport adapter
- SSE MCP transport adapter
- structured logging adapter
- OTEL-compatible observability exporter adapter
- session memory adapter backed by SQLite
- artifact adapter backed by filesystem and metadata in SQLite

### 10.2 Recommended Additional Adapters

- GitHub repository adapter
- website crawler/fetch adapter
- PDF ingestion adapter
- Context7-compatible dataset adapter
- embedding adapter for local model or remote API
- optional vector adapter
- optional Honker-style job queue adapter if it materially improves refresh reliability

Note: if a Honker-style adapter is implemented, it MUST remain behind scheduler/job ports and MUST
NOT leak queue-specific semantics into the domain model.

## 11. Storage Specification

### 11.1 Baseline Storage Model

SQLite is REQUIRED.

The implementation MUST use SQLite for:
- metadata
- documents/chunks
- session memory
- refresh jobs
- configuration snapshots or schema state if needed

SQLite FTS5 is REQUIRED for lexical retrieval over:
- document/chunk text
- session events
- selected artifact summaries/excerpts

### 11.2 Artifact Payload Storage

Large payload bodies SHOULD be stored on the filesystem with references in SQLite.

The implementation MAY store smaller payloads inline in SQLite when efficient.

### 11.3 Transactions

Indexing writes for one logical document SHOULD be transactional:
- document row
- chunk rows
- FTS rows
- embeddings/vector metadata if applicable

### 11.4 Migrations

Schema migrations MUST be versioned.

Startup behavior:
- validate schema version
- apply safe forward migrations
- fail startup with operator-visible error on incompatible or partial schema states

## 12. Indexing and Normalization Pipeline

### 12.1 Pipeline Stages

1. discover source units
2. fetch raw payload
3. detect content type
4. normalize into documents
5. canonicalize metadata/identity
6. chunk documents
7. compute checksums
8. write storage/indexes
9. optionally generate embeddings
10. mark refresh state

### 12.2 Normalization Rules

Normalization SHOULD:
- preserve title and hierarchy
- extract headings and anchors
- preserve code blocks
- preserve version and source metadata
- lowercase/filter normalized tags where applicable
- avoid lossy transformations that hurt retrieval fidelity

### 12.3 Deduplication

Deduplication MUST support at least:
- exact duplicate detection via checksum
- document supersession by source/version/uri identity
- chunk replacement on changed parent document

### 12.4 Incremental Refresh

If a source adapter can provide change tokens, ETags, timestamps, commit SHAs, or other incremental
signals, the implementation SHOULD use them.

If not, full refresh is conforming.

## 13. Refresh and Freshness Model

### 13.1 Required Refresh Modes

- one-time indexing
- manual refresh on demand
- scheduled refresh

### 13.2 Recommended Refresh Modes

- refresh-on-startup for stale sources
- webhook/event-driven refresh
- dependency-version-triggered refresh

### 13.3 Freshness Metadata

Each source and document SHOULD track:
- first indexed time
- last attempted refresh
- last successful refresh
- freshness TTL or policy
- upstream revision/version where available

### 13.4 Recommended Policy

At minimum, the default refresh policy SHOULD be at least as good as the existing ecosystems’
typical “fetch current docs when needed” posture by supporting:
- short TTL for high-change web docs
- revision-aware refresh for GitHub or package-based sources
- manual invalidation by library/source ID

This recommendation is an inference from the stated “up-to-date docs” goal of Context7 rather than
a claim about any exact internal implementation detail. ([github.com](https://github.com/mcp/upstash/context7?utm_source=openai))

## 14. Retrieval Model

### 14.1 Retrieval Modes

The service MUST support:

- `lexical`
- `semantic`
- `hybrid`

The default SHOULD be `hybrid`.

### 14.2 Lexical Retrieval

Baseline:
- SQLite FTS5
- BM25 or documented equivalent scoring
- field-aware weighting if practical

### 14.3 Semantic Retrieval

Embeddings are OPTIONAL but first-class.

If semantic retrieval is enabled:
- document/chunk embeddings MUST be generated through the Embedding Port
- a vector search adapter MUST be used
- model identity/version MUST be recorded

### 14.4 Hybrid Ranking

Hybrid ranking is REQUIRED.

It MUST support combining at least:
- lexical score
- metadata prior
- freshness score
- source trust prior
- optional semantic score
- optional session affinity prior

### 14.5 Ranking Transparency

For debugging and evaluation, the ranker SHOULD expose a score breakdown.

### 14.6 Query Intent Handling

Intent-specific logic is REQUIRED.

Examples:
- `resolve_library`: prioritize exact/alias/name matches and canonical IDs
- `query_docs`: prioritize matching library/version and doc sections
- `search_refs`: broaden to public/private sources
- `recall_context`: prioritize session-local events and summaries

## 15. Unified MCP Tool Surface

The MCP tool surface MUST be unified and non-redundant.

### 15.1 Compatibility Principle

Context Kernel SHOULD preserve familiar outcomes from Context7-compatible flows such as resolving a
library ID and querying docs, because Context7 publicly documents those two MCP actions. ([github.com](https://github.com/mcp/upstash/context7?utm_source=openai))

### 15.2 Required Tools

A conforming v1 implementation MUST expose at least these logical tools:

1. `resolve_reference`
   - Purpose: resolve a library, package, repo, document set, or known corpus target into one or
     more canonical IDs.
   - Replaces overlapping “resolve library”/reference lookup behaviors.
   - Inputs:
     - `query` (required)
     - `kind` (optional; e.g. `library`, `repo`, `docs`, `session`, `auto`)
     - `version` (optional)
     - `source_scope` (optional)
   - Outputs:
     - ranked candidates with canonical IDs, aliases, summaries, version/provider info

2. `query_reference`
   - Purpose: retrieve documentation/reference content for a canonical ID or query.
   - Inputs:
     - `canonical_id` (optional if `query` provided)
     - `query` (required)
     - `top_k` (optional)
     - `mode` (`lexical`, `semantic`, `hybrid`, optional)
     - `version` (optional)
     - `filters` (optional)
   - Outputs:
     - ranked excerpts/chunks with citations and metadata

3. `store_context`
   - Purpose: store session facts, summaries, decisions, task state, or large artifacts outside the
     prompt.
   - Inputs:
     - `session_id` (optional depending on MCP session binding)
     - `kind`
     - `text` or `artifact_ref`
     - `metadata` (optional)
   - Outputs:
     - stored reference/ID

4. `recall_context`
   - Purpose: retrieve relevant prior session continuity context.
   - Inputs:
     - `query`
     - `session_id` (optional depending on binding)
     - `top_k` (optional)
     - `scope` (`current`, `recent`, `all`, optional)
   - Outputs:
     - ranked relevant session events/summaries/artifact references

5. `ingest_source`
   - Purpose: one-time or manual ingestion of a configured or ad hoc source where policy allows.
   - Inputs:
     - source definition or source reference
     - refresh mode
   - Outputs:
     - job reference/status

6. `refresh_source`
   - Purpose: refresh configured sources or canonical references.
   - Inputs:
     - `source_id` or `canonical_id`
   - Outputs:
     - job reference/status

7. `get_status`
   - Purpose: return service/index/session status suitable for operators or MCP-aware clients.

### 15.3 Optional Tools

- `list_sources`
- `list_corpora`
- `explain_ranking`
- `evict_context`
- `pin_context`
- `summarize_artifact`

### 15.4 Tool Result Constraints

Tool responses SHOULD be token-efficient:
- prefer structured summaries, excerpts, and references over full raw payloads
- attach artifact references instead of large blobs
- include enough metadata to let a client fetch more if needed

This aligns with Context Mode and Ref MCP public positioning around minimizing context waste and
token-efficient access. ([github.com](https://github.com/mksglu/context-mode/blob/main/README.md?utm_source=openai))

## 16. MCP Transports

### 16.1 stdio

The service MUST support MCP over stdio for local usage.

### 16.2 SSE

The service MUST support MCP over SSE for remote self-hosted internal usage.

### 16.3 SSE Deployment Model

Recommended model:
- standalone HTTP server
- loopback bind by default
- reverse proxy/TLS termination handled externally for production
- auth required for non-loopback or explicitly remote binds

### 16.4 Authentication

For SSE:
- auth SHOULD be enabled by default when binding to anything except loopback-only trusted mode
- supported schemes MAY include bearer token, static secret, or mTLS via external proxy
- the auth mechanism is implementation-defined but MUST be documented

### 16.5 Session Binding

Remote sessions SHOULD have stable `session_id` values so continuity retrieval can be scoped.

## 17. Session Continuity Specification

### 17.1 Principle

Session continuity is REQUIRED.

Large raw data SHOULD NOT be pushed wholesale into the model context if it can instead be:
- stored as an artifact
- summarized
- indexed
- recalled later via targeted retrieval

This reflects Context Mode’s public design philosophy of storing session continuity in SQLite and
retrieving relevant memory with FTS/BM25. ([github.com](https://github.com/mksglu/context-mode/blob/main/README.md?utm_source=openai))

### 17.2 Stored Event Categories

At minimum:
- user goal
- task state
- decisions
- file/repo references
- tool calls and summarized outputs
- errors
- summaries
- pinned facts/preferences
- artifact references

### 17.3 Continuation Semantics

The implementation MUST support:
- ephemeral sessions
- resumeable sessions
- pinned continuity entries

### 17.4 Expiration and Retention

Defaults MAY differ by continuity mode:
- ephemeral: delete on session end or short TTL
- resumeable: retain until configured TTL
- pinned: retain until explicit removal

### 17.5 Privacy

By default, session continuity data MUST remain local unless the operator explicitly configures a
shared/internal deployment.

## 18. Source and Plugin Model

### 18.1 Plugin Types

The system SHOULD support plugin/adapters for:
- public docs datasets
- repo/docs crawlers
- filesystem content
- PDF/document bundles
- package/version metadata
- private reference stores
- session memory corpora

### 18.2 Context7-Compatible Dataset Support

If a source contains Context7-compatible library IDs or equivalent metadata, the implementation
SHOULD preserve and expose them.

This follows from Context7’s documented user-facing ID model. ([github.com](https://github.com/mcp/upstash/context7?utm_source=openai))

### 18.3 Plugin Lifecycle

A plugin MAY support:
- config validation
- discovery
- fetch
- refresh
- deletion/eviction
- health reporting

### 18.4 Fault Isolation

A plugin failure MUST NOT crash the entire service.
- index/refresh failure -> job failed
- source unavailable -> retrieval may degrade gracefully
- plugin init failure -> plugin disabled with operator-visible error

## 19. Security and Safety

### 19.1 Trust Boundary

The target deployment is:
- trusted local use
- self-hosted internal remote use

The service is not required to be safe for arbitrary public internet exposure by default.

### 19.2 Required Safety Controls

- local filesystem path validation
- source allowlists/denylists where applicable
- remote auth for SSE unless trusted loopback mode
- output size truncation/summarization for artifacts/logging
- secret redaction in logs
- safe default binds

### 19.3 SSRF and Fetch Safety

Remote-fetching adapters SHOULD:
- restrict local network access by default when fetching arbitrary URLs
- support allowlists for internal/private hosts
- validate schemes
- reject dangerous or unsupported schemes

### 19.4 Secret Handling

- support `$VAR` indirection
- do not log secrets
- validate secret presence without printing values

## 20. Observability Specification

### 20.1 Logging

Structured logs are REQUIRED.

Recommended fields:
- `component`
- `operation`
- `session_id`
- `source_id`
- `corpus_id`
- `job_id`
- `query_id`
- `duration_ms`
- `outcome`
- `error`

### 20.2 OTEL

An OpenTelemetry-compatible export path is REQUIRED.

This MAY include:
- OTLP HTTP
- OTLP gRPC
- tracing spans
- metrics
- event logs

### 20.3 Status Surface

A human-readable status endpoint is OPTIONAL.

### 20.4 Health Endpoints

If SSE/HTTP server is enabled, the implementation SHOULD expose:
- `/healthz`
- `/readyz`
- `/metrics` or OTEL-compatible integration endpoint as configured
- OPTIONAL `/api/v1/status`

## 21. Failure Model and Recovery

### 21.1 Failure Classes

1. `Config Failures`
   - missing file
   - parse error
   - invalid values

2. `Storage Failures`
   - SQLite unavailable
   - migration failure
   - disk full

3. `Source Failures`
   - fetch error
   - auth error
   - unsupported payload
   - refresh timeout

4. `Retrieval Failures`
   - FTS query error
   - embedding provider unavailable
   - vector store unavailable
   - ranker failure

5. `Session/Artifact Failures`
   - artifact write/read failure
   - session persistence error

6. `Transport Failures`
   - stdio protocol failure
   - SSE bind/auth/stream failure

7. `Observability Failures`
   - OTEL exporter unavailable
   - log sink failure

### 21.2 Recovery Behavior

- Config reload failure:
  - keep last known good config
- Source refresh failure:
  - mark source degraded
  - retain previous indexed content
- Embedding subsystem failure:
  - degrade to lexical/hybrid-without-semantic where possible
- OTEL failure:
  - continue service with local logs
- SSE failure:
  - stdio mode MAY continue independently

### 21.3 Restart Recovery

On restart, the service MUST be able to recover from persisted state:
- SQLite-backed metadata
- indexed corpora
- session memory according to retention
- refresh history

In-flight refresh jobs need not survive process restart unless a job adapter explicitly supports it.

## 22. Reference Algorithms

### 22.1 Service Startup

```text
function start_service():
  config = load_config()
  validate_config(config)

  storage = init_storage(config)
  run_migrations(storage)

  ports = wire_ports_and_adapters(config, storage)

  load_enabled_plugins()
  recover_persisted_state()

  if config.indexing.refresh_on_startup:
    enqueue_stale_refresh_jobs()

  start_stdio_mcp_if_enabled()
  start_sse_mcp_if_enabled()
  start_observability()

  event_loop()
```

### 22.2 Index One Source

```text
function index_source(source_id):
  source = source_registry.get(source_id)
  units = source.discover_units()

  for unit in units:
    raw = source.fetch(unit)
    docs = normalizer.normalize(raw, source)

    for doc in docs:
      chunks = chunker.chunk(doc)
      begin_tx()

      storage.upsert_document(doc)
      storage.replace_chunks(doc.document_id, chunks)
      lexical_index.upsert(doc, chunks)

      if embeddings.enabled:
        vectors = embedding_port.embed(chunks.texts)
        vector_index.upsert(chunks, vectors)

      commit_tx()

  mark_refresh_success(source_id)
```

### 22.3 Resolve Reference

```text
function resolve_reference(query, kind, version, scope):
  candidates = identity_catalog.lookup(query, kind, scope)

  ranked = ranker.rank_resolution_candidates(
    query=query,
    candidates=candidates,
    version_hint=version
  )

  return top(ranked)
```

### 22.4 Query Reference

```text
function query_reference(query, canonical_id, mode, filters, top_k):
  target_set = corpus_selector.select(canonical_id, filters)

  lexical = lexical_index.search(query, target_set, top_k * 5)

  semantic = []
  if mode in [semantic, hybrid] and embeddings.enabled:
    qv = embedding_port.embed_query(query)
    semantic = vector_index.search(qv, target_set, top_k * 5)

  ranked = ranker.combine(
    query=query,
    lexical=lexical,
    semantic=semantic,
    filters=filters,
    freshness=source_freshness(target_set),
    trust=source_trust(target_set)
  )

  return truncate_for_budget(ranked, top_k)
```

### 22.5 Store Context

```text
function store_context(session_id, kind, text_or_artifact, metadata):
  if payload_is_large(text_or_artifact):
    artifact = artifact_store.put(session_id, text_or_artifact)
    summary = summarize_or_extract(text_or_artifact)
    event = session_memory.add_event(session_id, kind, summary, artifact.id, metadata)
  else:
    event = session_memory.add_event(session_id, kind, text_or_artifact, null, metadata)

  lexical_index.upsert_session_event(event)

  if embeddings.enabled and should_embed_event(event):
    vector_index.upsert_session_event(event)

  return event
```

### 22.6 Recall Context

```text
function recall_context(session_id, query, scope, top_k):
  events = session_memory.search(session_id, query, scope, top_k * 5)

  if embeddings.enabled:
    rerank = semantic_rerank_if_enabled(events, query)
  else:
    rerank = events

  return top(rerank, top_k)
```

## 23. API and Status Extension (OPTIONAL)

If the SSE/HTTP server is implemented beyond the MCP endpoint, the implementation SHOULD provide:

- `GET /healthz`
- `GET /readyz`
- `GET /api/v1/status`
- `GET /api/v1/sources`
- `POST /api/v1/refresh/:source_id`

These endpoints are OPTIONAL and MUST NOT be required for MCP correctness.

## 24. Compatibility and Naming Guidance

### 24.1 Upstream Outcome Compatibility

The implementation SHOULD aim for outcome compatibility with:
- Context7 docs resolution/query flows
- Context Mode continuity/storage/retrieval flows
- Ref MCP public/private reference retrieval flows

This is based on the public descriptions and exposed tools of those projects, not an assertion of
their private internals. ([github.com](https://github.com/mcp/upstash/context7?utm_source=openai))

### 24.2 Unified Naming

A conforming implementation MAY expose compatibility aliases for user familiarity, such as mapping:
- `resolve-library-id` -> `resolve_reference`
- `query-docs` -> `query_reference`

because Context7 publicly documents those names. ([github.com](https://github.com/mcp/upstash/context7?utm_source=openai))

## 25. Test and Validation Matrix

A conforming implementation SHOULD include tests that cover the behaviors defined in this
specification.

### 25.1 Config and Startup

- config path precedence
- TOML parse errors
- `$VAR` resolution
- invalid reload keeps last known good config
- migration behavior

### 25.2 Ports-and-Adapters Conformance

- domain logic can be tested without real adapters
- adapter substitution does not change domain semantics unexpectedly
- plugin failures are isolated

### 25.3 Storage and Indexing

- SQLite schema initializes correctly
- FTS5 indexes document/chunk/session text
- document deduplication works
- chunk replacement on update works
- artifact references remain valid
- transaction rollback preserves consistency

### 25.4 Retrieval

- lexical search returns expected hits
- semantic retrieval is skipped safely when disabled
- hybrid ranking combines signals deterministically
- version hints influence result ranking
- score breakdowns are available when enabled

### 25.5 Unified MCP Tools

- `resolve_reference` returns canonical IDs
- `query_reference` returns ranked excerpts/citations
- `store_context` persists events and large artifacts appropriately
- `recall_context` returns relevant prior session information
- `ingest_source` and `refresh_source` create jobs/status

### 25.6 Continuity

- compaction-safe recall works from persisted events
- retention policies delete ephemeral state correctly
- pinned entries survive ordinary cleanup
- session-local search prefers current session over unrelated history

### 25.7 Transports

- stdio MCP startup and request flow
- SSE bind/auth/stream behavior
- loopback default bind
- auth enforcement for remote mode

### 25.8 Security

- secrets never appear in logs
- unsafe fetch schemes are rejected
- filesystem path traversal is prevented
- unauthorized SSE requests are denied

### 25.9 Observability

- structured logs include required context
- OTEL exporter failure does not crash service
- health/readiness endpoints reflect state if implemented

## 26. Implementation Checklist (Definition of Done)

### 26.1 REQUIRED for Conformance

- single Rust binary
- TOML config
- ports-and-adapters architecture across major subsystems
- SQLite baseline storage
- SQLite FTS5 lexical retrieval
- MCP over stdio
- MCP over SSE
- unified MCP tool surface
- one-time/manual/scheduled refresh
- hybrid ranking
- session continuity persistence and recall
- artifact storage outside prompt context
- structured logging
- OTEL-compatible observability export
- safe local/internal deployment defaults

### 26.2 RECOMMENDED Extensions

- Context7-compatible dataset adapter
- GitHub repo/docs adapter
- PDF ingestion adapter
- optional embeddings provider(s)
- optional vector search adapter
- compatibility aliases for familiar tool names
- admin/status API
- webhook-triggered refresh
- Honker-style job adapter if useful and well-isolated

### 26.3 Out of Scope for v1

- multi-tenant SaaS
- billing
- rich mandatory web UI
- distributed cluster
- GPU requirement
- cloud-managed dependencies as a requirement

## Appendix A. Recommended Rust Module Boundaries

Suggested crate/module layout:

- `context-kernel-core`
  - domain entities
  - policies
  - ports
- `context-kernel-app`
  - use cases
  - orchestration
- `context-kernel-adapters-sqlite`
- `context-kernel-adapters-filesystem`
- `context-kernel-adapters-http`
- `context-kernel-adapters-mcp`
- `context-kernel-adapters-embeddings`
- `context-kernel-adapters-sources`
- `context-kernel-bin`

A single workspace MAY compile into one final binary while retaining these separations.

## Appendix B. Recommended Default Behavior

- local-first install
- loopback-only SSE bind by default
- auth required when not strictly local
- hybrid retrieval on by default
- semantic retrieval disabled until embeddings are configured
- periodic refresh enabled
- ephemeral session continuity by default with explicit resume modes
- artifact truncation and summarization enabled