# Domain Concepts & Vocabulary

This guide defines the core domain concepts and terminology used throughout Context Kernel. It clarifies what each concept means, what it deliberately does not mean, how it relates to neighboring concepts, and where it is implemented in the codebase.

---

## 1. Terminology Index

| Term | Category | Primary Code Representation | Storage Location |
| :--- | :--- | :--- | :--- |
| [Source](#source) | Ingestion | `ck_core::entities::Source` | `sources` table |
| [Corpus](#corpus) | Retrieval / Security | `ck_core::entities::Corpus` | `corpora`, `corpus_sources` tables |
| [Document](#document) | Content | `ck_core::entities::Document` | `documents` table |
| [Chunk](#chunk) | Retrieval | `ck_core::entities::Chunk` | `chunks`, `chunks_fts` tables |
| [Library Identity](#library-identity) | Resolution | `ck_core::entities::LibraryIdentity` | `library_identities` table |
| [Library Alias](#library-alias) | Resolution | `ck_core::entities::LibraryAlias` | `library_aliases`, `library_aliases_fts` tables |
| [Session](#session) | Memory | `ck_core::entities::Session` | `sessions` table |
| [Session Event](#session-event) | Memory | `ck_core::entities::SessionEvent` | `session_events`, `session_events_fts` tables |
| [Artifact](#artifact) | Storage | `ck_core::entities::Artifact` | `artifacts` table + `<data_dir>/artifacts/*.bin` |
| [Refresh Job](#refresh-job) | Background Task | `ck_core::entities::RefreshJob` | `refresh_jobs` table |
| [Context Packet](#context-packet) | Convergence | `ck_mcp::agentic_capability_loop::ContextPacket` | In-flight MCP JSON payload |
| [Query Intent](#query-intent) | Ranking | `ck_core::types::QueryIntent` | Used at runtime by `ck-ranker` |
| [Event Retention Class](#event-retention-class) | Memory | `ck_memory::EventRetentionClass` | Evaluated during retention GC |

---

## 2. Concept Definitions

### Source
* **What it is**: An operational origin of external documentation. A source encapsulates a specific provider kind (e.g. `filesystem`, `github`, or `website`), a configuration JSON blob (e.g. root folder, owner/repo, or URLs), a freshness time-to-live (`freshness_ttl_secs`), and an upstream revision tracking string.
* **What it is not**: A source is not a document or chunk. It is the persistent generator of documents.
* **Relationships**: A source is associated with one or more corpora via the `corpus_sources` junction table. When refreshed, a source replaces its existing documents.
* **Source Location**: Defined in [`crates/ck-core/src/entities/source.rs`](../crates/ck-core/src/entities/source.rs#L8-L25); stored in SQLite `sources` table.

### Corpus
* **What it is**: A named administrative boundary used to group sources and scope retrieval queries. Corpora can have public or private visibility and serve as the enforcement target for authorization policies (`allow_corpora`).
* **What it is not**: A corpus is not a physical database file or folder. It is a logical partition across documents.
* **Relationships**: Groups sources through `corpus_sources`. Queries specifying a `source_scope` restrict search results to chunks belonging to those corpus IDs.
* **Source Location**: Defined in [`crates/ck-core/src/entities/corpus.rs`](../crates/ck-core/src/entities/corpus.rs#L6-L15); stored in SQLite `corpora` table.

### Document
* **What it is**: A single cohesive content unit ingested from a source (e.g. a Markdown file, text file, or scraped web page). It records the originating URI, MIME type, version, raw content, metadata JSON, and a SHA-256 checksum.
* **What it is not**: A document is not directly searched by the lexical engine. Search operates on the smaller *chunks* generated from the document.
* **Relationships**: Belongs to exactly one `Source` and one `Corpus`. Optionally linked to a `LibraryIdentity`. Split into one or more `Chunks`.
* **Source Location**: Defined in [`crates/ck-core/src/entities/document.rs`](../crates/ck-core/src/entities/document.rs#L8-L26); stored in SQLite `documents` table.

### Chunk
* **What it is**: The fundamental unit of retrieval in Context Kernel. A chunk represents an extract of a document partitioned along heading boundaries or token limits (maximum ~800 words). It contains the chunk text, section heading, anchor, and estimated token count.
* **What it is not**: A chunk is not an independent document. It carries no independent provenance other than its relationship to its parent document.
* **Relationships**: Belongs to one `Document` and `Corpus`. Indexed in SQLite FTS5 (`chunks_fts`) for BM25 ranking and optionally in `chunk_embeddings` for vector search.
* **Source Location**: Defined in [`crates/ck-core/src/entities/chunk.rs`](../crates/ck-core/src/entities/chunk.rs#L8-L23); stored in SQLite `chunks` and `chunks_fts` tables.

### Library Identity
* **What it is**: A canonical package or framework descriptor (e.g. `provider="crates.io"`, `name="tokio"`, `version="1.38.0"`, `language="rust"`). Carries a `trust_score` between 0.0 and 1.0 used during search ranking.
* **What it is not**: A library identity is not an arbitrary tag. It represents an authoritative release identity in an ecosystem.
* **Relationships**: Has one or more `LibraryAlias` entries. Associated with `Document` records.
* **Source Location**: Defined in [`crates/ck-core/src/entities/library.rs`](../crates/ck-core/src/entities/library.rs#L6-L20); stored in SQLite `library_identities` table.

### Library Alias
* **What it is**: An alternative name, abbreviation, or colloquial term mapped to a canonical `LibraryIdentity` (e.g., mapping `"tokio-rs"` or `"tokio_core"` to canonical `"tokio"`).
* **What it is not**: An alias does not store documentation; it is purely a lookup pointer.
* **Relationships**: Linked to a `LibraryIdentity` with foreign key cascade delete. Indexed in `library_aliases_fts` for full-text lookup during `resolve_reference`.
* **Source Location**: Defined in [`crates/ck-core/src/entities/library.rs`](../crates/ck-core/src/entities/library.rs#L22-L28); stored in SQLite `library_aliases` and `library_aliases_fts` tables.

### Session
* **What it is**: A durable conversational or task context representing an ongoing interaction with an agent. Characterized by a `client_id`, `transport` (`stdio` or `sse`), `continuation_mode` (`ephemeral`, `resumeable`, or `pinned`), and `status` (`active`, `closed`, `compacted`).
* **What it is not**: A session is not an ephemeral in-memory socket connection. It is a persistent database record.
* **Relationships**: Contains an ordered sequence of `SessionEvent` records.
* **Source Location**: Defined in [`crates/ck-core/src/entities/session.rs`](../crates/ck-core/src/entities/session.rs#L6-L16); stored in SQLite `sessions` table.

### Session Event
* **What it is**: A timestamped memory record within a session. Captures semantic interaction milestones categorized by `kind`: `"decision"`, `"architecture"`, `"requirement"`, `"constraint"`, `"milestone"`, or `"note"`.
* **What it is not**: A session event is not a raw chat transcript dump. It is a curated memory entry.
* **Relationships**: Belongs to a `Session`. If the event payload exceeds inline text limits or is binary, it offloads content to an `Artifact` and stores the `artifact_id`. Indexed in `session_events_fts`.
* **Source Location**: Defined in [`crates/ck-core/src/entities/session.rs`](../crates/ck-core/src/entities/session.rs#L18-L29); stored in SQLite `session_events` and `session_events_fts` tables.

### Artifact
* **What it is**: A content-addressed filesystem payload file stored in `<data_dir>/artifacts/<artifact_id>.bin`, accompanied by `<artifact_id>.meta.json` and a database metadata record. Used to store large outputs, logs, code diffs, or images outside the agent's prompt context.
* **What it is not**: An artifact is not stored as a BLOB inside the SQLite database file. SQLite only stores the artifact metadata.
* **Relationships**: Referenced by `SessionEvent` via `artifact_id`. Purged during retention GC unless referenced by active/pinned sessions.
* **Source Location**: Defined in [`crates/ck-core/src/entities/artifact.rs`](../crates/ck-core/src/entities/artifact.rs#L6-L18); managed by [`crates/ck-artifacts/src/lib.rs`](../crates/ck-artifacts/src/lib.rs).

### Refresh Job
* **What it is**: A tracked background task representing the re-indexing of a `Source`. Contains status (`queued`, `running`, `completed`, `failed`, `cancelled`), trigger kind (`startup`, `manual`, `scheduled`, `webhook`, `dependency_change`), and execution statistics.
* **What it is not**: An unconstrained worker queue. Context Kernel enforces a partial unique index (`idx_refresh_jobs_one_active_per_source`) ensuring that at most **one** active (queued or running) job exists per source at any time. New refresh requests are coalesced into the existing active job.
* **Relationships**: Belongs to a `Source`.
* **Source Location**: Defined in [`crates/ck-core/src/entities/job.rs`](../crates/ck-core/src/entities/job.rs#L6-L20); stored in SQLite `refresh_jobs` table.

### Context Packet
* **What it is**: A bounded, citation-bearing JSON bundle assembled by Context Kernel to satisfy a `ContextRequired` request on the E2/E3 boundary with SWE_SEED. Contains a `context_packet_id`, `corpus_id`, citations with line and heading anchors, and pass-through authority references.
* **What it is not**: A prompt or a complete file dump. Context packets are strictly bounded relevance packages.
* **Relationships**: Emitted by `ck-mcp::AclContextAgent` in response to an authoritative `ContextRequired` event produced by `swe_seed`.
* **Source Location**: Defined in [`crates/ck-mcp/src/agentic_capability_loop.rs`](../crates/ck-mcp/src/agentic_capability_loop.rs).

---

## 3. Enums & Core Classifications

### Query Intent (`ck_core::types::QueryIntent`)
Determines the dynamic weighting profile applied by `ck-ranker` during search:
* `ResolveLibrary`: Focuses on exact alias and library identity matches (FTS: 0.5, Trust: 0.3, Freshness: 0.1, Alias Exact: 0.1).
* `QueryDocs`: Balances lexical FTS with documentation freshness and semantic similarity (FTS: 0.6, Trust: 0.15, Freshness: 0.15, Semantic: 0.1).
* `SearchRefs`: Prioritizes freshness and semantic matching across broad references (FTS: 0.5, Trust: 0.1, Freshness: 0.2, Semantic: 0.2).
* `RecallContext`: Prioritizes session affinity to boost events from the active session (FTS: 0.4, Trust: 0.1, Freshness: 0.2, Session Affinity: 0.3).
* `General`: Default balanced lexical search (FTS: 0.7, Trust: 0.2, Freshness: 0.1).

### Event Retention Class (`ck_memory::EventRetentionClass`)
Governs whether an event is preserved across retention GC sweeps:
* `Permanent`: Retained indefinitely. Includes events with kind `"pinned"` or `"milestone"`.
* `Pinnable`: Eligible for pinning. Defaults to ephemeral unless explicitly pinned by an operator or agent. Includes `"decision"`, `"architecture"`, `"requirement"`, and `"constraint"`.
* `Ephemeral`: Purged after `session_event_days` (default: 90 days). Includes `"note"` and generic agent dialogue.

### Continuation Mode (`ck_core::types::ContinuationMode`)
* `Ephemeral`: The session is temporary and expected to be cleaned up after inactivity.
* `Resumeable`: The session timeline can be reopened and extended in future agent runs.
* `Pinned`: The session and its events are protected against automated retention cleanup.
