# ck-transport-stdio

## Overview

`ck-transport-stdio` is the stdio transport adapter for Context Kernel's MCP surface. It contains a single line handler that maps one JSON-RPC 2.0 request line to one serialized response line, delegating `tools/call` requests to the `ck-mcp` crate's `McpService`; its only workspace dependency is `ck-mcp`. Process-level stdio framing and output handling live in the binary composition layer (`ck-bin`), not here.

## Subdirectories & Boundaries

- src: [./src/README.md](./src/README.md) - JSON-RPC 2.0 line handler (`handle_line`) bridging request lines to `ck_mcp::McpService::call_tool` with a fixed error wire shape.
