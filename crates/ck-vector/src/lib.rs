use std::path::{Path, PathBuf};

use anyhow::Result;
use ck_core::entities::ScoredItem;
use ck_core::ports::VectorSearchPort;
use rusqlite::Connection;

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS chunk_embeddings (
  chunk_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  vector_blob BLOB NOT NULL,
  corpus_id TEXT,
  library_id TEXT,
  document_id TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (chunk_id, model_id)
);

CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_corpus_id ON chunk_embeddings(corpus_id);
CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_library_id ON chunk_embeddings(library_id);
CREATE INDEX IF NOT EXISTS idx_chunk_embeddings_document_id ON chunk_embeddings(document_id);

CREATE TABLE IF NOT EXISTS session_event_embeddings (
  event_id TEXT NOT NULL,
  model_id TEXT NOT NULL,
  vector_blob BLOB NOT NULL,
  session_id TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  PRIMARY KEY (event_id, model_id)
);

CREATE INDEX IF NOT EXISTS idx_session_event_embeddings_session_id ON session_event_embeddings(session_id);
"#;

#[derive(Clone)]
pub struct SqliteVectorStore {
    path: PathBuf,
}

impl SqliteVectorStore {
    pub fn open(db_path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = db_path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let store = Self {
            path: db_path.as_ref().to_path_buf(),
        };
        let conn = store.conn()?;
        conn.execute_batch(SCHEMA)?;
        Ok(store)
    }

    fn conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
        Ok(conn)
    }
}

fn encode_vector(vector: &[f64]) -> Vec<u8> {
    let dim = vector.len() as u32;
    let mut buf = Vec::with_capacity(4 + vector.len() * 8);
    buf.extend_from_slice(&dim.to_le_bytes());
    for v in vector {
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

fn decode_vector(blob: &[u8]) -> Result<Vec<f64>> {
    if blob.len() < 4 {
        anyhow::bail!("vector blob too short: {} bytes", blob.len());
    }
    let dim = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]) as usize;
    let expected = 4 + dim * 8;
    if blob.len() != expected {
        anyhow::bail!(
            "vector blob size mismatch: expected {} bytes, got {}",
            expected,
            blob.len()
        );
    }
    let mut out = Vec::with_capacity(dim);
    for i in 0..dim {
        let offset = 4 + i * 8;
        let bytes: [u8; 8] = blob[offset..offset + 8].try_into()?;
        out.push(f64::from_le_bytes(bytes));
    }
    Ok(out)
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

impl VectorSearchPort for SqliteVectorStore {
    fn upsert_chunk_embedding(&self, chunk_id: &str, model_id: &str, vector: &[f64]) -> Result<()> {
        let conn = self.conn()?;
        let blob = encode_vector(vector);
        conn.execute(
            "INSERT OR REPLACE INTO chunk_embeddings (chunk_id, model_id, vector_blob, corpus_id, library_id, document_id) \
             VALUES (?1, ?2, ?3, NULL, NULL, NULL)",
            rusqlite::params![chunk_id, model_id, blob],
        )?;
        Ok(())
    }

    fn upsert_chunk_embedding_with_metadata(
        &self,
        chunk_id: &str,
        model_id: &str,
        vector: &[f64],
        corpus_id: Option<&str>,
        library_id: Option<&str>,
        document_id: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn()?;
        let blob = encode_vector(vector);
        conn.execute(
            "INSERT OR REPLACE INTO chunk_embeddings (chunk_id, model_id, vector_blob, corpus_id, library_id, document_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![chunk_id, model_id, blob, corpus_id, library_id, document_id],
        )?;
        Ok(())
    }

    fn upsert_session_event_embedding(
        &self,
        event_id: &str,
        model_id: &str,
        vector: &[f64],
    ) -> Result<()> {
        let conn = self.conn()?;
        let blob = encode_vector(vector);
        conn.execute(
            "INSERT OR REPLACE INTO session_event_embeddings (event_id, model_id, vector_blob) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params![event_id, model_id, blob],
        )?;
        Ok(())
    }

    fn search_chunks(
        &self,
        query_vector: &[f64],
        model_id: &str,
        corpus_id: Option<&str>,
        library_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>> {
        let conn = self.conn()?;

        let mut sql =
            "SELECT chunk_id, vector_blob FROM chunk_embeddings WHERE model_id = ?".to_string();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(model_id.to_string())];

        if let Some(c) = corpus_id {
            sql.push_str(" AND corpus_id = ?");
            params.push(Box::new(c.to_string()));
        }
        if let Some(l) = library_id {
            sql.push_str(" AND library_id = ?");
            params.push(Box::new(l.to_string()));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;

        let mut scored: Vec<ScoredItem> = Vec::new();
        for row_result in rows {
            let (id, blob) = row_result?;
            let vec = decode_vector(&blob)?;
            let sim = cosine_similarity(query_vector, &vec);
            scored.push(ScoredItem {
                id,
                score: sim,
                score_breakdown: serde_json::json!({ "semantic": sim }),
            });
        }

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit.max(0) as usize);
        Ok(scored)
    }

    fn search_session_events(
        &self,
        query_vector: &[f64],
        model_id: &str,
        session_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>> {
        let conn = self.conn()?;

        let mut sql =
            "SELECT event_id, vector_blob FROM session_event_embeddings WHERE model_id = ?"
                .to_string();
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![Box::new(model_id.to_string())];

        if let Some(s) = session_id {
            sql.push_str(" AND session_id = ?");
            params.push(Box::new(s.to_string()));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?;

        let mut scored: Vec<ScoredItem> = Vec::new();
        for row_result in rows {
            let (id, blob) = row_result?;
            let vec = decode_vector(&blob)?;
            let sim = cosine_similarity(query_vector, &vec);
            scored.push(ScoredItem {
                id,
                score: sim,
                score_breakdown: serde_json::json!({ "semantic": sim }),
            });
        }

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit.max(0) as usize);
        Ok(scored)
    }

    fn remove_chunk_embeddings_by_document(&self, document_id: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM chunk_embeddings WHERE document_id = ?1",
            rusqlite::params![document_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> (SqliteVectorStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test_vectors.db");
        let store = SqliteVectorStore::open(&db_path).unwrap();
        (store, dir)
    }

    #[test]
    fn open_creates_tables() {
        let (store, _dir) = test_store();
        let conn = store.conn().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('chunk_embeddings', 'session_event_embeddings')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn encode_decode_roundtrip() {
        let original = vec![1.0, 2.5, -3.7, 0.0, 42.0];
        let blob = encode_vector(&original);
        let decoded = decode_vector(&blob).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn cosine_similarity_same_vector() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_known_value() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        let dot = 1.0 * 4.0 + 2.0 * 5.0 + 3.0 * 6.0;
        let mag_a = (1.0 + 4.0 + 9.0_f64).sqrt();
        let mag_b = (16.0 + 25.0 + 36.0_f64).sqrt();
        let expected = dot / (mag_a * mag_b);
        let sim = cosine_similarity(&a, &b);
        assert!((sim - expected).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_zero_vector() {
        let a = vec![0.0, 0.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim - 0.0).abs() < 1e-10);
    }

    #[test]
    fn upsert_and_search_chunk_embeddings() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let query = vec![0.9, 0.1, 0.0];

        store
            .upsert_chunk_embedding("chunk-a", "model-1", &vec_a)
            .unwrap();
        store
            .upsert_chunk_embedding("chunk-b", "model-1", &vec_b)
            .unwrap();

        let results = store
            .search_chunks(&query, "model-1", None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "chunk-a");
        assert!(results[0].score > results[1].score);
        assert!(results[0].score_breakdown["semantic"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn search_chunks_filters_by_model() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0];
        let query = vec![1.0, 0.0];

        store
            .upsert_chunk_embedding("chunk-a", "model-1", &vec_a)
            .unwrap();
        store
            .upsert_chunk_embedding("chunk-b", "model-2", &vec_a)
            .unwrap();

        let results = store
            .search_chunks(&query, "model-1", None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "chunk-a");
    }

    #[test]
    fn search_chunks_respects_limit() {
        let (store, _dir) = test_store();
        let v = vec![1.0, 0.0];

        for i in 0..5 {
            store
                .upsert_chunk_embedding(&format!("chunk-{}", i), "model-1", &v)
                .unwrap();
        }

        let results = store.search_chunks(&v, "model-1", None, None, 3).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn upsert_chunk_replaces_existing() {
        let (store, _dir) = test_store();
        let vec_v1 = vec![1.0, 0.0];
        let vec_v2 = vec![0.0, 1.0];
        let query = vec![0.0, 1.0];

        store
            .upsert_chunk_embedding("chunk-x", "model-1", &vec_v1)
            .unwrap();
        store
            .upsert_chunk_embedding("chunk-x", "model-1", &vec_v2)
            .unwrap();

        let results = store
            .search_chunks(&query, "model-1", None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn upsert_and_search_session_event_embeddings() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let query = vec![0.1, 0.9, 0.0];

        store
            .upsert_session_event_embedding("evt-a", "model-1", &vec_a)
            .unwrap();
        store
            .upsert_session_event_embedding("evt-b", "model-1", &vec_b)
            .unwrap();

        let results = store
            .search_session_events(&query, "model-1", None, 10)
            .unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "evt-b");
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn search_session_events_filters_by_model() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0];

        store
            .upsert_session_event_embedding("evt-a", "model-1", &vec_a)
            .unwrap();
        store
            .upsert_session_event_embedding("evt-b", "model-2", &vec_a)
            .unwrap();

        let results = store
            .search_session_events(&vec_a, "model-2", None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "evt-b");
    }

    #[test]
    fn clone_reopens_database() {
        let (store, _dir) = test_store();
        let v = vec![1.0, 0.0];
        store.upsert_chunk_embedding("chunk-1", "m1", &v).unwrap();

        let cloned = store.clone();
        let results = cloned.search_chunks(&v, "m1", None, None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "chunk-1");
    }

    #[test]
    fn search_chunks_filters_by_corpus_id() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0, 0.0];
        let vec_b = vec![0.0, 1.0, 0.0];
        let query = vec![1.0, 0.0, 0.0];

        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-a",
                "model-1",
                &vec_a,
                Some("corpus-1"),
                None,
                None,
            )
            .unwrap();
        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-b",
                "model-1",
                &vec_b,
                Some("corpus-2"),
                None,
                None,
            )
            .unwrap();

        let results = store
            .search_chunks(&query, "model-1", Some("corpus-1"), None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "chunk-a");
    }

    #[test]
    fn search_chunks_filters_by_library_id() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0];
        let query = vec![1.0, 0.0];

        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-a",
                "model-1",
                &vec_a,
                None,
                Some("lib-1"),
                None,
            )
            .unwrap();
        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-b",
                "model-1",
                &vec_a,
                None,
                Some("lib-2"),
                None,
            )
            .unwrap();

        let results = store
            .search_chunks(&query, "model-1", None, Some("lib-1"), 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "chunk-a");
    }

    #[test]
    fn remove_chunk_embeddings_by_document_deletes_rows() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0];
        let query = vec![1.0, 0.0];

        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-a",
                "model-1",
                &vec_a,
                None,
                None,
                Some("doc-1"),
            )
            .unwrap();
        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-b",
                "model-1",
                &vec_a,
                None,
                None,
                Some("doc-1"),
            )
            .unwrap();
        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-c",
                "model-1",
                &vec_a,
                None,
                None,
                Some("doc-2"),
            )
            .unwrap();

        store.remove_chunk_embeddings_by_document("doc-1").unwrap();

        let results = store
            .search_chunks(&query, "model-1", None, None, 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "chunk-c");
    }

    #[test]
    fn search_chunks_filters_by_both_corpus_and_library() {
        let (store, _dir) = test_store();
        let vec_a = vec![1.0, 0.0];
        let query = vec![1.0, 0.0];

        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-a",
                "model-1",
                &vec_a,
                Some("corpus-1"),
                Some("lib-1"),
                None,
            )
            .unwrap();
        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-b",
                "model-1",
                &vec_a,
                Some("corpus-1"),
                Some("lib-2"),
                None,
            )
            .unwrap();
        store
            .upsert_chunk_embedding_with_metadata(
                "chunk-c",
                "model-1",
                &vec_a,
                Some("corpus-2"),
                Some("lib-1"),
                None,
            )
            .unwrap();

        let results = store
            .search_chunks(&query, "model-1", Some("corpus-1"), Some("lib-1"), 10)
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "chunk-a");
    }
}
