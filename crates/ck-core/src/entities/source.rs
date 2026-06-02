use serde::{Deserialize, Serialize};

/// Represents a configured content source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub source_id: String,
    pub kind: String,
    pub display_name: String,
    pub enabled: bool,
    pub adapter_config_json: String,
    pub auth_ref: Option<String>,
    pub first_indexed_at: Option<String>,
    pub last_refresh_attempt_at: Option<String>,
    pub last_refresh_success_at: Option<String>,
    pub last_refresh_error: Option<String>,
    pub freshness_ttl_secs: Option<i64>,
    pub upstream_revision: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A discovered unit from a source (e.g., a file path).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceUnit {
    pub external_id: String,
    pub uri: Option<String>,
    pub mime_type_hint: Option<String>,
}

/// Raw content fetched from a source unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContent {
    pub external_id: String,
    pub uri: Option<String>,
    pub mime_type: String,
    pub bytes: Vec<u8>,
    pub metadata_json: String,
}
