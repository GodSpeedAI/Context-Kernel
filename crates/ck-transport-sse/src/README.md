# ck-transport-sse Source

## Overview
This crate is the HTTP transport adapter for Context Kernel's MCP endpoint, built on axum. `serve_sse` wires a `Router` with health, readiness, status, and MCP POST routes onto a caller-provided `tokio::net::TcpListener` and dispatches requests into the `McpService` from `ck-mcp`, with authorization delegated to an `AuthPort` from `ck-core`. `ReloadableSseState` holds the live `McpService` and auth backend behind `RwLock`-protected `Arc`s so the binary can hot-swap either at runtime without restarting the server. The transport is composed into the runtime by `ck-bin` (`ck_transport_sse::ReloadableSseState` / `ck_transport_sse::serve_sse` in `crates/ck-bin/src/main.rs`).

## Interface
- [lib.rs:ReloadableSseState] - Public state container pairing `RwLock<Arc<McpService>>` with `RwLock<Arc<dyn AuthPort>>`; axum shared state for all routes.
- [lib.rs:ReloadableSseState::new] - Constructs the state from the initial service and auth backends.
- [lib.rs:ReloadableSseState::mcp] / [lib.rs:ReloadableSseState::auth] - Clone the current `Arc` snapshot out under the read lock.
- [lib.rs:ReloadableSseState::replace_service] / [lib.rs:ReloadableSseState::replace_auth] - Hot-swap the service or auth backend under the write lock.
- [lib.rs:serve_sse] - Public async fn building the axum `Router` and serving it on the provided `tokio::net::TcpListener`. Routes: `GET /healthz`, `GET /readyz`, `GET /api/v1/status`, `POST /mcp`.
- Handlers `healthz`, `readyz`, `api_status_reloadable`, and `mcp_handler_reloadable` are private but define the HTTP contract documented below.

## Invariants
- Hot-swap consistency: readers obtain a coherent `Arc` snapshot per request; swapping via `replace_service`/`replace_auth` affects only requests that start afterwards ([lib.rs:ReloadableSseState]).
- Readiness is live: [lib.rs:readyz] computes `{"ready": bool}` by actually dispatching `tools/call get_status` against the current `McpService`, not from a hardcoded constant; [lib.rs:healthz] by contrast is a static `{"status": "ok"}`.
- Auth gate: [lib.rs:mcp_handler_reloadable] requires a valid `Authorization` header token validated through `AuthPort::validate_token` whenever the backend is not in local-trust mode (`is_local_trust_mode() == false`); failures return HTTP 401 with error code `auth_required`.
- Method constraint: [lib.rs:mcp_handler_reloadable] accepts only JSON-RPC method `"tools/call"`; any other method returns HTTP 200 with a JSON-RPC error object `unsupported method: {method}`.
- Error mapping: a non-JSON body returns HTTP 400 with error code `invalid_arguments`; a tool execution failure returns HTTP 200 with a JSON-RPC error object carrying code `internal` ([lib.rs:mcp_handler_reloadable]).
- Response envelope: successful calls are wrapped as `{"jsonrpc": "2.0", "id": <request id or null>, "result": ...}` ([lib.rs:mcp_handler_reloadable]).

## Side Effects
- Serves an HTTP server on the caller-provided `tokio::net::TcpListener` via `axum::serve`, logging `SSE MCP server starting` with the local address ([lib.rs:serve_sse]).
- Dispatches into `McpService` (`call_tool` for status/readiness, `call_tool_authorized` for POST /mcp), executing the full kernel tool pipeline per request ([lib.rs:mcp_handler_reloadable], [lib.rs:readyz], [lib.rs:api_status_reloadable]).
- Internal synchronization via two `RwLock`s whose lock results are unwrapped (poisoned locks panic) ([lib.rs:ReloadableSseState]).
- Emits `tracing::info!` / `tracing::debug!` events, including received body lengths ([lib.rs:serve_sse], [lib.rs:mcp_handler_reloadable]).

## Verification
- Run: `cargo test -p ck-transport-sse`
