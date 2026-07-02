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
- `context_required` (WP-5, F-05) — the agentic-capability-loop context agent.
  Returns a citation-bearing `ContextPacketCreated` payload + the POL-ACL-002
  `ContextAuthorized` decision. SWE_SEED's context stage calls this to satisfy
  POL-ACL-001/003 with a CK-sourced packet. Served only when
  `CK_CONTEXT_CORPUS_ROOT` (filesystem corpus) and/or
  `CK_LEARNING_PROPOSALS_LEDGER` is set; otherwise the tool reports itself
  unconfigured rather than failing the server.

Implemented compatibility aliases:
- `resolve-library-id` maps to `resolve_reference`
- `get-library-docs` maps to `query_reference`
- `query-docs` maps to `query_reference`

`context_required` request:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"context_required","arguments":{
  "work_request_id":"wr-1","context_requirement_id":"cr-1","corpus_id":"docs",
  "query":"context packet","max_results":10,"is_private":false,"scope_claim":null
}}}
```

Response result shape:

```json
{"context_packet":{"context_packet_id":"ctx_...","corpus_id":"docs","citations":[...],"authorized":true},
 "authorization":{"result":"allow","policy_code":"POL-ACL-002","reason":"public corpus"}}
```

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
