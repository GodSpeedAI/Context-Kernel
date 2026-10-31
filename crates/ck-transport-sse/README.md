# ck-transport-sse

## Overview
`ck-transport-sse` (package `ck-transport-sse`) is the HTTP transport adapter exposing Context Kernel's MCP service. It serves axum routes for health (`/healthz`), live readiness (`/readyz`), status (`/api/v1/status`), and the MCP JSON-RPC entry point (`POST /mcp`) on a caller-provided TCP listener, with bearer-token authorization delegated to an `AuthPort`. A reloadable state container lets the `ck-bin` runtime hot-swap the `McpService` and auth backend without restarting the server.

## Subdirectories & Boundaries
- `src`: [./src/README.md](./src/README.md) - Component contract for `ReloadableSseState` and `serve_sse`: route table, hot-swap locking rules, auth gating and error mapping for `POST /mcp`, and the live-probe readiness contract.

## Verification
- Run: `cargo test -p ck-transport-sse`
