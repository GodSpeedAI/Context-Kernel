# Reference: Configuration Schema

Context Kernel uses TOML configuration files. This document details every section, key, data type, default value, and validation constraint supported by `ck-config`.

---

## 1. Environment Variable Expansion

Any string value in the configuration file can reference environment variables using either `${VAR_NAME}` or `$VAR_NAME` syntax:

```toml
[auth]
tokens = ["${CONTEXT_KERNEL_AUTH_TOKEN}"]

[embeddings]
api_key = "$OPENAI_API_KEY"
```

If the referenced environment variable is not defined in the shell, it expands to an empty string.

---

## 2. Table-by-Table Reference

### `[service]`
Controls the identity reported in MCP status responses.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `name` | string | `"context-kernel"` | Name of the service emitted in `get_status` and logs |

---

### `[storage]`
Controls persistent database paths.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `database_path` | string | `"kernel.db"` | Relative or absolute path to the SQLite database file |

---

### `[retrieval]`
Controls default search behavior.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `default_mode` | string | `"hybrid"` | Retrieval mode: `"lexical"`, `"semantic"`, or `"hybrid"` |
| `semantic_enabled` | bool | `false` | *Parsed but ignored (controlled by `[embeddings]`)* |

---

### `[mcp]`
Controls transport serving.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enable_stdio` | bool | `true` | *Parsed but ignored (stdio is always enabled)* |
| `enable_sse` | bool | `false` | Enable the HTTP Server-Sent Events (SSE) server |

#### `[mcp.sse]`
Settings for the HTTP SSE server.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `bind` | string | `"127.0.0.1:8765"` | IP and port to bind the HTTP server |
| `base_path` | string | `"/mcp"` | *Parsed but ignored (server binds `/mcp`)* |
| `require_auth` | bool | `false` | Require `Authorization: Bearer <token>` on HTTP calls |

---

### `[auth]`
Authentication settings.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `trust_mode` | string | `"internal"` | *Parsed but ignored* |
| `tokens` | string[] | `[]` | List of valid bearer tokens for SSE authentication |

---

### `[authorization]`
Fine-grained access control policies.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `allow_tools` | string[] | `["*"]` | Allowed MCP tools (e.g. `["get_status", "query_reference"]`) |
| `allow_corpora` | string[] | `["*"]` | Allowed corpus IDs (e.g. `["public_docs"]`) |

---

### `[scheduler]`
Controls background source refresh workers.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enable_periodic_refresh` | bool | `false` | Enable background periodic source re-indexing thread |
| `refresh_interval_secs` | integer | `1800` | Sleep interval between refresh polling sweeps |

---

### `[embeddings]`
Controls optional vector embeddings and semantic search.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Enable semantic embeddings |
| `provider` | string | `"none"` | Embedding provider: `"none"` or `"openai"` |
| `api_base` | string | `""` | API base URL (e.g. `https://api.openai.com/v1`) |
| `api_key` | string | `""` | Bearer API key for the embedding provider |
| `model` | string | `"text-embedding-3-small"` | Model name |
| `dimensions` | integer | `1536` | Embedding vector dimensions |

---

### `[reload]`
Hot configuration file reload.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Enable on-disk config file change detection in stdio loop |

---

### `[retention]`
Startup garbage collection rules.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `enabled` | bool | `false` | Enable retention GC on service startup |
| `session_event_days` | integer | `90` | Days to retain unpinned session events |
| `inactive_session_days` | integer | `30` | Days to retain closed or compacted sessions |
| `refresh_job_days` | integer | `30` | Days to retain completed refresh jobs |
| `artifact_days` | integer | `90` | Days to retain artifact files and metadata |
| `disabled_source_days` | integer | `30` | Days to retain disabled sources |

---

### `[observability]`
Logging format and telemetry.

| Field | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `log_format` | string | `"text"` | Log formatting: `"text"` or `"json"` (written to stderr) |
| `otel_endpoint` | string / null| `null` | *Reserved: detected in config but exporter is unimplemented* |
