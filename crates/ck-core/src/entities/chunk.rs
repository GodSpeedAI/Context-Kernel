use serde::{Deserialize, Serialize};

/// Atomic retrieval unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub chunk_id: String,
    pub document_id: String,
    pub corpus_id: String,
    pub ordinal: i64,
    pub text: String,
    pub token_count_estimate: i64,
    pub heading: Option<String>,
    pub anchor: Option<String>,
    pub section_path_json: String,
    pub metadata_json: String,
    pub checksum: String,
    pub created_at: String,
    pub updated_at: String,
}
