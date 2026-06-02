use anyhow::Result;

use crate::entities::Chunk;

/// Port for splitting documents into retrieval chunks.
pub trait Chunker: Send + Sync {
    fn chunk(
        &self,
        document_id: &str,
        corpus_id: &str,
        text: &str,
        title: &str,
    ) -> Result<Vec<Chunk>>;
}
