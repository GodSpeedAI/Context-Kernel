# Configuration Reference

Context Kernel uses TOML configuration files. See `config/` for example templates.

## Top-Level Sections

| Section | Purpose |
|---------|---------|
| `[service]` | Service identity |
| `[storage]` | Database path |
| `[retrieval]` | Search mode settings |
| `[mcp]` | Transport configuration |
| `[auth]` | Authentication tokens |
| `[scheduler]` | Background refresh |
| `[embeddings]` | Semantic retrieval |
| `[reload]` | Hot config reload |
| `[retention]` | Garbage collection |
| `[authorization]` | Tool and corpus access |
| `[observability]` | Logging and telemetry |

## [service]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | `"context-kernal"` | Service name in status responses |

## [storage]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `database_path` | string | `"kernel.db"` | Path to SQLite database file |

## [retrieval]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `default_mode` | string | `"hybrid"` | Search mode: `lexical`, `semantic`, or `hybrid` |
| `semantic_enabled` | bool | `false` | Whether semantic search is active |

## [mcp]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable_stdio` | bool | `true` | Enable stdio JSON-RPC transport |
| `enable_sse` | bool | `false` | Enable SSE HTTP transport |

### [mcp.sse]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bind` | string | `"127.0.0.1:8765"` | Bind address for SSE server |
| `base_path` | string | `"/mcp"` | URL base path |
| `require_auth` | bool | `false` | Require bearer token authentication |

## [auth]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `trust_mode` | string | `"internal"` | Trust mode: `internal` for local use |
| `tokens` | string[] | `[]` | Bearer tokens. Use `${ENV_VAR}` for env vars |

## [scheduler]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enable_periodic_refresh` | bool | `false` | Enable background refresh |
| `refresh_interval_secs` | u64 | `1800` | Refresh check interval |

## [embeddings]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable semantic retrieval |
| `provider` | string | `"none"` | Provider: `none` or `openai` |
| `api_base` | string | `""` | API base URL |
| `api_key` | string | `""` | API key. Use `$ENV_VAR` syntax |
| `model` | string | `"text-embedding-3-small"` | Embedding model name |
| `dimensions` | usize | `1536` | Vector dimensions |

## [reload]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable config file hot reload |

## [retention]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable retention GC on startup |
| `session_event_days` | i64 | `90` | Days to retain unpinned events |
| `inactive_session_days` | i64 | `30` | Days to retain inactive sessions |
| `refresh_job_days` | i64 | `30` | Days to retain finished jobs |
| `artifact_days` | i64 | `90` | Days to retain artifacts |
| `disabled_source_days` | i64 | `30` | Days to retain disabled sources |

## [authorization]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `allow_tools` | string[] | `["*"]` | Allowed MCP tool names |
| `allow_corpora` | string[] | `["*"]` | Allowed corpus IDs |

## [observability]

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `log_format` | string | `"text"` | Log format: `text` or `json` |
| `otel_endpoint` | string | `null` | OpenTelemetry collector endpoint (reserved) |

## Environment Variable Expansion

Use `${ENV_VAR}` syntax in config values to load from environment variables:

```toml
[auth]
tokens = ["${CONTEXT_KERNEL_AUTH_TOKEN}"]

[embeddings]
api_key = "$OPENAI_API_KEY"
```
