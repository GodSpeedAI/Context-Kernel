use anyhow::Result;
use ck_core::entities::{RecalledEvent, ResolvedCandidate, RetrievalResult, ScoredItem};
use ck_core::ports::RankerPort;
use tracing::instrument;

/// Weight constants for hybrid scoring.
const FTS_WEIGHT: f64 = 0.7;
const TRUST_WEIGHT: f64 = 0.2;
const FRESHNESS_WEIGHT: f64 = 0.1;

pub struct LexicalRanker;

impl LexicalRanker {
    pub fn new() -> Self {
        Self
    }

    /// Combine FTS score with optional trust and freshness priors from score_breakdown.
    /// The breakdown may contain: "fts", "trust_score", "freshness".
    fn combine_score(breakdown: &serde_json::Value, fts_score: f64) -> (f64, serde_json::Value) {
        let normalized_fts = 1.0 / (1.0 + (-fts_score).exp());
        let trust = breakdown
            .get("trust_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let freshness = breakdown
            .get("freshness")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let combined =
            FTS_WEIGHT * normalized_fts + TRUST_WEIGHT * trust + FRESHNESS_WEIGHT * freshness;

        let new_breakdown = serde_json::json!({
            "fts": fts_score,
            "normalized_fts": normalized_fts,
            "trust": trust,
            "freshness": freshness,
            "combined": combined,
        });

        (combined, new_breakdown)
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
        candidates: Vec<ScoredItem>,
    ) -> Result<Vec<ResolvedCandidate>> {
        // Enrich scores with trust/freshness if available
        let mut enriched: Vec<_> = candidates
            .into_iter()
            .map(|item| {
                let (combined, breakdown) = Self::combine_score(&item.score_breakdown, item.score);
                (item, combined, breakdown)
            })
            .collect();

        enriched.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results = enriched
            .into_iter()
            .map(|(item, combined, breakdown)| ResolvedCandidate {
                canonical_id: item.id.clone(),
                provider: "unknown".to_string(),
                name: item.id,
                version: None,
                aliases: vec![],
                summary: None,
                trust_score: None,
                score: combined,
                score_breakdown: breakdown,
            })
            .collect();

        Ok(results)
    }

    #[instrument(skip(self, items), fields(item_count = items.len()))]
    fn rank_retrieval_results(
        &self,
        _query: &str,
        items: Vec<ScoredItem>,
    ) -> Result<Vec<RetrievalResult>> {
        let mut enriched: Vec<_> = items
            .into_iter()
            .map(|item| {
                let (combined, breakdown) = Self::combine_score(&item.score_breakdown, item.score);
                (item, combined, breakdown)
            })
            .collect();

        enriched.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results = enriched
            .into_iter()
            .map(|(item, combined, breakdown)| {
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
                    score: combined,
                    score_breakdown: breakdown,
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

    fn scored_item_with_trust(id: &str, score: f64, trust: f64) -> ScoredItem {
        ScoredItem {
            id: id.to_string(),
            score,
            score_breakdown: serde_json::json!({ "trust_score": trust }),
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
    }

    #[test]
    fn trust_score_influences_ranking() {
        let ranker = LexicalRanker::new();
        // lib_a has higher FTS but no trust; lib_b has lower FTS but high trust
        let candidates = vec![
            scored_item("lib_a", 10.0),
            scored_item_with_trust("lib_b", 8.0, 0.95),
        ];

        let results = ranker
            .rank_resolution_candidates("test query", candidates)
            .unwrap();

        // FTS is bounded, so strong trust can now influence the final order.
        assert_eq!(results[0].canonical_id, "lib_b");
        assert!(results[0].score_breakdown["trust"].as_f64().unwrap() > 0.0);
        assert!(results[1].score_breakdown["trust"].as_f64().unwrap() == 0.0);
    }

    #[test]
    fn fts_score_is_normalized_before_combining() {
        let ranker = LexicalRanker::new();
        let candidates = vec![scored_item("lib_a", 1000.0)];

        let results = ranker
            .rank_resolution_candidates("test query", candidates)
            .unwrap();

        let breakdown = &results[0].score_breakdown;
        assert_eq!(breakdown["fts"].as_f64().unwrap(), 1000.0);
        let normalized = breakdown["normalized_fts"].as_f64().unwrap();
        assert!((0.0..=1.0).contains(&normalized));
        assert!(results[0].score <= 1.0);
    }

    #[test]
    fn rank_retrieval_results_returns_proper_ids() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item("chunk_1", 0.7), scored_item("chunk_2", 0.4)];

        let results = ranker.rank_retrieval_results("test query", items).unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].result_id.starts_with("res_"));
        assert_eq!(results[0].chunk_id.as_deref(), Some("chunk_1"));
        assert!(results[0].score_breakdown["combined"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn rank_recall_results_returns_events() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item("evt_1", 0.8), scored_item("evt_2", 0.6)];

        let results = ranker.rank_recall_results("test query", items).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].event_id, "evt_1");
        assert_eq!(results[1].event_id, "evt_2");
    }
}
