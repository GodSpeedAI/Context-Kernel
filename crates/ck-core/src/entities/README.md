# ck-core entities

## Overview

Domain entity records for Context Kernel: the plain data structures that flow between ports and use cases. Each file defines one aggregate area — ingestion sources and their discovered units (`source.rs`), normalized documents and retrieval chunks (`document.rs`, `chunk.rs`), corpora (`corpus.rs`), out-of-band artifact payloads (`artifact.rs`), session-continuity records (`session.rs`), library resolution records (`library.rs`), retrieval query/result envelopes (`retrieval.rs`), and refresh jobs (`job.rs`). All structs are pure serializable data with no behavior; `mod.rs` re-exports every child module publicly.

## Interface

- [mod.rs] - `pub use` re-exports of all child modules (`artifact::*`, `chunk::*`, `corpus::*`, `document::*`, `job::*`, `library::*`, `retrieval::*`, `session::*`, `source::*`).
- [artifact.rs:Artifact] - large payload stored out-of-band: `artifact_id`, optional `session_id`, `kind`, `mime_type`, `storage_uri`, `size_bytes`, `summary`, `checksum`, `created_at`.
- [chunk.rs:Chunk] - atomic retrieval unit: `chunk_id`, `document_id`, `corpus_id`, `ordinal`, `text`, `token_count_estimate`, `heading`, `anchor`, `section_path_json`, `metadata_json`, `checksum`, timestamps.
- [corpus.rs:Corpus] - logical grouping of indexed content: `corpus_id`, `name`, `kind`, `visibility`.
- [document.rs:Document] - normalized retrievable unit before chunking: `document_id`, `corpus_id`, `source_id`, `content_text`, `metadata_json`, `checksum`, `superseded_by_document_id`.
- [job.rs:RefreshJob] - a refresh job: `job_id`, `source_id`, `trigger_kind` (JobTrigger), `status` (JobStatus), `started_at`/`finished_at`, `error`, `stats_json`.
- [job.rs:JobSummary] - compact job projection for MCP responses (`job_id`, `source_id`, `status`, `trigger` as strings).
- [library.rs:LibraryIdentity] - library/document target comparable to Context7 library IDs: `library_id`, `provider`, `name`, optional `version`/`namespace`/`language`/`summary`/`trust_score`, `metadata_json`.
- [library.rs:LibraryAlias] - alias mapping (`alias`, `library_id`, `alias_kind`).
- [library.rs:ResolvedCandidate] - scored resolution candidate: `canonical_id`, `provider`, `name`, `aliases`, `score` (f64), `score_breakdown` (serde_json::Value).
- [retrieval.rs:RetrievalQuery] - retrieval query: `query_id`, `query_text`, `intent` (QueryIntent), optional `canonical_id`, `filters` (serde_json::Value), `budget`, `session_id`, `top_k`, `mode`.
- [retrieval.rs:Budget] - budget constraints (`max_tokens`, `max_chunks`); derives PartialEq/Eq.
- [retrieval.rs:RetrievalResult] - scored retrieval result: `result_id`, optional `canonical_id`/`document_id`/`chunk_id`/`title`/`heading`, `excerpt`, `citation`, `score`, `score_breakdown`, `metadata`.
- [retrieval.rs:ScoredItem] - shared scored search item: `id`, `score` (f64), `score_breakdown`.
- [session.rs:Session] - session record: `session_id`, optional `client_id`, `transport` (SessionTransport), `continuation_mode` (ContinuationMode), `status` (SessionStatus).
- [session.rs:SessionEvent] - session event: `event_id`, `session_id`, `kind`, `text`, optional `artifact_id`, `metadata_json`, `pinned` flag.
- [session.rs:StoredContext] - result of storing context (`event_id`, `session_id`, `kind`, `stored_as`, optional `artifact_id`).
- [session.rs:RecalledEvent] - scored recalled event (`event_id`, `kind`, `text`, optional `artifact_id`, `score`, `score_breakdown`).
- [source.rs:Source] - configured content source: `source_id`, `kind`, `display_name`, `enabled`, `adapter_config_json`, optional `auth_ref`, refresh bookkeeping (`first_indexed_at`, `last_refresh_attempt_at`, `last_refresh_success_at`, `last_refresh_error`), `freshness_ttl_secs`, `upstream_revision`.
- [source.rs:SourceUnit] - discovered unit from a source (e.g. a file path): `external_id`, optional `uri`, `mime_type_hint`.
- [source.rs:RawContent] - raw content fetched from a source unit: `external_id`, optional `uri`, `mime_type`, `bytes` (Vec&lt;u8&gt;), `metadata_json`.

## Invariants

- Entities are plain data: every struct derives `Debug, Clone, Serialize, Deserialize` and carries no methods or I/O; all behavior lives outside this directory ([mod.rs] and child files).
- [chunk.rs:Chunk] is documented as the "Atomic retrieval unit"; each chunk is bound to one `document_id`/`corpus_id` pair and carries an `ordinal` plus a `checksum`.
- [artifact.rs:Artifact] is documented as "Large payload stored out-of-band": it carries `storage_uri` and `checksum` only, never inline content bytes.
- [document.rs:Document] models version supersession via `superseded_by_document_id`; it is the "Normalized retrievable unit before chunking".
- State vocabulary is delegated to `crate::types` enums rather than free strings: [job.rs:RefreshJob] uses `JobTrigger`/`JobStatus`, [session.rs:Session] uses `SessionTransport`/`ContinuationMode`/`SessionStatus`, [retrieval.rs:RetrievalQuery] uses `QueryIntent`.
- Nested structured data is carried in `*_json` String fields (e.g. `section_path_json`, `metadata_json`, `stats_json`, `adapter_config_json`) or `serde_json::Value` fields (`filters`, `citation`, `score_breakdown`).

## Side Effects

- None

## Verification

- Run: `cargo test -p ck-core`
