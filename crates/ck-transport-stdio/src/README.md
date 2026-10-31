# ck-transport-stdio src

## Overview

Minimal stdio transport adapter for Context Kernel's MCP surface. `handle_line` converts a single JSON-RPC 2.0 request line into a single serialized JSON-RPC 2.0 response line, dispatching `tools/call` requests to `ck_mcp::McpService::call_tool`. The handler itself performs no stdio, filesystem, or network I/O; line framing and output handling belong to the binary composition layer.

## Interface

- [lib.rs:handle_line] - `handle_line(service: &McpService, line: &str) -> anyhow::Result<String>`: parses one JSON-RPC request, and for method `tools/call` invokes `McpService::call_tool(name, arguments)` from the `params`
  - Success wire shape: `{"jsonrpc":"2.0","id":<id>,"result":<tool result>}`
  - Tool failure wire shape: `{"jsonrpc":"2.0","id":<id>,"error":{"code":"invalid_arguments","message":<McpError display>,"retryable":false,"details":{}}}`
  - Any method other than `tools/call` receives the same error shape with message `unsupported method: <method>`
  - A request without an `id` field is answered with `"id": null`

## Invariants

- None

## Side Effects

- None

## Verification

- Run: `cargo test -p ck-transport-stdio`
