use anyhow::Result;
use chrono::{Duration, Utc};
use ck_core::entities::*;
use ck_core::ports::*;
use ck_core::types::*;
use serde_json::json;
use ulid::Ulid;

pub mod context;
pub mod context_effect;

pub use context::*;
pub use context_effect::*;

pub use ck_memory::{
    classify_event_kind, EventRetentionClass, SessionRetentionPolicy as MemoryRetentionPolicy,
};

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
    embedding: Option<Box<dyn EmbeddingPort>>,
    vector_search: Option<Box<dyn VectorSearchPort>>,
    /// Additional source adapters selected by `SourceAdapter::kind()` during
    /// ingestion (e.g. the learning-proposals JSONL ledger adapter).
    kind_adapters: Vec<(String, Box<dyn SourceAdapter>)>,
    /// Settlement-backed context-effect ledger for prospective retrieval
    /// calibration. Absent = calibration disabled (never faked).
    context_effects: Option<std::sync::Arc<context_effect::ContextEffectStore>>,
}

// --- retention GC ---

#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    pub session_event_days: i64,
    pub inactive_session_days: i64,
    pub refresh_job_days: i64,
    pub artifact_days: i64,
    pub disabled_source_days: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RetentionGcReport {
    pub session_events_deleted: u64,
    pub inactive_sessions_deleted: u64,
    pub refresh_jobs_deleted: u64,
    pub artifacts_deleted: u64,
    pub disabled_sources_deleted: u64,
    pub artifact_payload_errors: u64,
}

impl AppServices {
    pub fn run_retention_gc(&self, policy: RetentionPolicy) -> Result<RetentionGcReport> {
        let now = Utc::now();
        let event_cutoff = (now - Duration::days(policy.session_event_days)).to_rfc3339();
        let session_cutoff = (now - Duration::days(policy.inactive_session_days)).to_rfc3339();
        let job_cutoff = (now - Duration::days(policy.refresh_job_days)).to_rfc3339();
        let artifact_cutoff = (now - Duration::days(policy.artifact_days)).to_rfc3339();
        let source_cutoff = (now - Duration::days(policy.disabled_source_days)).to_rfc3339();

        let expired_artifact_ids = self.storage.list_artifacts_older_than(&artifact_cutoff)?;
        let mut artifact_payload_errors: u64 = 0;
        for artifact_id in &expired_artifact_ids {
            if let Err(e) = self.artifact_store.delete_artifact_payload(artifact_id) {
                tracing::warn!(artifact_id = %artifact_id, error = %e, "failed to delete artifact payload");
                artifact_payload_errors += 1;
            }
        }

        let report = RetentionGcReport {
            session_events_deleted: self
                .storage
                .delete_unpinned_session_events_older_than(&event_cutoff)?,
            inactive_sessions_deleted: self
                .storage
                .delete_inactive_sessions_older_than(&session_cutoff)?,
            refresh_jobs_deleted: self
                .storage
                .delete_finished_refresh_jobs_older_than(&job_cutoff)?,
            artifacts_deleted: self.storage.delete_artifacts_older_than(&artifact_cutoff)?,
            disabled_sources_deleted: self
                .storage
                .delete_disabled_sources_older_than(&source_cutoff)?,
            artifact_payload_errors,
        };

        tracing::info!(
            session_events_deleted = report.session_events_deleted,
            inactive_sessions_deleted = report.inactive_sessions_deleted,
            refresh_jobs_deleted = report.refresh_jobs_deleted,
            artifacts_deleted = report.artifacts_deleted,
            disabled_sources_deleted = report.disabled_sources_deleted,
            artifact_payload_errors = report.artifact_payload_errors,
            "retention gc completed"
        );

        Ok(report)
    }
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
            embedding: None,
            vector_search: None,
            kind_adapters: Vec::new(),
            context_effects: None,
        }
    }

    /// Set optional embedding and vector search ports for semantic retrieval.
    pub fn with_semantic(
        mut self,
        embedding: Box<dyn EmbeddingPort>,
        vector_search: Box<dyn VectorSearchPort>,
    ) -> Self {
        self.embedding = Some(embedding);
        self.vector_search = Some(vector_search);
        self
    }

    /// Register an additional source adapter by kind. Ingestion selects the
    /// adapter whose `SourceAdapter::kind()` matches `IngestSourceRequest::source_kind`;
    /// unregistered kinds fall back to the default adapter.
    pub fn with_source_adapter_kind(
        mut self,
        kind: impl Into<String>,
        adapter: Box<dyn SourceAdapter>,
    ) -> Self {
        self.kind_adapters.push((kind.into(), adapter));
        self
    }

    /// Attach the settlement-backed context-effect ledger so prospective
    /// retrieval calibration is enabled.
    pub fn with_context_effect_store(mut self, store: context_effect::ContextEffectStore) -> Self {
        self.context_effects = Some(std::sync::Arc::new(store));
        self
    }

    /// Whether semantic retrieval is available.
    pub fn semantic_available(&self) -> bool {
        self.embedding.as_ref().is_some_and(|e| e.is_available())
    }

    fn adapter_for_kind(&self, kind: &str) -> &dyn SourceAdapter {
        self.kind_adapters
            .iter()
            .find(|(k, _)| k == kind)
            .map(|(_, a)| a.as_ref())
            .unwrap_or(self.source_adapter.as_ref())
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

            if let Err(e) = self.storage.upsert_artifact(&artifact) {
                let _ = self
                    .artifact_store
                    .delete_artifact_payload(&artifact.artifact_id);
                return Err(e);
            }

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
                let artifact = self.artifact_store.store_artifact(
                    &artifact_id,
                    Some(&req.session_id),
                    &req.kind,
                    Some(&content_type),
                    text.as_bytes(),
                )?;

                if let Err(e) = self.storage.upsert_artifact(&artifact) {
                    let _ = self
                        .artifact_store
                        .delete_artifact_payload(&artifact.artifact_id);
                    return Err(e);
                }

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

        let session_filter = match req.scope {
            RecallScope::All => None,
            RecallScope::Current | RecallScope::Recent => req.session_id.as_deref(),
        };

        let scored =
            self.search
                .search_session_events(&req.query, session_filter, safe_top_k * 3)?;

        let rank_ctx = RankContext {
            intent: QueryIntent::RecallContext,
            session_id: req.session_id.clone(),
            debug: false,
        };
        let results = self
            .ranker
            .rank_recall_results(&req.query, scored, rank_ctx)?;

        let results = results.into_iter().take(safe_top_k as usize).collect();

        Ok(RecallContextResponse { results, query_id })
    }
}

// --- ingest_source ---

fn stable_document_id(source_id: &str, external_id: &str, uri: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    source_id.hash(&mut hasher);
    external_id.hash(&mut hasher);
    uri.hash(&mut hasher);
    format!("doc_{:016x}", hasher.finish())
}

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

        let ingest_result: Result<(i64, i64)> = (|| {
            // Ensure corpus exists. An existing corpus KEEPS its registered
            // visibility: re-ingestion must never open a private corpus.
            let existing = self.storage.get_corpus(&corpus_id)?;
            let corpus = Corpus {
                corpus_id: corpus_id.clone(),
                name: existing
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| req.source_id.clone()),
                kind: "library_docs".to_string(),
                visibility: existing
                    .as_ref()
                    .map(|c| c.visibility.clone())
                    .unwrap_or_else(|| "local".to_string()),
                created_at: existing
                    .as_ref()
                    .map(|c| c.created_at.clone())
                    .unwrap_or_else(|| now.clone()),
                updated_at: now.clone(),
            };
            self.storage.upsert_corpus(&corpus)?;
            self.storage
                .link_source_to_corpus(&corpus_id, &req.source_id)?;

            // Discover and fetch the entire snapshot BEFORE mutating previous
            // rows. An adapter/fetch failure leaves the old snapshot untouched.
            let adapter = self.adapter_for_kind(&req.source_kind);
            let units = adapter.discover_units(&req.source_id, &req.adapter_config_json)?;

            // Fetch + normalize + chunk all units into memory first.
            struct PreparedDoc {
                document: Document,
                chunks: Vec<Chunk>,
            }
            let mut prepared: Vec<PreparedDoc> = Vec::new();

            for unit in &units {
                let raw = adapter.fetch_content(&req.source_id, unit, &req.adapter_config_json)?;

                let documents = self
                    .normalizer
                    .normalize(&raw, &req.source_id, &corpus_id)?;

                let stable_documents: Vec<Document> = documents
                    .into_iter()
                    .map(|mut doc| {
                        doc.document_id = stable_document_id(
                            &req.source_id,
                            doc.external_id.as_deref().unwrap_or(&raw.external_id),
                            doc.uri.as_deref().unwrap_or(&raw.external_id),
                        );
                        doc
                    })
                    .collect();

                for doc in &stable_documents {
                    let chunks = self.chunker.chunk(
                        &doc.document_id,
                        &corpus_id,
                        &doc.content_text,
                        &doc.title,
                    )?;
                    prepared.push(PreparedDoc {
                        document: doc.clone(),
                        chunks,
                    });
                }
            }

            // Snapshot prepared — now it's safe to replace previous rows.
            // Delete old documents + FTS entries.
            for doc in self.storage.list_documents_by_source(&req.source_id)? {
                self.search.remove_chunks_by_document(&doc.document_id)?;
                self.storage.delete_document(&doc.document_id)?;
            }

            let mut docs_indexed: i64 = 0;
            let mut chunks_indexed: i64 = 0;

            for prepared_doc in &prepared {
                self.storage.upsert_document(&prepared_doc.document)?;

                self.storage
                    .replace_chunks(&prepared_doc.document.document_id, &prepared_doc.chunks)?;

                for chunk in &prepared_doc.chunks {
                    self.search.index_chunk(
                        &chunk.chunk_id,
                        &chunk.document_id,
                        &chunk.corpus_id,
                        prepared_doc.document.library_id.as_deref(),
                        &prepared_doc.document.title,
                        chunk.heading.as_deref().unwrap_or(""),
                        &chunk.text,
                    )?;
                }

                docs_indexed += 1;
                chunks_indexed += prepared_doc.chunks.len() as i64;
            }
            Ok((docs_indexed, chunks_indexed))
        })();

        let (docs_indexed, chunks_indexed) = match ingest_result {
            Ok(stats) => stats,
            Err(e) => {
                let error = e.to_string();
                self.storage.update_source_health(
                    &req.source_id,
                    SourceHealth::Degraded,
                    Some(&error),
                )?;
                return Err(e);
            }
        };

        // Mark source as healthy
        self.storage
            .update_source_health(&req.source_id, SourceHealth::Healthy, None)?;

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
        let mode = req.mode.unwrap_or_else(|| "hybrid".to_string());
        let safe_top_k = req.top_k.max(1);

        // Determine corpus/library filter from canonical_id
        let (corpus_id, library_id): (Option<String>, Option<String>) =
            if let Some(cid) = &req.canonical_id {
                if self.storage.get_library_identity(cid)?.is_some() {
                    (None, Some(cid.clone()))
                } else {
                    (Some(cid.clone()), None)
                }
            } else {
                (None, None)
            };

        if let Some(cid) = &corpus_id {
            if let Some(corpus) = self.storage.get_corpus(cid)? {
                if corpus.visibility == "private" {
                    // POL-ACL-002: a private corpus requires EXPLICIT scope.
                    // A missing or empty source_scope never grants access.
                    let allowed_scopes: Vec<String> = req
                        .filters
                        .get("source_scope")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    if !allowed_scopes.contains(&cid.to_string()) {
                        anyhow::bail!(
                            "unauthorized corpus: {} is private and not in source_scope",
                            cid
                        );
                    }
                }
            }
        }

        // Search chunks via FTS (lexical)
        let mut scored = self.search.search_chunks(
            &req.query,
            corpus_id.as_deref(),
            library_id.as_deref(),
            safe_top_k * 3,
        )?;

        // Semantic search when available and mode allows it
        let use_semantic = (mode == "semantic" || mode == "hybrid") && self.semantic_available();
        if use_semantic {
            if let (Some(embedding), Some(vector_search)) = (&self.embedding, &self.vector_search) {
                if let Ok(Some(query_vector)) = embedding.embed(&req.query) {
                    let model_id = embedding.model_id();
                    match vector_search.search_chunks(
                        &query_vector,
                        model_id,
                        corpus_id.as_deref(),
                        library_id.as_deref(),
                        safe_top_k * 3,
                    ) {
                        Ok(semantic_results) => {
                            // Merge semantic results into scored items
                            for sr in semantic_results {
                                if let Some(existing) = scored.iter_mut().find(|s| s.id == sr.id) {
                                    // Add semantic score to existing breakdown
                                    if existing.score_breakdown.as_object_mut().is_none() {
                                        existing.score_breakdown = serde_json::json!({});
                                    }
                                    if let Some(m) = existing.score_breakdown.as_object_mut() {
                                        m.insert(
                                            "semantic".to_string(),
                                            serde_json::json!(sr.score),
                                        );
                                    }
                                } else {
                                    scored.push(sr);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "semantic search failed, using lexical only");
                        }
                    }
                }
            }
        }

        // Enrich scored items with trust and freshness metadata
        let enriched = self.enrich_scored_items(scored)?;

        // Build rank context with query intent
        let context = RankContext {
            intent: QueryIntent::QueryDocs,
            session_id: None,
            debug: false,
        };

        let results = self
            .ranker
            .rank_retrieval_results(&req.query, enriched, context)?;

        // Build citations and excerpts for each result
        let mut final_results = Vec::new();
        for mut result in results {
            if let Some(chunk_id) = &result.chunk_id {
                if let Some((chunk, doc)) = self.storage.get_chunk_with_document(chunk_id)? {
                    result.document_id = Some(doc.document_id.clone());
                    result.title = Some(doc.title.clone());
                    result.heading = chunk.heading;
                    result.excerpt = build_excerpt(&chunk.text, 200);
                    result.citation = build_citation(&doc, chunk.anchor.as_deref());
                    result.canonical_id = doc.library_id.clone();
                    result.metadata = serde_json::json!({
                        "source_id": doc.source_id,
                        "corpus_id": doc.corpus_id,
                        "version": doc.version,
                    });
                }
            }
            final_results.push(result);
        }

        let truncated = final_results.len() > safe_top_k as usize;
        let final_results = final_results
            .into_iter()
            .take(safe_top_k as usize)
            .collect();

        Ok(QueryReferenceResponse {
            results: final_results,
            query_id,
            mode,
            truncated,
        })
    }

    /// Enrich scored items with trust_score and freshness from storage metadata.
    fn enrich_scored_items(&self, items: Vec<ScoredItem>) -> Result<Vec<ScoredItem>> {
        let mut enriched = Vec::new();
        for item in items {
            let mut breakdown = item.score_breakdown;

            // Try to get trust score from library identity
            if let Some((chunk, doc)) = self.storage.get_chunk_with_document(&item.id)? {
                if let Some(lib_id) = &doc.library_id {
                    if let Some(lib) = self.storage.get_library_identity(lib_id)? {
                        if let Some(ts) = lib.trust_score {
                            if let Some(m) = breakdown.as_object_mut() {
                                m.insert("trust_score".to_string(), serde_json::json!(ts));
                            }
                        }
                    }
                }

                // Compute freshness from source
                if let Some(source) = self.storage.get_source(&doc.source_id)? {
                    let freshness = compute_freshness(
                        source.last_refresh_success_at.as_deref(),
                        source.freshness_ttl_secs,
                    );
                    if let Some(m) = breakdown.as_object_mut() {
                        m.insert("freshness".to_string(), serde_json::json!(freshness));
                    }
                }
                // We don't need the chunk/doc anymore, but we had to query them
                let _ = (chunk, doc);
            }

            enriched.push(ScoredItem {
                id: item.id,
                score: item.score,
                score_breakdown: breakdown,
            });
        }
        Ok(enriched)
    }
}

/// Build a token-efficient excerpt from chunk text.
fn build_excerpt(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars).collect();
    format!("{}…", truncated)
}

/// Build a citation object from document metadata.
fn build_citation(doc: &Document, anchor: Option<&str>) -> serde_json::Value {
    let mut citation = serde_json::Map::new();
    if let Some(uri) = &doc.uri {
        citation.insert("uri".to_string(), serde_json::json!(uri));
    }
    if let Some(a) = anchor {
        citation.insert("anchor".to_string(), serde_json::json!(a));
    }
    serde_json::Value::Object(citation)
}

/// Compute a freshness score (0.0 to 1.0). 1.0 = just refreshed; 0.0 = older
/// than the TTL window. When no TTL is configured, a 30-day default decay is
/// used. Malformed/empty timestamps score 0.5 (uncertain) or 0.0 (absent).
fn compute_freshness(last_success: Option<&str>, ttl_secs: Option<i64>) -> f64 {
    let Some(ts) = last_success.filter(|s| !s.is_empty()) else {
        return 0.0;
    };
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts) else {
        return 0.5;
    };
    let age_secs = (Utc::now()
        .signed_duration_since(parsed.with_timezone(&Utc))
        .num_seconds()
        .max(0)) as f64;
    let ttl = ttl_secs
        .filter(|t| *t > 0)
        .map(|t| t as f64)
        .unwrap_or(30.0 * 24.0 * 3600.0);
    (1.0 - (age_secs / ttl)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct AppTestState {
        sources: Vec<Source>,
        corpora: Vec<Corpus>,
        documents: Vec<Document>,
        refresh_jobs: Vec<RefreshJob>,
        source_corpora: Vec<(String, String)>,
        health_updates: Vec<(String, SourceHealth, Option<String>)>,
        normalized_corpora: Vec<String>,
        fetch_should_fail: bool,
        retention_calls: Vec<String>,
        fail_upsert_artifact: bool,
        artifact_upsert_calls: Vec<String>,
    }

    #[derive(Clone)]
    struct SharedState(Arc<Mutex<AppTestState>>);

    impl SharedState {
        fn new() -> Self {
            Self(Arc::new(Mutex::new(AppTestState::default())))
        }
    }

    struct TestStorage {
        state: SharedState,
    }

    impl StoragePort for TestStorage {
        fn upsert_source(&self, source: &Source) -> Result<()> {
            let mut state = self.state.0.lock().unwrap();
            if let Some(existing) = state
                .sources
                .iter_mut()
                .find(|s| s.source_id == source.source_id)
            {
                *existing = source.clone();
            } else {
                state.sources.push(source.clone());
            }
            Ok(())
        }

        fn get_source(&self, source_id: &str) -> Result<Option<Source>> {
            Ok(self
                .state
                .0
                .lock()
                .unwrap()
                .sources
                .iter()
                .find(|s| s.source_id == source_id)
                .cloned())
        }

        fn list_sources(&self) -> Result<Vec<Source>> {
            Ok(self.state.0.lock().unwrap().sources.clone())
        }

        fn update_source_health(
            &self,
            source_id: &str,
            health: SourceHealth,
            error: Option<&str>,
        ) -> Result<()> {
            self.state.0.lock().unwrap().health_updates.push((
                source_id.to_string(),
                health,
                error.map(str::to_string),
            ));
            Ok(())
        }

        fn list_stale_sources(&self) -> Result<Vec<Source>> {
            Ok(vec![])
        }

        fn upsert_corpus(&self, corpus: &Corpus) -> Result<()> {
            self.state.0.lock().unwrap().corpora.push(corpus.clone());
            Ok(())
        }

        fn get_corpus(&self, corpus_id: &str) -> Result<Option<Corpus>> {
            Ok(self
                .state
                .0
                .lock()
                .unwrap()
                .corpora
                .iter()
                .find(|c| c.corpus_id == corpus_id)
                .cloned())
        }

        fn list_corpora(&self) -> Result<Vec<Corpus>> {
            Ok(self.state.0.lock().unwrap().corpora.clone())
        }

        fn link_source_to_corpus(&self, corpus_id: &str, source_id: &str) -> Result<()> {
            let mut state = self.state.0.lock().unwrap();
            if !state
                .source_corpora
                .iter()
                .any(|(s, c)| s == source_id && c == corpus_id)
            {
                state
                    .source_corpora
                    .push((source_id.to_string(), corpus_id.to_string()));
            }
            Ok(())
        }

        fn find_corpus_id_for_source(&self, source_id: &str) -> Result<Option<String>> {
            Ok(self
                .state
                .0
                .lock()
                .unwrap()
                .source_corpora
                .iter()
                .find(|(s, _)| s == source_id)
                .map(|(_, c)| c.clone()))
        }

        fn upsert_document(&self, doc: &Document) -> Result<()> {
            self.state.0.lock().unwrap().documents.push(doc.clone());
            Ok(())
        }

        fn get_document(&self, _document_id: &str) -> Result<Option<Document>> {
            Ok(None)
        }

        fn delete_document(&self, document_id: &str) -> Result<()> {
            let mut state = self.state.0.lock().unwrap();
            state.documents.retain(|d| d.document_id != document_id);
            Ok(())
        }

        fn list_documents_by_source(&self, source_id: &str) -> Result<Vec<Document>> {
            Ok(self
                .state
                .0
                .lock()
                .unwrap()
                .documents
                .iter()
                .filter(|d| d.source_id == source_id)
                .cloned()
                .collect())
        }

        fn list_documents_by_corpus(&self, _corpus_id: &str) -> Result<Vec<Document>> {
            Ok(vec![])
        }

        fn get_chunk_with_document(&self, _chunk_id: &str) -> Result<Option<(Chunk, Document)>> {
            Ok(None)
        }

        fn replace_chunks(&self, _document_id: &str, _chunks: &[Chunk]) -> Result<()> {
            Ok(())
        }

        fn list_chunks_by_document(&self, _document_id: &str) -> Result<Vec<Chunk>> {
            Ok(vec![])
        }

        fn list_chunks_by_corpus(&self, _corpus_id: &str) -> Result<Vec<Chunk>> {
            Ok(vec![])
        }

        fn upsert_library_identity(&self, _lib: &LibraryIdentity) -> Result<()> {
            Ok(())
        }

        fn get_library_identity(&self, _library_id: &str) -> Result<Option<LibraryIdentity>> {
            Ok(None)
        }

        fn list_library_identities(&self) -> Result<Vec<LibraryIdentity>> {
            Ok(vec![])
        }

        fn upsert_library_alias(&self, _alias: &LibraryAlias) -> Result<()> {
            Ok(())
        }

        fn list_library_aliases(&self, _library_id: &str) -> Result<Vec<LibraryAlias>> {
            Ok(vec![])
        }

        fn upsert_refresh_job(&self, job: &RefreshJob) -> Result<()> {
            self.state.0.lock().unwrap().refresh_jobs.push(job.clone());
            Ok(())
        }

        fn insert_active_refresh_job(&self, job: &RefreshJob) -> Result<RefreshJob> {
            self.upsert_refresh_job(job)?;
            Ok(job.clone())
        }

        fn get_refresh_job(&self, _job_id: &str) -> Result<Option<RefreshJob>> {
            Ok(None)
        }

        fn list_refresh_jobs_by_source(&self, _source_id: &str) -> Result<Vec<RefreshJob>> {
            Ok(vec![])
        }

        fn find_active_job_for_source(&self, _source_id: &str) -> Result<Option<RefreshJob>> {
            Ok(None)
        }

        fn claim_next_refresh_job_for_source(
            &self,
            _source_id: &str,
        ) -> Result<Option<RefreshJob>> {
            Ok(None)
        }

        fn count_jobs_by_status(&self, _status_str: &str) -> Result<u64> {
            Ok(0)
        }

        fn delete_unpinned_session_events_older_than(&self, _cutoff_rfc3339: &str) -> Result<u64> {
            self.state
                .0
                .lock()
                .unwrap()
                .retention_calls
                .push("session_events".to_string());
            Ok(1)
        }

        fn delete_inactive_sessions_older_than(&self, _cutoff_rfc3339: &str) -> Result<u64> {
            self.state
                .0
                .lock()
                .unwrap()
                .retention_calls
                .push("inactive_sessions".to_string());
            Ok(2)
        }

        fn delete_finished_refresh_jobs_older_than(&self, _cutoff_rfc3339: &str) -> Result<u64> {
            self.state
                .0
                .lock()
                .unwrap()
                .retention_calls
                .push("refresh_jobs".to_string());
            Ok(3)
        }

        fn delete_disabled_sources_older_than(&self, _cutoff_rfc3339: &str) -> Result<u64> {
            self.state
                .0
                .lock()
                .unwrap()
                .retention_calls
                .push("disabled_sources".to_string());
            Ok(5)
        }

        fn delete_artifacts_older_than(&self, _cutoff_rfc3339: &str) -> Result<u64> {
            self.state
                .0
                .lock()
                .unwrap()
                .retention_calls
                .push("artifacts".to_string());
            Ok(4)
        }

        fn list_artifacts_older_than(&self, _cutoff_rfc3339: &str) -> Result<Vec<String>> {
            Ok(vec![])
        }

        fn upsert_artifact(&self, artifact: &Artifact) -> Result<()> {
            let mut state = self.state.0.lock().unwrap();
            state
                .artifact_upsert_calls
                .push(artifact.artifact_id.clone());
            if state.fail_upsert_artifact {
                anyhow::bail!("injected upsert_artifact failure");
            }
            Ok(())
        }
    }

    struct NoopSessionMemory;

    impl SessionMemoryPort for NoopSessionMemory {
        fn create_session(
            &self,
            _session_id: &str,
            _transport: SessionTransport,
            _continuation_mode: ContinuationMode,
        ) -> Result<Session> {
            unimplemented!()
        }

        fn get_session(&self, _session_id: &str) -> Result<Option<Session>> {
            Ok(None)
        }

        fn add_event(
            &self,
            _session_id: &str,
            _kind: &str,
            _text: &str,
            _artifact_id: Option<&str>,
            _metadata_json: &str,
            _pinned: bool,
        ) -> Result<StoredContext> {
            unimplemented!()
        }

        fn get_event(&self, _event_id: &str) -> Result<Option<SessionEvent>> {
            Ok(None)
        }

        fn list_events_by_session(&self, _session_id: &str) -> Result<Vec<SessionEvent>> {
            Ok(vec![])
        }

        fn count_active_sessions(&self) -> Result<u64> {
            Ok(0)
        }
    }

    struct NoopArtifactStore;

    impl ArtifactPort for NoopArtifactStore {
        fn store_artifact(
            &self,
            _artifact_id: &str,
            _session_id: Option<&str>,
            _kind: &str,
            _mime_type: Option<&str>,
            _content: &[u8],
        ) -> Result<Artifact> {
            unimplemented!()
        }

        fn get_artifact(&self, _artifact_id: &str) -> Result<Option<Artifact>> {
            Ok(None)
        }

        fn read_artifact_content(&self, _artifact_id: &str) -> Result<Option<Vec<u8>>> {
            Ok(None)
        }

        fn delete_artifact_payload(&self, _artifact_id: &str) -> Result<bool> {
            Ok(false)
        }
    }

    struct NoopSearch;

    impl LexicalSearchPort for NoopSearch {
        fn index_chunk(
            &self,
            _chunk_id: &str,
            _document_id: &str,
            _corpus_id: &str,
            _library_id: Option<&str>,
            _title: &str,
            _heading: &str,
            _text: &str,
        ) -> Result<()> {
            Ok(())
        }

        fn remove_chunks_by_document(&self, _document_id: &str) -> Result<()> {
            Ok(())
        }

        fn search_chunks(
            &self,
            _query: &str,
            _corpus_id: Option<&str>,
            _library_id: Option<&str>,
            _limit: i64,
        ) -> Result<Vec<ScoredItem>> {
            Ok(vec![])
        }

        fn index_session_event(
            &self,
            _event_id: &str,
            _session_id: &str,
            _kind: &str,
            _text: &str,
        ) -> Result<()> {
            Ok(())
        }

        fn search_session_events(
            &self,
            _query: &str,
            _session_id: Option<&str>,
            _limit: i64,
        ) -> Result<Vec<ScoredItem>> {
            Ok(vec![])
        }

        fn index_library_alias(
            &self,
            _alias: &str,
            _library_id: &str,
            _name: &str,
            _provider: &str,
        ) -> Result<()> {
            Ok(())
        }

        fn search_library_aliases(&self, _query: &str, _limit: i64) -> Result<Vec<ScoredItem>> {
            Ok(vec![])
        }
    }

    struct NoopRanker;

    impl RankerPort for NoopRanker {
        fn rank_resolution_candidates(
            &self,
            _query: &str,
            _candidates: Vec<ScoredItem>,
            _context: RankContext,
        ) -> Result<Vec<ResolvedCandidate>> {
            Ok(vec![])
        }

        fn rank_retrieval_results(
            &self,
            _query: &str,
            _items: Vec<ScoredItem>,
            _context: RankContext,
        ) -> Result<Vec<RetrievalResult>> {
            Ok(vec![])
        }

        fn rank_recall_results(
            &self,
            _query: &str,
            _items: Vec<ScoredItem>,
            _context: RankContext,
        ) -> Result<Vec<RecalledEvent>> {
            Ok(vec![])
        }
    }

    struct TestSourceAdapter {
        state: SharedState,
    }

    impl SourceAdapter for TestSourceAdapter {
        fn kind(&self) -> &str {
            "test"
        }

        fn discover_units(&self, _source_id: &str, _config_json: &str) -> Result<Vec<SourceUnit>> {
            Ok(vec![SourceUnit {
                external_id: "unit-1".to_string(),
                uri: None,
                mime_type_hint: Some("text/plain".to_string()),
            }])
        }

        fn fetch_content(
            &self,
            _source_id: &str,
            unit: &SourceUnit,
            _config_json: &str,
        ) -> Result<RawContent> {
            if self.state.0.lock().unwrap().fetch_should_fail {
                anyhow::bail!("fetch failed");
            }
            Ok(RawContent {
                external_id: unit.external_id.clone(),
                uri: None,
                mime_type: "text/plain".to_string(),
                bytes: b"hello".to_vec(),
                metadata_json: "{}".to_string(),
            })
        }
    }

    struct TestNormalizer {
        state: SharedState,
    }

    impl Normalizer for TestNormalizer {
        fn normalize(
            &self,
            raw: &RawContent,
            source_id: &str,
            corpus_id: &str,
        ) -> Result<Vec<Document>> {
            self.state
                .0
                .lock()
                .unwrap()
                .normalized_corpora
                .push(corpus_id.to_string());
            Ok(vec![Document {
                document_id: format!("doc_{}", raw.external_id),
                corpus_id: corpus_id.to_string(),
                source_id: source_id.to_string(),
                external_id: Some(raw.external_id.clone()),
                library_id: None,
                title: "Test".to_string(),
                uri: None,
                version: None,
                mime_type: Some(raw.mime_type.clone()),
                content_text: String::from_utf8(raw.bytes.clone())?,
                metadata_json: "{}".to_string(),
                checksum: "checksum".to_string(),
                superseded_by_document_id: None,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            }])
        }
    }

    struct TestChunker;

    impl Chunker for TestChunker {
        fn chunk(
            &self,
            document_id: &str,
            corpus_id: &str,
            text: &str,
            _title: &str,
        ) -> Result<Vec<Chunk>> {
            Ok(vec![Chunk {
                chunk_id: format!("chunk_{}", document_id),
                document_id: document_id.to_string(),
                corpus_id: corpus_id.to_string(),
                ordinal: 0,
                text: text.to_string(),
                token_count_estimate: 1,
                heading: None,
                anchor: None,
                section_path_json: "[]".to_string(),
                metadata_json: "{}".to_string(),
                checksum: "checksum".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            }])
        }
    }

    fn test_app(state: SharedState) -> AppServices {
        AppServices::new(
            Box::new(TestStorage {
                state: state.clone(),
            }),
            Box::new(NoopSessionMemory),
            Box::new(NoopArtifactStore),
            Box::new(NoopSearch),
            Box::new(NoopRanker),
            Box::new(TestSourceAdapter {
                state: state.clone(),
            }),
            Box::new(TestNormalizer {
                state: state.clone(),
            }),
            Box::new(TestChunker),
        )
    }

    fn source_with_corpus(source_id: &str, corpus_id: &str) -> Source {
        Source {
            source_id: source_id.to_string(),
            kind: "test".to_string(),
            display_name: source_id.to_string(),
            enabled: true,
            adapter_config_json: format!(r#"{{"corpus_id":"{}"}}"#, corpus_id),
            auth_ref: None,
            first_indexed_at: None,
            last_refresh_attempt_at: None,
            last_refresh_success_at: None,
            last_refresh_error: None,
            freshness_ttl_secs: None,
            upstream_revision: None,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn build_excerpt_uses_character_budget_for_utf8() {
        assert_eq!(build_excerpt("ééé", 3), "ééé");
        assert_eq!(build_excerpt("éééé", 3), "ééé…");
    }

    #[test]
    fn refresh_source_reuses_persisted_corpus_id() {
        let state = SharedState::new();
        state
            .0
            .lock()
            .unwrap()
            .sources
            .push(source_with_corpus("src1", "corp_existing"));
        state
            .0
            .lock()
            .unwrap()
            .source_corpora
            .push(("src1".to_string(), "corp_existing".to_string()));
        let app = test_app(state.clone());

        app.refresh_source(RefreshSourceRequest {
            source_id: "src1".to_string(),
        })
        .unwrap();

        assert_eq!(
            state.0.lock().unwrap().normalized_corpora,
            vec!["corp_existing".to_string()]
        );
    }

    #[test]
    fn ingest_source_marks_source_degraded_on_failure() {
        let state = SharedState::new();
        state.0.lock().unwrap().fetch_should_fail = true;
        let app = test_app(state.clone());

        let err = app
            .ingest_source(IngestSourceRequest {
                source_id: "src1".to_string(),
                source_kind: "test".to_string(),
                adapter_config_json: "{}".to_string(),
                corpus_id: Some("corp1".to_string()),
            })
            .unwrap_err();

        assert!(err.to_string().contains("fetch failed"));
        assert_eq!(
            state.0.lock().unwrap().health_updates,
            vec![(
                "src1".to_string(),
                SourceHealth::Degraded,
                Some("fetch failed".to_string())
            )]
        );
    }

    #[test]
    fn retention_gc_delegates_to_storage_and_reports_counts() {
        let state = SharedState::new();
        let app = test_app(state.clone());

        let report = app
            .run_retention_gc(RetentionPolicy {
                session_event_days: 90,
                inactive_session_days: 30,
                refresh_job_days: 30,
                artifact_days: 90,
                disabled_source_days: 30,
            })
            .unwrap();

        assert_eq!(
            report,
            RetentionGcReport {
                session_events_deleted: 1,
                inactive_sessions_deleted: 2,
                refresh_jobs_deleted: 3,
                artifacts_deleted: 4,
                disabled_sources_deleted: 5,
                artifact_payload_errors: 0,
            }
        );
        assert_eq!(
            state.0.lock().unwrap().retention_calls,
            vec![
                "session_events",
                "inactive_sessions",
                "refresh_jobs",
                "artifacts",
                "disabled_sources"
            ]
        );
    }

    #[test]
    fn query_reference_rejects_private_corpus_outside_source_scope() {
        let state = SharedState::new();
        {
            let mut s = state.0.lock().unwrap();
            s.corpora.push(Corpus {
                corpus_id: "private_docs".to_string(),
                name: "Private Docs".to_string(),
                kind: "library_docs".to_string(),
                visibility: "private".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
                updated_at: "2026-01-01T00:00:00Z".to_string(),
            });
        }
        let app = test_app(state);

        let err = app
            .query_reference(QueryReferenceRequest {
                canonical_id: Some("private_docs".to_string()),
                query: "secret".to_string(),
                mode: None,
                top_k: 5,
                version: None,
                filters: serde_json::json!({"source_scope": ["public_docs"]}),
            })
            .unwrap_err();

        assert!(
            err.to_string().contains("unauthorized corpus"),
            "got: {}",
            err
        );
    }

    #[test]
    fn refresh_does_not_accumulate_duplicate_documents() {
        let state = SharedState::new();
        let app = test_app(state.clone());

        let req = IngestSourceRequest {
            source_id: "src1".to_string(),
            source_kind: "test".to_string(),
            adapter_config_json: "{}".to_string(),
            corpus_id: Some("corp1".to_string()),
        };
        app.ingest_source(req.clone()).unwrap();

        let docs_after_ingest = state
            .0
            .lock()
            .unwrap()
            .documents
            .iter()
            .filter(|d| d.source_id == "src1")
            .count();
        assert_eq!(docs_after_ingest, 1);

        app.refresh_source(RefreshSourceRequest {
            source_id: "src1".to_string(),
        })
        .unwrap();

        let docs_after_refresh = state
            .0
            .lock()
            .unwrap()
            .documents
            .iter()
            .filter(|d| d.source_id == "src1")
            .count();
        assert_eq!(
            docs_after_refresh, 1,
            "refresh should replace, not accumulate, documents"
        );
    }

    #[test]
    fn compute_freshness_reflects_age_and_ttl() {
        let recent = Utc::now().to_rfc3339();

        // ~now with ttl 3600 → near 1.0
        assert!(
            compute_freshness(Some(&recent), Some(3600)) > 0.99,
            "recent timestamp should be near-fresh"
        );

        // Far in the past with ttl 3600 → 0.0
        assert_eq!(
            compute_freshness(Some("2020-01-01T00:00:00Z"), Some(3600)),
            0.0
        );

        // Absent → 0.0
        assert_eq!(compute_freshness(None, Some(3600)), 0.0);

        // Malformed → 0.5 (uncertain)
        assert_eq!(compute_freshness(Some("not-a-date"), Some(3600)), 0.5);

        // No ttl configured, recent timestamp → uses 30-day default, near 1.0
        assert!(
            compute_freshness(Some(&recent), None) > 0.99,
            "recent timestamp with default ttl should be near-fresh"
        );
    }

    #[test]
    fn compute_freshness_table() {
        let now = Utc::now();
        let rfc = |secs_ago: i64| -> String {
            now.checked_sub_signed(chrono::Duration::seconds(secs_ago))
                .unwrap()
                .to_rfc3339()
        };
        let ttl = 3600i64;

        // Half TTL: age = ttl/2 → freshness 0.5.
        let half = compute_freshness(Some(&rfc(ttl / 2)), Some(ttl));
        assert!(
            (half - 0.5).abs() < 0.02,
            "half-TTL age should be ~0.5, got {half}"
        );

        // Exact TTL: age = ttl → freshness 0.0 (boundary).
        let exact = compute_freshness(Some(&rfc(ttl)), Some(ttl));
        assert!(
            (exact - 0.0).abs() < 0.02,
            "exact-TTL age should be ~0.0, got {exact}"
        );

        // Past TTL: age > ttl → clamped to 0.0.
        let past = compute_freshness(Some(&rfc(ttl * 2)), Some(ttl));
        assert_eq!(past, 0.0, "past-TTL age must clamp to 0.0, got {past}");

        // Future timestamp: age < 0 → clamp(0.0, 1.0) gives 1.0.
        let future_rfc = (now + chrono::Duration::seconds(60)).to_rfc3339();
        let future = compute_freshness(Some(&future_rfc), Some(ttl));
        assert_eq!(
            future, 1.0,
            "future timestamp must clamp to 1.0, got {future}"
        );

        // Zero TTL configured: filtered to default (30 days), so a recent
        // timestamp is near-fresh, not 1.0-from-zero (which would divide by
        // zero). Confirms the `*t > 0` guard.
        let zero_ttl = compute_freshness(Some(&rfc(60)), Some(0));
        assert!(
            zero_ttl > 0.99,
            "zero TTL must fall back to default TTL, got {zero_ttl}"
        );

        // Negative TTL: also filtered to default.
        let neg_ttl = compute_freshness(Some(&rfc(60)), Some(-1));
        assert!(
            neg_ttl > 0.99,
            "negative TTL must fall back to default TTL, got {neg_ttl}"
        );
    }

    struct WorkingSessionMemory;

    impl SessionMemoryPort for WorkingSessionMemory {
        fn create_session(
            &self,
            session_id: &str,
            transport: SessionTransport,
            continuation_mode: ContinuationMode,
        ) -> Result<Session> {
            let now = chrono::Utc::now().to_rfc3339();
            Ok(Session {
                session_id: session_id.to_string(),
                client_id: None,
                transport,
                continuation_mode,
                status: SessionStatus::Active,
                created_at: now.clone(),
                updated_at: now,
            })
        }

        fn get_session(&self, _session_id: &str) -> Result<Option<Session>> {
            Ok(None)
        }

        fn add_event(
            &self,
            _session_id: &str,
            _kind: &str,
            _text: &str,
            _artifact_id: Option<&str>,
            _metadata_json: &str,
            _pinned: bool,
        ) -> Result<StoredContext> {
            unimplemented!()
        }

        fn get_event(&self, _event_id: &str) -> Result<Option<SessionEvent>> {
            Ok(None)
        }

        fn list_events_by_session(&self, _session_id: &str) -> Result<Vec<SessionEvent>> {
            Ok(vec![])
        }

        fn count_active_sessions(&self) -> Result<u64> {
            Ok(0)
        }
    }

    fn test_app_with_artifact_store(
        state: SharedState,
        artifact_dir: &std::path::Path,
    ) -> AppServices {
        AppServices::new(
            Box::new(TestStorage {
                state: state.clone(),
            }),
            Box::new(WorkingSessionMemory),
            Box::new(ck_artifacts::FileArtifactStore::new(
                artifact_dir.to_path_buf(),
            )),
            Box::new(NoopSearch),
            Box::new(NoopRanker),
            Box::new(TestSourceAdapter {
                state: state.clone(),
            }),
            Box::new(TestNormalizer {
                state: state.clone(),
            }),
            Box::new(TestChunker),
        )
    }

    #[test]
    fn artifact_payload_is_compensated_on_metadata_failure() {
        let state = SharedState::new();
        {
            let mut s = state.0.lock().unwrap();
            s.fail_upsert_artifact = true;
        }
        let tmp = tempfile::tempdir().unwrap();
        let app = test_app_with_artifact_store(state.clone(), tmp.path());

        let req = StoreContextRequest {
            session_id: "sess-comp".to_string(),
            kind: "decision".to_string(),
            text: None,
            artifact_content: Some(vec![1u8, 2, 3, 4, 5]),
            artifact_content_type: Some("application/octet-stream".to_string()),
            metadata: serde_json::json!({}),
        };
        let err = app.store_context(req).unwrap_err();
        assert!(
            err.to_string().contains("injected upsert_artifact failure"),
            "store_context must surface the metadata failure; got: {err}"
        );

        let calls = state.0.lock().unwrap().artifact_upsert_calls.clone();
        assert_eq!(
            calls.len(),
            1,
            "upsert_artifact must be called exactly once on the failure path"
        );
        let artifact_id = calls[0].clone();

        let artifacts_subdir = tmp.path().join("artifacts");
        let dir_listing = std::fs::read_dir(&artifacts_subdir).unwrap().count();
        assert_eq!(
            dir_listing, 0,
            "artifact payload + sidecar must be deleted on metadata failure"
        );
        let _ = artifact_id;
    }

    #[test]
    fn stable_document_id_is_deterministic() {
        let id1 = stable_document_id("src-a", "ext-1", "/path/doc.md");
        let id2 = stable_document_id("src-a", "ext-1", "/path/doc.md");
        assert_eq!(id1, id2, "same inputs must produce same ID");

        let id3 = stable_document_id("src-b", "ext-1", "/path/doc.md");
        assert_ne!(id1, id3, "different source must produce different ID");

        let id4 = stable_document_id("src-a", "ext-2", "/path/doc.md");
        assert_ne!(id1, id4, "different external_id must produce different ID");

        assert!(id1.starts_with("doc_"));
    }

    #[test]
    fn document_identity_survives_discovery_reorder_and_insertion() {
        let src = "src-reorder";
        let doc_a = stable_document_id(src, "file-a.md", "/path/file-a.md");
        let doc_b = stable_document_id(src, "file-b.md", "/path/file-b.md");
        let doc_c = stable_document_id(src, "file-c.md", "/path/file-c.md");

        // Reorder: B, C, A — IDs must be the same as original order A, B, C.
        let reordered_a = stable_document_id(src, "file-a.md", "/path/file-a.md");
        let reordered_b = stable_document_id(src, "file-b.md", "/path/file-b.md");
        let reordered_c = stable_document_id(src, "file-c.md", "/path/file-c.md");
        assert_eq!(doc_a, reordered_a);
        assert_eq!(doc_b, reordered_b);
        assert_eq!(doc_c, reordered_c);

        // Insertion: a new file D doesn't change existing IDs.
        let _doc_d = stable_document_id(src, "file-d.md", "/path/file-d.md");
        assert_eq!(
            stable_document_id(src, "file-a.md", "/path/file-a.md"),
            doc_a,
            "new file must not shift existing IDs"
        );
    }
}

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

        // Enrich scored items with trust metadata before ranking
        let mut enriched = Vec::new();
        for item in scored {
            let mut breakdown = item.score_breakdown;
            if let Some(lib) = self.storage.get_library_identity(&item.id)? {
                if let Some(ts) = lib.trust_score {
                    if let Some(m) = breakdown.as_object_mut() {
                        m.insert("trust_score".to_string(), serde_json::json!(ts));
                    }
                }
            }
            enriched.push(ScoredItem {
                id: item.id,
                score: item.score,
                score_breakdown: breakdown,
            });
        }

        let context = RankContext {
            intent: QueryIntent::ResolveLibrary,
            session_id: None,
            debug: false,
        };

        let ranked = self
            .ranker
            .rank_resolution_candidates(&req.query, enriched, context)?;

        // Enrich ranked candidates with library metadata
        let mut candidates_with_metadata = Vec::new();
        for candidate in ranked {
            if let Some(lib) = self.storage.get_library_identity(&candidate.canonical_id)? {
                let aliases = self.storage.list_library_aliases(&candidate.canonical_id)?;
                let alias_strs: Vec<String> = aliases.iter().map(|a| a.alias.clone()).collect();

                candidates_with_metadata.push(ResolvedCandidate {
                    canonical_id: lib.library_id,
                    provider: lib.provider,
                    name: lib.name,
                    version: lib.version,
                    aliases: alias_strs,
                    summary: lib.summary,
                    trust_score: lib.trust_score,
                    score: candidate.score,
                    score_breakdown: candidate.score_breakdown,
                });
            }
        }

        let candidates = if let Some(scope) = &req.source_scope {
            candidates_with_metadata
                .into_iter()
                .filter(|c| scope.contains(&c.canonical_id))
                .take(req.limit as usize)
                .collect()
        } else {
            candidates_with_metadata
                .into_iter()
                .take(req.limit as usize)
                .collect()
        };

        Ok(ResolveReferenceResponse {
            candidates,
            query_id,
            mode: "resolve".to_string(),
        })
    }
}

// --- refresh_source ---

#[derive(Debug, Clone)]
pub struct RefreshSourceRequest {
    pub source_id: String,
}

#[derive(Debug, Clone)]
pub struct RefreshSourceResponse {
    pub job: JobSummary,
}

impl AppServices {
    pub fn refresh_source(&self, req: RefreshSourceRequest) -> Result<RefreshSourceResponse> {
        let source = self
            .storage
            .get_source(&req.source_id)?
            .ok_or_else(|| anyhow::anyhow!("source not found: {}", req.source_id))?;

        // Re-ingest using the source's existing config
        let ingest_req = IngestSourceRequest {
            source_id: source.source_id.clone(),
            source_kind: source.kind.clone(),
            adapter_config_json: source.adapter_config_json.clone(),
            corpus_id: self.storage.find_corpus_id_for_source(&source.source_id)?,
        };

        let resp = self.ingest_source(ingest_req)?;

        Ok(RefreshSourceResponse {
            job: JobSummary {
                job_id: resp.job.job_id,
                source_id: req.source_id,
                status: "completed".to_string(),
                trigger: "manual".to_string(),
            },
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
        let degraded_sources = sources
            .iter()
            .filter(|s| s.last_refresh_error.is_some())
            .count() as u64;

        let active_sessions = self.session_memory.count_active_sessions()?;

        let queued_jobs = self.storage.count_jobs_by_status("queued")?;
        let running_jobs = self.storage.count_jobs_by_status("running")?;

        let mut status = KernelStatus::new(
            started_at.to_string(),
            storage_status,
            default_mode,
            semantic_enabled,
        );
        status.sources.total = total_sources;
        status.sources.enabled = enabled_sources;
        status.sources.degraded = degraded_sources;
        status.sessions.active = active_sessions;
        status.jobs.queued = queued_jobs;
        status.jobs.running = running_jobs;

        Ok(status)
    }
}
