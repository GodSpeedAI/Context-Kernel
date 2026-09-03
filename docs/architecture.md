# Architecture Overview

This document specifies the canonical architecture of Context Kernel. It presents the logical, runtime, dependency, data, control flow, and security architectures that govern the implementation.

---

## 1. Logical Architecture (Ports-and-Adapters)

Context Kernel strictly adheres to the **ports-and-adapters (hexagonal) architecture** pattern. The domain core defines entity structures and abstract port traits. Infrastructure concerns (SQLite, FTS5, HTTP, filesystem IO, external APIs) exist strictly as pluggable adapters.

```mermaid
graph TB
    subgraph Drivers["Driving Adapters (Inbound)"]
        StdioTransport["ck-transport-stdio\n(JSON-RPC line parser)"]
        SseTransport["ck-transport-sse\n(Axum HTTP server)"]
        CLI["ck-bin\n(Clap CLI subcommands)"]
    end

    subgraph PortsIn["Inbound Protocol Surface"]
        McpService["ck-mcp::McpService\n(Canonical Tools & Aliases)"]
        ContextAgent["ck-mcp::AclContextAgent\n(E2/E3 Canonical Ingress)"]
    end

    subgraph Application["Application Layer"]
        AppServices["ck-app::AppServices\n(Use-Case Orchestration)"]
        MemoryPolicy["ck-memory\n(Retention Classifier)"]
    end

    subgraph Core["Domain Core (ck-core)"]
        Entities["Entities: Source, Corpus, Doc, Chunk, Session, Event, Artifact"]
        Ports["Port Traits: StoragePort, SessionMemoryPort, LexicalSearchPort,\nRankerPort, ArtifactPort, SourceAdapter, EmbeddingPort, VectorSearchPort"]
    end

    subgraph Driven["Driven Adapters (Outbound)"]
        Sqlite["ck-storage-sqlite\n(SqliteStore)"]
        Fts["ck-search-fts\n(FtsSearch FTS5)"]
        Ranker["ck-ranker\n(LexicalRanker)"]
        ArtifactStore["ck-artifacts\n(FileArtifactStore)"]
        VectorStore["ck-vector\n(SqliteVectorStore)"]
        Embedder["ck-embeddings\n(OpenAI / Noop)"]
        FSAdapter["ck-adapter-filesystem\n(FilesystemSourceAdapter)"]
        GHAdapter["ck-adapter-github\n(GitHubSourceAdapter) [unwired]"]
        WebAdapter["ck-adapter-web\n(WebSourceAdapter) [unwired]"]
    end

    StdioTransport --> McpService
    SseTransport --> McpService
    CLI --> AppServices
    McpService --> AppServices
    McpService --> ContextAgent

    AppServices --> Core
    AppServices --> MemoryPolicy
    ContextAgent --> Core

    AppServices -.->|invokes port| Ports
    Sqlite -.->|implements| Ports
    Fts -.->|implements| Ports
    Ranker -.->|implements| Ports
    ArtifactStore -.->|implements| Ports
    VectorStore -.->|implements| Ports
    Embedder -.->|implements| Ports
    FSAdapter -.->|implements| Ports
    GHAdapter -.->|implements| Ports
    WebAdapter -.->|implements| Ports
```

### Architectural Rules
1. **No Infrastructure Leaks in `ck-core`**: `ck-core` contains zero SQL statements, zero HTTP dependencies, and zero filesystem calls. It depends only on standard libraries, `anyhow`, `chrono`, `serde`, and `ulid`.
2. **Ports Own the Contracts**: Driving callers (e.g. `AppServices`) hold `Box<dyn Port>` traits. Concrete storage or network crates never call each other directly; they communicate across domain entity boundaries.
3. **Composition at the Edge**: Concrete adapter instantiation and trait object wiring happens exclusively in the composition root (`crates/ck-bin/src/main.rs::wire_app`).

---

## 2. Runtime & Process Topology

Context Kernel executes as a **single, standalone Linux/Unix or Windows binary** named `context-kernal`. It requires no external background daemon, database server, or container runtime.

```mermaid
flowchart TD
    subgraph Process["context-kernal OS Process"]
        subgraph MainThread["Main Thread"]
            Bootstrap["Bootstrap & CLI Dispatch\n(clap parse, open_store, migrate)"]
            InitTracing["init_logging()\n(Global Tracing Subscriber)"]
            RetSweep["Startup Retention GC\n(app.run_retention_gc)"]
            StdioLoop["Stdio Serve Loop\n(reads lines from stdin, calls handle_line)"]
        end

        subgraph TokioThread["Tokio Worker Thread (When SSE Enabled)"]
            TcpBind["Pre-bound std::net::TcpListener\n(bound on main thread before spawning)"]
            AxumServer["Axum HTTP Server\n(/mcp, /healthz, /readyz, /api/v1/status)"]
        end

        subgraph RefreshThread["Background Refresh Thread (When Enabled)"]
            RefreshLoop["Periodic Refresh Loop\n(sleeps refresh_interval_secs, calls refresh_source)"]
        end

        subgraph LocalState["Local Storage & IO Boundaries"]
            SQLiteFile[("kernel.db (SQLite WAL mode)")]
            ArtifactDir[("<data_dir>/artifacts/*.bin")]
        end
    end

    Bootstrap --> InitTracing
    InitTracing --> RetSweep
    RetSweep --> StdioLoop

    Bootstrap -.->|spawns| TokioThread
    Bootstrap -.->|spawns| RefreshThread

    StdioLoop -->|rusqlite queries| SQLiteFile
    StdioLoop -->|read/write payloads| ArtifactDir

    AxumServer -->|McpService read lock| StdioLoop
    AxumServer -->|rusqlite queries| SQLiteFile

    RefreshLoop -->|list_stale_sources & refresh| SQLiteFile
```

### Key Operational Properties
* **Deterministic Fail-Early Bind**: The TCP listener for SSE (`std::net::TcpListener::bind(sse_bind)`) is executed on the **main thread** before spawning the Tokio runtime thread. If the port is occupied or permission is denied, the process immediately aborts with an exit code, avoiding "silent headless zombie" execution.
* **Synchronous SQLite Engine**: Storage operations use synchronous `rusqlite` connections protected by SQLite's internal locking and WAL (`Write-Ahead Logging`) concurrency mode.
* **Independent Background Workers**: The periodic refresh worker runs as an OS thread, polling `list_stale_sources` at the configured interval (`scheduler.refresh_interval_secs`).

---

## 3. Dependency Architecture

The workspace consists of 20 crates organized in a strictly unidirectional dependency tree. No cyclic dependencies exist.

```mermaid
graph BT
    ck-core["ck-core (Domain & Ports)"]

    ck-config["ck-config"]
    ck-memory["ck-memory"]
    ck-observability["ck-observability"]
    ck-auth["ck-auth"]

    ck-storage-sqlite["ck-storage-sqlite"]
    ck-search-fts["ck-search-fts"]
    ck-ranker["ck-ranker"]
    ck-artifacts["ck-artifacts"]
    ck-vector["ck-vector"]
    ck-embeddings["ck-embeddings"]
    ck-adapter-filesystem["ck-adapter-filesystem"]
    ck-adapter-github["ck-adapter-github"]
    ck-adapter-web["ck-adapter-web"]
    ck-scheduler["ck-scheduler"]

    ck-app["ck-app"]
    ck-mcp["ck-mcp"]
    ck-transport-stdio["ck-transport-stdio"]
    ck-transport-sse["ck-transport-sse"]
    ck-bin["ck-bin (Application Root)"]

    ck-memory --> ck-core
    ck-auth --> ck-core
    ck-storage-sqlite --> ck-core
    ck-search-fts --> ck-core
    ck-ranker --> ck-core
    ck-artifacts --> ck-core
    ck-vector --> ck-core
    ck-embeddings --> ck-core
    ck-adapter-filesystem --> ck-core
    ck-adapter-github --> ck-core
    ck-adapter-web --> ck-core
    ck-scheduler --> ck-core

    ck-app --> ck-core
    ck-app --> ck-memory

    ck-mcp --> ck-core
    ck-mcp --> ck-app

    ck-transport-stdio --> ck-mcp
    ck-transport-sse --> ck-core
    ck-transport-sse --> ck-mcp

    ck-bin --> ck-core
    ck-bin --> ck-config
    ck-bin --> ck-app
    ck-bin --> ck-mcp
    ck-bin --> ck-auth
    ck-bin --> ck-observability
    ck-bin --> ck-storage-sqlite
    ck-bin --> ck-search-fts
    ck-bin --> ck-ranker
    ck-bin --> ck-artifacts
    ck-bin --> ck-vector
    ck-bin --> ck-embeddings
    ck-bin --> ck-adapter-filesystem
    ck-bin --> ck-transport-stdio
    ck-bin --> ck-transport-sse
```

### Dependency Observations
* `ck-bin` is the composition root. It depends on all functional adapters to wire them into `ck-app::AppServices`.
* Notice that `ck-adapter-github`, `ck-adapter-web`, and `ck-scheduler` compile and pass tests within the workspace, but are **deliberately not imported** by `ck-bin/Cargo.toml`. They exist as fully implemented library crates pending runtime composition.

---

## 4. Data Architecture & Storage Pipelines

Context Kernel moves data through two primary pipelines: Ingestion and Retrieval.

### Ingestion Pipeline (External Sources → Chunks)

```mermaid
flowchart LR
    SourceData["Raw Source (Filesystem / Git / Web)"] -->|SourceAdapter::discover_units| Units["SourceUnits"]
    Units -->|SourceAdapter::fetch_content| Raw["RawContent"]
    Raw -->|Normalizer::normalize| Clean["Clean Document Text"]
    Clean -->|Chunker::chunk| Chunks["Text Chunks (~800 tokens)"]
    Chunks -->|StoragePort::replace_chunks| SQLiteDB[("SQLite: documents & chunks")]
    Chunks -->|LexicalSearchPort::index_chunk| FTS5Table[("SQLite FTS5: chunks_fts")]
    Chunks -->|Optional EmbeddingPort::embed| VectorDB[("SQLite: chunk_embeddings")]
```

### Retrieval Pipeline (Query → Scored Excerpts)

```mermaid
flowchart LR
    AgentQuery["Agent Query String"] --> SearchBranch

    subgraph SearchBranch["Parallel Retrieval"]
        FTSQuery["chunks_fts (BM25 lexical)"]
        VectorQuery["chunk_embeddings (Cosine vector)"]
    end

    FTSQuery --> RawScores["Raw Lexical Scores (-raw_score negated)"]
    VectorQuery --> VectorScores["Raw Cosine Scores (0.0 to 1.0)"]

    RawScores --> Ranker["ck-ranker::LexicalRanker"]
    VectorScores --> Ranker

    Ranker -->|QueryIntent Weight Profile| HybridScores["Combined Weighted Scores"]
    HybridScores -->|De-duplication| TopChunks["Top K Document Chunks"]
    TopChunks -->|StoragePort::get_chunk_with_document| Citations["Formatted Citations + Excerpts"]
```

---

## 5. Control Flow & Hot Config Reload

In Stdio mode, Context Kernel can detect on-disk configuration changes on every incoming JSON-RPC line without dropping connection state.

```mermaid
sequenceDiagram
    participant Stdin as Stdin Loop (ck-bin)
    participant Config as Config File (TOML)
    participant SseState as ReloadableSseState (RwLock)
    participant Stdio as ck-transport-stdio
    participant Mcp as McpService

    Stdin->>Stdin: Read JSON-RPC Line
    alt reload.enabled == true
        Stdin->>Config: Check modification timestamp
        alt File Changed
            Stdin->>Config: load_config()
            Stdin->>Stdin: Rebuild SqliteStore & AppServices
            Stdin->>Stdin: Construct new McpService
            Stdin->>SseState: replace_service() & replace_auth()
        end
    end
    Stdin->>Stdio: handle_line(&service, line)
    Stdio->>Mcp: call_tool(name, arguments)
    Mcp-->>Stdio: Tool Result
    Stdio-->>Stdin: Return JSON-RPC Response String
    Stdin->>Stdin: Print to Stdout
```

---

## 6. Trust & Security Boundaries

Context Kernel operates under two distinct trust profiles:

1. **Local Stdio Trust Mode**:
   - Boundary: OS process boundary.
   - Intended for local CLI, IDE extensions (VS Code, Cursor), and local developer agents.
   - Authentication is disabled by default (`BearerAuth::no_auth()`).
   - The caller is assumed to have direct system execution privileges.

2. **Remote SSE Boundary**:
   - Boundary: Network TCP / HTTP boundary.
   - When `require_auth = true`, every HTTP request to `/mcp` must present a `Bearer <token>` header.
   - Token validation uses `subtle::ConstantTimeEq` to prevent timing side-channel attacks.
   - Granular tool allowlists (`allow_tools`) and corpus allowlists (`allow_corpora`) restrict which tools and corpora the token holder may access.
   - SSRF Protection: The Web adapter blocks private IP networks, loopback addresses, non-HTTP schemes, and unresolvable hostnames.

---

## 7. Further Subsystem Exploration

To inspect specific implementation details, proceed to the subsystem guides:
* [Core & Domain Subsystem](subsystems/core-and-domain.md)
* [Storage & Persistence Subsystem](subsystems/storage-and-persistence.md)
* [Retrieval & Ranking Subsystem](subsystems/retrieval-and-ranking.md)
* [Source Adapters Subsystem](subsystems/source-adapters.md)
* [Application Orchestration Subsystem](subsystems/application-orchestration.md)
* [MCP & Protocol Serving Subsystem](subsystems/mcp-and-protocols.md)
* [Runtime & CLI Subsystem](subsystems/runtime-and-cli.md)
