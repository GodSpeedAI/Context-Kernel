use anyhow::Result;
use ck_core::entities::{RecalledEvent, ResolvedCandidate, RetrievalResult, ScoredItem};
use ck_core::ports::RankerPort;
use tracing::instrument;

pub struct LexicalRanker;

impl LexicalRanker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LexicalRanker {
    fn default() -> Self {
        Self::new()
    }
}

impl RankerPort for LexicalRanker {
    #[instrument(skip(self, candidates), fields(candidate_count = candidates.len()))]
    fn rank_resolution_candidates(
        &self,
        _query: &str,
        mut candidates: Vec<ScoredItem>,
    ) -> Result<Vec<ResolvedCandidate>> {
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let results = candidates
            .into_iter()
            .map(|item| ResolvedCandidate {
                canonical_id: item.id.clone(),
                provider: "unknown".to_string(),
                name: item.id,
                version: None,
                aliases: vec![],
                summary: None,
                trust_score: None,
                score: item.score,
                score_breakdown: item.score_breakdown,
            })
            .collect();

        Ok(results)
    }

    #[instrument(skip(self, items), fields(item_count = items.len()))]
    fn rank_retrieval_results(
        &self,
        _query: &str,
        mut items: Vec<ScoredItem>,
    ) -> Result<Vec<RetrievalResult>> {
        items.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let results = items
            .into_iter()
            .map(|item| {
                let result_id = format!("res_{}", ulid::Ulid::new());
                RetrievalResult {
                    result_id,
                    canonical_id: None,
                    document_id: None,
                    chunk_id: Some(item.id),
                    title: None,
                    heading: None,
                    excerpt: String::new(),
                    citation: serde_json::Value::Object(serde_json::Map::new()),
                    score: item.score,
                    score_breakdown: item.score_breakdown,
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                }
            })
            .collect();

        Ok(results)
    }

    #[instrument(skip(self, items), fields(item_count = items.len()))]
    fn rank_recall_results(
        &self,
        _query: &str,
        mut items: Vec<ScoredItem>,
    ) -> Result<Vec<RecalledEvent>> {
        items.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let results = items
            .into_iter()
            .map(|item| RecalledEvent {
                event_id: item.id,
                kind: "unknown".to_string(),
                text: String::new(),
                artifact_id: None,
                score: item.score,
                score_breakdown: item.score_breakdown,
                created_at: String::new(),
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ck_core::entities::ScoredItem;

    fn scored_item(id: &str, score: f64) -> ScoredItem {
        ScoredItem {
            id: id.to_string(),
            score,
            score_breakdown: serde_json::json!({}),
        }
    }

    #[test]
    fn rank_resolution_candidates_returns_sorted_by_score() {
        let ranker = LexicalRanker::new();
        let candidates = vec![
            scored_item("lib_a", 0.5),
            scored_item("lib_b", 0.9),
            scored_item("lib_c", 0.3),
        ];

        let results = ranker
            .rank_resolution_candidates("test query", candidates)
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].canonical_id, "lib_b");
        assert_eq!(results[1].canonical_id, "lib_a");
        assert_eq!(results[2].canonical_id, "lib_c");
        assert!((results[0].score - 0.9).abs() < f64::EPSILON);
        assert_eq!(results[0].provider, "unknown");
        assert!(results[0].version.is_none());
        assert!(results[0].aliases.is_empty());
    }

    #[test]
    fn rank_retrieval_results_returns_proper_ids() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item("chunk_1", 0.7), scored_item("chunk_2", 0.4)];

        let results = ranker.rank_retrieval_results("test query", items).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].result_id.starts_with("res_"));
        assert_eq!(results[0].chunk_id.as_deref(), Some("chunk_1"));
        assert_eq!(results[1].chunk_id.as_deref(), Some("chunk_2"));
        assert!((results[0].score - 0.7).abs() < f64::EPSILON);
        assert!(results[0].excerpt.is_empty());
        assert!(results[0].citation.is_object());
        assert!(results[0].metadata.is_object());
    }

    #[test]
    fn rank_recall_results_returns_events() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item("evt_1", 0.8), scored_item("evt_2", 0.6)];

        let results = ranker.rank_recall_results("test query", items).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].event_id, "evt_1");
        assert_eq!(results[1].event_id, "evt_2");
        assert_eq!(results[0].kind, "unknown");
        assert!(results[0].text.is_empty());
        assert!((results[0].score - 0.8).abs() < f64::EPSILON);
        assert!(results[0].artifact_id.is_none());
        assert!(results[0].created_at.is_empty());
    }
}
