use anyhow::Result;

use crate::entities::{RecalledEvent, ResolvedCandidate, RetrievalResult, ScoredItem};
use crate::types::QueryIntent;

/// Context for ranking operations.
#[derive(Debug, Clone, Default)]
pub struct RankContext {
    /// The query intent (resolve_library, query_docs, search_refs, recall_context, general).
    pub intent: QueryIntent,
    /// Optional session ID for session affinity scoring.
    pub session_id: Option<String>,
    /// Whether to include detailed score breakdowns (debug mode).
    pub debug: bool,
}

/// Port for ranking and combining retrieval signals.
pub trait RankerPort: Send + Sync {
    /// Rank resolution candidates for resolve_reference.
    fn rank_resolution_candidates(
        &self,
        query: &str,
        candidates: Vec<ScoredItem>,
        context: RankContext,
    ) -> Result<Vec<ResolvedCandidate>>;

    /// Rank retrieval results for query_reference.
    fn rank_retrieval_results(
        &self,
        query: &str,
        items: Vec<ScoredItem>,
        context: RankContext,
    ) -> Result<Vec<RetrievalResult>>;

    /// Rank recalled events for recall_context.
    fn rank_recall_results(
        &self,
        query: &str,
        items: Vec<ScoredItem>,
        context: RankContext,
    ) -> Result<Vec<RecalledEvent>>;
}
