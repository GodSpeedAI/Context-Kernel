use anyhow::Result;
use chrono::Utc;
use ck_core::entities::RefreshJob;
use ck_core::ports::{SchedulerPort, StoragePort};
use ck_core::types::{JobStatus, JobTrigger, SourceHealth};
use tracing::{debug, instrument};
use ulid::Ulid;

pub struct RefreshScheduler {
    storage: Box<dyn StoragePort>,
}

impl RefreshScheduler {
    pub fn new(storage: Box<dyn StoragePort>) -> Self {
        Self { storage }
    }
}

impl SchedulerPort for RefreshScheduler {
    #[instrument(skip(self), fields(source_id = %source_id))]
    fn enqueue_refresh(&self, source_id: &str, trigger: &str) -> Result<RefreshJob> {
        let now = Utc::now().to_rfc3339();
        let trigger_kind = match trigger {
            "startup" => JobTrigger::Startup,
            "scheduled" => JobTrigger::Scheduled,
            "webhook" => JobTrigger::Webhook,
            "dependency_change" => JobTrigger::DependencyChange,
            _ => JobTrigger::Manual,
        };

        let job = RefreshJob {
            job_id: format!("job_{}", Ulid::new()),
            source_id: source_id.to_string(),
            trigger_kind,
            status: JobStatus::Queued,
            started_at: None,
            finished_at: None,
            error: None,
            stats_json: "{}".to_string(),
            created_at: now.clone(),
            updated_at: now,
        };

        let stored = self.storage.insert_active_refresh_job(&job)?;
        if stored.job_id == job.job_id {
            debug!(job_id = %stored.job_id, source_id, "enqueued refresh job");
        } else {
            debug!(job_id = %stored.job_id, "coalescing: active job already exists");
        }
        Ok(stored)
    }

    #[instrument(skip(self))]
    fn claim_next_job(&self) -> Result<Option<RefreshJob>> {
        let sources = self.storage.list_sources()?;
        for source in &sources {
            if let Some(claimed) = self
                .storage
                .claim_next_refresh_job_for_source(&source.source_id)?
            {
                debug!(job_id = %claimed.job_id, "claimed refresh job");
                return Ok(Some(claimed));
            }
        }
        Ok(None)
    }

    #[instrument(skip(self), fields(job_id = %job_id))]
    fn complete_job(&self, job_id: &str, stats_json: &str) -> Result<()> {
        if let Some(mut job) = self.storage.get_refresh_job(job_id)? {
            let now = Utc::now().to_rfc3339();
            job.status = JobStatus::Completed;
            job.finished_at = Some(now.clone());
            job.stats_json = stats_json.to_string();
            job.updated_at = now;
            self.storage.upsert_refresh_job(&job)?;
            self.storage
                .update_source_health(&job.source_id, SourceHealth::Healthy, None)?;
            debug!(job_id, "completed refresh job");
        }
        Ok(())
    }

    #[instrument(skip(self), fields(job_id = %job_id))]
    fn fail_job(&self, job_id: &str, error: &str) -> Result<()> {
        if let Some(mut job) = self.storage.get_refresh_job(job_id)? {
            let now = Utc::now().to_rfc3339();
            job.status = JobStatus::Failed;
            job.finished_at = Some(now.clone());
            job.error = Some(error.to_string());
            job.updated_at = now;
            self.storage.upsert_refresh_job(&job)?;
            self.storage.update_source_health(
                &job.source_id,
                SourceHealth::Degraded,
                Some(error),
            )?;
            debug!(job_id, error, "failed refresh job");
        }
        Ok(())
    }

    #[instrument(skip(self))]
    fn list_stale_sources(&self) -> Result<Vec<String>> {
        let sources = self.storage.list_stale_sources()?;
        Ok(sources.into_iter().map(|s| s.source_id).collect())
    }

    #[instrument(skip(self))]
    fn count_jobs_by_status(&self, status: JobStatus) -> Result<u64> {
        let status_str = match status {
            JobStatus::Queued => "queued",
            JobStatus::Running => "running",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
        };
        self.storage.count_jobs_by_status(status_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ck_core::entities::Source;
    use ck_core::types::SourceHealth;
    use std::sync::Mutex;

    struct MockStorage {
        jobs: Mutex<Vec<RefreshJob>>,
        sources: Mutex<Vec<Source>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                jobs: Mutex::new(Vec::new()),
                sources: Mutex::new(Vec::new()),
            }
        }

        fn add_source(&self, source: Source) {
            self.sources.lock().unwrap().push(source);
        }
    }

    impl StoragePort for MockStorage {
        fn upsert_source(&self, _source: &Source) -> Result<()> {
            Ok(())
        }
        fn get_source(&self, _source_id: &str) -> Result<Option<Source>> {
            Ok(None)
        }
        fn list_sources(&self) -> Result<Vec<Source>> {
            Ok(self.sources.lock().unwrap().clone())
        }
        fn update_source_health(
            &self,
            _source_id: &str,
            _health: SourceHealth,
            _error: Option<&str>,
        ) -> Result<()> {
            Ok(())
        }
        fn list_stale_sources(&self) -> Result<Vec<Source>> {
            Ok(self.sources.lock().unwrap().clone())
        }
        fn upsert_corpus(&self, _corpus: &ck_core::entities::Corpus) -> Result<()> {
            Ok(())
        }
        fn get_corpus(&self, _corpus_id: &str) -> Result<Option<ck_core::entities::Corpus>> {
            Ok(None)
        }
        fn list_corpora(&self) -> Result<Vec<ck_core::entities::Corpus>> {
            Ok(vec![])
        }
        fn link_source_to_corpus(&self, _corpus_id: &str, _source_id: &str) -> Result<()> {
            Ok(())
        }
        fn find_corpus_id_for_source(&self, _source_id: &str) -> Result<Option<String>> {
            Ok(None)
        }
        fn upsert_document(&self, _doc: &ck_core::entities::Document) -> Result<()> {
            Ok(())
        }
        fn get_document(&self, _document_id: &str) -> Result<Option<ck_core::entities::Document>> {
            Ok(None)
        }
        fn delete_document(&self, _document_id: &str) -> Result<()> {
            Ok(())
        }
        fn list_documents_by_source(
            &self,
            _source_id: &str,
        ) -> Result<Vec<ck_core::entities::Document>> {
            Ok(vec![])
        }
        fn list_documents_by_corpus(
            &self,
            _corpus_id: &str,
        ) -> Result<Vec<ck_core::entities::Document>> {
            Ok(vec![])
        }
        fn get_chunk_with_document(
            &self,
            _chunk_id: &str,
        ) -> Result<Option<(ck_core::entities::Chunk, ck_core::entities::Document)>> {
            Ok(None)
        }
        fn replace_chunks(
            &self,
            _document_id: &str,
            _chunks: &[ck_core::entities::Chunk],
        ) -> Result<()> {
            Ok(())
        }
        fn list_chunks_by_document(
            &self,
            _document_id: &str,
        ) -> Result<Vec<ck_core::entities::Chunk>> {
            Ok(vec![])
        }
        fn list_chunks_by_corpus(&self, _corpus_id: &str) -> Result<Vec<ck_core::entities::Chunk>> {
            Ok(vec![])
        }
        fn upsert_library_identity(&self, _lib: &ck_core::entities::LibraryIdentity) -> Result<()> {
            Ok(())
        }
        fn get_library_identity(
            &self,
            _library_id: &str,
        ) -> Result<Option<ck_core::entities::LibraryIdentity>> {
            Ok(None)
        }
        fn list_library_identities(&self) -> Result<Vec<ck_core::entities::LibraryIdentity>> {
            Ok(vec![])
        }
        fn upsert_library_alias(&self, _alias: &ck_core::entities::LibraryAlias) -> Result<()> {
            Ok(())
        }
        fn list_library_aliases(
            &self,
            _library_id: &str,
        ) -> Result<Vec<ck_core::entities::LibraryAlias>> {
            Ok(vec![])
        }
        fn upsert_refresh_job(&self, job: &RefreshJob) -> Result<()> {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(idx) = jobs.iter().position(|j| j.job_id == job.job_id) {
                jobs[idx] = job.clone();
            } else {
                jobs.push(job.clone());
            }
            Ok(())
        }
        fn insert_active_refresh_job(&self, job: &RefreshJob) -> Result<RefreshJob> {
            if let Some(existing) = self.find_active_job_for_source(&job.source_id)? {
                return Ok(existing);
            }
            self.upsert_refresh_job(job)?;
            Ok(job.clone())
        }
        fn get_refresh_job(&self, job_id: &str) -> Result<Option<RefreshJob>> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .find(|j| j.job_id == job_id)
                .cloned())
        }
        fn list_refresh_jobs_by_source(&self, source_id: &str) -> Result<Vec<RefreshJob>> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .filter(|j| j.source_id == source_id)
                .cloned()
                .collect())
        }
        fn find_active_job_for_source(&self, source_id: &str) -> Result<Option<RefreshJob>> {
            Ok(self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .find(|j| {
                    j.source_id == source_id
                        && (j.status == JobStatus::Queued || j.status == JobStatus::Running)
                })
                .cloned())
        }
        fn claim_next_refresh_job_for_source(&self, source_id: &str) -> Result<Option<RefreshJob>> {
            let mut jobs = self.jobs.lock().unwrap();
            let Some(job) = jobs
                .iter_mut()
                .filter(|j| j.source_id == source_id && j.status == JobStatus::Queued)
                .min_by(|a, b| a.created_at.cmp(&b.created_at))
            else {
                return Ok(None);
            };
            let now = Utc::now().to_rfc3339();
            job.status = JobStatus::Running;
            job.started_at = Some(now.clone());
            job.updated_at = now;
            Ok(Some(job.clone()))
        }
        fn count_jobs_by_status(&self, status_str: &str) -> Result<u64> {
            let count = self
                .jobs
                .lock()
                .unwrap()
                .iter()
                .filter(|j| {
                    let s = match j.status {
                        JobStatus::Queued => "queued",
                        JobStatus::Running => "running",
                        JobStatus::Completed => "completed",
                        JobStatus::Failed => "failed",
                        JobStatus::Cancelled => "cancelled",
                    };
                    s == status_str
                })
                .count() as u64;
            Ok(count)
        }
    }

    fn test_source(id: &str) -> Source {
        Source {
            source_id: id.to_string(),
            kind: "filesystem".to_string(),
            display_name: id.to_string(),
            enabled: true,
            adapter_config_json: "{}".to_string(),
            auth_ref: None,
            first_indexed_at: None,
            last_refresh_attempt_at: None,
            last_refresh_success_at: None,
            last_refresh_error: None,
            freshness_ttl_secs: None,
            upstream_revision: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn enqueue_coalesces_when_active_job_exists() {
        let storage = Box::new(MockStorage::new());
        let scheduler = RefreshScheduler::new(storage);

        let job1 = scheduler.enqueue_refresh("src1", "manual").unwrap();
        assert_eq!(job1.status, JobStatus::Queued);
        assert_eq!(job1.source_id, "src1");

        let job2 = scheduler.enqueue_refresh("src1", "manual").unwrap();
        assert_eq!(job1.job_id, job2.job_id, "should return the same job");
    }

    #[test]
    fn enqueue_creates_new_job_when_no_active_job() {
        let storage = Box::new(MockStorage::new());
        let scheduler = RefreshScheduler::new(storage);

        let job1 = scheduler.enqueue_refresh("src1", "manual").unwrap();
        let job2 = scheduler.enqueue_refresh("src2", "manual").unwrap();
        assert_ne!(job1.job_id, job2.job_id);
    }

    #[test]
    fn claim_next_job_claims_first_queued() {
        let mock = Box::new(MockStorage::new());
        mock.add_source(test_source("src1"));
        let scheduler = RefreshScheduler::new(mock);

        scheduler.enqueue_refresh("src1", "manual").unwrap();
        let claimed = scheduler.claim_next_job().unwrap();
        assert!(claimed.is_some());
        let claimed = claimed.unwrap();
        assert_eq!(claimed.status, JobStatus::Running);
        assert!(claimed.started_at.is_some());
    }

    #[test]
    fn claim_next_job_returns_none_when_empty() {
        let mock = Box::new(MockStorage::new());
        let scheduler = RefreshScheduler::new(mock);
        assert!(scheduler.claim_next_job().unwrap().is_none());
    }

    #[test]
    fn complete_job_transitions_status() {
        let mock = Box::new(MockStorage::new());
        mock.add_source(test_source("src1"));
        let scheduler = RefreshScheduler::new(mock);

        let job = scheduler.enqueue_refresh("src1", "manual").unwrap();
        scheduler.claim_next_job().unwrap();
        scheduler
            .complete_job(&job.job_id, r#"{"docs":5}"#)
            .unwrap();

        let stored = scheduler
            .storage
            .get_refresh_job(&job.job_id)
            .unwrap()
            .unwrap();
        assert_eq!(stored.status, JobStatus::Completed);
        assert!(stored.finished_at.is_some());
        assert_eq!(stored.stats_json, r#"{"docs":5}"#);
    }

    #[test]
    fn fail_job_transitions_status() {
        let mock = Box::new(MockStorage::new());
        mock.add_source(test_source("src1"));
        let scheduler = RefreshScheduler::new(mock);

        let job = scheduler.enqueue_refresh("src1", "manual").unwrap();
        scheduler.claim_next_job().unwrap();
        scheduler
            .fail_job(&job.job_id, "connection refused")
            .unwrap();

        let stored = scheduler
            .storage
            .get_refresh_job(&job.job_id)
            .unwrap()
            .unwrap();
        assert_eq!(stored.status, JobStatus::Failed);
        assert_eq!(stored.error.as_deref(), Some("connection refused"));
        assert!(stored.finished_at.is_some());
    }

    #[test]
    fn count_jobs_by_status_counts_correctly() {
        let mock = Box::new(MockStorage::new());
        mock.add_source(test_source("src1"));
        mock.add_source(test_source("src2"));
        let scheduler = RefreshScheduler::new(mock);

        scheduler.enqueue_refresh("src1", "manual").unwrap();
        scheduler.enqueue_refresh("src2", "manual").unwrap();

        assert_eq!(
            scheduler.count_jobs_by_status(JobStatus::Queued).unwrap(),
            2
        );
        assert_eq!(
            scheduler.count_jobs_by_status(JobStatus::Running).unwrap(),
            0
        );
    }

    #[test]
    fn list_stale_sources_returns_ids() {
        let mock = Box::new(MockStorage::new());
        mock.add_source(test_source("stale1"));
        let scheduler = RefreshScheduler::new(mock);

        let stale = scheduler.list_stale_sources().unwrap();
        assert_eq!(stale, vec!["stale1".to_string()]);
    }

    #[test]
    fn trigger_kind_mapping() {
        let storage = Box::new(MockStorage::new());
        let scheduler = RefreshScheduler::new(storage);

        let job = scheduler.enqueue_refresh("s1", "startup").unwrap();
        assert_eq!(job.trigger_kind, JobTrigger::Startup);

        let job = scheduler.enqueue_refresh("s2", "scheduled").unwrap();
        assert_eq!(job.trigger_kind, JobTrigger::Scheduled);

        let job = scheduler.enqueue_refresh("s3", "webhook").unwrap();
        assert_eq!(job.trigger_kind, JobTrigger::Webhook);

        let job = scheduler
            .enqueue_refresh("s4", "dependency_change")
            .unwrap();
        assert_eq!(job.trigger_kind, JobTrigger::DependencyChange);

        let job = scheduler.enqueue_refresh("s5", "unknown").unwrap();
        assert_eq!(job.trigger_kind, JobTrigger::Manual);
    }
}
