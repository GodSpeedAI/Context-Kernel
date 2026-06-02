use anyhow::Result;

use crate::entities::RefreshJob;
use crate::types::JobStatus;

/// Port for refresh job scheduling and queue management.
pub trait SchedulerPort: Send + Sync {
    /// Enqueue a refresh job for a source. Returns the job.
    /// If a queued or running job already exists for this source, returns the existing job (coalescing).
    fn enqueue_refresh(&self, source_id: &str, trigger: &str) -> Result<RefreshJob>;

    /// Claim the next queued job atomically through the storage layer (set status to running).
    fn claim_next_job(&self) -> Result<Option<RefreshJob>>;

    /// Mark a job as completed with stats.
    fn complete_job(&self, job_id: &str, stats_json: &str) -> Result<()>;

    /// Mark a job as failed with an error message.
    fn fail_job(&self, job_id: &str, error: &str) -> Result<()>;

    /// List sources that are stale (last refresh success + TTL < now).
    fn list_stale_sources(&self) -> Result<Vec<String>>;

    /// Count jobs by status.
    fn count_jobs_by_status(&self, status: JobStatus) -> Result<u64>;
}
