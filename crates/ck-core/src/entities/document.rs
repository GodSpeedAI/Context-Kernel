use serde::{Deserialize, Serialize};

/// Normalized retrievable unit before chunking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub document_id: String,
    pub corpus_id: String,
    pub source_id: String,
    pub external_id: Option<String>,
    pub library_id: Option<String>,
    pub title: String,
    pub uri: Option<String>,
    pub version: Option<String>,
    pub mime_type: Option<String>,
    pub content_text: String,
    pub metadata_json: String,
    pub checksum: String,
    pub superseded_by_document_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
