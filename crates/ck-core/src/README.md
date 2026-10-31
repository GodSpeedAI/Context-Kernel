# ck-core src

## Overview

Crate root module of `ck-core`, the innermost domain crate of Context Kernel. [lib.rs] declares the three public modules that make up the entire crate surface and aggregates them: `entities` (domain records), `ports` (hexagonal trait contracts for adapters), and `types` (shared vocabulary enums and context-boundary types). It also re-exports the status types at the crate root and defines a `prelude` module that re-exports all three modules at once.

## Interface

- [lib.rs] `pub mod entities` - domain entity records (see ./entities/README.md).
- [lib.rs] `pub mod ports` - port traits implemented by adapters (see ./ports/README.md).
- [lib.rs] `pub mod types` - shared vocabulary types (see ./types/README.md).
- [lib.rs] `pub use types::status::*` - re-export of the status types at the crate root for backward compatibility.
- [lib.rs:prelude] - `pub mod prelude` re-exporting `crate::entities::*`, `crate::ports::*`, and `crate::types::*`.

Child contracts:

- Entities: ./entities/README.md
- Ports: ./ports/README.md
- Types: ./types/README.md

## Invariants

- [lib.rs] comment: status types are re-exported at the crate root "for backward compatibility"; consumers may rely on root paths such as `ck_core::JobStatus` and `ck_core::KernelStatus` as well as the `types::status` module path.
- [lib.rs:prelude] must stay a flat re-export aggregation of `entities`, `ports`, and `types`; the crate root itself adds no domain logic.

## Side Effects

- None

## Verification

- Run: `cargo test -p ck-core`
