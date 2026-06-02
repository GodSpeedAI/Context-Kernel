use anyhow::Result;

use crate::entities::*;
use crate::types::SourceHealth;

/// Port for persistent storage of all entities.
pub trait StoragePort: Send + Sync {
    // Sources
    fn upsert_source(&self, source: &Source) -> Result<()>;
    fn get_source(&self, source_id: &str) -> Result<Option<Source>>;
    fn list_sources(&self) -> Result<Vec<Source>>;

    /// Update source health state (sets last_refresh_error for degraded).
    fn update_source_health(
        &self,
        source_id: &str,
        health: SourceHealth,
        error: Option<&str>,
    ) -> Result<()>;

    /// List sources that are stale based on freshness_ttl_secs.
    fn list_stale_sources(&self) -> Result<Vec<Source>>;

    // Corpora
    fn upsert_corpus(&self, corpus: &Corpus) -> Result<()>;
    fn get_corpus(&self, corpus_id: &str) -> Result<Option<Corpus>>;
    fn list_corpora(&self) -> Result<Vec<Corpus>>;
    fn link_source_to_corpus(&self, corpus_id: &str, source_id: &str) -> Result<()>;
    fn find_corpus_id_for_source(&self, source_id: &str) -> Result<Option<String>>;

    // Documents
    fn upsert_document(&self, doc: &Document) -> Result<()>;
    fn get_document(&self, document_id: &str) -> Result<Option<Document>>;
    fn delete_document(&self, document_id: &str) -> Result<()>;
    fn list_documents_by_source(&self, source_id: &str) -> Result<Vec<Document>>;
    fn list_documents_by_corpus(&self, corpus_id: &str) -> Result<Vec<Document>>;

    /// Get a chunk with its parent document for citation building.
    fn get_chunk_with_document(&self, chunk_id: &str) -> Result<Option<(Chunk, Document)>>;

    // Chunks
    fn replace_chunks(&self, document_id: &str, chunks: &[Chunk]) -> Result<()>;
    fn list_chunks_by_document(&self, document_id: &str) -> Result<Vec<Chunk>>;
    fn list_chunks_by_corpus(&self, corpus_id: &str) -> Result<Vec<Chunk>>;

    // Library identities
    fn upsert_library_identity(&self, lib: &LibraryIdentity) -> Result<()>;
    fn get_library_identity(&self, library_id: &str) -> Result<Option<LibraryIdentity>>;
    fn list_library_identities(&self) -> Result<Vec<LibraryIdentity>>;
    fn upsert_library_alias(&self, alias: &LibraryAlias) -> Result<()>;
    fn list_library_aliases(&self, library_id: &str) -> Result<Vec<LibraryAlias>>;

    // Refresh jobs
    fn upsert_refresh_job(&self, job: &RefreshJob) -> Result<()>;
    fn insert_active_refresh_job(&self, job: &RefreshJob) -> Result<RefreshJob>;
    fn get_refresh_job(&self, job_id: &str) -> Result<Option<RefreshJob>>;
    fn list_refresh_jobs_by_source(&self, source_id: &str) -> Result<Vec<RefreshJob>>;

    /// Find a queued or running job for a source (for coalescing).
    fn find_active_job_for_source(&self, source_id: &str) -> Result<Option<RefreshJob>>;

    /// Atomically claim the oldest queued refresh job for a source.
    fn claim_next_refresh_job_for_source(&self, source_id: &str) -> Result<Option<RefreshJob>>;

    /// Count jobs by status string.
    fn count_jobs_by_status(&self, status_str: &str) -> Result<u64>;

    /// Delete retention-eligible session events older than a timestamp, preserving pinned events.
    fn delete_unpinned_session_events_older_than(&self, cutoff_rfc3339: &str) -> Result<u64>;

    /// Delete closed/compacted sessions older than a timestamp.
    fn delete_inactive_sessions_older_than(&self, cutoff_rfc3339: &str) -> Result<u64>;

    /// Delete finished refresh jobs older than a timestamp.
    fn delete_finished_refresh_jobs_older_than(&self, cutoff_rfc3339: &str) -> Result<u64>;

    /// Delete disabled sources older than a timestamp.
    fn delete_disabled_sources_older_than(&self, cutoff_rfc3339: &str) -> Result<u64>;

    /// Insert or replace artifact metadata.
    fn upsert_artifact(&self, artifact: &Artifact) -> Result<()>;

    /// Delete artifact metadata rows older than a timestamp.
    fn delete_artifacts_older_than(&self, cutoff_rfc3339: &str) -> Result<u64>;

    /// List artifact IDs older than a cutoff timestamp.
    fn list_artifacts_older_than(&self, cutoff_rfc3339: &str) -> Result<Vec<String>>;
}
