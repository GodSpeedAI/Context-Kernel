# integration

## Overview
The `integration` directory groups cross-system integration adapter code for Context-Kernel and contains no direct source files of its own. Its single subdirectory, `agentic_capability_loop`, hosts a Python-level integration adapter package for the `agentic_capability_loop` domain, documented in its own contract as a legacy, deprecated compatibility shim whose production counterpart is `crates/ck-mcp`.

## Subdirectories & Boundaries
- `agentic_capability_loop`: [./agentic_capability_loop/README.md](./agentic_capability_loop/README.md) - Python integration adapter for the `agentic_capability_loop` domain: consumes `ContextRequired` (from SWE_SEED), emits `ContextPacketCreated` (legacy wire, deprecated) and `ContextAccessEvaluated` (context access only, never action authority); LEGACY/DEPRECATED shim kept for old integration tests, with production retrieval in `crates/ck-mcp`.
