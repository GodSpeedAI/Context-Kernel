# ck-app

## Overview

`ck-app` is the application-layer crate of the Context Kernel workspace. It defines `AppServices`, the use-case orchestrator that composes `ck-core` ports and implements the kernel's operations: session context store/recall, source ingestion and refresh, reference query/resolve, retention GC, status aggregation, typed context resolution (`ContextRequirement` to `ContextResolution`), and settlement-backed context-effect calibration over an append-only JSONL ledger. Persistence and search run exclusively through ports; the one exception is the JSONL `ContextEffectStore`, which performs direct `std::fs` file I/O inside `src/context_effect.rs` (known layering debt: a durable store living in the application layer rather than behind a port). There is no transport (MCP/HTTP) code and no SQL in this crate; the crate also re-exports retention-classification helpers from the sibling `ck-memory` crate.

## Subdirectories & Boundaries

- `src`: [./src/README.md](./src/README.md) - Component package contract for the crate's three source files (`lib.rs`, `context.rs`, `context_effect.rs`): the `AppServices` use-case orchestrator, the typed context resolution pipeline, and the settlement-backed context-effect ledger.

## Verification

- Run: `cargo test -p ck-app`
