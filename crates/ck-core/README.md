# ck-core

## Overview

`ck-core` is the innermost crate of the Context Kernel workspace: per the workspace layering rules it holds domain types and ports only, with no transport, storage, or MCP concerns. [Cargo.toml] confines its dependencies to `anyhow`, `chrono`, `serde`, and `serde_json` — no infrastructure crates. The crate root exposes exactly three public modules (`entities`, `ports`, `types`), re-exports the status types at the crate root for backward compatibility, and provides a `prelude` that re-exports all three modules.

## Subdirectories & Boundaries

- src: ./src/README.md - crate root module; aggregates the domain records (entities), adapter trait contracts (ports), and shared vocabulary (types), and re-exports status types at the crate root.

## Verification

- Run: `cargo test -p ck-core`
