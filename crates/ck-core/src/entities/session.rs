use serde::{Deserialize, Serialize};

use crate::types::{ContinuationMode, SessionStatus, SessionTransport};

/// A session for context continuity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub client_id: Option<String>,
    pub transport: SessionTransport,
    pub continuation_mode: ContinuationMode,
    pub status: SessionStatus,
    pub created_at: String,
    pub updated_at: String,
}

/// A session event for context continuity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEvent {
    pub event_id: String,
    pub session_id: String,
    pub kind: String,
    pub text: String,
    pub artifact_id: Option<String>,
    pub metadata_json: String,
    pub pinned: bool,
    pub created_at: String,
}

/// Result from storing context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredContext {
    pub event_id: String,
    pub session_id: String,
    pub kind: String,
    pub stored_as: String,
    pub artifact_id: Option<String>,
    pub created_at: String,
}

/// Result from recalling context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecalledEvent {
    pub event_id: String,
    pub kind: String,
    pub text: String,
    pub artifact_id: Option<String>,
    pub score: f64,
    pub score_breakdown: serde_json::Value,
    pub created_at: String,
}
