# ck-core ports

## Overview

The hexagonal port surface of Context Kernel. Each file declares one `Send + Sync` trait that the application layer depends on and infrastructure adapters implement: persistence (`StoragePort`, `ArtifactPort`, `SessionMemoryPort`), search indices (`LexicalSearchPort`, `VectorSearchPort`), ingestion pipeline stages (`SourceAdapter`, `Normalizer`, `Chunker`), retrieval signals (`EmbeddingPort`, `RankerPort`), refresh-job scheduling (`SchedulerPort`), and authn/authz (`AuthPort`). `mod.rs` re-exports every child module publicly. The traits declare no infrastructure themselves; concrete effects live in the adapter crates that implement them.

## Interface

- [mod.rs] - `pub use` re-exports of all child modules.
- [storage.rs:StoragePort] - persistence for all entities: upsert/get/list for `Source`, `Corpus`, `Document`, `Chunk`, `LibraryIdentity`, `LibraryAlias`, `RefreshJob`, `Artifact`; corpus-source linking (`link_source_to_corpus`, `find_corpus_id_for_source`); `get_chunk_with_document`; active-job lookup/claim; staleness and health queries; retention deletes for session events, sessions, refresh jobs, disabled sources, and artifacts.
- [artifact.rs:ArtifactPort] - artifact payload persistence: `store_artifact`, `get_artifact`, `read_artifact_content`, `delete_artifact_payload`.
- [session_memory.rs:SessionMemoryPort] - session persistence and recall: `create_session`, `get_session`, `ensure_session` (default impl: get-or-create), `add_event`, `get_event`, `list_events_by_session`, `count_active_sessions`.
- [lexical_search.rs:LexicalSearchPort] - lexical search documented as "using FTS5/BM25": `index_chunk` / `remove_chunks_by_document` / `search_chunks`, `index_session_event` / `search_session_events`, `index_library_alias` / `search_library_aliases`; searches return `Vec<ScoredItem>`.
- [vector_search.rs:VectorSearchPort] - vector similarity search: `upsert_chunk_embedding` / `upsert_session_event_embedding` (keyed by `model_id`), `search_chunks` / `search_session_events`, `remove_chunk_embeddings_by_document`; default method `upsert_chunk_embedding_with_metadata` delegates to `upsert_chunk_embedding`.
- [embedding.rs:EmbeddingPort] - text embedding provider: `embed` / `embed_batch` (Option-returning for unavailable provider), `model_id`, `dimensions`, `is_available`.
- [ranker.rs:RankerPort] - ranking/combination of retrieval signals: `rank_resolution_candidates`, `rank_retrieval_results`, `rank_recall_results`; [ranker.rs:RankContext] carries `intent` (QueryIntent), optional `session_id`, and a `debug` flag.
- [scheduler.rs:SchedulerPort] - refresh job queue: `enqueue_refresh`, `claim_next_job`, `complete_job`, `fail_job`, `list_stale_sources`, `count_jobs_by_status`.
- [source_adapter.rs:SourceAdapter] - content discovery and fetch: `kind`, `discover_units`, `fetch_content`.
- [normalizer.rs:Normalizer] - converts `RawContent` into `Vec<Document>` (`normalize`).
- [chunker.rs:Chunker] - splits document text into `Vec<Chunk>` (`chunk`).
- [auth.rs:AuthPort] - `validate_token`, `is_local_trust_mode`, `authorize_tool`, `authorize_corpus`.

## Invariants

- [session_memory.rs:SessionMemoryPort] doc comment: "Implementations MUST make `create_session` idempotent (e.g. INSERT OR IGNORE / upsert with a UNIQUE constraint on `session_id`) so that `ensure_session` does not race under concurrent callers."
- [scheduler.rs:SchedulerPort::claim_next_job] doc comment: "Claim the next queued job atomically through the storage layer (set status to running)."
- [scheduler.rs:SchedulerPort::enqueue_refresh] doc comment: coalescing — "If a queued or running job already exists for this source, returns the existing job."
- [storage.rs:StoragePort::claim_next_refresh_job_for_source] doc comment: "Atomically claim the oldest queued refresh job for a source."
- [storage.rs:StoragePort::delete_unpinned_session_events_older_than] doc comment: retention deletes are "preserving pinned events".
- [storage.rs:StoragePort::update_source_health] doc comment: "sets last_refresh_error for degraded".
- [storage.rs:StoragePort::list_stale_sources] doc comment: staleness is "based on freshness_ttl_secs".
- [embedding.rs:EmbeddingPort] doc comments: `embed`/`embed_batch` return `None` per item when the embedding provider is unavailable, instead of erroring.
- [vector_search.rs:VectorSearchPort::search_chunks] doc comment: results carry "cosine similarity scores".
- Every trait requires `Send + Sync` and returns `anyhow::Result` (except [auth.rs:AuthPort], whose methods return `bool`).

## Side Effects

- None directly — the traits declare no I/O; documented implementation expectations are FTS5/BM25 backing for lexical search ([lexical_search.rs:LexicalSearchPort]) and storage-layer atomic job claims ([scheduler.rs:SchedulerPort::claim_next_job]), fulfilled by adapter crates.

## Verification

- Run: `cargo test -p ck-core`
