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
