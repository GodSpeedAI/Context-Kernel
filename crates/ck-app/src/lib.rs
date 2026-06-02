use anyhow::Result;
use chrono::Utc;
use ck_core::entities::*;
use ck_core::ports::*;
use ck_core::types::*;
use serde_json::json;
use ulid::Ulid;

/// Application use cases for Context Kernel.
/// Each use case orchestrates ports to accomplish a domain operation.
pub struct AppServices {
    storage: Box<dyn StoragePort>,
    session_memory: Box<dyn SessionMemoryPort>,
    artifact_store: Box<dyn ArtifactPort>,
    search: Box<dyn LexicalSearchPort>,
    ranker: Box<dyn RankerPort>,
    source_adapter: Box<dyn SourceAdapter>,
    normalizer: Box<dyn Normalizer>,
    chunker: Box<dyn Chunker>,
}

impl AppServices {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        storage: Box<dyn StoragePort>,
        session_memory: Box<dyn SessionMemoryPort>,
        artifact_store: Box<dyn ArtifactPort>,
        search: Box<dyn LexicalSearchPort>,
        ranker: Box<dyn RankerPort>,
        source_adapter: Box<dyn SourceAdapter>,
        normalizer: Box<dyn Normalizer>,
        chunker: Box<dyn Chunker>,
    ) -> Self {
        Self {
            storage,
            session_memory,
            artifact_store,
            search,
            ranker,
            source_adapter,
            normalizer,
            chunker,
        }
    }
}

// --- store_context ---

#[derive(Debug, Clone)]
pub struct StoreContextRequest {
    pub session_id: String,
    pub kind: String,
    pub text: Option<String>,
    pub artifact_content: Option<Vec<u8>>,
    pub artifact_content_type: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct StoreContextResponse {
    pub event: StoredContext,
}

/// Large payload threshold in bytes (10 KB).
const LARGE_PAYLOAD_THRESHOLD: usize = 10_240;

impl AppServices {
    pub fn store_context(&self, req: StoreContextRequest) -> Result<StoreContextResponse> {
        // Ensure session exists
        self.session_memory.ensure_session(
            &req.session_id,
            SessionTransport::Stdio,
            ContinuationMode::Ephemeral,
        )?;

        let metadata_str = serde_json::to_string(&req.metadata)?;

        if let Some(content) = &req.artifact_content {
            // Store as artifact
            let artifact_id = format!("art_{}", Ulid::new());
            let artifact = self.artifact_store.store_artifact(
                &artifact_id,
                Some(&req.session_id),
                &req.kind,
                req.artifact_content_type.as_deref(),
                content,
            )?;

            let text = req.text.as_deref().unwrap_or_default();
            let display_text = if text.is_empty() {
                artifact.summary.as_deref().unwrap_or_default()
            } else {
                text
            };
            let event = self.session_memory.add_event(
                &req.session_id,
                &req.kind,
                display_text,
                Some(&artifact_id),
                &metadata_str,
                false,
            )?;

            // Index the event for search
            self.search.index_session_event(
                &event.event_id,
                &req.session_id,
                &req.kind,
                display_text,
            )?;

            Ok(StoreContextResponse { event })
        } else if let Some(text) = &req.text {
            if text.len() > LARGE_PAYLOAD_THRESHOLD {
                // Store large text as artifact
                let artifact_id = format!("art_{}", Ulid::new());
                let content_type = "text/plain".to_string();
                self.artifact_store.store_artifact(
                    &artifact_id,
                    Some(&req.session_id),
                    &req.kind,
                    Some(&content_type),
                    text.as_bytes(),
                )?;

                let summary: String = text.chars().take(500).collect();
                let event = self.session_memory.add_event(
                    &req.session_id,
                    &req.kind,
                    &summary,
                    Some(&artifact_id),
                    &metadata_str,
                    false,
                )?;

                self.search.index_session_event(
                    &event.event_id,
                    &req.session_id,
                    &req.kind,
                    &summary,
                )?;

                Ok(StoreContextResponse { event })
            } else {
                // Store as event directly
                let event = self.session_memory.add_event(
                    &req.session_id,
                    &req.kind,
                    text,
                    None,
                    &metadata_str,
                    false,
                )?;

                self.search.index_session_event(
                    &event.event_id,
                    &req.session_id,
                    &req.kind,
                    text,
                )?;

                Ok(StoreContextResponse { event })
            }
        } else {
            anyhow::bail!("store_context requires either text or artifact_content");
        }
    }
}

// --- recall_context ---

#[derive(Debug, Clone)]
pub struct RecallContextRequest {
    pub session_id: Option<String>,
    pub query: String,
    pub scope: RecallScope,
    pub top_k: i64,
}

#[derive(Debug, Clone)]
pub struct RecallContextResponse {
    pub results: Vec<RecalledEvent>,
    pub query_id: String,
}

impl AppServices {
    pub fn recall_context(&self, req: RecallContextRequest) -> Result<RecallContextResponse> {
        let query_id = format!("qry_{}", Ulid::new());
        let safe_top_k = req.top_k.max(1);

        // Honor RecallScope:
        //   Current → restrict to req.session_id
        //   Recent  → restrict to req.session_id (same session filter)
        //   All     → no session filter
        let session_filter = match req.scope {
            RecallScope::All => None,
            RecallScope::Current | RecallScope::Recent => req.session_id.as_deref(),
        };

        let scored =
            self.search
                .search_session_events(&req.query, session_filter, safe_top_k * 3)?;

        let results = self.ranker.rank_recall_results(&req.query, scored)?;

        let results = results.into_iter().take(safe_top_k as usize).collect();

        Ok(RecallContextResponse { results, query_id })
    }
}

// --- ingest_source ---

#[derive(Debug, Clone)]
pub struct IngestSourceRequest {
    pub source_id: String,
    pub source_kind: String,
    pub adapter_config_json: String,
    pub corpus_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IngestSourceResponse {
    pub job: JobSummary,
}

impl AppServices {
    pub fn ingest_source(&self, req: IngestSourceRequest) -> Result<IngestSourceResponse> {
        let now = Utc::now().to_rfc3339();
        let corpus_id = req
            .corpus_id
            .unwrap_or_else(|| format!("corp_{}", req.source_id));

        // Ensure source exists in storage
        let source = Source {
            source_id: req.source_id.clone(),
            kind: req.source_kind.clone(),
            display_name: req.source_id.clone(),
            enabled: true,
            adapter_config_json: req.adapter_config_json.clone(),
            auth_ref: None,
            first_indexed_at: Some(now.clone()),
            last_refresh_attempt_at: Some(now.clone()),
            last_refresh_success_at: None,
            last_refresh_error: None,
            freshness_ttl_secs: None,
            upstream_revision: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        self.storage.upsert_source(&source)?;

        // Ensure corpus exists
        let corpus = Corpus {
            corpus_id: corpus_id.clone(),
            name: req.source_id.clone(),
            kind: "library_docs".to_string(),
            visibility: "local".to_string(),
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        self.storage.upsert_corpus(&corpus)?;

        // Discover and fetch units
        let units = self
            .source_adapter
            .discover_units(&req.source_id, &req.adapter_config_json)?;

        let mut docs_indexed: i64 = 0;
        let mut chunks_indexed: i64 = 0;

        for unit in &units {
            let raw = self.source_adapter.fetch_content(
                &req.source_id,
                unit,
                &req.adapter_config_json,
            )?;

            let documents = self
                .normalizer
                .normalize(&raw, &req.source_id, &corpus_id)?;

            for doc in &documents {
                // Upsert document
                self.storage.upsert_document(doc)?;

                // Chunk the document
                let chunks = self.chunker.chunk(
                    &doc.document_id,
                    &corpus_id,
                    &doc.content_text,
                    &doc.title,
                )?;

                // Remove old FTS entries for this document
                self.search.remove_chunks_by_document(&doc.document_id)?;

                // Replace chunks in storage
                self.storage.replace_chunks(&doc.document_id, &chunks)?;

                // Index chunks in FTS
                for chunk in &chunks {
                    self.search.index_chunk(
                        &chunk.chunk_id,
                        &chunk.document_id,
                        &chunk.corpus_id,
                        doc.library_id.as_deref(),
                        &doc.title,
                        chunk.heading.as_deref().unwrap_or(""),
                        &chunk.text,
                    )?;
                }

                docs_indexed += 1;
                chunks_indexed += chunks.len() as i64;
            }
        }

        // Create a refresh job record
        let job_id = format!("job_{}", Ulid::new());
        let job = RefreshJob {
            job_id: job_id.clone(),
            source_id: req.source_id.clone(),
            trigger_kind: JobTrigger::Manual,
            status: JobStatus::Completed,
            started_at: Some(now.clone()),
            finished_at: Some(now.clone()),
            error: None,
            stats_json: serde_json::to_string(&json!({
                "docs_indexed": docs_indexed,
                "chunks_indexed": chunks_indexed
            }))?,
            created_at: now.clone(),
            updated_at: now,
        };
        self.storage.upsert_refresh_job(&job)?;

        Ok(IngestSourceResponse {
            job: JobSummary {
                job_id,
                source_id: req.source_id,
                status: "completed".to_string(),
                trigger: "manual".to_string(),
            },
        })
    }
}

// --- query_reference ---

#[derive(Debug, Clone)]
pub struct QueryReferenceRequest {
    pub canonical_id: Option<String>,
    pub query: String,
    pub mode: Option<String>,
    pub top_k: i64,
    pub version: Option<String>,
    pub filters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct QueryReferenceResponse {
    pub results: Vec<RetrievalResult>,
    pub query_id: String,
    pub mode: String,
    pub truncated: bool,
}

impl AppServices {
    pub fn query_reference(&self, req: QueryReferenceRequest) -> Result<QueryReferenceResponse> {
        let query_id = format!("qry_{}", Ulid::new());
        let mode = req.mode.unwrap_or_else(|| "lexical".to_string());
        let safe_top_k = req.top_k.max(1);

        // Determine corpus/library filter from canonical_id
        let (corpus_id, library_id): (Option<String>, Option<String>) =
            if let Some(cid) = &req.canonical_id {
                // Try to find a library identity
                if self.storage.get_library_identity(cid)?.is_some() {
                    (None, Some(cid.clone())) // library filter only
                } else {
                    // Treat as corpus_id
                    (Some(cid.clone()), None)
                }
            } else {
                (None, None)
            };

        // Search chunks via FTS
        let scored = self.search.search_chunks(
            &req.query,
            corpus_id.as_deref(),
            library_id.as_deref(),
            safe_top_k * 3,
        )?;

        let results = self.ranker.rank_retrieval_results(&req.query, scored)?;

        let truncated = results.len() > safe_top_k as usize;
        let results = results.into_iter().take(safe_top_k as usize).collect();

        Ok(QueryReferenceResponse {
            results,
            query_id,
            mode,
            truncated,
        })
    }
}

// --- resolve_reference ---

#[derive(Debug, Clone)]
pub struct ResolveReferenceRequest {
    pub query: String,
    pub kind: Option<String>,
    pub version: Option<String>,
    pub source_scope: Option<Vec<String>>,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct ResolveReferenceResponse {
    pub candidates: Vec<ResolvedCandidate>,
    pub query_id: String,
    pub mode: String,
}

impl AppServices {
    pub fn resolve_reference(
        &self,
        req: ResolveReferenceRequest,
    ) -> Result<ResolveReferenceResponse> {
        let query_id = format!("qry_{}", Ulid::new());

        // Search library aliases via FTS
        let scored = self
            .search
            .search_library_aliases(&req.query, req.limit * 3)?;

        // Enrich with library metadata
        let mut candidates_with_metadata = Vec::new();
        for item in scored {
            if let Some(lib) = self.storage.get_library_identity(&item.id)? {
                let aliases = self.storage.list_library_aliases(&item.id)?;
                let alias_strs: Vec<String> = aliases.iter().map(|a| a.alias.clone()).collect();

                candidates_with_metadata.push(ResolvedCandidate {
                    canonical_id: lib.library_id,
                    provider: lib.provider,
                    name: lib.name,
                    version: lib.version,
                    aliases: alias_strs,
                    summary: lib.summary,
                    trust_score: lib.trust_score,
                    score: item.score,
                    score_breakdown: item.score_breakdown,
                });
            }
        }

        // Sort by score descending
        candidates_with_metadata.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let candidates = candidates_with_metadata
            .into_iter()
            .take(req.limit as usize)
            .collect();

        Ok(ResolveReferenceResponse {
            candidates,
            query_id,
            mode: "resolve".to_string(),
        })
    }
}

// --- get_status ---

impl AppServices {
    pub fn get_status(
        &self,
        started_at: &str,
        storage_status: StorageStatus,
        default_mode: &str,
        semantic_enabled: bool,
    ) -> Result<KernelStatus> {
        let sources = self.storage.list_sources()?;
        let total_sources = sources.len() as u64;
        let enabled_sources = sources.iter().filter(|s| s.enabled).count() as u64;

        let active_sessions = self.session_memory.count_active_sessions()?;

        let mut status = KernelStatus::new(
            started_at.to_string(),
            storage_status,
            default_mode,
            semantic_enabled,
        );
        status.sources.total = total_sources;
        status.sources.enabled = enabled_sources;
        status.sessions.active = active_sessions;

        Ok(status)
    }
}
