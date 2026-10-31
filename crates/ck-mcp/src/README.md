# ck-mcp src

## Overview

This package is Context Kernel's MCP boundary adapter. `McpService` maps MCP tool-call arguments onto `ck-app::AppServices` use cases; `AclContextAgent` serves the frozen E2/E3 agentic-capability-loop boundary (`ContextRequired` ingress, `ContextPacketCreated` egress) by delegating all retrieval to `AppServices::resolve_context`; `cep_bundle` is the one place a native Context Kernel resolution is restated as a CEP-0008 `godspeed.context_bundle` envelope with SHA-256 canonical-JSON integrity. The package contains no transport or storage code: every tool except `get_status` requires an initialized `AppServices` and delegates persistence, retrieval, and ingestion to the application layer.

## Interface

- [lib.rs:McpService] - the MCP tool dispatcher; constructors `new`/`with_app`/`with_app_and_auth`/`with_context_agent` (builder), entry points `call_tool` and `call_tool_authorized(token, name, arguments)`
- [lib.rs:dispatch_tool] - tool table: `get_status`, `store_context`, `recall_context`, `ingest_source`, `refresh_source`, `query_reference` (aliases `get-library-docs`, `query-docs`), `resolve_reference` (alias `resolve-library-id`), `context_required`, `ingest_context_effects`; unknown names return `McpError::UnknownTool`
- [lib.rs:McpError] - error enum `UnknownTool` / `InvalidArguments` / `Internal`, with `From<anyhow::Error>`
- [lib.rs:ToolTelemetry] - per-call telemetry record with `success`/`failure` constructors
- [agentic_capability_loop.rs:AclContextAgent] - GodSpeed-facing context acquisition adapter (fields `corpus_root`, optional `learning_proposals_ledger`); `acquire_context` returns [agentic_capability_loop.rs:ContextAcquisitionResult] with `outcome(required)` and `reason()`
- [agentic_capability_loop.rs:ContextRequiredPayload] - E2 `ContextRequired` wire payload; `to_requirement()` projects it onto the native `ck_app::ContextRequirement`
- [agentic_capability_loop.rs:ContextPacketCreatedPayload] and [agentic_capability_loop.rs:Citation] - E3 packet and citation wire shapes
- [agentic_capability_loop.rs:verify_domain_identity] - domain identity ingress gate
- [agentic_capability_loop.rs:DomainIdentityError] - error enum of `verify_domain_identity`: `Malformed` (not a 64-character lowercase hex digest) and `Placeholder` (a known pseudo-identity / all-zero)
- [agentic_capability_loop.rs:EventEnvelope] - canonical envelope struct (`event_id`, `event_type`, `namespace`, `occurred_at`, `payload`)
- [agentic_capability_loop.rs:context_packet_envelope] - builds the canonical v1 `ContextPacketCreated` envelope (idempotency key, provenance chain with `caused_by:` lineage)
- [agentic_capability_loop.rs] re-exports `verify_world_ref`, `NoContextReason`, `RetrievalCompleteness`, `WorldRefError`, `OMISSION_CONTENT_TRUNCATED`, `OMISSION_MAX_RESULTS` from `ck-core::types`
- Constants: [agentic_capability_loop.rs:SCHEMA_VERSION_V1], [agentic_capability_loop.rs:CONTEXT_KERNEL_AGENT], [agentic_capability_loop.rs:CAUSALITY_PREFIX], [agentic_capability_loop.rs:FALLBACK_PSEUDO_HASH]
- [cep_bundle.rs:project_context_bundle] - canonical projection of a native resolution ([cep_bundle.rs:BundleInput]) onto the CEP-0008 `godspeed.context_bundle` envelope
- [cep_bundle.rs:to_cep_context_bundle] - deprecated legacy dual-read projection from a `ContextPacketCreatedPayload` alone
- [cep_bundle.rs:validate_context_bundle] - local fail-closed structural mirror of the CEP `context_bundle.v1` profile rules
- [cep_bundle.rs:canonical_json], [cep_bundle.rs:content_hash], [cep_bundle.rs:verify_envelope_integrity] - canonical JSON, `sha256:<hex>` integrity hashing, and post-emission tamper detection; error types [cep_bundle.rs:BundleError] and [cep_bundle.rs:IntegrityError]

## Invariants

- [lib.rs:handle_context_required] - `context_required` is served only when an ACL context agent is attached via `with_context_agent`; requests must be schema_version `v1`, `event_type` `ContextRequired`, produced exclusively by `swe-seed`/`swe_seed`, carry a non-empty `event_id`, a decodable payload, verified domain identity and world identity, and `work_request_id`/`context_requirement_id` correlation - all gates fail closed
- [lib.rs:handle_context_required] - egress is bundle-first: the canonical cross-system artifact is the CEP `godspeed.context_bundle` under the `context_bundle` key; the legacy `context_envelope`/`context_packet` wire shapes are retained for bounded dual-read compatibility and are DEPRECATED - write CEP only
- [lib.rs:handle_context_required] - `access` reports context access (accessible / inaccessible / requires_scope), never action authority; SEA-Forge owns that
- [lib.rs:handle_ingest_context_effects] - not a memory/capability surface: every effect must cite a settlement/evidence artifact (`evidence_ref`), self-reports are rejected, and historical bundles are never rewritten
- [lib.rs:authorize] - with an `AuthPort` attached (`with_app_and_auth`), `authorize_tool` and per-corpus `authorize_corpus` must pass before dispatch; without one, `call_tool_authorized` permits every call
- [lib.rs:context_required_corpus_ids] - a present-but-empty (or non-string) `corpus_id` is rejected rather than treated as "no corpus", so the corpus allowlist is enforced on every wire shape (canonical nested or legacy flat)
- [agentic_capability_loop.rs:verify_domain_identity] - a digest is admissible only as a real SHA-256 hex form and not a known pseudo-identity (`FALLBACK_PSEUDO_HASH` or all-zero); CK never manufactures an identity
- [agentic_capability_loop.rs:AclContextAgent] - retrieval only: CK does not create authority decisions (SEA-Forge owns `AuthorityChecked`); the packet carries authority references passed through from the caller, never computed locally, and all retrieval is delegated to `AppServices::resolve_context` with no degraded, integration-only path
- [agentic_capability_loop.rs:acquire_context] - a caller-declared private corpus must already be registered with its visibility and is never auto-ingested open; operator-configured public corpora are auto-ingested exactly once, and anything else resolves to an explicit no-context reason, never fabricated context
- [agentic_capability_loop.rs:ensure_corpus_ingested] - a `corpus_id` that escapes `corpus_root` via traversal is a hard refusal, not a quiet no-context
- [agentic_capability_loop.rs:ContextAcquisitionResult::outcome] - only `cited` is a successful acquisition; zero citations are explicit and governed (`no_context_governed` when required, `empty_optional` otherwise), never silently successful
- [agentic_capability_loop.rs:acquire_context] - when the resolution reports no context, packet citations are cleared
- [cep_bundle.rs:assemble] - absence is never silent: a non-complete retrieval naming no omission is refused (`BundleError::UnexplainedIncompleteness`) and self-contradictory retrievals are refused (`BundleError::Inconsistent`: complete with no references or with omissions, partial with no references, none with references); an empty retrieval is projected as an unanswered `questions` entry, not references
- [cep_bundle.rs:canonical_json] - float normalization to 6 decimal places and negative-zero flattening is MANDATORY for hash stability across JSON parse round-trips; consumers recomputing a bundle hash MUST apply the same rule
- [cep_bundle.rs:content_hash] - `integrity.content_hash` is `sha256:<hex>` over the canonical JSON of the envelope with the `integrity` block removed; occurrence identity (`envelope_id`), exact content identity (the hash), and semantic-world identity (`scope.world_ref`) are kept separate and never collapsed
- [cep_bundle.rs:to_cep_context_bundle] - deprecated dual-read compatibility; the canonical projection is `project_context_bundle`
- [cep_bundle.rs:validate_context_bundle] - a local fail-closed mirror only; the authoritative validator is the CEP reference implementation

## Side Effects

- [lib.rs:dispatch_tool] - emits `tracing::info!`/`tracing::warn!` telemetry (`tool_name`, `duration_ms`, `outcome`) for every tool call
- [lib.rs:handle_store_context] - base64-decodes `artifact_ref.content` (falling back to raw UTF-8 bytes) before delegating to `AppServices::store_context`
- [agentic_capability_loop.rs:ensure_corpus_ingested] - filesystem access: `canonicalize` and existence checks against `corpus_root` and the learning-proposals ledger, plus triggering `AppServices::ingest_source` for one-time auto-ingestion
- [agentic_capability_loop.rs:packet_for] and [agentic_capability_loop.rs:context_packet_envelope] - generate non-deterministic ULID identifiers and `chrono::Utc::now()` RFC 3339 timestamps
- All persistent storage, retrieval, and ingestion I/O is delegated to `ck_app::AppServices`; this package performs no direct database, network, or stdio I/O

## Verification

- Run: `cargo test -p ck-mcp`
