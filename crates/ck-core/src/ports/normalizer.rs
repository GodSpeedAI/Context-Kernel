use anyhow::Result;

use crate::entities::{Document, RawContent};

/// Port for normalizing raw content into documents.
pub trait Normalizer: Send + Sync {
    fn normalize(
        &self,
        raw: &RawContent,
        source_id: &str,
        corpus_id: &str,
    ) -> Result<Vec<Document>>;
}
