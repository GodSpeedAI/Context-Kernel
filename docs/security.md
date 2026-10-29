# Security

## Authentication

### Stdio Transport
Stdio runs in local trust mode. No authentication is required. This is intended for local development and IDE integration where the process boundary provides isolation.

### SSE Transport
SSE supports bearer token authentication. Enable with:

```toml
[mcp.sse]
require_auth = true

[auth]
tokens = ["${CONTEXT_KERNEL_AUTH_TOKEN}"]
```

When `require_auth = true` and no tokens are configured, the service fails to start with a clear error.

## Authorization

### Tool Authorization
Control which MCP tools are accessible:

```toml
[authorization]
allow_tools = ["get_status", "query_reference"]
```

Use `["*"]` to allow all tools (default).

### Corpus Authorization
Control which corpora are accessible:

```toml
[authorization]
allow_corpora = ["public_docs"]
```

Use `["*"]` to allow all corpora (default).

#### POL-ACL-002: Private Corpus Scope

Corpus authorization has two independent layers. The transport allowlist (`allow_corpora`) decides whether a caller may touch a corpus at all. The resolution-level scope claim decides whether a private corpus may be opened for this specific request:

* A private corpus requires an explicit scope claim that includes its `corpus_id`. An empty claim produces `requires_scope`; a claim naming only other corpora produces `inaccessible` with reason `out_of_scope`. Both are reported as the no-context reason `corpus_scope_unresolved`.
* The gate runs before retrieval. When it refuses, no chunk is fetched and the returned bundle is minimal.
* A caller assertion of privacy (`is_private: true`) is authoritative for ingest decisions: an unregistered private corpus is never auto-ingested open. It resolves to `corpus_not_found` instead.
* `query_reference` and `resolve_reference` apply the same scope rule through their `source_scope` filter, so private corpora cannot be read through the retrieval tools either.

The `access` object on `context_required` reports context access only: `{"result":"accessible"}`, `{"result":"inaccessible","corpus_id":"...","reason":"excluded"|"out_of_scope"}`, or `{"result":"requires_scope","corpus_id":"..."}`. It never conveys action authority.

### Source Scope
The `source_scope` filter in `query_reference` and `resolve_reference` restricts results to specified corpora. Private corpora require explicit inclusion in the scope.

## SSRF Protection

The web adapter (`ck-adapter-web`) blocks requests to:

- Private IPv4 ranges: 10/8, 172.16/12, 192.168/16, 127/8, 169.254/16, 0/8
- IPv6 loopback, link-local, and unique-local addresses
- Non-HTTP schemes (file, ftp, data, etc.)
- DNS hostnames that resolve to private IPs
- Hostnames not in `allowed_domains` (when configured)

## Token Storage

- Use environment variables for sensitive tokens: `${CONTEXT_KERNEL_AUTH_TOKEN}`
- The `subtle` crate provides constant-time token comparison
- Tokens are never logged

## Artifact Storage

Artifacts are stored as files in the configured `data_dir/artifacts/` directory. Artifact IDs are validated to prevent path traversal attacks (only alphanumeric, `-`, and `_` characters allowed).
