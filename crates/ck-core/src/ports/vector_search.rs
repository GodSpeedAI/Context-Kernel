use anyhow::Result;

use crate::entities::ScoredItem;

/// Port for vector similarity search over embeddings.
pub trait VectorSearchPort: Send + Sync {
    /// Store an embedding for a chunk.
    fn upsert_chunk_embedding(&self, chunk_id: &str, model_id: &str, vector: &[f64]) -> Result<()>;

    /// Store an embedding for a session event.
    fn upsert_session_event_embedding(
        &self,
        event_id: &str,
        model_id: &str,
        vector: &[f64],
    ) -> Result<()>;

    /// Search chunks by vector similarity.
    /// Returns scored items with cosine similarity scores.
    fn search_chunks(
        &self,
        query_vector: &[f64],
        model_id: &str,
        corpus_id: Option<&str>,
        library_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>>;

    /// Search session events by vector similarity.
    fn search_session_events(
        &self,
        query_vector: &[f64],
        model_id: &str,
        session_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>>;

    /// Remove embeddings for chunks belonging to a document.
    fn remove_chunk_embeddings_by_document(&self, document_id: &str) -> Result<()>;
}
