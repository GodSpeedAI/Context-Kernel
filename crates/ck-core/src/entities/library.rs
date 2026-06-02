use serde::{Deserialize, Serialize};

/// Represents a library/document target comparable to Context7 library IDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryIdentity {
    pub library_id: String,
    pub provider: String,
    pub name: String,
    pub version: Option<String>,
    pub namespace: Option<String>,
    pub language: Option<String>,
    pub summary: Option<String>,
    pub trust_score: Option<f64>,
    pub metadata_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// An alias for a library identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryAlias {
    pub alias: String,
    pub library_id: String,
    pub alias_kind: String,
}

/// A resolved candidate from library resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedCandidate {
    pub canonical_id: String,
    pub provider: String,
    pub name: String,
    pub version: Option<String>,
    pub aliases: Vec<String>,
    pub summary: Option<String>,
    pub trust_score: Option<f64>,
    pub score: f64,
    pub score_breakdown: serde_json::Value,
}
