use serde::{Deserialize, Serialize};

use crate::types::QueryIntent;

/// A retrieval query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalQuery {
    pub query_id: String,
    pub query_text: String,
    pub intent: QueryIntent,
    pub canonical_id: Option<String>,
    pub filters: serde_json::Value,
    pub budget: Option<Budget>,
    pub session_id: Option<String>,
    pub top_k: i64,
    pub mode: Option<String>,
}

/// Budget constraints for retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    pub max_tokens: i64,
    pub max_chunks: i64,
}

/// A retrieval result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub result_id: String,
    pub canonical_id: Option<String>,
    pub document_id: Option<String>,
    pub chunk_id: Option<String>,
    pub title: Option<String>,
    pub heading: Option<String>,
    pub excerpt: String,
    pub citation: serde_json::Value,
    pub score: f64,
    pub score_breakdown: serde_json::Value,
    pub metadata: serde_json::Value,
}

/// A scored item from lexical search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredItem {
    pub id: String,
    pub score: f64,
    pub score_breakdown: serde_json::Value,
}
