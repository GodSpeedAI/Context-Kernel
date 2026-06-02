use anyhow::Result;

use crate::entities::{ResolvedCandidate, RetrievalResult, ScoredItem};

/// Port for ranking and combining retrieval signals.
pub trait RankerPort: Send + Sync {
    /// Rank resolution candidates for resolve_reference.
    fn rank_resolution_candidates(
        &self,
        query: &str,
        candidates: Vec<ScoredItem>,
    ) -> Result<Vec<ResolvedCandidate>>;

    /// Rank retrieval results for query_reference.
    fn rank_retrieval_results(
        &self,
        query: &str,
        items: Vec<ScoredItem>,
    ) -> Result<Vec<RetrievalResult>>;

    /// Rank recalled events for recall_context.
    fn rank_recall_results(
        &self,
        query: &str,
        items: Vec<ScoredItem>,
    ) -> Result<Vec<crate::entities::RecalledEvent>>;
}
