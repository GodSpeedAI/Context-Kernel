use std::path::PathBuf;

use anyhow::Result;
use ck_core::entities::ScoredItem;
use ck_core::ports::LexicalSearchPort;
use rusqlite::Connection;
use tracing::{debug, instrument};

pub struct FtsSearch {
    path: PathBuf,
}

impl FtsSearch {
    pub fn open(path: PathBuf) -> Result<Self> {
        Ok(Self { path })
    }

    fn conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        Ok(conn)
    }

    fn fts_query(raw: &str) -> Option<String> {
        let result = raw
            .split_whitespace()
            .filter(|t| !t.is_empty())
            .map(|t| format!("\"{}\"", t.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(" OR ");
        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }

    fn collect_scored(
        conn: &Connection,
        sql: &str,
        params: Vec<Box<dyn rusqlite::types::ToSql>>,
    ) -> Result<Vec<ScoredItem>> {
        let mut stmt = conn.prepare(sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt.query_map(param_refs.as_slice(), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;
        let mut items = Vec::new();
        for row in rows {
            let (id, raw_score): (String, f64) = row?;
            let score = -raw_score;
            items.push(ScoredItem {
                id,
                score,
                score_breakdown: serde_json::json!({ "fts": score }),
            });
        }
        Ok(items)
    }
}

impl LexicalSearchPort for FtsSearch {
    #[instrument(skip(self, text), fields(chunk_id = %chunk_id))]
    fn index_chunk(
        &self,
        chunk_id: &str,
        document_id: &str,
        corpus_id: &str,
        library_id: Option<&str>,
        title: &str,
        heading: &str,
        text: &str,
    ) -> Result<()> {
        let conn = self.conn()?;
        let lib = library_id.unwrap_or("");
        conn.execute(
            "INSERT INTO chunks_fts(chunk_id, document_id, corpus_id, library_id, title, heading, text) VALUES(?, ?, ?, ?, ?, ?, ?)",
            rusqlite::params![chunk_id, document_id, corpus_id, lib, title, heading, text],
        )?;
        debug!(chunk_id, "indexed chunk in FTS");
        Ok(())
    }

    #[instrument(skip(self), fields(document_id = %document_id))]
    fn remove_chunks_by_document(&self, document_id: &str) -> Result<()> {
        let conn = self.conn()?;
        let chunk_ids: Vec<String> = {
            let mut stmt = conn.prepare("SELECT chunk_id FROM chunks WHERE document_id = ?")?;
            let rows = stmt.query_map(rusqlite::params![document_id], |row| row.get(0))?;
            rows.collect::<Result<Vec<_>, _>>()?
        };
        for cid in &chunk_ids {
            conn.execute(
                "DELETE FROM chunks_fts WHERE chunk_id = ?",
                rusqlite::params![cid],
            )?;
        }
        debug!(
            document_id,
            count = chunk_ids.len(),
            "removed chunks from FTS"
        );
        Ok(())
    }

    #[instrument(skip(self), fields(query = %query))]
    fn search_chunks(
        &self,
        query: &str,
        corpus_id: Option<&str>,
        library_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>> {
        let conn = self.conn()?;
        let fts_expr = match Self::fts_query(query) {
            Some(e) => e,
            None => return Ok(Vec::new()),
        };

        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) =
            match (corpus_id, library_id) {
                (Some(cid), Some(lid)) => (
                    "SELECT f.chunk_id, bm25(chunks_fts) \
                 FROM chunks_fts f \
                 JOIN chunks c ON f.chunk_id = c.chunk_id \
                 JOIN documents d ON c.document_id = d.document_id \
                 WHERE f.chunks_fts MATCH ? AND c.corpus_id = ? AND d.library_id = ? \
                 ORDER BY bm25(chunks_fts) LIMIT ?",
                    vec![
                        Box::new(fts_expr),
                        Box::new(cid.to_string()),
                        Box::new(lid.to_string()),
                        Box::new(limit),
                    ],
                ),
                (Some(cid), None) => (
                    "SELECT f.chunk_id, bm25(chunks_fts) \
                 FROM chunks_fts f \
                 JOIN chunks c ON f.chunk_id = c.chunk_id \
                 WHERE f.chunks_fts MATCH ? AND c.corpus_id = ? \
                 ORDER BY bm25(chunks_fts) LIMIT ?",
                    vec![
                        Box::new(fts_expr),
                        Box::new(cid.to_string()),
                        Box::new(limit),
                    ],
                ),
                (None, Some(lid)) => (
                    "SELECT f.chunk_id, bm25(chunks_fts) \
                 FROM chunks_fts f \
                 JOIN chunks c ON f.chunk_id = c.chunk_id \
                 JOIN documents d ON c.document_id = d.document_id \
                 WHERE f.chunks_fts MATCH ? AND d.library_id = ? \
                 ORDER BY bm25(chunks_fts) LIMIT ?",
                    vec![
                        Box::new(fts_expr),
                        Box::new(lid.to_string()),
                        Box::new(limit),
                    ],
                ),
                (None, None) => (
                    "SELECT chunk_id, bm25(chunks_fts) \
                 FROM chunks_fts \
                 WHERE chunks_fts MATCH ? \
                 ORDER BY bm25(chunks_fts) LIMIT ?",
                    vec![Box::new(fts_expr), Box::new(limit)],
                ),
            };

        let items = Self::collect_scored(&conn, sql, params)?;
        debug!(query, count = items.len(), "FTS chunk search complete");
        Ok(items)
    }

    #[instrument(skip(self, text), fields(event_id = %event_id))]
    fn index_session_event(
        &self,
        event_id: &str,
        session_id: &str,
        kind: &str,
        text: &str,
    ) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO session_events_fts(event_id, session_id, kind, text) VALUES(?, ?, ?, ?)",
            rusqlite::params![event_id, session_id, kind, text],
        )?;
        debug!(event_id, "indexed session event in FTS");
        Ok(())
    }

    #[instrument(skip(self), fields(query = %query))]
    fn search_session_events(
        &self,
        query: &str,
        session_id: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ScoredItem>> {
        let conn = self.conn()?;
        let fts_expr = match Self::fts_query(query) {
            Some(e) => e,
            None => return Ok(Vec::new()),
        };

        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match session_id {
            Some(sid) => (
                "SELECT f.event_id, bm25(session_events_fts) \
                 FROM session_events_fts f \
                 JOIN session_events e ON f.event_id = e.event_id \
                 WHERE f.session_events_fts MATCH ? AND e.session_id = ? \
                 ORDER BY bm25(session_events_fts) LIMIT ?",
                vec![
                    Box::new(fts_expr),
                    Box::new(sid.to_string()),
                    Box::new(limit),
                ],
            ),
            None => (
                "SELECT event_id, bm25(session_events_fts) \
                 FROM session_events_fts \
                 WHERE session_events_fts MATCH ? \
                 ORDER BY bm25(session_events_fts) LIMIT ?",
                vec![Box::new(fts_expr), Box::new(limit)],
            ),
        };

        let items = Self::collect_scored(&conn, sql, params)?;
        debug!(
            query,
            count = items.len(),
            "FTS session event search complete"
        );
        Ok(items)
    }

    #[instrument(skip(self), fields(alias = %alias))]
    fn index_library_alias(
        &self,
        alias: &str,
        library_id: &str,
        name: &str,
        provider: &str,
    ) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO library_aliases_fts(alias, library_id, name, provider) VALUES(?, ?, ?, ?)",
            rusqlite::params![alias, library_id, name, provider],
        )?;
        debug!(alias, library_id, "indexed library alias in FTS");
        Ok(())
    }

    #[instrument(skip(self), fields(query = %query))]
    fn search_library_aliases(&self, query: &str, limit: i64) -> Result<Vec<ScoredItem>> {
        let conn = self.conn()?;
        let fts_expr = match Self::fts_query(query) {
            Some(e) => e,
            None => return Ok(Vec::new()),
        };

        let sql = "SELECT library_id, bm25(library_aliases_fts) \
                   FROM library_aliases_fts \
                   WHERE library_aliases_fts MATCH ? \
                   ORDER BY bm25(library_aliases_fts) LIMIT ?";

        let items = Self::collect_scored(&conn, sql, vec![Box::new(fts_expr), Box::new(limit)])?;
        debug!(
            query,
            count = items.len(),
            "FTS library alias search complete"
        );
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_db() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
               chunk_id UNINDEXED, document_id UNINDEXED, corpus_id UNINDEXED,
               library_id UNINDEXED, title, heading, text, tokenize = 'unicode61'
             );
             CREATE VIRTUAL TABLE IF NOT EXISTS session_events_fts USING fts5(
               event_id UNINDEXED, session_id UNINDEXED, kind, text, tokenize = 'unicode61'
             );
             CREATE VIRTUAL TABLE IF NOT EXISTS library_aliases_fts USING fts5(
               alias, library_id UNINDEXED, name, provider, tokenize = 'unicode61'
             );
              CREATE TABLE IF NOT EXISTS chunks (
                chunk_id TEXT PRIMARY KEY, document_id TEXT NOT NULL,
                corpus_id TEXT NOT NULL
              );
              CREATE TABLE IF NOT EXISTS documents (
                document_id TEXT PRIMARY KEY, library_id TEXT
              );
              CREATE TABLE IF NOT EXISTS session_events (
               event_id TEXT PRIMARY KEY, session_id TEXT NOT NULL,
               kind TEXT NOT NULL, text TEXT NOT NULL
             );",
        )
        .unwrap();
        drop(conn);
        (dir, db_path)
    }

    #[test]
    fn index_and_search_chunks() {
        let (_dir, db_path) = setup_db();
        let fts = FtsSearch::open(db_path).unwrap();

        fts.index_chunk(
            "c1",
            "d1",
            "corp1",
            Some("lib1"),
            "Rust Guide",
            "Intro",
            "Rust is a systems programming language",
        )
        .unwrap();
        fts.index_chunk(
            "c2",
            "d1",
            "corp1",
            None,
            "Python Guide",
            "Intro",
            "Python is a scripting language",
        )
        .unwrap();
        fts.index_chunk(
            "c3",
            "d2",
            "corp2",
            Some("lib2"),
            "Go Guide",
            "Intro",
            "Go is a compiled language",
        )
        .unwrap();

        let conn = Connection::open(&fts.path).unwrap();
        conn.execute(
            "INSERT INTO documents(document_id, library_id) VALUES('d1', 'lib1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO documents(document_id, library_id) VALUES('d2', 'lib2')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chunks(chunk_id, document_id, corpus_id) VALUES('c1', 'd1', 'corp1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chunks(chunk_id, document_id, corpus_id) VALUES('c2', 'd1', 'corp1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chunks(chunk_id, document_id, corpus_id) VALUES('c3', 'd2', 'corp2')",
            [],
        )
        .unwrap();
        drop(conn);

        let results = fts.search_chunks("Rust", None, None, 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].id, "c1");
        assert!(results[0].score > 0.0);

        let filtered = fts
            .search_chunks("language", Some("corp1"), None, 10)
            .unwrap();
        assert!(filtered.len() >= 2);

        let filtered_lib = fts
            .search_chunks("language", None, Some("lib1"), 10)
            .unwrap();
        assert!(filtered_lib.iter().any(|r| r.id == "c1"));
    }

    #[test]
    fn remove_chunks_by_document() {
        let (_dir, db_path) = setup_db();
        let fts = FtsSearch::open(db_path).unwrap();

        fts.index_chunk(
            "c1",
            "d1",
            "corp1",
            None,
            "Title",
            "",
            "unique content alpha",
        )
        .unwrap();
        fts.index_chunk(
            "c2",
            "d1",
            "corp1",
            None,
            "Title",
            "",
            "unique content beta",
        )
        .unwrap();

        let conn = Connection::open(&fts.path).unwrap();
        conn.execute(
            "INSERT INTO chunks(chunk_id, document_id, corpus_id) VALUES('c1', 'd1', 'corp1')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO chunks(chunk_id, document_id, corpus_id) VALUES('c2', 'd1', 'corp1')",
            [],
        )
        .unwrap();
        drop(conn);

        let before = fts.search_chunks("unique", None, None, 10).unwrap();
        assert_eq!(before.len(), 2);

        fts.remove_chunks_by_document("d1").unwrap();

        let after = fts.search_chunks("unique", None, None, 10).unwrap();
        assert!(after.is_empty());
    }

    #[test]
    fn index_and_search_session_events() {
        let (_dir, db_path) = setup_db();
        let fts = FtsSearch::open(db_path).unwrap();

        fts.index_session_event("e1", "s1", "user", "How do I deploy applications?")
            .unwrap();
        fts.index_session_event("e2", "s1", "assistant", "You can deploy using Docker")
            .unwrap();
        fts.index_session_event("e3", "s2", "user", "What is the weather today?")
            .unwrap();

        let conn = Connection::open(&fts.path).unwrap();
        conn.execute(
            "INSERT INTO session_events(event_id, session_id, kind, text) VALUES('e1', 's1', 'user', 'How do I deploy applications?')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO session_events(event_id, session_id, kind, text) VALUES('e2', 's1', 'assistant', 'You can deploy using Docker')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO session_events(event_id, session_id, kind, text) VALUES('e3', 's2', 'user', 'What is the weather today?')",
            [],
        ).unwrap();
        drop(conn);

        let results = fts.search_session_events("deploy", None, 10).unwrap();
        assert!(!results.is_empty());
        assert!(results.len() >= 2);

        let filtered = fts.search_session_events("deploy", Some("s1"), 10).unwrap();
        assert!(!filtered.is_empty());
        assert!(filtered.iter().any(|r| r.id == "e1"));
    }

    #[test]
    fn index_and_search_library_aliases() {
        let (_dir, db_path) = setup_db();
        let fts = FtsSearch::open(db_path).unwrap();

        fts.index_library_alias("react", "/facebook/react", "React", "npm")
            .unwrap();
        fts.index_library_alias("react-dom", "/facebook/react", "React DOM", "npm")
            .unwrap();
        fts.index_library_alias("vue", "/vuejs/vue", "Vue.js", "npm")
            .unwrap();

        let results = fts.search_library_aliases("React", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.id == "/facebook/react"));

        let vue = fts.search_library_aliases("Vue", 10).unwrap();
        assert!(!vue.is_empty());
        assert_eq!(vue[0].id, "/vuejs/vue");
    }

    #[test]
    fn bm25_score_is_positive_in_results() {
        let (_dir, db_path) = setup_db();
        let fts = FtsSearch::open(db_path).unwrap();

        fts.index_library_alias("tokio", "/tokio-rs/tokio", "Tokio", "crates")
            .unwrap();
        fts.index_library_alias("async-std", "/async-rs/async-std", "Async Std", "crates")
            .unwrap();

        let results = fts.search_library_aliases("tokio", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);
        assert!(results[0].score_breakdown["fts"].as_f64().unwrap() > 0.0);
    }
}
