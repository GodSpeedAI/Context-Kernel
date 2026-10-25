use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::Utc;
use ck_core::entities::{
    Artifact, Chunk, Corpus, Document, LibraryAlias, LibraryIdentity, RefreshJob, Session,
    SessionEvent, Source, StoredContext,
};
use ck_core::ports::{SessionMemoryPort, StoragePort};
use ck_core::types::{
    ContinuationMode, JobStatus, JobTrigger, SessionStatus, SessionTransport, SourceHealth,
    StorageStatus,
};
use rusqlite::Connection;
use ulid::Ulid;

#[derive(Clone)]
pub struct SqliteStore {
    path: PathBuf,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            path: path.as_ref().to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Opens a connection with PRAGMA foreign_keys = ON enabled.
    fn conn(&self) -> Result<Connection> {
        let conn = Connection::open(&self.path)?;
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
        Ok(conn)
    }

    pub fn migrate(&self) -> Result<()> {
        let mut conn = self.conn()?;
        conn.execute_batch(SCHEMA_V1)?;
        migrate_to_v2(&mut conn)
    }

    pub fn schema_sql() -> &'static str {
        SCHEMA_V1
    }

    pub fn status(&self) -> Result<StorageStatus> {
        let conn = self.conn()?;
        let fts5_enabled = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name = 'chunks_fts')",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            == 1;

        Ok(StorageStatus {
            kind: "sqlite".to_string(),
            path: self.path.display().to_string(),
            fts5_enabled,
        })
    }

    /// Clears all FTS tables and repopulates them from canonical rows.
    /// Returns (chunks_reindexed, session_events_reindexed, aliases_reindexed).
    pub fn rebuild_fts_index(&self) -> Result<(u64, u64, u64)> {
        // One transaction: a failure part-way leaves the existing index intact
        // rather than cleared or half-populated.
        let mut conn = self.conn()?;
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM chunks_fts", [])?;
        tx.execute("DELETE FROM session_events_fts", [])?;
        tx.execute("DELETE FROM library_aliases_fts", [])?;

        let chunks = tx.execute(
            "INSERT INTO chunks_fts (chunk_id, document_id, corpus_id, library_id, title, heading, text)
             SELECT c.chunk_id, c.document_id, c.corpus_id,
                    COALESCE(d.library_id, ''), COALESCE(d.title, ''),
                    COALESCE(c.heading, ''), c.text
             FROM chunks c LEFT JOIN documents d ON c.document_id = d.document_id",
            [],
        )? as u64;

        let events = tx.execute(
            "INSERT INTO session_events_fts (event_id, session_id, kind, text)
             SELECT event_id, session_id, kind, text FROM session_events",
            [],
        )? as u64;

        let aliases = tx.execute(
            "INSERT INTO library_aliases_fts (alias, library_id, name, provider)
             SELECT la.alias, la.library_id, COALESCE(li.name, ''), COALESCE(li.provider, '')
             FROM library_aliases la
             LEFT JOIN library_identities li ON la.library_id = li.library_id",
            [],
        )? as u64;

        tx.commit()?;
        Ok((chunks, events, aliases))
    }
}

fn transport_str(t: SessionTransport) -> &'static str {
    match t {
        SessionTransport::Stdio => "stdio",
        SessionTransport::Sse => "sse",
    }
}

fn transport_from(s: &str) -> Result<SessionTransport> {
    match s {
        "stdio" => Ok(SessionTransport::Stdio),
        "sse" => Ok(SessionTransport::Sse),
        other => Err(anyhow::anyhow!("unknown transport: {}", other)),
    }
}

fn cont_mode_str(m: ContinuationMode) -> &'static str {
    match m {
        ContinuationMode::Ephemeral => "ephemeral",
        ContinuationMode::Resumeable => "resumeable",
        ContinuationMode::Pinned => "pinned",
    }
}

fn cont_mode_from(s: &str) -> Result<ContinuationMode> {
    match s {
        "ephemeral" => Ok(ContinuationMode::Ephemeral),
        "resumeable" => Ok(ContinuationMode::Resumeable),
        "pinned" => Ok(ContinuationMode::Pinned),
        other => Err(anyhow::anyhow!("unknown continuation_mode: {}", other)),
    }
}

fn sess_status_str(s: SessionStatus) -> &'static str {
    match s {
        SessionStatus::Active => "active",
        SessionStatus::Closed => "closed",
        SessionStatus::Compacted => "compacted",
    }
}

fn sess_status_from(s: &str) -> Result<SessionStatus> {
    match s {
        "active" => Ok(SessionStatus::Active),
        "closed" => Ok(SessionStatus::Closed),
        "compacted" => Ok(SessionStatus::Compacted),
        other => Err(anyhow::anyhow!("unknown session_status: {}", other)),
    }
}

fn trigger_str(t: JobTrigger) -> &'static str {
    match t {
        JobTrigger::Startup => "startup",
        JobTrigger::Manual => "manual",
        JobTrigger::Scheduled => "scheduled",
        JobTrigger::Webhook => "webhook",
        JobTrigger::DependencyChange => "dependency_change",
    }
}

fn trigger_from(s: &str) -> Result<JobTrigger> {
    match s {
        "startup" => Ok(JobTrigger::Startup),
        "manual" => Ok(JobTrigger::Manual),
        "scheduled" => Ok(JobTrigger::Scheduled),
        "webhook" => Ok(JobTrigger::Webhook),
        "dependency_change" => Ok(JobTrigger::DependencyChange),
        other => Err(anyhow::anyhow!("unknown trigger: {}", other)),
    }
}

fn job_status_str(s: JobStatus) -> &'static str {
    match s {
        JobStatus::Queued => "queued",
        JobStatus::Running => "running",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Cancelled => "cancelled",
    }
}

fn job_status_from(s: &str) -> Result<JobStatus> {
    match s {
        "queued" => Ok(JobStatus::Queued),
        "running" => Ok(JobStatus::Running),
        "completed" => Ok(JobStatus::Completed),
        "failed" => Ok(JobStatus::Failed),
        "cancelled" => Ok(JobStatus::Cancelled),
        other => Err(anyhow::anyhow!("unknown job_status: {}", other)),
    }
}

fn row_to_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<Source> {
    Ok(Source {
        source_id: row.get(0)?,
        kind: row.get(1)?,
        display_name: row.get(2)?,
        enabled: row.get::<_, i64>(3)? != 0,
        adapter_config_json: row.get(4)?,
        auth_ref: row.get(5)?,
        first_indexed_at: row.get(6)?,
        last_refresh_attempt_at: row.get(7)?,
        last_refresh_success_at: row.get(8)?,
        last_refresh_error: row.get(9)?,
        freshness_ttl_secs: row.get(10)?,
        upstream_revision: row.get(11)?,
        created_at: row.get(12)?,
        updated_at: row.get(13)?,
    })
}

fn row_to_corpus(row: &rusqlite::Row<'_>) -> rusqlite::Result<Corpus> {
    Ok(Corpus {
        corpus_id: row.get(0)?,
        name: row.get(1)?,
        kind: row.get(2)?,
        visibility: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn row_to_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<Document> {
    Ok(Document {
        document_id: row.get(0)?,
        corpus_id: row.get(1)?,
        source_id: row.get(2)?,
        external_id: row.get(3)?,
        library_id: row.get(4)?,
        title: row.get(5)?,
        uri: row.get(6)?,
        version: row.get(7)?,
        mime_type: row.get(8)?,
        content_text: row.get(9)?,
        metadata_json: row.get(10)?,
        checksum: row.get(11)?,
        superseded_by_document_id: row.get(12)?,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

fn row_to_chunk(row: &rusqlite::Row<'_>) -> rusqlite::Result<Chunk> {
    Ok(Chunk {
        chunk_id: row.get(0)?,
        document_id: row.get(1)?,
        corpus_id: row.get(2)?,
        ordinal: row.get(3)?,
        text: row.get(4)?,
        token_count_estimate: row.get(5)?,
        heading: row.get(6)?,
        anchor: row.get(7)?,
        section_path_json: row.get(8)?,
        metadata_json: row.get(9)?,
        checksum: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn row_to_library_identity(row: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryIdentity> {
    Ok(LibraryIdentity {
        library_id: row.get(0)?,
        provider: row.get(1)?,
        name: row.get(2)?,
        version: row.get(3)?,
        namespace: row.get(4)?,
        language: row.get(5)?,
        summary: row.get(6)?,
        trust_score: row.get(7)?,
        metadata_json: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

fn row_to_library_alias(row: &rusqlite::Row<'_>) -> rusqlite::Result<LibraryAlias> {
    Ok(LibraryAlias {
        alias: row.get(0)?,
        library_id: row.get(1)?,
        alias_kind: row.get(2)?,
    })
}

fn row_to_refresh_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<RefreshJob> {
    let trigger_s: String = row.get(2)?;
    let status_s: String = row.get(3)?;
    let trigger_kind = trigger_from(&trigger_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;
    let status = job_status_from(&status_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;
    Ok(RefreshJob {
        job_id: row.get(0)?,
        source_id: row.get(1)?,
        trigger_kind,
        status,
        started_at: row.get(4)?,
        finished_at: row.get(5)?,
        error: row.get(6)?,
        stats_json: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusqlite::Result<Session> {
    let transport_s: String = row.get(2)?;
    let cont_s: String = row.get(3)?;
    let status_s: String = row.get(4)?;
    let transport = transport_from(&transport_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;
    let continuation_mode = cont_mode_from(&cont_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;
    let status = sess_status_from(&status_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            4,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;
    Ok(Session {
        session_id: row.get(0)?,
        client_id: row.get(1)?,
        transport,
        continuation_mode,
        status,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn row_to_session_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<SessionEvent> {
    Ok(SessionEvent {
        event_id: row.get(0)?,
        session_id: row.get(1)?,
        kind: row.get(2)?,
        text: row.get(3)?,
        artifact_id: row.get(4)?,
        metadata_json: row.get(5)?,
        pinned: row.get::<_, i64>(6)? != 0,
        created_at: row.get(7)?,
    })
}

impl StoragePort for SqliteStore {
    fn upsert_source(&self, source: &Source) -> Result<()> {
        let conn = self.conn()?;
        // UPSERT (not INSERT OR REPLACE): a replace deletes the old row and
        // cascades away every document/chunk of the source.
        conn.execute(
            "INSERT INTO sources \
             (source_id, kind, display_name, enabled, adapter_config_json, auth_ref, \
              first_indexed_at, last_refresh_attempt_at, last_refresh_success_at, \
              last_refresh_error, freshness_ttl_secs, upstream_revision, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14) \
             ON CONFLICT(source_id) DO UPDATE SET \
              kind = excluded.kind, display_name = excluded.display_name, \
              enabled = excluded.enabled, adapter_config_json = excluded.adapter_config_json, \
              auth_ref = excluded.auth_ref, first_indexed_at = excluded.first_indexed_at, \
              last_refresh_attempt_at = excluded.last_refresh_attempt_at, \
              last_refresh_success_at = excluded.last_refresh_success_at, \
              last_refresh_error = excluded.last_refresh_error, \
              freshness_ttl_secs = excluded.freshness_ttl_secs, \
              upstream_revision = excluded.upstream_revision, \
              updated_at = excluded.updated_at",
            rusqlite::params![
                source.source_id,
                source.kind,
                source.display_name,
                if source.enabled { 1i64 } else { 0i64 },
                source.adapter_config_json,
                source.auth_ref,
                source.first_indexed_at,
                source.last_refresh_attempt_at,
                source.last_refresh_success_at,
                source.last_refresh_error,
                source.freshness_ttl_secs,
                source.upstream_revision,
                source.created_at,
                source.updated_at,
            ],
        )?;
        Ok(())
    }

    fn get_source(&self, source_id: &str) -> Result<Option<Source>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM sources WHERE source_id = ?1",
            rusqlite::params![source_id],
            row_to_source,
        ) {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_sources(&self) -> Result<Vec<Source>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM sources")?;
        let rows = stmt.query_map([], row_to_source)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn upsert_corpus(&self, corpus: &Corpus) -> Result<()> {
        let conn = self.conn()?;
        // UPSERT (not INSERT OR REPLACE): a replace deletes the old row and
        // cascades away every document/chunk of the corpus.
        conn.execute(
            "INSERT INTO corpora \
             (corpus_id, name, kind, visibility, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT(corpus_id) DO UPDATE SET \
              name = excluded.name, kind = excluded.kind, \
              visibility = excluded.visibility, updated_at = excluded.updated_at",
            rusqlite::params![
                corpus.corpus_id,
                corpus.name,
                corpus.kind,
                corpus.visibility,
                corpus.created_at,
                corpus.updated_at,
            ],
        )?;
        Ok(())
    }

    fn get_corpus(&self, corpus_id: &str) -> Result<Option<Corpus>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM corpora WHERE corpus_id = ?1",
            rusqlite::params![corpus_id],
            row_to_corpus,
        ) {
            Ok(c) => Ok(Some(c)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_corpora(&self) -> Result<Vec<Corpus>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM corpora")?;
        let rows = stmt.query_map([], row_to_corpus)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn link_source_to_corpus(&self, corpus_id: &str, source_id: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR IGNORE INTO corpus_sources (corpus_id, source_id) VALUES (?1, ?2)",
            rusqlite::params![corpus_id, source_id],
        )?;
        Ok(())
    }

    fn find_corpus_id_for_source(&self, source_id: &str) -> Result<Option<String>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT corpus_id FROM corpus_sources WHERE source_id = ?1 ORDER BY corpus_id LIMIT 1",
            rusqlite::params![source_id],
            |row| row.get(0),
        ) {
            Ok(corpus_id) => Ok(Some(corpus_id)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn upsert_document(&self, doc: &Document) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO documents \
             (document_id, corpus_id, source_id, external_id, library_id, title, uri, version, \
              mime_type, content_text, metadata_json, checksum, superseded_by_document_id, \
              created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            rusqlite::params![
                doc.document_id,
                doc.corpus_id,
                doc.source_id,
                doc.external_id,
                doc.library_id,
                doc.title,
                doc.uri,
                doc.version,
                doc.mime_type,
                doc.content_text,
                doc.metadata_json,
                doc.checksum,
                doc.superseded_by_document_id,
                doc.created_at,
                doc.updated_at,
            ],
        )?;
        Ok(())
    }

    fn get_document(&self, document_id: &str) -> Result<Option<Document>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM documents WHERE document_id = ?1",
            rusqlite::params![document_id],
            row_to_document,
        ) {
            Ok(d) => Ok(Some(d)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn delete_document(&self, document_id: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM documents WHERE document_id = ?1",
            rusqlite::params![document_id],
        )?;
        Ok(())
    }

    fn list_documents_by_source(&self, source_id: &str) -> Result<Vec<Document>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM documents WHERE source_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![source_id], row_to_document)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn list_documents_by_corpus(&self, corpus_id: &str) -> Result<Vec<Document>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM documents WHERE corpus_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![corpus_id], row_to_document)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn replace_chunks(&self, document_id: &str, chunks: &[Chunk]) -> Result<()> {
        let conn = self.conn()?;
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM chunks WHERE document_id = ?1",
            rusqlite::params![document_id],
        )?;
        for chunk in chunks {
            tx.execute(
                "INSERT INTO chunks \
                 (chunk_id, document_id, corpus_id, ordinal, text, token_count_estimate, \
                  heading, anchor, section_path_json, metadata_json, checksum, \
                  created_at, updated_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                rusqlite::params![
                    chunk.chunk_id,
                    chunk.document_id,
                    chunk.corpus_id,
                    chunk.ordinal,
                    chunk.text,
                    chunk.token_count_estimate,
                    chunk.heading,
                    chunk.anchor,
                    chunk.section_path_json,
                    chunk.metadata_json,
                    chunk.checksum,
                    chunk.created_at,
                    chunk.updated_at,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    fn list_chunks_by_document(&self, document_id: &str) -> Result<Vec<Chunk>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM chunks WHERE document_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![document_id], row_to_chunk)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn list_chunks_by_corpus(&self, corpus_id: &str) -> Result<Vec<Chunk>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM chunks WHERE corpus_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![corpus_id], row_to_chunk)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn upsert_library_identity(&self, lib: &LibraryIdentity) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO library_identities \
             (library_id, provider, name, version, namespace, language, summary, \
              trust_score, metadata_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                lib.library_id,
                lib.provider,
                lib.name,
                lib.version,
                lib.namespace,
                lib.language,
                lib.summary,
                lib.trust_score,
                lib.metadata_json,
                lib.created_at,
                lib.updated_at,
            ],
        )?;
        Ok(())
    }

    fn get_library_identity(&self, library_id: &str) -> Result<Option<LibraryIdentity>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM library_identities WHERE library_id = ?1",
            rusqlite::params![library_id],
            row_to_library_identity,
        ) {
            Ok(l) => Ok(Some(l)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_library_identities(&self) -> Result<Vec<LibraryIdentity>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM library_identities")?;
        let rows = stmt.query_map([], row_to_library_identity)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn upsert_library_alias(&self, alias: &LibraryAlias) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO library_aliases (alias, library_id, alias_kind) \
             VALUES (?1, ?2, ?3)",
            rusqlite::params![alias.alias, alias.library_id, alias.alias_kind],
        )?;
        Ok(())
    }

    fn list_library_aliases(&self, library_id: &str) -> Result<Vec<LibraryAlias>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM library_aliases WHERE library_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![library_id], row_to_library_alias)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn upsert_refresh_job(&self, job: &RefreshJob) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO refresh_jobs \
             (job_id, source_id, trigger_kind, status, started_at, finished_at, error, \
              stats_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                job.job_id,
                job.source_id,
                trigger_str(job.trigger_kind),
                job_status_str(job.status),
                job.started_at,
                job.finished_at,
                job.error,
                job.stats_json,
                job.created_at,
                job.updated_at,
            ],
        )?;
        Ok(())
    }

    fn insert_active_refresh_job(&self, job: &RefreshJob) -> Result<RefreshJob> {
        let conn = self.conn()?;
        let inserted = conn.execute(
            "INSERT INTO refresh_jobs \
             (job_id, source_id, trigger_kind, status, started_at, finished_at, error, \
              stats_json, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(source_id) WHERE status IN ('queued', 'running') DO NOTHING",
            rusqlite::params![
                job.job_id,
                job.source_id,
                trigger_str(job.trigger_kind),
                job_status_str(job.status),
                job.started_at,
                job.finished_at,
                job.error,
                job.stats_json,
                job.created_at,
                job.updated_at,
            ],
        )?;

        if inserted == 1 {
            Ok(job.clone())
        } else {
            self.find_active_job_for_source(&job.source_id)?
                .ok_or_else(|| anyhow::anyhow!("active refresh job conflict without stored job"))
        }
    }

    fn get_refresh_job(&self, job_id: &str) -> Result<Option<RefreshJob>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM refresh_jobs WHERE job_id = ?1",
            rusqlite::params![job_id],
            row_to_refresh_job,
        ) {
            Ok(j) => Ok(Some(j)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_refresh_jobs_by_source(&self, source_id: &str) -> Result<Vec<RefreshJob>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM refresh_jobs WHERE source_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![source_id], row_to_refresh_job)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn update_source_health(
        &self,
        source_id: &str,
        health: SourceHealth,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn()?;
        let now = Utc::now().to_rfc3339();
        match health {
            SourceHealth::Healthy => {
                conn.execute(
                    "UPDATE sources SET last_refresh_success_at = ?1, last_refresh_error = NULL, updated_at = ?2 WHERE source_id = ?3",
                    rusqlite::params![now, now, source_id],
                )?;
            }
            SourceHealth::Degraded => {
                conn.execute(
                    "UPDATE sources SET last_refresh_error = ?1, updated_at = ?2 WHERE source_id = ?3",
                    rusqlite::params![error.unwrap_or("unknown error"), now, source_id],
                )?;
            }
            SourceHealth::Unknown => {}
        }
        Ok(())
    }

    fn list_stale_sources(&self) -> Result<Vec<Source>> {
        let conn = self.conn()?;
        // Sources where freshness_ttl_secs is set AND
        // (last_refresh_success_at + freshness_ttl_secs) < now OR never refreshed
        let mut stmt = conn.prepare(
            "SELECT * FROM sources WHERE enabled = 1 AND freshness_ttl_secs IS NOT NULL AND (
                last_refresh_success_at IS NULL
                OR datetime(last_refresh_success_at, '+' || freshness_ttl_secs || ' seconds') < datetime('now')
            )",
        )?;
        let rows = stmt.query_map([], row_to_source)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn get_chunk_with_document(&self, chunk_id: &str) -> Result<Option<(Chunk, Document)>> {
        let conn = self.conn()?;
        let chunk = match conn.query_row(
            "SELECT * FROM chunks WHERE chunk_id = ?1",
            rusqlite::params![chunk_id],
            row_to_chunk,
        ) {
            Ok(c) => c,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        let doc = match conn.query_row(
            "SELECT * FROM documents WHERE document_id = ?1",
            rusqlite::params![chunk.document_id],
            row_to_document,
        ) {
            Ok(d) => d,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        Ok(Some((chunk, doc)))
    }

    fn find_active_job_for_source(&self, source_id: &str) -> Result<Option<RefreshJob>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM refresh_jobs WHERE source_id = ?1 AND status IN ('queued', 'running') ORDER BY created_at DESC LIMIT 1",
            rusqlite::params![source_id],
            row_to_refresh_job,
        ) {
            Ok(j) => Ok(Some(j)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn claim_next_refresh_job_for_source(&self, source_id: &str) -> Result<Option<RefreshJob>> {
        let conn = self.conn()?;
        let now = Utc::now().to_rfc3339();
        match conn.query_row(
            "UPDATE refresh_jobs
             SET status = 'running', started_at = ?2, updated_at = ?2
             WHERE job_id = (
                 SELECT job_id FROM refresh_jobs
                 WHERE source_id = ?1 AND status = 'queued'
                 ORDER BY created_at ASC
                 LIMIT 1
             )
             AND status = 'queued'
             RETURNING *",
            rusqlite::params![source_id, now],
            row_to_refresh_job,
        ) {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn count_jobs_by_status(&self, status_str: &str) -> Result<u64> {
        let conn = self.conn()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM refresh_jobs WHERE status = ?1",
            rusqlite::params![status_str],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    fn delete_unpinned_session_events_older_than(&self, cutoff_rfc3339: &str) -> Result<u64> {
        let conn = self.conn()?;
        let deleted = conn.execute(
            "DELETE FROM session_events WHERE pinned = 0 AND created_at < ?1",
            rusqlite::params![cutoff_rfc3339],
        )?;
        Ok(deleted as u64)
    }

    fn delete_inactive_sessions_older_than(&self, cutoff_rfc3339: &str) -> Result<u64> {
        let conn = self.conn()?;
        let deleted = conn.execute(
            "DELETE FROM sessions \
             WHERE status IN ('closed', 'compacted') AND updated_at < ?1 \
             AND NOT EXISTS (SELECT 1 FROM session_events e \
                 WHERE e.session_id = sessions.session_id AND e.pinned = 1)",
            rusqlite::params![cutoff_rfc3339],
        )?;
        Ok(deleted as u64)
    }

    fn delete_finished_refresh_jobs_older_than(&self, cutoff_rfc3339: &str) -> Result<u64> {
        let conn = self.conn()?;
        let deleted = conn.execute(
            "DELETE FROM refresh_jobs \
             WHERE status IN ('completed', 'failed', 'cancelled') \
             AND COALESCE(finished_at, updated_at, created_at) < ?1",
            rusqlite::params![cutoff_rfc3339],
        )?;
        Ok(deleted as u64)
    }

    fn delete_disabled_sources_older_than(&self, cutoff_rfc3339: &str) -> Result<u64> {
        let conn = self.conn()?;
        let deleted = conn.execute(
            "DELETE FROM sources WHERE enabled = 0 AND updated_at < ?1",
            rusqlite::params![cutoff_rfc3339],
        )?;
        Ok(deleted as u64)
    }

    fn delete_artifacts_older_than(&self, cutoff_rfc3339: &str) -> Result<u64> {
        let conn = self.conn()?;
        let deleted = conn.execute(
            "DELETE FROM artifacts WHERE created_at < ?1 \
             AND NOT EXISTS (SELECT 1 FROM session_events e \
                 WHERE e.artifact_id = artifacts.artifact_id AND e.pinned = 1)",
            rusqlite::params![cutoff_rfc3339],
        )?;
        Ok(deleted as u64)
    }

    fn upsert_artifact(&self, artifact: &Artifact) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR REPLACE INTO artifacts \
             (artifact_id, session_id, kind, mime_type, storage_uri, size_bytes, summary, checksum, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                artifact.artifact_id,
                artifact.session_id,
                artifact.kind,
                artifact.mime_type,
                artifact.storage_uri,
                artifact.size_bytes,
                artifact.summary,
                artifact.checksum,
                artifact.created_at,
            ],
        )?;
        Ok(())
    }

    fn list_artifacts_older_than(&self, cutoff_rfc3339: &str) -> Result<Vec<String>> {
        let conn = self.conn()?;
        // Artifacts referenced by a pinned event are retained: their payloads
        // must not be deleted either, so this list gates payload removal too.
        let mut stmt = conn.prepare(
            "SELECT artifact_id FROM artifacts WHERE created_at < ?1 \
             AND NOT EXISTS (SELECT 1 FROM session_events e \
                 WHERE e.artifact_id = artifacts.artifact_id AND e.pinned = 1)",
        )?;
        let rows = stmt.query_map(rusqlite::params![cutoff_rfc3339], |row| row.get(0))?;
        let mut ids = Vec::new();
        for r in rows {
            ids.push(r?);
        }
        Ok(ids)
    }
}

impl SessionMemoryPort for SqliteStore {
    fn create_session(
        &self,
        session_id: &str,
        transport: SessionTransport,
        continuation_mode: ContinuationMode,
    ) -> Result<Session> {
        let now = Utc::now().to_rfc3339();
        let session = Session {
            session_id: session_id.to_string(),
            client_id: None,
            transport,
            continuation_mode,
            status: SessionStatus::Active,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        let conn = self.conn()?;
        conn.execute(
            "INSERT OR IGNORE INTO sessions \
             (session_id, client_id, transport, continuation_mode, status, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                session.session_id,
                session.client_id,
                transport_str(session.transport),
                cont_mode_str(session.continuation_mode),
                sess_status_str(session.status),
                session.created_at,
                session.updated_at,
            ],
        )?;
        Ok(session)
    }

    fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM sessions WHERE session_id = ?1",
            rusqlite::params![session_id],
            row_to_session,
        ) {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn add_event(
        &self,
        session_id: &str,
        kind: &str,
        text: &str,
        artifact_id: Option<&str>,
        metadata_json: &str,
        pinned: bool,
    ) -> Result<StoredContext> {
        let event_id = Ulid::new().to_string();
        let now = Utc::now().to_rfc3339();
        let conn = self.conn()?;
        conn.execute(
            "INSERT INTO session_events \
             (event_id, session_id, kind, text, artifact_id, metadata_json, pinned, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                event_id,
                session_id,
                kind,
                text,
                artifact_id,
                metadata_json,
                if pinned { 1i64 } else { 0i64 },
                now,
            ],
        )?;
        Ok(StoredContext {
            event_id,
            session_id: session_id.to_string(),
            kind: kind.to_string(),
            stored_as: "session_event".to_string(),
            artifact_id: artifact_id.map(|a| a.to_string()),
            created_at: now,
        })
    }

    fn get_event(&self, event_id: &str) -> Result<Option<SessionEvent>> {
        let conn = self.conn()?;
        match conn.query_row(
            "SELECT * FROM session_events WHERE event_id = ?1",
            rusqlite::params![event_id],
            row_to_session_event,
        ) {
            Ok(e) => Ok(Some(e)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_events_by_session(&self, session_id: &str) -> Result<Vec<SessionEvent>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare("SELECT * FROM session_events WHERE session_id = ?1")?;
        let rows = stmt.query_map(rusqlite::params![session_id], row_to_session_event)?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn count_active_sessions(&self) -> Result<u64> {
        let conn = self.conn()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sessions WHERE status = 'active'",
            [],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }
}

/// Version 2 re-keys document IDs from `std::hash::DefaultHasher` (not stable
/// across Rust releases) to `ck_core::entities::stable_document_id`.
///
/// The v1 ID was a hash of `(source_id, external_id, uri)` and those inputs are
/// recoverable only when the stored row carries the exact values that were hashed:
/// `source_id` is always stored, and `external_id`/`uri` are stored when they were
/// `Some` at ingest (ingest fell back to the raw external id otherwise, which is
/// not stored). So a v1 store with any document whose `external_id` or `uri` is
/// NULL is refused with a re-ingest message instead of guessing. Otherwise every
/// document, its superseded reference, its chunks, the chunk FTS rows and any
/// chunk embeddings are re-keyed in one transaction, and the foreign-key check
/// must pass before commit.
fn migrate_to_v2(conn: &mut Connection) -> Result<()> {
    let recorded: i64 = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = 2)",
        [],
        |row| row.get(0),
    )?;
    if recorded == 1 {
        return Ok(());
    }

    let docs: Vec<(String, String, Option<String>, Option<String>)> = {
        let mut stmt =
            conn.prepare("SELECT document_id, source_id, external_id, uri FROM documents")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        rows.collect::<rusqlite::Result<_>>()?
    };

    let unrecoverable = docs
        .iter()
        .filter(|(_, _, external_id, uri)| external_id.is_none() || uri.is_none())
        .count();
    if unrecoverable > 0 {
        anyhow::bail!(
            "store schema v1 holds {unrecoverable} document(s) whose external_id or uri is NULL; \
             their v1 document IDs cannot be recomputed, so the store cannot be migrated to v2. \
             Re-ingest required: move this database aside and re-ingest every source"
        );
    }

    let tx = conn.transaction()?;
    if !docs.is_empty() {
        tx.execute_batch(
            "DROP TABLE IF EXISTS temp.doc_id_map;
             CREATE TEMP TABLE doc_id_map (old_id TEXT PRIMARY KEY, new_id TEXT NOT NULL);
             PRAGMA defer_foreign_keys = ON;",
        )?;
        {
            let mut insert =
                tx.prepare("INSERT INTO temp.doc_id_map (old_id, new_id) VALUES (?1, ?2)")?;
            for (old_id, source_id, external_id, uri) in &docs {
                let new_id = ck_core::entities::stable_document_id(
                    source_id,
                    external_id.as_deref().unwrap_or_default(),
                    uri.as_deref().unwrap_or_default(),
                );
                insert.execute(rusqlite::params![old_id, new_id])?;
            }
        }
        let remap = "(SELECT new_id FROM temp.doc_id_map WHERE old_id = ";
        tx.execute_batch(&format!(
            "UPDATE documents SET superseded_by_document_id = {remap}documents.superseded_by_document_id)
               WHERE superseded_by_document_id IN (SELECT old_id FROM temp.doc_id_map);
             UPDATE documents SET document_id = {remap}documents.document_id);
             UPDATE chunks SET document_id = {remap}chunks.document_id)
               WHERE document_id IN (SELECT old_id FROM temp.doc_id_map);
             UPDATE chunks_fts SET document_id = {remap}chunks_fts.document_id)
               WHERE document_id IN (SELECT old_id FROM temp.doc_id_map);"
        ))?;
        let has_embeddings: i64 = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'chunk_embeddings')",
            [],
            |row| row.get(0),
        )?;
        if has_embeddings == 1 {
            tx.execute_batch(&format!(
                "UPDATE chunk_embeddings SET document_id = {remap}chunk_embeddings.document_id)
                   WHERE document_id IN (SELECT old_id FROM temp.doc_id_map);"
            ))?;
        }
        tx.execute_batch("DROP TABLE temp.doc_id_map;")?;

        let violations: i64 =
            tx.query_row("SELECT COUNT(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })?;
        if violations > 0 {
            anyhow::bail!(
                "v2 document re-keying left {violations} dangling foreign key(s); rolled back"
            );
        }
    }
    tx.execute(
        "INSERT OR IGNORE INTO schema_migrations (version, applied_at)
         VALUES (2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
        [],
    )?;
    tx.commit()?;
    Ok(())
}

const SCHEMA_V1: &str = r#"
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
PRAGMA synchronous=NORMAL;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sources (
  source_id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  display_name TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  adapter_config_json TEXT NOT NULL,
  auth_ref TEXT,
  first_indexed_at TEXT,
  last_refresh_attempt_at TEXT,
  last_refresh_success_at TEXT,
  last_refresh_error TEXT,
  freshness_ttl_secs INTEGER,
  upstream_revision TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS corpora (
  corpus_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  kind TEXT NOT NULL,
  visibility TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS corpus_sources (
  corpus_id TEXT NOT NULL,
  source_id TEXT NOT NULL,
  PRIMARY KEY (corpus_id, source_id),
  FOREIGN KEY (corpus_id) REFERENCES corpora(corpus_id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES sources(source_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS library_identities (
  library_id TEXT PRIMARY KEY,
  provider TEXT NOT NULL,
  name TEXT NOT NULL,
  version TEXT,
  namespace TEXT,
  language TEXT,
  summary TEXT,
  trust_score REAL,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS library_aliases (
  alias TEXT NOT NULL,
  library_id TEXT NOT NULL,
  alias_kind TEXT NOT NULL DEFAULT 'general',
  PRIMARY KEY (alias, library_id),
  FOREIGN KEY (library_id) REFERENCES library_identities(library_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS documents (
  document_id TEXT PRIMARY KEY,
  corpus_id TEXT NOT NULL,
  source_id TEXT NOT NULL,
  external_id TEXT,
  library_id TEXT,
  title TEXT NOT NULL,
  uri TEXT,
  version TEXT,
  mime_type TEXT,
  content_text TEXT NOT NULL,
  metadata_json TEXT NOT NULL,
  checksum TEXT NOT NULL,
  superseded_by_document_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (corpus_id) REFERENCES corpora(corpus_id) ON DELETE CASCADE,
  FOREIGN KEY (source_id) REFERENCES sources(source_id) ON DELETE CASCADE,
  FOREIGN KEY (library_id) REFERENCES library_identities(library_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_documents_source_id ON documents(source_id);
CREATE INDEX IF NOT EXISTS idx_documents_corpus_id ON documents(corpus_id);
CREATE INDEX IF NOT EXISTS idx_documents_library_id ON documents(library_id);
CREATE INDEX IF NOT EXISTS idx_documents_uri ON documents(uri);
CREATE INDEX IF NOT EXISTS idx_documents_checksum ON documents(checksum);

CREATE TABLE IF NOT EXISTS chunks (
  chunk_id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL,
  corpus_id TEXT NOT NULL,
  ordinal INTEGER NOT NULL,
  text TEXT NOT NULL,
  token_count_estimate INTEGER NOT NULL,
  heading TEXT,
  anchor TEXT,
  section_path_json TEXT NOT NULL DEFAULT '[]',
  metadata_json TEXT NOT NULL DEFAULT '{}',
  checksum TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (document_id) REFERENCES documents(document_id) ON DELETE CASCADE,
  FOREIGN KEY (corpus_id) REFERENCES corpora(corpus_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_chunks_document_id ON chunks(document_id);
CREATE INDEX IF NOT EXISTS idx_chunks_corpus_id ON chunks(corpus_id);
CREATE INDEX IF NOT EXISTS idx_chunks_ordinal ON chunks(document_id, ordinal);
CREATE INDEX IF NOT EXISTS idx_chunks_checksum ON chunks(checksum);

CREATE TABLE IF NOT EXISTS artifacts (
  artifact_id TEXT PRIMARY KEY,
  session_id TEXT,
  kind TEXT NOT NULL,
  mime_type TEXT,
  storage_uri TEXT NOT NULL,
  size_bytes INTEGER NOT NULL,
  summary TEXT,
  checksum TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
  session_id TEXT PRIMARY KEY,
  client_id TEXT,
  transport TEXT NOT NULL,
  continuation_mode TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS session_events (
  event_id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  kind TEXT NOT NULL,
  text TEXT NOT NULL,
  artifact_id TEXT,
  metadata_json TEXT NOT NULL DEFAULT '{}',
  pinned INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE,
  FOREIGN KEY (artifact_id) REFERENCES artifacts(artifact_id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_session_events_session_id ON session_events(session_id);
CREATE INDEX IF NOT EXISTS idx_session_events_kind ON session_events(kind);
CREATE INDEX IF NOT EXISTS idx_session_events_created_at ON session_events(created_at);

CREATE TABLE IF NOT EXISTS refresh_jobs (
  job_id TEXT PRIMARY KEY,
  source_id TEXT NOT NULL,
  trigger_kind TEXT NOT NULL,
  status TEXT NOT NULL,
  started_at TEXT,
  finished_at TEXT,
  error TEXT,
  stats_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (source_id) REFERENCES sources(source_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_refresh_jobs_source_id ON refresh_jobs(source_id);
CREATE INDEX IF NOT EXISTS idx_refresh_jobs_status ON refresh_jobs(status);
CREATE INDEX IF NOT EXISTS idx_refresh_jobs_created_at ON refresh_jobs(created_at);
CREATE UNIQUE INDEX IF NOT EXISTS idx_refresh_jobs_one_active_per_source
  ON refresh_jobs(source_id)
  WHERE status IN ('queued', 'running');

CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
  chunk_id UNINDEXED,
  document_id UNINDEXED,
  corpus_id UNINDEXED,
  library_id UNINDEXED,
  title,
  heading,
  text,
  tokenize = 'unicode61'
);

CREATE VIRTUAL TABLE IF NOT EXISTS session_events_fts USING fts5(
  event_id UNINDEXED,
  session_id UNINDEXED,
  kind,
  text,
  tokenize = 'unicode61'
);

CREATE VIRTUAL TABLE IF NOT EXISTS library_aliases_fts USING fts5(
  alias,
  library_id UNINDEXED,
  name,
  provider,
  tokenize = 'unicode61'
);

INSERT OR IGNORE INTO schema_migrations (version, applied_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn test_store() -> (SqliteStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = SqliteStore::open(&db_path).unwrap();
        store.migrate().unwrap();
        (store, dir)
    }

    fn now() -> String {
        Utc::now().to_rfc3339()
    }

    #[test]
    fn session_create_and_retrieve() {
        let (store, _dir) = test_store();

        let session = store
            .create_session(
                "sess-1",
                SessionTransport::Stdio,
                ContinuationMode::Ephemeral,
            )
            .unwrap();

        assert_eq!(session.session_id, "sess-1");
        assert_eq!(session.transport, SessionTransport::Stdio);
        assert_eq!(session.continuation_mode, ContinuationMode::Ephemeral);
        assert_eq!(session.status, SessionStatus::Active);

        let fetched = store.get_session("sess-1").unwrap().unwrap();
        assert_eq!(fetched.session_id, "sess-1");
        assert_eq!(fetched.transport, SessionTransport::Stdio);
        assert_eq!(fetched.status, SessionStatus::Active);

        assert!(store.get_session("nonexistent").unwrap().is_none());
        assert_eq!(store.count_active_sessions().unwrap(), 1);
    }

    #[test]
    fn add_events_and_list() {
        let (store, _dir) = test_store();
        store
            .create_session("sess-2", SessionTransport::Sse, ContinuationMode::Pinned)
            .unwrap();

        let ctx1 = store
            .add_event("sess-2", "user_message", "hello", None, "{}", false)
            .unwrap();
        assert_eq!(ctx1.session_id, "sess-2");
        assert_eq!(ctx1.kind, "user_message");
        assert_eq!(ctx1.stored_as, "session_event");
        assert!(ctx1.artifact_id.is_none());

        let _ctx2 = store
            .add_event(
                "sess-2",
                "assistant_reply",
                "world",
                None,
                r#"{"key":"val"}"#,
                true,
            )
            .unwrap();

        let events = store.list_events_by_session("sess-2").unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, "user_message");
        assert_eq!(events[0].text, "hello");
        assert!(!events[0].pinned);
        assert_eq!(events[1].kind, "assistant_reply");
        assert_eq!(events[1].text, "world");
        assert!(events[1].pinned);

        let fetched = store.get_event(&ctx1.event_id).unwrap().unwrap();
        assert_eq!(fetched.text, "hello");
    }

    #[test]
    fn persisted_enums_roundtrip_every_domain_variant() {
        // Persist every domain enum variant through real tables and read the
        // typed values back. This exercises the Sse/Pinned/Compacted/
        // DependencyChange/Cancelled decoder arms that the audit fixed; a
        // reverted decoder makes one of these `get_*` calls return a
        // FromSqlConversionFailure instead of the typed enum.
        let (store, _dir) = test_store();
        let t = now();

        store
            .upsert_source(&Source {
                source_id: "src-persist".into(),
                kind: "local".into(),
                display_name: "S".into(),
                enabled: true,
                adapter_config_json: "{}".into(),
                auth_ref: None,
                first_indexed_at: None,
                last_refresh_attempt_at: None,
                last_refresh_success_at: None,
                last_refresh_error: None,
                freshness_ttl_secs: None,
                upstream_revision: None,
                created_at: t.clone(),
                updated_at: t.clone(),
            })
            .unwrap();

        // 1. Sessions: every transport × continuation_mode through create_session.
        let session_specs = [
            (
                "sess-stdio-eph",
                SessionTransport::Stdio,
                ContinuationMode::Ephemeral,
            ),
            (
                "sess-stdio-res",
                SessionTransport::Stdio,
                ContinuationMode::Resumeable,
            ),
            (
                "sess-stdio-pin",
                SessionTransport::Stdio,
                ContinuationMode::Pinned,
            ),
            (
                "sess-sse-eph",
                SessionTransport::Sse,
                ContinuationMode::Ephemeral,
            ),
            (
                "sess-sse-res",
                SessionTransport::Sse,
                ContinuationMode::Resumeable,
            ),
            (
                "sess-sse-pin",
                SessionTransport::Sse,
                ContinuationMode::Pinned,
            ),
        ];
        for (id, transport, cont) in session_specs {
            let created = store.create_session(id, transport, cont).unwrap();
            assert_eq!(created.transport, transport);
            assert_eq!(created.continuation_mode, cont);
            assert_eq!(created.status, SessionStatus::Active);

            let fetched = store.get_session(id).unwrap().unwrap();
            assert_eq!(fetched.transport, transport, "transport lost for {id}");
            assert_eq!(fetched.continuation_mode, cont, "cont_mode lost for {id}");
            assert_eq!(
                fetched.status,
                SessionStatus::Active,
                "status lost for {id}"
            );
        }

        // 2. SessionStatus: Active is exercised above; Closed/Compacted require a
        // status update through SQL (create_session only writes Active), then a
        // fresh get_session to prove the decoder accepts the persisted string.
        {
            let conn = store.conn().unwrap();
            conn.execute(
                "UPDATE sessions SET status = 'closed', updated_at = ?1 \
                 WHERE session_id = 'sess-stdio-eph'",
                rusqlite::params![t],
            )
            .unwrap();
            conn.execute(
                "UPDATE sessions SET status = 'compacted', updated_at = ?1 \
                 WHERE session_id = 'sess-stdio-res'",
                rusqlite::params![t],
            )
            .unwrap();
        }
        assert_eq!(
            store.get_session("sess-stdio-eph").unwrap().unwrap().status,
            SessionStatus::Closed
        );
        assert_eq!(
            store.get_session("sess-stdio-res").unwrap().unwrap().status,
            SessionStatus::Compacted
        );

        // 3. Refresh jobs: every JobTrigger × JobStatus through upsert_refresh_job.
        let trigger_status_specs: [(JobTrigger, JobStatus); 6] = [
            (JobTrigger::Manual, JobStatus::Queued),
            (JobTrigger::Manual, JobStatus::Running),
            (JobTrigger::Manual, JobStatus::Completed),
            (JobTrigger::Manual, JobStatus::Failed),
            (JobTrigger::Manual, JobStatus::Cancelled),
            (JobTrigger::DependencyChange, JobStatus::Completed),
        ];
        let triggers = [
            JobTrigger::Startup,
            JobTrigger::Manual,
            JobTrigger::Scheduled,
            JobTrigger::Webhook,
            JobTrigger::DependencyChange,
        ];
        for (i, trigger) in triggers.iter().enumerate() {
            store
                .upsert_refresh_job(&RefreshJob {
                    job_id: format!("job-tr-{i}"),
                    source_id: "src-persist".into(),
                    trigger_kind: *trigger,
                    status: JobStatus::Queued,
                    started_at: None,
                    finished_at: None,
                    error: None,
                    stats_json: "{}".into(),
                    created_at: t.clone(),
                    updated_at: t.clone(),
                })
                .unwrap();
            let fetched = store
                .get_refresh_job(&format!("job-tr-{i}"))
                .unwrap()
                .unwrap();
            assert_eq!(
                fetched.trigger_kind, *trigger,
                "trigger lost for job-tr-{i}"
            );
        }

        // Status transitions: queue → run → each terminal, asserting the
        // decoder accepts the persisted status string on each read.
        for (i, (_trigger, status)) in trigger_status_specs.iter().enumerate() {
            let job_id = format!("job-st-{i}");
            store
                .upsert_refresh_job(&RefreshJob {
                    job_id: job_id.clone(),
                    source_id: "src-persist".into(),
                    trigger_kind: JobTrigger::Manual,
                    status: *status,
                    started_at: Some(t.clone()),
                    finished_at: matches!(
                        *status,
                        JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
                    )
                    .then(|| t.clone()),
                    error: matches!(*status, JobStatus::Failed).then(|| "boom".into()),
                    stats_json: "{}".into(),
                    created_at: t.clone(),
                    updated_at: t.clone(),
                })
                .unwrap();
            let fetched = store.get_refresh_job(&job_id).unwrap().unwrap();
            assert_eq!(fetched.status, *status, "status lost for {job_id}");
        }
    }

    #[test]
    fn retention_deletes_only_eligible_rows() {
        let (store, _dir) = test_store();
        let old = "2026-01-01T00:00:00Z";
        let cutoff = "2026-02-01T00:00:00Z";
        let recent = "2026-03-01T00:00:00Z";

        store
            .create_session(
                "sess-old",
                SessionTransport::Stdio,
                ContinuationMode::Ephemeral,
            )
            .unwrap();
        let old_event = store
            .add_event("sess-old", "note", "old", None, "{}", false)
            .unwrap();
        let pinned_event = store
            .add_event("sess-old", "note", "pinned", None, "{}", true)
            .unwrap();
        let recent_event = store
            .add_event("sess-old", "note", "recent", None, "{}", false)
            .unwrap();

        {
            let conn = store.conn().unwrap();
            conn.execute(
                "UPDATE session_events SET created_at = ?1 WHERE event_id IN (?2, ?3)",
                rusqlite::params![old, old_event.event_id, pinned_event.event_id],
            )
            .unwrap();
            conn.execute(
                "UPDATE session_events SET created_at = ?1 WHERE event_id = ?2",
                rusqlite::params![recent, recent_event.event_id],
            )
            .unwrap();
            conn.execute(
                "UPDATE sessions SET status = 'closed', updated_at = ?1 WHERE session_id = 'sess-old'",
                rusqlite::params![old],
            )
            .unwrap();
        }

        assert_eq!(
            store
                .delete_unpinned_session_events_older_than(cutoff)
                .unwrap(),
            1
        );
        assert!(store.get_event(&old_event.event_id).unwrap().is_none());
        assert!(store.get_event(&pinned_event.event_id).unwrap().is_some());
        assert!(store.get_event(&recent_event.event_id).unwrap().is_some());

        // The session still holds a pinned event, so it is retained: deleting
        // it would cascade-delete the pinned event.
        assert_eq!(
            store.delete_inactive_sessions_older_than(cutoff).unwrap(),
            0
        );
        assert!(store.get_session("sess-old").unwrap().is_some());
        assert!(store.get_event(&pinned_event.event_id).unwrap().is_some());

        store
            .upsert_source(&Source {
                source_id: "src-retention".into(),
                kind: "local".into(),
                display_name: "S".into(),
                enabled: true,
                adapter_config_json: "{}".into(),
                auth_ref: None,
                first_indexed_at: None,
                last_refresh_attempt_at: None,
                last_refresh_success_at: None,
                last_refresh_error: None,
                freshness_ttl_secs: None,
                upstream_revision: None,
                created_at: recent.into(),
                updated_at: recent.into(),
            })
            .unwrap();

        store
            .upsert_refresh_job(&RefreshJob {
                job_id: "job-old".into(),
                source_id: "src-retention".into(),
                trigger_kind: JobTrigger::Manual,
                status: JobStatus::Completed,
                started_at: Some(old.into()),
                finished_at: Some(old.into()),
                error: None,
                stats_json: "{}".into(),
                created_at: old.into(),
                updated_at: old.into(),
            })
            .unwrap();
        store
            .upsert_refresh_job(&RefreshJob {
                job_id: "job-active".into(),
                source_id: "src-retention".into(),
                trigger_kind: JobTrigger::Manual,
                status: JobStatus::Running,
                started_at: Some(old.into()),
                finished_at: None,
                error: None,
                stats_json: "{}".into(),
                created_at: old.into(),
                updated_at: old.into(),
            })
            .unwrap();

        assert_eq!(
            store
                .delete_finished_refresh_jobs_older_than(cutoff)
                .unwrap(),
            1
        );
        assert!(store.get_refresh_job("job-old").unwrap().is_none());
        assert!(store.get_refresh_job("job-active").unwrap().is_some());
    }

    #[test]
    fn rebuild_fts_index_is_atomic_on_failure() {
        let (store, _dir) = test_store();
        {
            let conn = store.conn().unwrap();
            conn.execute(
                "INSERT INTO chunks_fts (chunk_id, document_id, corpus_id, library_id, title, heading, text) \
                 VALUES ('chunk-keep', 'doc-1', 'corp-1', '', 't', '', 'still indexed')",
                [],
            )
            .unwrap();
            // Force the last INSERT to fail after the earlier DELETEs ran.
            conn.execute("DROP TABLE library_aliases_fts", []).unwrap();
        }

        assert!(store.rebuild_fts_index().is_err());

        let conn = store.conn().unwrap();
        let kept: i64 = conn
            .query_row("SELECT COUNT(*) FROM chunks_fts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(kept, 1, "failed rebuild must roll back the chunk DELETE");
    }

    fn artifact_row(artifact_id: &str, created_at: &str) -> Artifact {
        Artifact {
            artifact_id: artifact_id.into(),
            session_id: None,
            kind: "blob".into(),
            mime_type: None,
            storage_uri: format!("file:///{artifact_id}"),
            size_bytes: 1,
            summary: None,
            checksum: "sha256:00".into(),
            created_at: created_at.into(),
        }
    }

    #[test]
    fn retention_keeps_artifacts_referenced_by_pinned_events() {
        let (store, _dir) = test_store();
        let old = "2026-01-01T00:00:00Z";
        let cutoff = "2026-02-01T00:00:00Z";
        store
            .create_session(
                "sess-art",
                SessionTransport::Stdio,
                ContinuationMode::Ephemeral,
            )
            .unwrap();
        store
            .upsert_artifact(&artifact_row("art-pinned", old))
            .unwrap();
        store
            .upsert_artifact(&artifact_row("art-plain", old))
            .unwrap();
        store
            .add_event("sess-art", "note", "pinned", Some("art-pinned"), "{}", true)
            .unwrap();

        // Only the unreferenced artifact is eligible; the payload list gates
        // payload deletion too, so the pinned artifact is never offered.
        assert_eq!(
            store.list_artifacts_older_than(cutoff).unwrap(),
            vec!["art-plain".to_string()]
        );
        assert_eq!(store.delete_artifacts_older_than(cutoff).unwrap(), 1);
        let conn = store.conn().unwrap();
        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM artifacts WHERE artifact_id = 'art-pinned'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(remaining, 1, "pinned-event artifact must survive retention");
    }

    #[test]
    fn retention_keeps_sessions_with_pinned_events_and_their_events() {
        let (store, _dir) = test_store();
        let old = "2026-01-01T00:00:00Z";
        let cutoff = "2026-02-01T00:00:00Z";
        store
            .create_session(
                "sess-pinned",
                SessionTransport::Stdio,
                ContinuationMode::Ephemeral,
            )
            .unwrap();
        let pinned = store
            .add_event("sess-pinned", "decision", "keep me", None, "{}", true)
            .unwrap();
        {
            let conn = store.conn().unwrap();
            conn.execute(
                "UPDATE sessions SET status = 'closed', updated_at = ?1 WHERE session_id = 'sess-pinned'",
                rusqlite::params![old],
            )
            .unwrap();
        }

        assert_eq!(
            store.delete_inactive_sessions_older_than(cutoff).unwrap(),
            0
        );
        assert!(store.get_session("sess-pinned").unwrap().is_some());
        assert!(store.get_event(&pinned.event_id).unwrap().is_some());
    }

    #[test]
    fn upsert_source_and_list() {
        let (store, _dir) = test_store();
        let t = now();

        let src = Source {
            source_id: "src-1".into(),
            kind: "local".into(),
            display_name: "Test Source".into(),
            enabled: true,
            adapter_config_json: "{}".into(),
            auth_ref: None,
            first_indexed_at: None,
            last_refresh_attempt_at: None,
            last_refresh_success_at: None,
            last_refresh_error: None,
            freshness_ttl_secs: Some(3600),
            upstream_revision: Some("abc123".into()),
            created_at: t.clone(),
            updated_at: t,
        };

        store.upsert_source(&src).unwrap();

        let fetched = store.get_source("src-1").unwrap().unwrap();
        assert_eq!(fetched.source_id, "src-1");
        assert!(fetched.enabled);
        assert_eq!(fetched.freshness_ttl_secs, Some(3600));
        assert_eq!(fetched.upstream_revision, Some("abc123".into()));

        let listed = store.list_sources().unwrap();
        assert_eq!(listed.len(), 1);

        let mut src2 = src.clone();
        src2.source_id = "src-2".into();
        src2.enabled = false;
        store.upsert_source(&src2).unwrap();
        assert_eq!(store.list_sources().unwrap().len(), 2);

        let fetched2 = store.get_source("src-2").unwrap().unwrap();
        assert!(!fetched2.enabled);
    }

    #[test]
    fn upsert_document_and_replace_chunks() {
        let (store, _dir) = test_store();
        let t = now();

        store
            .upsert_source(&Source {
                source_id: "src-1".into(),
                kind: "local".into(),
                display_name: "S".into(),
                enabled: true,
                adapter_config_json: "{}".into(),
                auth_ref: None,
                first_indexed_at: None,
                last_refresh_attempt_at: None,
                last_refresh_success_at: None,
                last_refresh_error: None,
                freshness_ttl_secs: None,
                upstream_revision: None,
                created_at: t.clone(),
                updated_at: t.clone(),
            })
            .unwrap();

        store
            .upsert_corpus(&Corpus {
                corpus_id: "corp-1".into(),
                name: "C".into(),
                kind: "general".into(),
                visibility: "private".into(),
                created_at: t.clone(),
                updated_at: t.clone(),
            })
            .unwrap();

        let doc = Document {
            document_id: "doc-1".into(),
            corpus_id: "corp-1".into(),
            source_id: "src-1".into(),
            external_id: Some("ext-1".into()),
            library_id: None,
            title: "Test Doc".into(),
            uri: Some("file:///test.md".into()),
            version: None,
            mime_type: Some("text/markdown".into()),
            content_text: "hello world".into(),
            metadata_json: "{}".into(),
            checksum: "abc".into(),
            superseded_by_document_id: None,
            created_at: t.clone(),
            updated_at: t.clone(),
        };
        store.upsert_document(&doc).unwrap();

        let fetched = store.get_document("doc-1").unwrap().unwrap();
        assert_eq!(fetched.title, "Test Doc");
        assert_eq!(store.list_documents_by_source("src-1").unwrap().len(), 1);
        assert_eq!(store.list_documents_by_corpus("corp-1").unwrap().len(), 1);

        let chunks = vec![
            Chunk {
                chunk_id: "ch-1".into(),
                document_id: "doc-1".into(),
                corpus_id: "corp-1".into(),
                ordinal: 0,
                text: "hello".into(),
                token_count_estimate: 1,
                heading: None,
                anchor: None,
                section_path_json: "[]".into(),
                metadata_json: "{}".into(),
                checksum: "h1".into(),
                created_at: t.clone(),
                updated_at: t.clone(),
            },
            Chunk {
                chunk_id: "ch-2".into(),
                document_id: "doc-1".into(),
                corpus_id: "corp-1".into(),
                ordinal: 1,
                text: "world".into(),
                token_count_estimate: 1,
                heading: Some("Intro".into()),
                anchor: None,
                section_path_json: "[]".into(),
                metadata_json: "{}".into(),
                checksum: "h2".into(),
                created_at: t.clone(),
                updated_at: t.clone(),
            },
        ];
        store.replace_chunks("doc-1", &chunks).unwrap();

        let listed = store.list_chunks_by_document("doc-1").unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].text, "hello");
        assert_eq!(listed[1].heading, Some("Intro".into()));

        let by_corpus = store.list_chunks_by_corpus("corp-1").unwrap();
        assert_eq!(by_corpus.len(), 2);

        let new_chunks = vec![Chunk {
            chunk_id: "ch-3".into(),
            document_id: "doc-1".into(),
            corpus_id: "corp-1".into(),
            ordinal: 0,
            text: "replaced".into(),
            token_count_estimate: 1,
            heading: None,
            anchor: None,
            section_path_json: "[]".into(),
            metadata_json: "{}".into(),
            checksum: "h3".into(),
            created_at: t.clone(),
            updated_at: t,
        }];
        store.replace_chunks("doc-1", &new_chunks).unwrap();

        let after = store.list_chunks_by_document("doc-1").unwrap();
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].text, "replaced");

        store.delete_document("doc-1").unwrap();
        assert!(store.get_document("doc-1").unwrap().is_none());
        assert!(store.list_chunks_by_document("doc-1").unwrap().is_empty());
    }

    #[test]
    fn upsert_library_identity_and_alias() {
        let (store, _dir) = test_store();
        let t = now();

        let lib = LibraryIdentity {
            library_id: "lib-1".into(),
            provider: "context7".into(),
            name: "react".into(),
            version: Some("18.0.0".into()),
            namespace: None,
            language: Some("typescript".into()),
            summary: Some("React library".into()),
            trust_score: Some(0.95),
            metadata_json: "{}".into(),
            created_at: t.clone(),
            updated_at: t,
        };
        store.upsert_library_identity(&lib).unwrap();

        let fetched = store.get_library_identity("lib-1").unwrap().unwrap();
        assert_eq!(fetched.name, "react");
        assert_eq!(fetched.trust_score, Some(0.95));

        let listed = store.list_library_identities().unwrap();
        assert_eq!(listed.len(), 1);

        let alias = LibraryAlias {
            alias: "react".into(),
            library_id: "lib-1".into(),
            alias_kind: "primary".into(),
        };
        store.upsert_library_alias(&alias).unwrap();

        let aliases = store.list_library_aliases("lib-1").unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].alias, "react");
        assert_eq!(aliases[0].alias_kind, "primary");
    }

    #[test]
    fn upsert_refresh_job() {
        let (store, _dir) = test_store();
        let t = now();

        store
            .upsert_source(&Source {
                source_id: "src-rj".into(),
                kind: "local".into(),
                display_name: "S".into(),
                enabled: true,
                adapter_config_json: "{}".into(),
                auth_ref: None,
                first_indexed_at: None,
                last_refresh_attempt_at: None,
                last_refresh_success_at: None,
                last_refresh_error: None,
                freshness_ttl_secs: None,
                upstream_revision: None,
                created_at: t.clone(),
                updated_at: t.clone(),
            })
            .unwrap();

        let job = RefreshJob {
            job_id: "job-1".into(),
            source_id: "src-rj".into(),
            trigger_kind: JobTrigger::Manual,
            status: JobStatus::Running,
            started_at: Some(t.clone()),
            finished_at: None,
            error: None,
            stats_json: "{}".into(),
            created_at: t.clone(),
            updated_at: t,
        };
        store.upsert_refresh_job(&job).unwrap();

        let fetched = store.get_refresh_job("job-1").unwrap().unwrap();
        assert_eq!(fetched.job_id, "job-1");
        assert_eq!(fetched.trigger_kind, JobTrigger::Manual);
        assert_eq!(fetched.status, JobStatus::Running);

        let listed = store.list_refresh_jobs_by_source("src-rj").unwrap();
        assert_eq!(listed.len(), 1);
    }

    #[test]
    fn claim_next_refresh_job_for_source_is_single_use() {
        let (store, _dir) = test_store();
        let t = now();

        store
            .upsert_source(&Source {
                source_id: "src-claim".into(),
                kind: "local".into(),
                display_name: "S".into(),
                enabled: true,
                adapter_config_json: "{}".into(),
                auth_ref: None,
                first_indexed_at: None,
                last_refresh_attempt_at: None,
                last_refresh_success_at: None,
                last_refresh_error: None,
                freshness_ttl_secs: None,
                upstream_revision: None,
                created_at: t.clone(),
                updated_at: t.clone(),
            })
            .unwrap();

        store
            .upsert_refresh_job(&RefreshJob {
                job_id: "job-claim".into(),
                source_id: "src-claim".into(),
                trigger_kind: JobTrigger::Manual,
                status: JobStatus::Queued,
                started_at: None,
                finished_at: None,
                error: None,
                stats_json: "{}".into(),
                created_at: t.clone(),
                updated_at: t,
            })
            .unwrap();

        let claimed = store
            .claim_next_refresh_job_for_source("src-claim")
            .unwrap()
            .unwrap();
        assert_eq!(claimed.job_id, "job-claim");
        assert_eq!(claimed.status, JobStatus::Running);
        assert!(claimed.started_at.is_some());
        assert!(store
            .claim_next_refresh_job_for_source("src-claim")
            .unwrap()
            .is_none());
    }

    #[test]
    fn insert_active_refresh_job_returns_existing_active_job() {
        let (store, _dir) = test_store();
        let t = now();

        store
            .upsert_source(&Source {
                source_id: "src-active".into(),
                kind: "local".into(),
                display_name: "S".into(),
                enabled: true,
                adapter_config_json: "{}".into(),
                auth_ref: None,
                first_indexed_at: None,
                last_refresh_attempt_at: None,
                last_refresh_success_at: None,
                last_refresh_error: None,
                freshness_ttl_secs: None,
                upstream_revision: None,
                created_at: t.clone(),
                updated_at: t.clone(),
            })
            .unwrap();

        let first = RefreshJob {
            job_id: "job-active-1".into(),
            source_id: "src-active".into(),
            trigger_kind: JobTrigger::Manual,
            status: JobStatus::Queued,
            started_at: None,
            finished_at: None,
            error: None,
            stats_json: "{}".into(),
            created_at: t.clone(),
            updated_at: t.clone(),
        };
        let second = RefreshJob {
            job_id: "job-active-2".into(),
            created_at: now(),
            updated_at: now(),
            ..first.clone()
        };

        let inserted = store.insert_active_refresh_job(&first).unwrap();
        let coalesced = store.insert_active_refresh_job(&second).unwrap();

        assert_eq!(inserted.job_id, "job-active-1");
        assert_eq!(coalesced.job_id, "job-active-1");
        assert_eq!(
            store
                .list_refresh_jobs_by_source("src-active")
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn enum_from_rejects_unknown_strings() {
        assert!(transport_from("invalid").is_err());
        assert!(cont_mode_from("invalid").is_err());
        assert!(sess_status_from("invalid").is_err());
        assert!(trigger_from("invalid").is_err());
        assert!(job_status_from("invalid").is_err());

        assert!(transport_from("stdio").is_ok());
        assert!(cont_mode_from("ephemeral").is_ok());
        assert!(sess_status_from("active").is_ok());
        assert!(trigger_from("manual").is_ok());
        assert!(job_status_from("completed").is_ok());
    }

    #[test]
    fn enum_roundtrips_all_writable_variants() {
        for v in [SessionTransport::Stdio, SessionTransport::Sse] {
            assert_eq!(transport_from(transport_str(v)).unwrap(), v);
        }
        for v in [
            ContinuationMode::Ephemeral,
            ContinuationMode::Resumeable,
            ContinuationMode::Pinned,
        ] {
            assert_eq!(cont_mode_from(cont_mode_str(v)).unwrap(), v);
        }
        for v in [
            SessionStatus::Active,
            SessionStatus::Closed,
            SessionStatus::Compacted,
        ] {
            assert_eq!(sess_status_from(sess_status_str(v)).unwrap(), v);
        }
        for v in [
            JobTrigger::Startup,
            JobTrigger::Manual,
            JobTrigger::Scheduled,
            JobTrigger::Webhook,
            JobTrigger::DependencyChange,
        ] {
            assert_eq!(trigger_from(trigger_str(v)).unwrap(), v);
        }
        for v in [
            JobStatus::Queued,
            JobStatus::Running,
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ] {
            assert_eq!(job_status_from(job_status_str(v)).unwrap(), v);
        }
    }

    /// The v1 document ID exactly as the pre-v2 code computed it.
    fn legacy_v1_document_id(source_id: &str, external_id: &str, uri: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        source_id.hash(&mut hasher);
        external_id.hash(&mut hasher);
        uri.hash(&mut hasher);
        format!("doc_{:016x}", hasher.finish())
    }

    /// Builds a v1 database with the pre-v2 code path: `SCHEMA_V1` only (what the
    /// old `migrate()` ran), then rows keyed by the legacy IDs.
    /// `docs` = (external_id, Option<uri>) under source "src-v1".
    fn build_v1_store(path: &Path, docs: &[(&str, Option<&str>)]) -> Vec<String> {
        let conn = Connection::open(path).unwrap();
        conn.execute_batch(SCHEMA_V1).unwrap();
        conn.execute_batch(
            "INSERT INTO corpora (corpus_id, name, kind, visibility, created_at, updated_at)
               VALUES ('c1', 'c1', 'docs', 'private', 'now', 'now');
             INSERT INTO sources (source_id, kind, display_name, adapter_config_json, created_at, updated_at)
               VALUES ('src-v1', 'filesystem', 'v1', '{}', 'now', 'now');",
        )
        .unwrap();
        let mut ids = Vec::new();
        for (external_id, uri) in docs {
            let old_id = legacy_v1_document_id("src-v1", external_id, uri.unwrap_or(external_id));
            conn.execute(
                "INSERT INTO documents (document_id, corpus_id, source_id, external_id, uri, title,
                   content_text, metadata_json, checksum, created_at, updated_at)
                 VALUES (?1, 'c1', 'src-v1', ?2, ?3, 't', 'body', '{}', 'ck', 'now', 'now')",
                rusqlite::params![old_id, external_id, uri],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO chunks (chunk_id, document_id, corpus_id, ordinal, text,
                   token_count_estimate, checksum, created_at, updated_at)
                 VALUES (?1, ?2, 'c1', 0, 'body text', 2, 'ck', 'now', 'now')",
                rusqlite::params![format!("chunk-{external_id}"), old_id],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO chunks_fts (chunk_id, document_id, corpus_id, library_id, title, heading, text)
                 VALUES (?1, ?2, 'c1', '', 't', '', 'body text')",
                rusqlite::params![format!("chunk-{external_id}"), old_id],
            )
            .unwrap();
            ids.push(old_id);
        }
        // Second document supersedes the first, so the reference must be re-keyed too.
        if ids.len() >= 2 {
            conn.execute(
                "UPDATE documents SET superseded_by_document_id = ?1 WHERE document_id = ?2",
                rusqlite::params![ids[1], ids[0]],
            )
            .unwrap();
        }
        ids
    }

    #[test]
    fn v1_store_is_rekeyed_to_stable_ids_by_migrate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1.db");
        let old = build_v1_store(
            &path,
            &[("a.md", Some("/docs/a.md")), ("b.md", Some("/docs/b.md"))],
        );
        let new_a = ck_core::entities::stable_document_id("src-v1", "a.md", "/docs/a.md");
        let new_b = ck_core::entities::stable_document_id("src-v1", "b.md", "/docs/b.md");

        SqliteStore::open(&path).unwrap().migrate().unwrap();

        let conn = Connection::open(&path).unwrap();
        let count = |sql: &str, id: &str| -> i64 {
            conn.query_row(sql, rusqlite::params![id], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(
            count(
                "SELECT COUNT(*) FROM documents WHERE document_id = ?1",
                &new_a
            ),
            1
        );
        assert_eq!(
            count(
                "SELECT COUNT(*) FROM documents WHERE document_id = ?1",
                &old[0]
            ),
            0
        );
        assert_eq!(
            count("SELECT COUNT(*) FROM chunks WHERE document_id = ?1", &new_a),
            1
        );
        assert_eq!(
            count(
                "SELECT COUNT(*) FROM chunks WHERE document_id = ?1",
                &old[0]
            ),
            0
        );
        assert_eq!(
            count(
                "SELECT COUNT(*) FROM chunks_fts WHERE document_id = ?1",
                &new_a
            ),
            1
        );
        assert_eq!(
            count(
                "SELECT COUNT(*) FROM chunks_fts WHERE document_id = ?1",
                &old[0]
            ),
            0
        );
        let superseded: String = conn
            .query_row(
                "SELECT superseded_by_document_id FROM documents WHERE document_id = ?1",
                rusqlite::params![new_a],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(superseded, new_b);
        let versions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 2",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(versions, 1);

        // Idempotent: a second open/migrate changes nothing and does not fail.
        SqliteStore::open(&path).unwrap().migrate().unwrap();
        let docs: i64 = conn
            .query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))
            .unwrap();
        assert_eq!(docs, 2);
    }

    #[test]
    fn v1_store_with_unrecoverable_document_is_refused_with_reingest_message() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("v1-null-uri.db");
        let old = build_v1_store(&path, &[("c.md", None)]);

        let err = SqliteStore::open(&path)
            .unwrap()
            .migrate()
            .unwrap_err()
            .to_string();
        assert!(err.contains("Re-ingest required"), "got: {err}");

        // Refused without rekeying anything or recording v2.
        let conn = Connection::open(&path).unwrap();
        let still_old: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM documents WHERE document_id = ?1",
                rusqlite::params![old[0]],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(still_old, 1);
        let v2: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 2",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(v2, 0);
    }

    #[test]
    fn fresh_store_records_v1_and_v2() {
        let (store, _dir) = test_store();
        store.migrate().unwrap();
        let conn = Connection::open(store.path()).unwrap();
        let versions: Vec<i64> = conn
            .prepare("SELECT version FROM schema_migrations ORDER BY version")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(versions, vec![1, 2]);
    }
}
