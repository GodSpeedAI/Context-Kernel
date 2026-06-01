# MCP Contracts

Milestone 0 implements the `get_status` tool over stdio JSON-RPC `tools/call`.

Request:

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
