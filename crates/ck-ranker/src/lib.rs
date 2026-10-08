use anyhow::Result;
use ck_core::entities::{RecalledEvent, ResolvedCandidate, RetrievalResult, ScoredItem};
use ck_core::ports::{RankContext, RankerPort};
use ck_core::types::QueryIntent;
use std::collections::HashMap;
use tracing::instrument;

struct WeightProfile {
    fts: f64,
    trust: f64,
    freshness: f64,
    semantic: f64,
    session_affinity: f64,
    alias_exact: f64,
}

impl WeightProfile {
    fn for_intent(intent: &QueryIntent) -> Self {
        match intent {
            QueryIntent::ResolveLibrary => WeightProfile {
                fts: 0.5,
                trust: 0.3,
                freshness: 0.1,
                semantic: 0.0,
                session_affinity: 0.0,
                alias_exact: 0.1,
            },
            QueryIntent::QueryDocs => WeightProfile {
                fts: 0.6,
                trust: 0.15,
                freshness: 0.15,
                semantic: 0.1,
                session_affinity: 0.0,
                alias_exact: 0.0,
            },
            QueryIntent::SearchRefs => WeightProfile {
                fts: 0.5,
                trust: 0.1,
                freshness: 0.2,
                semantic: 0.2,
                session_affinity: 0.0,
                alias_exact: 0.0,
            },
            QueryIntent::RecallContext => WeightProfile {
                fts: 0.4,
                trust: 0.1,
                freshness: 0.2,
                semantic: 0.0,
                session_affinity: 0.3,
                alias_exact: 0.0,
            },
            QueryIntent::General => WeightProfile {
                fts: 0.7,
                trust: 0.2,
                freshness: 0.1,
                semantic: 0.0,
                session_affinity: 0.0,
                alias_exact: 0.0,
            },
        }
    }
}

pub struct LexicalRanker;

impl LexicalRanker {
    pub fn new() -> Self {
        Self
    }

    fn extract_f64(breakdown: &serde_json::Value, key: &str) -> f64 {
        breakdown.get(key).and_then(|v| v.as_f64()).unwrap_or(0.0)
    }

    fn extract_string(breakdown: &serde_json::Value, key: &str) -> Option<String> {
        breakdown
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    fn compute_session_affinity(
        breakdown: &serde_json::Value,
        context_session_id: &Option<String>,
    ) -> f64 {
        match context_session_id {
            Some(sid) => {
                let item_session = Self::extract_string(breakdown, "session_id");
                if item_session.as_deref() == Some(sid.as_str()) {
                    1.0
                } else {
                    0.0
                }
            }
            None => 0.0,
        }
    }

    fn combine_score(
        breakdown: &serde_json::Value,
        fts_score: f64,
        weights: &WeightProfile,
        context: &RankContext,
    ) -> (f64, serde_json::Value) {
        let normalized_fts = 1.0 / (1.0 + (-fts_score).exp());
        let trust = Self::extract_f64(breakdown, "trust_score");
        let freshness = Self::extract_f64(breakdown, "freshness");
        let semantic = Self::extract_f64(breakdown, "semantic");
        let alias_exact = Self::extract_f64(breakdown, "alias_exact");
        let session_affinity = Self::compute_session_affinity(breakdown, &context.session_id);

        let combined = weights.fts * normalized_fts
            + weights.trust * trust
            + weights.freshness * freshness
            + weights.semantic * semantic
            + weights.session_affinity * session_affinity
            + weights.alias_exact * alias_exact;

        if context.debug {
            let debug_breakdown = serde_json::json!({
                "fts": fts_score,
                "normalized_fts": normalized_fts,
                "trust": trust,
                "freshness": freshness,
                "semantic": semantic,
                "alias_exact": alias_exact,
                "session_affinity": session_affinity,
                "weights": {
                    "fts": weights.fts,
                    "trust": weights.trust,
                    "freshness": weights.freshness,
                    "semantic": weights.semantic,
                    "session_affinity": weights.session_affinity,
                    "alias_exact": weights.alias_exact,
                },
                "combined": combined,
            });
            (combined, debug_breakdown)
        } else {
            // Compact breakdown: the combined score plus the component
            // evaluations that actually participated, so downstream (and the
            // context bundle projection) can see WHY this result ranked.
            let mut compact = serde_json::Map::new();
            compact.insert("combined".to_string(), serde_json::json!(combined));
            for key in ["fts", "trust_score", "freshness", "semantic", "calibration"] {
                if let Some(v) = breakdown.get(key) {
                    compact.insert(key.to_string(), v.clone());
                }
            }
            (combined, serde_json::Value::Object(compact))
        }
    }
}

impl Default for LexicalRanker {
    fn default() -> Self {
        Self::new()
    }
}

impl RankerPort for LexicalRanker {
    #[instrument(skip(self, candidates, context), fields(candidate_count = candidates.len()))]
    fn rank_resolution_candidates(
        &self,
        _query: &str,
        candidates: Vec<ScoredItem>,
        context: RankContext,
    ) -> Result<Vec<ResolvedCandidate>> {
        let weights = WeightProfile::for_intent(&context.intent);

        let mut enriched: Vec<_> = candidates
            .into_iter()
            .map(|item| {
                let (combined, breakdown) =
                    Self::combine_score(&item.score_breakdown, item.score, &weights, &context);
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

    #[instrument(skip(self, items, context), fields(item_count = items.len()))]
    fn rank_retrieval_results(
        &self,
        _query: &str,
        items: Vec<ScoredItem>,
        context: RankContext,
    ) -> Result<Vec<RetrievalResult>> {
        let weights = WeightProfile::for_intent(&context.intent);

        let mut enriched: Vec<_> = items
            .into_iter()
            .map(|item| {
                let doc_id = Self::extract_string(&item.score_breakdown, "document_id");
                let (combined, breakdown) =
                    Self::combine_score(&item.score_breakdown, item.score, &weights, &context);
                (item, combined, breakdown, doc_id)
            })
            .collect();

        enriched.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut seen_docs: HashMap<String, usize> = HashMap::new();
        let mut results: Vec<RetrievalResult> = Vec::new();

        for (item, combined, breakdown, doc_id) in enriched {
            if let Some(ref did) = doc_id {
                if seen_docs.contains_key(did) {
                    continue;
                }
                seen_docs.insert(did.clone(), results.len());
            }

            let result_id = format!("res_{}", ulid::Ulid::new());
            results.push(RetrievalResult {
                result_id,
                canonical_id: None,
                document_id: doc_id,
                chunk_id: Some(item.id),
                title: None,
                heading: None,
                excerpt: String::new(),
                citation: serde_json::Value::Object(serde_json::Map::new()),
                score: combined,
                score_breakdown: breakdown,
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            });
        }

        Ok(results)
    }

    #[instrument(skip(self, items, context), fields(item_count = items.len()))]
    fn rank_recall_results(
        &self,
        _query: &str,
        items: Vec<ScoredItem>,
        context: RankContext,
    ) -> Result<Vec<RecalledEvent>> {
        let weights = WeightProfile::for_intent(&context.intent);

        let mut enriched: Vec<_> = items
            .into_iter()
            .map(|item| {
                let (combined, breakdown) =
                    Self::combine_score(&item.score_breakdown, item.score, &weights, &context);
                (item, combined, breakdown)
            })
            .collect();

        enriched.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let results = enriched
            .into_iter()
            .map(|(item, combined, breakdown)| RecalledEvent {
                event_id: item.id,
                kind: "unknown".to_string(),
                text: String::new(),
                artifact_id: None,
                score: combined,
                score_breakdown: breakdown,
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
    use ck_core::ports::RankContext;
    use ck_core::types::QueryIntent;

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

    fn scored_item_with_session(id: &str, score: f64, session_id: &str) -> ScoredItem {
        ScoredItem {
            id: id.to_string(),
            score,
            score_breakdown: serde_json::json!({ "session_id": session_id }),
        }
    }

    fn scored_item_with_doc(id: &str, score: f64, doc_id: &str) -> ScoredItem {
        ScoredItem {
            id: id.to_string(),
            score,
            score_breakdown: serde_json::json!({ "document_id": doc_id }),
        }
    }

    fn scored_item_with_breakdown(
        id: &str,
        score: f64,
        breakdown: serde_json::Value,
    ) -> ScoredItem {
        ScoredItem {
            id: id.to_string(),
            score,
            score_breakdown: breakdown,
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
            .rank_resolution_candidates("test query", candidates, RankContext::default())
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].canonical_id, "lib_b");
        assert_eq!(results[1].canonical_id, "lib_a");
        assert_eq!(results[2].canonical_id, "lib_c");
    }

    #[test]
    fn trust_score_influences_ranking() {
        let ranker = LexicalRanker::new();
        let candidates = vec![
            scored_item("lib_a", 10.0),
            scored_item_with_trust("lib_b", 8.0, 0.95),
        ];

        let results = ranker
            .rank_resolution_candidates("test query", candidates, RankContext::default())
            .unwrap();

        assert_eq!(results[0].canonical_id, "lib_b");
    }

    #[test]
    fn fts_score_is_normalized_before_combining() {
        let ranker = LexicalRanker::new();
        let candidates = vec![scored_item("lib_a", 1000.0)];

        let ctx = RankContext {
            debug: true,
            ..Default::default()
        };
        let results = ranker
            .rank_resolution_candidates("test query", candidates, ctx)
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

        let results = ranker
            .rank_retrieval_results("test query", items, RankContext::default())
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results[0].result_id.starts_with("res_"));
        assert_eq!(results[0].chunk_id.as_deref(), Some("chunk_1"));
        assert!(results[0].score_breakdown["combined"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn rank_recall_results_returns_events() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item("evt_1", 0.8), scored_item("evt_2", 0.6)];

        let results = ranker
            .rank_recall_results("test query", items, RankContext::default())
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].event_id, "evt_1");
        assert_eq!(results[1].event_id, "evt_2");
    }

    #[test]
    fn intent_weighting_changes_ranking_order() {
        let ranker = LexicalRanker::new();
        let candidates = vec![
            scored_item_with_breakdown(
                "lib_a",
                8.0,
                serde_json::json!({ "trust_score": 0.2, "alias_exact": 0.0 }),
            ),
            scored_item_with_breakdown(
                "lib_b",
                8.0,
                serde_json::json!({ "trust_score": 0.1, "alias_exact": 1.0 }),
            ),
        ];

        let general_ctx = RankContext {
            intent: QueryIntent::General,
            debug: true,
            ..Default::default()
        };
        let general_results = ranker
            .rank_resolution_candidates("test query", candidates.clone(), general_ctx)
            .unwrap();
        assert_eq!(general_results[0].canonical_id, "lib_a");

        let resolve_ctx = RankContext {
            intent: QueryIntent::ResolveLibrary,
            debug: true,
            ..Default::default()
        };
        let resolve_results = ranker
            .rank_resolution_candidates("test query", candidates, resolve_ctx)
            .unwrap();
        assert_eq!(resolve_results[0].canonical_id, "lib_b");
    }

    #[test]
    fn session_affinity_boosts_matching_session_items() {
        let ranker = LexicalRanker::new();
        let items = vec![
            scored_item("evt_no_session", 0.9),
            scored_item_with_session("evt_matching", 0.5, "sess_abc"),
        ];

        let ctx = RankContext {
            intent: QueryIntent::RecallContext,
            session_id: Some("sess_abc".to_string()),
            debug: true,
        };

        let results = ranker
            .rank_recall_results("test query", items, ctx)
            .unwrap();

        assert_eq!(results[0].event_id, "evt_matching");
        let affinity = results[0].score_breakdown["session_affinity"]
            .as_f64()
            .unwrap();
        assert!((affinity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn session_affinity_zero_when_no_context_session() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item_with_session("evt_1", 0.8, "sess_abc")];

        let ctx = RankContext {
            intent: QueryIntent::RecallContext,
            session_id: None,
            debug: true,
        };

        let results = ranker
            .rank_recall_results("test query", items, ctx)
            .unwrap();

        let affinity = results[0].score_breakdown["session_affinity"]
            .as_f64()
            .unwrap();
        assert!(affinity.abs() < f64::EPSILON);
    }

    #[test]
    fn deduplication_collapses_same_document_chunks() {
        let ranker = LexicalRanker::new();
        let items = vec![
            scored_item_with_doc("chunk_1", 0.9, "doc_a"),
            scored_item_with_doc("chunk_2", 0.7, "doc_a"),
            scored_item_with_doc("chunk_3", 0.5, "doc_b"),
        ];

        let results = ranker
            .rank_retrieval_results("test query", items, RankContext::default())
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].chunk_id.as_deref(), Some("chunk_1"));
        assert_eq!(results[1].chunk_id.as_deref(), Some("chunk_3"));
    }

    #[test]
    fn deduplication_keeps_highest_scored_chunk() {
        let ranker = LexicalRanker::new();
        let items = vec![
            scored_item_with_doc("chunk_low", 0.3, "doc_x"),
            scored_item_with_doc("chunk_high", 0.9, "doc_x"),
        ];

        let results = ranker
            .rank_retrieval_results("test query", items, RankContext::default())
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk_id.as_deref(), Some("chunk_high"));
    }

    #[test]
    fn debug_mode_includes_detailed_breakdown() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item_with_breakdown(
            "item_1",
            5.0,
            serde_json::json!({ "trust_score": 0.8, "freshness": 0.5, "semantic": 0.6 }),
        )];

        let ctx = RankContext {
            intent: QueryIntent::QueryDocs,
            debug: true,
            ..Default::default()
        };

        let results = ranker
            .rank_retrieval_results("test query", items, ctx)
            .unwrap();

        let bd = &results[0].score_breakdown;
        assert!(bd.get("fts").is_some());
        assert!(bd.get("normalized_fts").is_some());
        assert!(bd.get("trust").is_some());
        assert!(bd.get("freshness").is_some());
        assert!(bd.get("semantic").is_some());
        assert!(bd.get("combined").is_some());
        assert!(bd.get("weights").is_some());
    }

    #[test]
    fn non_debug_mode_shows_only_combined() {
        let ranker = LexicalRanker::new();
        let items = vec![scored_item_with_breakdown(
            "item_1",
            5.0,
            serde_json::json!({ "trust_score": 0.8, "freshness": 0.5 }),
        )];

        let ctx = RankContext {
            intent: QueryIntent::General,
            debug: false,
            ..Default::default()
        };

        let results = ranker
            .rank_retrieval_results("test query", items, ctx)
            .unwrap();

        let bd = &results[0].score_breakdown;
        assert!(bd.get("combined").is_some());
        assert!(bd.get("fts").is_none());
        assert!(bd.get("normalized_fts").is_none());
        assert!(bd.get("trust").is_none());
        assert!(bd.get("weights").is_none());
    }

    #[test]
    fn semantic_score_included_in_weighted_combination() {
        let ranker = LexicalRanker::new();
        let items = vec![
            scored_item_with_breakdown(
                "item_no_sem",
                5.0,
                serde_json::json!({ "trust_score": 0.5, "freshness": 0.5, "semantic": 0.0 }),
            ),
            scored_item_with_breakdown(
                "item_with_sem",
                5.0,
                serde_json::json!({ "trust_score": 0.5, "freshness": 0.5, "semantic": 1.0 }),
            ),
        ];

        let ctx = RankContext {
            intent: QueryIntent::SearchRefs,
            debug: true,
            ..Default::default()
        };

        let results = ranker
            .rank_retrieval_results("test query", items, ctx)
            .unwrap();

        assert_eq!(results[0].chunk_id.as_deref(), Some("item_with_sem"));
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn items_without_document_id_not_deduplicated() {
        let ranker = LexicalRanker::new();
        let items = vec![
            scored_item("chunk_1", 0.9),
            scored_item("chunk_2", 0.7),
            scored_item("chunk_3", 0.5),
        ];

        let results = ranker
            .rank_retrieval_results("test query", items, RankContext::default())
            .unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn mixed_document_id_and_no_document_id() {
        let ranker = LexicalRanker::new();
        let items = vec![
            scored_item_with_doc("chunk_1", 0.9, "doc_a"),
            scored_item("chunk_2", 0.8),
            scored_item_with_doc("chunk_3", 0.7, "doc_a"),
            scored_item("chunk_4", 0.6),
        ];

        let results = ranker
            .rank_retrieval_results("test query", items, RankContext::default())
            .unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].chunk_id.as_deref(), Some("chunk_1"));
    }
}
