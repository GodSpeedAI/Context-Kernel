# MCP Contracts

The service exposes MCP tools over stdio JSON-RPC `tools/call` and, when enabled, the SSE HTTP transport.

Implemented canonical tools:
- `get_status`
- `store_context`
- `recall_context`
- `ingest_source`
- `refresh_source`
- `query_reference`
- `resolve_reference`

Implemented compatibility aliases:
- `resolve-library-id` maps to `resolve_reference`
- `get-library-docs` maps to `query_reference`
- `query-docs` maps to `query_reference`

`get_status` request:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}
```

Response result shape:

```json
{
  "service": {"name": "context-kernal", "version": "0.1.0", "started_at": "...", "healthy": true},
  "storage": {"kind": "sqlite", "path": "kernel.db", "fts5_enabled": true},
  "sources": {"total": 0, "enabled": 0, "degraded": 0},
  "sessions": {"active": 0},
  "jobs": {"queued": 0, "running": 0},
  "retrieval": {"default_mode": "hybrid", "semantic_enabled": false}
}
```

When SSE auth is enabled, bearer token validation happens at the HTTP boundary and tool/corpus authorization is enforced before the MCP tool dispatch. Plain stdio calls remain local/internal and use the direct `call_tool` path.
