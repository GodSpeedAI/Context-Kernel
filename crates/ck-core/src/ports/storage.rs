use anyhow::Result;

use crate::entities::*;

/// Port for persistent storage of all entities.
pub trait StoragePort: Send + Sync {
    // Sources
    fn upsert_source(&self, source: &Source) -> Result<()>;
    fn get_source(&self, source_id: &str) -> Result<Option<Source>>;
    fn list_sources(&self) -> Result<Vec<Source>>;

    // Corpora
    fn upsert_corpus(&self, corpus: &Corpus) -> Result<()>;
    fn get_corpus(&self, corpus_id: &str) -> Result<Option<Corpus>>;
    fn list_corpora(&self) -> Result<Vec<Corpus>>;

    // Documents
    fn upsert_document(&self, doc: &Document) -> Result<()>;
    fn get_document(&self, document_id: &str) -> Result<Option<Document>>;
    fn delete_document(&self, document_id: &str) -> Result<()>;
    fn list_documents_by_source(&self, source_id: &str) -> Result<Vec<Document>>;
    fn list_documents_by_corpus(&self, corpus_id: &str) -> Result<Vec<Document>>;

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
    fn get_refresh_job(&self, job_id: &str) -> Result<Option<RefreshJob>>;
    fn list_refresh_jobs_by_source(&self, source_id: &str) -> Result<Vec<RefreshJob>>;
}
