use anyhow::Result;

use crate::entities::{Session, SessionEvent, StoredContext};
use crate::types::{ContinuationMode, SessionTransport};

/// Port for session memory persistence and recall.
pub trait SessionMemoryPort: Send + Sync {
    fn create_session(
        &self,
        session_id: &str,
        transport: SessionTransport,
        continuation_mode: ContinuationMode,
    ) -> Result<Session>;

    /// Implementations MUST make `create_session` idempotent (e.g. INSERT OR
    /// IGNORE / upsert with a UNIQUE constraint on `session_id`) so that
    /// `ensure_session` does not race under concurrent callers.
    fn get_session(&self, session_id: &str) -> Result<Option<Session>>;

    fn ensure_session(
        &self,
        session_id: &str,
        transport: SessionTransport,
        continuation_mode: ContinuationMode,
    ) -> Result<Session> {
        if let Some(session) = self.get_session(session_id)? {
            Ok(session)
        } else {
            self.create_session(session_id, transport, continuation_mode)
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
    ) -> Result<StoredContext>;

    fn get_event(&self, event_id: &str) -> Result<Option<SessionEvent>>;

    fn list_events_by_session(&self, session_id: &str) -> Result<Vec<SessionEvent>>;

    fn count_active_sessions(&self) -> Result<u64>;
}
