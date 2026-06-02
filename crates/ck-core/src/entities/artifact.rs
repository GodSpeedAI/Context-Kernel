use serde::{Deserialize, Serialize};

/// Large payload stored out-of-band.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub artifact_id: String,
    pub session_id: Option<String>,
    pub kind: String,
    pub mime_type: Option<String>,
    pub storage_uri: String,
    pub size_bytes: i64,
    pub summary: Option<String>,
    pub checksum: String,
    pub created_at: String,
}
