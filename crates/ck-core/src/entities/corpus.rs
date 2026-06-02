use serde::{Deserialize, Serialize};

/// Logical grouping of indexed content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corpus {
    pub corpus_id: String,
    pub name: String,
    pub kind: String,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
}
