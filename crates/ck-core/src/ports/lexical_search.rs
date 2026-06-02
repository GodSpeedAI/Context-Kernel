use anyhow::Result;

use crate::entities::ScoredItem;

/// Port for lexical search using FTS5/BM25.
#[allow(clippy::too_many_arguments)]
pub trait LexicalSearchPort: Send + Sync {
    /// Index a chunk for search.
    fn index_chunk(
        &self,
        chunk_id: &str,
        document_id: &str,
        corpus_id: &str,
        library_id: Option<&str>,
        title: &str,
        heading: &str,
        text: &str,
    ) -> Result<()>;

    /// Remove chunks for a document from the search index.
    fn remove_chunks_by_document(&self, document_id: &str) -> Result<()>;

    /// Search chunks by query text.
    fn search_chunks(
        &self,
        query: &str,
        corpus_id: Option<&str>,
        library_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>>;

    /// Index a session event for search.
    fn index_session_event(
        &self,
        event_id: &str,
        session_id: &str,
        kind: &str,
        text: &str,
    ) -> Result<()>;

    /// Search session events by query text.
    fn search_session_events(
        &self,
        query: &str,
        session_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>>;

    /// Index a library alias for search.
    fn index_library_alias(
        &self,
        alias: &str,
        library_id: &str,
        name: &str,
        provider: &str,
    ) -> Result<()>;

    /// Search library aliases by query text.
    fn search_library_aliases(&self, query: &str, limit: i64) -> Result<Vec<ScoredItem>>;
}
