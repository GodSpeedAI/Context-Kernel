# How-To: Configure SSE with Bearer Authentication & Allowlists

This guide demonstrates how to configure Context Kernel to serve the Model Context Protocol over an HTTP Server-Sent Events (SSE) endpoint, secured with bearer tokens, tool allowlists, and corpus allowlists.

---

## Goal
Expose Context Kernel over HTTP so remote agents or CI runners can query documentation, while securing the service with bearer tokens and restricting which tools and corpora callers can access.

---

## 1. Create the Configuration File

Create a configuration file named `context-kernal.sse.toml`:

```toml
[service]
name = "context-kernel"

[storage]
database_path = "./data/kernel.db"

[retrieval]
default_mode = "hybrid"

[mcp]
enable_stdio = true
enable_sse = true

[mcp.sse]
bind = "127.0.0.1:8765"
base_path = "/mcp"
require_auth = true

[auth]
tokens = ["${CONTEXT_KERNEL_AUTH_TOKEN}"]

[authorization]
allow_tools = [
  "get_status",
  "query_reference",
  "resolve_reference"
]
allow_corpora = [
  "public_docs",
  "team_specs"
]

[observability]
log_format = "text"
```

---

## 2. Set the Environment Variable & Start the Server

Export a cryptographically secure token and launch `context-kernal`:

```bash
export CONTEXT_KERNEL_AUTH_TOKEN="$(openssl rand -hex 32)"
echo "CONTEXT_KERNEL_AUTH_TOKEN is set (value not printed)"

./target/release/context-kernal serve --config context-kernal.sse.toml
```

Expected log output:
```text
INFO SSE MCP server started bind="127.0.0.1:8765"
```

---

## 3. Verify Health & Readiness Endpoints

In a separate terminal, test the HTTP endpoints:

### Liveness Probe (`GET /healthz`)
```bash
curl -s http://127.0.0.1:8765/healthz
```
Response:
```json
{"status":"ok"}
```

### Readiness Probe (`GET /readyz`)
```bash
curl -s http://127.0.0.1:8765/readyz
```
Response:
```json
{"ready":true}
```
*(The readiness probe actively dispatches `get_status` internally to verify that the application core is operational).*

---

## 4. Test Authorized and Unauthorized Tool Calls

### Call Without Token (Rejected: 401 Unauthorized)
```bash
curl -i -X POST http://127.0.0.1:8765/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}'
```
Response:
```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{"error":{"code":"auth_required","message":"Unauthorized","retryable":false}}
```

### Call With Valid Token (Accepted: 200 OK)
```bash
curl -i -X POST http://127.0.0.1:8765/mcp \
  -H "Authorization: Bearer ${CONTEXT_KERNEL_AUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}'
```
Response:
```http
HTTP/1.1 200 OK
Content-Type: application/json

{"id":1,"jsonrpc":"2.0","result":{"service":{"healthy":true,"name":"context-kernel","version":"0.1.0"},...}}
```

### Call Restricted Tool (Rejected by Allowlist)
If a client attempts to call `store_context` (which is omitted from `allow_tools`):
```bash
curl -i -X POST http://127.0.0.1:8765/mcp \
  -H "Authorization: Bearer ${CONTEXT_KERNEL_AUTH_TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"store_context","arguments":{"session_id":"test","text":"hi"}}}'
```
Response:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": "invalid_arguments",
    "message": "invalid_arguments: unauthorized tool: store_context",
    "retryable": false
  }
}
```

---

## 5. Security Checklist
* Always set `mcp.sse.require_auth = true` when binding to non-loopback network interfaces (`0.0.0.0` or private LAN IPs).
* Use `${ENV_VAR}` interpolation rather than hardcoding tokens in TOML configuration files.
* Restrict `allow_tools` to read-only tools (`query_reference`, `resolve_reference`, `get_status`) for public or untrusted client agents.
