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
- `context_required` (WP-5, F-05) — the GodSpeed loop's context agent.
  Resolves the request through the REAL application retrieval pipeline and
  returns the canonical CEP-0008 `godspeed.context_bundle` envelope
  (`extensions["cep.profile"].profile_id = "godspeed.context_bundle"`,
  `profile_version = "1.0.0"`). The legacy `context_envelope` /
  `context_packet` wire shape is retained for bounded dual-read
  compatibility and is DEPRECATED (write CEP only). SWE_SEED's context stage
  consumes the bundle. The `access` field reports CONTEXT ACCESS
  (`accessible` / `inaccessible` / `requires_scope`) — Context Kernel never
  decides action authority; that is SEA-Forge's role. Served only when
  `CK_CONTEXT_CORPUS_ROOT` (filesystem corpus) and/or
  `CK_LEARNING_PROPOSALS_LEDGER` is set; otherwise the tool reports itself
  unconfigured rather than failing the server.
- `ingest_context_effects` — settlement-backed Context Effect ingestion for
  prospective retrieval calibration. Every effect must cite a
  settlement/evidence artifact (`evidence_ref`); self-reports are rejected.
  Calibration influences FUTURE retrieval only; historical bundles are
  immutable. Requires the `CK_CONTEXT_EFFECT_LEDGER` ledger to be configured.

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

Response result shape (canonical `context_bundle` first; legacy dual-read
fields deprecated):

```json
{"context_bundle":{"envelope_id":"env-ctx_...","envelope_kind":"context_bundle",
  "scope":{"world_ref":"world:<name>@sha256:<digest>"},
  "completeness_status":"complete_for_context",
  "extensions":{"cep.profile":{"profile_id":"godspeed.context_bundle","profile_version":"1.0.0"},
    "godspeed.context_bundle":{"work_request_ref":"wr-1","retrieval_completeness":"complete",...}},
  "references":[...],"omissions":[...],"integrity":{"content_hash":"sha256:..."}},
 "context_envelope":{"schema_version":"v1","event_type":"ContextPacketCreated",...},
 "context_packet":{"context_packet_id":"ctx_...","corpus_id":"docs","citations":[...]},
 "outcome":"cited","reason":null,
 "access":{"result":"accessible"}}
```

A partial or empty retrieval never reads as complete: `partial` bundles carry
`references` + `omissions`, `none` bundles carry the unanswered `questions` +
`omissions`, and absence is never converted into falsehood.

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
