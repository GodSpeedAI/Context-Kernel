# ck-core types

## Overview

Shared vocabulary types for the whole kernel. `mod.rs` defines the lifecycle/state enums (retrieval mode, session transport/continuation/status, job trigger/status, query intent, recall scope, source health). `context.rs` defines the context-boundary vocabulary: retrieval completeness, explicit no-context reasons, machine-readable omission codes, and the CEP-0008 `world:<name>@sha256:<64hex>` reference gate. `status.rs` defines the `KernelStatus` tree reported by the status surface. Both child modules are re-exported at the `types` level, and status types are additionally re-exported at the crate root.

## Interface

- [mod.rs:RetrievalMode] - `Lexical | Semantic | Hybrid`; `Hybrid` is the `#[default]`.
- [mod.rs:SessionTransport] - `Stdio | Sse`.
- [mod.rs:ContinuationMode] - `Ephemeral | Resumeable | Pinned`; `Ephemeral` is the default.
- [mod.rs:SessionStatus] - `Active | Closed | Compacted`; `Active` is the default.
- [mod.rs:JobTrigger] - `Startup | Manual | Scheduled | Webhook | DependencyChange`.
- [mod.rs:JobStatus] - `Queued | Running | Completed | Failed | Cancelled`; `Queued` is the default.
- [mod.rs:QueryIntent] - `ResolveLibrary | QueryDocs | SearchRefs | RecallContext | General`; `General` is the default.
- [mod.rs:RecallScope] - `Current | Recent | All`; `Current` is the default.
- [mod.rs:SourceHealth] - `Unknown | Healthy | Degraded`; `Unknown` is the default.
- [context.rs:RetrievalCompleteness] - `Complete | Partial | None`; serializes snake_case.
- [context.rs:NoContextReason] - eight explicit zero-result reasons (`CorpusRootUnconfigured`, `CorpusNotFound`, `NoMatchingContent`, `CorpusUnavailable`, `ScopeUnresolved`, `CorpusExcluded`, `BudgetExhausted`, `StaleContent`); [context.rs:NoContextReason::as_str] maps each to a stable string code.
- [context.rs:WorldRefError] - single `Malformed` variant; implements `Display` and `std::error::Error`.
- [context.rs:verify_world_ref] - public syntax gate for `world:<name>@sha256:<64 lowercase hex>` references.
- [context.rs] omission code constants: `OMISSION_MAX_RESULTS` = "max_results_reached", `OMISSION_CONTENT_TRUNCATED` = "citation_content_truncated", `OMISSION_TOKEN_BUDGET` = "token_budget_exhausted", `OMISSION_STALE_SOURCE` = "stale_source_excluded", `OMISSION_REQUIRED_CONCEPT` = "required_distinction_absent".
- [status.rs:KernelStatus] - aggregate status composed of [status.rs:ServiceStatus], [status.rs:StorageStatus], [status.rs:SourcesStatus], [status.rs:SessionsStatus], [status.rs:JobsStatus], [status.rs:RetrievalStatus]; constructors `new(...)` and `placeholder()`.

## Invariants

- [context.rs:RetrievalCompleteness] doc comment: only `Complete` may be read as "everything that matched within the declared retrieval contract is here"; "Partial must never masquerade as complete, and absence must never masquerade as anything."
- [context.rs:NoContextReason] doc comment: "Zero-citation outcomes MUST NOT silently settle as successful required context acquisition; absence is always explicit and machine-readable."
- [context.rs:NoContextReason::as_str] maps variants to fixed snake_case codes, including `ScopeUnresolved` -> "corpus_scope_unresolved" and `StaleContent` -> "no_fresh_content".
- [context.rs:verify_world_ref] is a syntax-only gate: the digest is never recomputed or verified here (DomainForge owns the world; SEA-Forge re-verifies it); it accepts only a `world:` name matching `[a-z][a-z0-9]*(?:[._-][a-z0-9]+)*` up to 64 chars plus `@sha256:` and exactly 64 lowercase hex chars, so "a mutable alias or label can never stand in for a world revision".
- [context.rs:OMISSION_CONTENT_TRUNCATED] is a legacy code "Kept for wire compatibility with historical packets".
- Module doc of [context.rs]: these types are "deliberately free of any CEP/transport structure: the boundary adapter does the projection".
- All [mod.rs] enums and [context.rs:RetrievalCompleteness] serialize snake_case via `#[serde(rename_all = "snake_case")]`; the `#[default]` variants are RetrievalMode::Hybrid, ContinuationMode::Ephemeral, SessionStatus::Active, JobStatus::Queued, QueryIntent::General, RecallScope::Current, SourceHealth::Unknown.
- [status.rs:KernelStatus::new] hardcodes service name "context-kernel" with `healthy: true` and zeroed source/session/job counters; [status.rs:KernelStatus::placeholder] returns the non-started fixture (`started_at` "not-started", storage kind "sqlite" path "kernel.db", `fts5_enabled` true, default mode "hybrid", `semantic_enabled` false).

## Side Effects

- Build-time environment read: [status.rs:KernelStatus::new] embeds the crate version via the compile-time `env!("CARGO_PKG_VERSION")` macro. No runtime I/O, network, locks, or global state exist in this directory.

## Verification

- Run: `cargo test -p ck-core`
