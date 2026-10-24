//! Shared test wiring: the REAL retrieval stack (SQLite storage + FTS5 +
//! LexicalRanker + filesystem adapter + normalizer + chunker) behind
//! `AppServices` — the same composition `ck-bin` serves. Tests must exercise
//! the production retrieval pipeline, not a degraded stub.
#![allow(dead_code)]

use std::path::Path;

use ck_adapter_filesystem::{
    FilesystemSourceAdapter, HeadingChunker, LearningProposalsAdapter, MarkdownNormalizer,
};
use ck_app::AppServices;
use ck_artifacts::FileArtifactStore;
use ck_ranker::LexicalRanker;
use ck_search_fts::FtsSearch;
use ck_storage_sqlite::SqliteStore;

/// Real CK application stack over a fresh temp database. Returns the
/// application services plus the backing store handle for fixture surgery
/// (e.g. aging a source for freshness tests).
pub fn test_kernel_with_store(dir: &Path) -> (AppServices, SqliteStore) {
    let db = dir.join("ck.db");
    let data = dir.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let store = SqliteStore::open(&db).unwrap();
    store.migrate().unwrap();
    let app = AppServices::new(
        Box::new(store.clone()),
        Box::new(store.clone()),
        Box::new(FileArtifactStore::new(data)),
        Box::new(FtsSearch::open(db).unwrap()),
        Box::new(LexicalRanker),
        Box::new(FilesystemSourceAdapter),
        Box::new(MarkdownNormalizer),
        Box::new(HeadingChunker),
    )
    .with_source_adapter_kind(
        "learning_proposals_jsonl",
        Box::new(LearningProposalsAdapter),
    );
    (app, store)
}

/// Real CK application stack over a fresh temp database.
pub fn test_kernel(dir: &Path) -> AppServices {
    test_kernel_with_store(dir).0
}

/// Deterministic embedding provider for semantic-path tests.
pub struct StubEmbedding;

impl ck_core::ports::EmbeddingPort for StubEmbedding {
    fn embed(&self, text: &str) -> anyhow::Result<Option<Vec<f64>>> {
        // Deterministic bag-of-words vector: identical texts -> identical vectors.
        let mut v = vec![0.0; 8];
        for (i, word) in text.split_whitespace().enumerate() {
            v[i % 8] += word.len() as f64;
        }
        Ok(Some(v))
    }
    fn embed_batch(&self, texts: &[&str]) -> anyhow::Result<Vec<Option<Vec<f64>>>> {
        texts.iter().map(|t| self.embed(t)).collect()
    }
    fn model_id(&self) -> &str {
        "stub-embedding"
    }
    fn dimensions(&self) -> usize {
        8
    }
    fn is_available(&self) -> bool {
        true
    }
}

/// Vector search stub returning pre-seeded scored items (cosine-simulated).
pub struct StubVectorSearch {
    pub items: Vec<ck_core::entities::ScoredItem>,
}

impl ck_core::ports::VectorSearchPort for StubVectorSearch {
    fn upsert_chunk_embedding(
        &self,
        _chunk_id: &str,
        _model_id: &str,
        _vector: &[f64],
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn upsert_session_event_embedding(
        &self,
        _event_id: &str,
        _model_id: &str,
        _vector: &[f64],
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn search_chunks(
        &self,
        _query_vector: &[f64],
        _model_id: &str,
        _corpus_id: Option<&str>,
        _library_id: Option<&str>,
        _limit: i64,
    ) -> anyhow::Result<Vec<ck_core::entities::ScoredItem>> {
        Ok(self.items.clone())
    }
    fn search_session_events(
        &self,
        _query_vector: &[f64],
        _model_id: &str,
        _session_id: Option<&str>,
        _limit: i64,
    ) -> anyhow::Result<Vec<ck_core::entities::ScoredItem>> {
        Ok(Vec::new())
    }
    fn remove_chunk_embeddings_by_document(&self, _document_id: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

/// A standard E2 `ContextRequired` payload for tool-level tests.
pub fn required_payload(corpus_id: &str, query: Option<&str>) -> serde_json::Value {
    serde_json::json!({
        "domain_model_hash": "a".repeat(64),
        "world_ref": "world:test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "work_request_id": "wr_test",
        "context_requirement_id": "cr_test",
        "corpus_id": corpus_id,
        "query": query,
        "max_results": 10,
    })
}
