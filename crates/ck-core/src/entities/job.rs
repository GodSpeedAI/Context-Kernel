use serde::{Deserialize, Serialize};

use crate::types::{JobStatus, JobTrigger};

/// A refresh job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshJob {
    pub job_id: String,
    pub source_id: String,
    pub trigger_kind: JobTrigger,
    pub status: JobStatus,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub error: Option<String>,
    pub stats_json: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Summary of a job for MCP responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub job_id: String,
    pub source_id: String,
    pub status: String,
    pub trigger: String,
}
