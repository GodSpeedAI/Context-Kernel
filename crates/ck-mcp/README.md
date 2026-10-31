# ck-mcp

## Overview

`ck-mcp` is Context Kernel's MCP boundary adapter crate. It dispatches the kernel's MCP tools (`get_status`, `store_context`, `recall_context`, `ingest_source`, `refresh_source`, `query_reference`, `resolve_reference`, `context_required`, `ingest_context_effects`, plus the Context7-style compatibility aliases `get-library-docs`/`query-docs`/`resolve-library-id`) onto `ck-app::AppServices` use cases, serves the frozen E2/E3 agentic-capability-loop boundary through `AclContextAgent` (with optional `AuthPort` tool/corpus authorization), and projects retrieval resolutions onto the CEP-0008 `godspeed.context_bundle` envelope. The crate depends on `ck-core` and `ck-app` and contains no transport framing or storage code; stdio line handling lives in `ck-transport-stdio` and composition in `ck-bin`.

## Subdirectories & Boundaries

- src: [./src/README.md](./src/README.md) - MCP tool dispatch (`McpService`), the ACL context agent (`AclContextAgent`) with the E2 `ContextRequired` / E3 `ContextPacketCreated` wire contracts, and the CEP-0008 `godspeed.context_bundle` projection (`cep_bundle`).
