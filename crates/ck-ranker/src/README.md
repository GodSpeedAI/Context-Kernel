# ck-ranker Source

## Overview
This crate implements the hybrid lexical/semantic ranking stage of Context Kernel as an adapter over the `RankerPort` defined in `ck-core`. It takes pre-scored candidates (`ScoredItem`) from retrieval and recall pipelines, re-scores them with an intent-weighted combination of FTS, trust, freshness, semantic, session-affinity, and alias-exactness signals, and returns the typed result entities (`ResolvedCandidate`, `RetrievalResult`, `RecalledEvent`). The crate is stateless: `LexicalRanker` is a unit struct and all scoring is a pure function of the inputs. It is composed into the runtime by `ck-bin` (`use ck_ranker::LexicalRanker` in `crates/ck-bin/src/main.rs`).

## Interface
- [lib.rs:LexicalRanker] - Public unit struct implementing `ck_core::ports::RankerPort`; constructed with [lib.rs:LexicalRanker::new] (and `Default`).
- [lib.rs:LexicalRanker::rank_resolution_candidates] - Re-scores `Vec<ScoredItem>` into `Vec<ResolvedCandidate>` sorted descending by combined score; provider/name fields are placeholders (`"unknown"`, the item id).
- [lib.rs:LexicalRanker::rank_retrieval_results] - Re-scores `Vec<ScoredItem>` into `Vec<RetrievalResult>` sorted descending by combined score, with per-document de-duplication; each result gets a `res_{ULID}` id.
- [lib.rs:LexicalRanker::rank_recall_results] - Re-scores `Vec<ScoredItem>` into `Vec<RecalledEvent>` sorted descending by combined score.
- Private scoring machinery (not public API, listed for contract clarity): [lib.rs:WeightProfile::for_intent] selects weights per `QueryIntent`; [lib.rs:LexicalRanker::combine_score] computes the weighted combination and the score breakdown JSON; [lib.rs:LexicalRanker::compute_session_affinity] computes the session-affinity signal.

## Invariants
- Intent-weighted scoring: [lib.rs:WeightProfile::for_intent] maps each `QueryIntent` variant (`ResolveLibrary`, `QueryDocs`, `SearchRefs`, `RecallContext`, `General`) to a fixed weight profile over `fts`, `trust`, `freshness`, `semantic`, `session_affinity`, `alias_exact`; each profile's weights sum to 1.0. Changing intents or weights changes ranking order.
- FTS normalization: [lib.rs:LexicalRanker::combine_score] squashes the raw FTS score through a sigmoid `1 / (1 + exp(-fts))` before weighting, so the combined score stays bounded regardless of raw BM25 magnitude. Missing breakdown components contribute 0.0.
- Session affinity: [lib.rs:LexicalRanker::compute_session_affinity] yields 1.0 only when the item breakdown's `session_id` equals `RankContext.session_id`, and 0.0 otherwise (including when the context has no session).
- De-duplication rule: [lib.rs:LexicalRanker::rank_retrieval_results] keeps only the first (highest-scored, post-sort) item per `document_id` found in the score breakdown; items whose breakdown has no `document_id` are never de-duplicated.
- Debug breakdown contract: when `RankContext.debug` is set, the returned breakdown contains every component plus the applied weight map; otherwise it is compact - `combined` plus whichever of `fts`, `trust_score`, `freshness`, `semantic`, `calibration` were present in the input breakdown ([lib.rs:LexicalRanker::combine_score]).
- Ordering: results are sorted descending by combined score using `partial_cmp`, so NaN scores compare as equal rather than panicking ([lib.rs:LexicalRanker::rank_retrieval_results] and siblings).

## Side Effects
- Emits `tracing` spans (via `#[instrument]`) with `candidate_count` / `item_count` fields for each ranking call ([lib.rs:LexicalRanker::rank_resolution_candidates], [lib.rs:LexicalRanker::rank_retrieval_results], [lib.rs:LexicalRanker::rank_recall_results]).
- Generates time/random-based ULID identifiers for retrieval result ids (`res_{ULID}`) in [lib.rs:LexicalRanker::rank_retrieval_results].
- No filesystem, network, database, lock, thread, or global-state side effects.

## Verification
- Run: `cargo test -p ck-ranker`
