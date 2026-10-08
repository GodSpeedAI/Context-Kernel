//! The typed Context Requirement and the real retrieval resolution behind it.
//!
//! Context Kernel answers: "What portion of the represented world is made
//! available to this observer for this bounded act?" — source/scope resolution,
//! retrieval, ranking, freshness/trust consideration, budgeting, and
//! completeness/omission accounting. It does NOT decide whether the
//! consequential action is authorized (SEA-Forge owns that) and it does NOT
//! turn retrieval outcomes into capability claims.
//!
//! The GodSpeed-facing adapter converts its boundary request into a
//! [`ContextRequirement`] and calls [`AppServices::resolve_context`]; there is
//! no second, degraded retrieval path.

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{AppServices, QueryReferenceRequest};
use ck_core::entities::{Budget, RetrievalResult};
use ulid::Ulid;
use ck_core::types::{
    NoContextReason, RetrievalCompleteness, OMISSION_MAX_RESULTS, OMISSION_REQUIRED_CONCEPT,
    OMISSION_STALE_SOURCE, OMISSION_TOKEN_BUDGET,
};

/// Identity of one bounded context demand.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequirementIdentity {
    /// The GodSpeed work request this context is for (CEP
    /// `extensions["godspeed.context_bundle"].work_request_ref`).
    pub work_request_ref: String,
    /// Stable id of this context requirement itself.
    pub context_requirement_id: String,
    /// Task/route identity where the caller has one.
    pub task_ref: Option<String>,
    pub route_ref: Option<String>,
}

/// Caller-declared data-access scope and exclusions for one requirement.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceConstraints {
    /// Scope the caller claims (from the request's `scope_claim`). A private
    /// corpus is only accessible when this explicitly contains it
    /// (POL-ACL-002: private corpus requires explicit scope).
    pub caller_scope: Vec<String>,
    /// Corpora/sources the caller excludes from this retrieval.
    pub exclusions: Vec<String>,
}

/// Freshness the caller is willing to accept. Absent = no freshness gate
/// (freshness is still *evaluated* and surfaced per result).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessRequirement {
    /// Maximum age (seconds) since the source's last successful refresh.
    pub max_age_secs: Option<u64>,
}

/// Trace/lineage identifiers supplied by the caller, carried through to the
/// boundary projection (correlation/causation).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceRefs {
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

/// A typed, first-class context demand. This is the native Context Kernel
/// input model for bounded retrieval — deliberately free of CEP structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextRequirement {
    pub identity: RequirementIdentity,
    /// Immutable DomainForge world reference
    /// (`world:<name>@sha256:<64 lowercase hex>`); validated at the boundary.
    pub world_ref: String,
    /// Why the caller wants context (purpose / retrieval intent).
    pub purpose: Option<String>,
    /// Free-text retrieval query; `None`/empty means "everything in the
    /// requested corpora, within budget".
    pub query: Option<String>,
    /// Retrieval mode: `lexical`, `semantic`, or `hybrid` (default).
    pub mode: Option<String>,
    /// Semantic concepts or distinctions that must be represented for this
    /// requirement to be satisfiable. Absent concepts become omissions —
    /// never silently dropped.
    pub required_concepts: Vec<String>,
    /// Requested corpora (empty = whole kernel scope).
    pub corpora: Vec<String>,
    pub constraints: SourceConstraints,
    pub freshness: FreshnessRequirement,
    /// Result/chunk cap for this acquisition.
    pub max_results: u32,
    /// Optional explicit retrieval budget (token/chunk ceilings).
    pub budget: Option<Budget>,
    /// Expected artifact/proof references, if the caller supplied any.
    pub expected_refs: Vec<String>,
    /// Risk classification supplied by the caller (informational here;
    /// authority classification belongs to SEA-Forge).
    pub risk_classification: Option<String>,
    pub trace: TraceRefs,
}

impl ContextRequirement {
    /// Effective result cap for this acquisition.
    pub fn effective_result_cap(&self) -> i64 {
        let mut cap = i64::from(self.max_results.max(1));
        if let Some(b) = &self.budget {
            if b.max_chunks > 0 {
                cap = cap.min(b.max_chunks);
            }
        }
        cap
    }
}

/// Context *access* result — data-access only. This is deliberately NOT an
/// action-authorization verdict: SEA-Forge decides whether consequential
/// actions are authorized; Context Kernel only decides whether requested
/// context is accessible to this caller under its declared scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "result")]
pub enum ContextAccess {
    /// The requested corpora are accessible to this caller.
    Accessible,
    /// The corpus exists but is out of this caller's declared scope.
    Inaccessible {
        corpus_id: String,
        reason: &'static str,
    },
    /// The corpus is private and the caller declared no scope: an explicit
    /// unresolved condition, never a silent grant or denial of action.
    RequiresScope { corpus_id: String },
}

impl ContextAccess {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Accessible => "accessible",
            Self::Inaccessible { .. } => "inaccessible",
            Self::RequiresScope { .. } => "requires_scope",
        }
    }
}


/// One machine-readable omission of a resolution (projected to CEP
/// `omissions` at the boundary).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalOmission {
    pub code: String,
    pub detail: Option<String>,
}

impl RetrievalOmission {
    pub fn code(code: &str) -> Self {
        Self {
            code: code.to_string(),
            detail: None,
        }
    }
    pub fn with_detail(code: &str, detail: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            detail: Some(detail.into()),
        }
    }
}

/// Requested vs consumed budget for one resolution.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetUsage {
    pub requested_results: i64,
    pub consumed_results: usize,
    pub requested_tokens: Option<i64>,
    pub consumed_tokens: i64,
    pub more_available: bool,
}

/// The bounded observer representation of one context demand: what was made
/// available (never "everything that exists").
#[derive(Debug, Clone)]
pub struct ContextResolution {
    pub query_id: String,
    pub mode: String,
    pub results: Vec<RetrievalResult>,
    pub access: ContextAccess,
    pub completeness: RetrievalCompleteness,
    pub omissions: Vec<RetrievalOmission>,
    /// Set exactly when `completeness == RetrievalCompleteness::None`.
    pub no_context_reason: Option<NoContextReason>,
    pub budget: BudgetUsage,
    /// Corpora actually searched (post scope/exclusion resolution).
    pub corpora: Vec<String>,
}

impl ContextResolution {
    /// True when at least one citation-worthy result was made available.
    pub fn has_context(&self) -> bool {
        !self.results.is_empty()
    }

    /// A blocked resolution: nothing retrieved, access explicit, reason
    /// machine-readable. Absence is never fabricated into context.
    pub fn blocked(
        query_id: String,
        mode: String,
        access: ContextAccess,
        reason: NoContextReason,
        corpora: Vec<String>,
        requested_results: i64,
        requested_tokens: Option<i64>,
    ) -> Self {
        Self {
            query_id,
            mode,
            results: Vec::new(),
            access,
            completeness: RetrievalCompleteness::None,
            omissions: vec![RetrievalOmission::code(reason.as_str())],
            no_context_reason: Some(reason),
            budget: BudgetUsage {
                requested_results,
                requested_tokens,
                ..BudgetUsage::default()
            },
            corpora,
        }
    }
}


impl AppServices {
    /// Corpus lookup — needed by boundary adapters for data-access decisions
    /// and by auto-ingest paths to avoid re-ingesting indexed corpora.
    pub fn corpus(&self, corpus_id: &str) -> Result<Option<ck_core::entities::Corpus>> {
        self.storage.get_corpus(corpus_id)
    }

    /// Resolve a typed context requirement through the real retrieval
    /// pipeline: lexical + semantic-when-configured retrieval, corpus
    /// filtering, private-source scope enforcement (POL-ACL-002), ranking,
    /// trust/freshness enrichment, budgets, freshness/concept accounting,
    /// settlement-backed calibration, and an explicit completeness verdict.
    ///
    /// Never fabricates context: a blocked or empty retrieval returns
    /// `completeness == None` with a machine-readable reason.
    pub fn resolve_context(&self, req: &ContextRequirement) -> Result<ContextResolution> {
        let cap = req.effective_result_cap();
        let token_budget = req.budget.as_ref().map(|b| b.max_tokens).filter(|t| *t > 0);
        let requested_tokens = token_budget;
        let generated_query_id = format!("qry_{}", Ulid::new());
        let mut query_id = generated_query_id.clone();
        let mut mode = req.mode.clone().unwrap_or_else(|| "hybrid".to_string());

        // --- 1. Data-access gate (before any retrieval) --------------------
        let mut searched: Vec<String> = Vec::new();
        for corpus in &req.corpora {
            if req.constraints.exclusions.iter().any(|e| e == corpus) {
                return Ok(ContextResolution::blocked(
                    query_id,
                    mode,
                    ContextAccess::Inaccessible {
                        corpus_id: corpus.clone(),
                        reason: "excluded",
                    },
                    NoContextReason::CorpusExcluded,
                    searched,
                    cap,
                    requested_tokens,
                ));
            }
            if let Some(c) = self.storage.get_corpus(corpus)? {
                if c.visibility == "private"
                    && !req.constraints.caller_scope.iter().any(|s| s == corpus)
                {
                    let access = if req.constraints.caller_scope.is_empty() {
                        ContextAccess::RequiresScope {
                            corpus_id: corpus.clone(),
                        }
                    } else {
                        ContextAccess::Inaccessible {
                            corpus_id: corpus.clone(),
                            reason: "out_of_scope",
                        }
                    };
                    return Ok(ContextResolution::blocked(
                        query_id,
                        mode,
                        access,
                        NoContextReason::ScopeUnresolved,
                        searched,
                        cap,
                        requested_tokens,
                    ));
                }
            }
            searched.push(corpus.clone());
        }

        // --- 2. Retrieval through the real query_reference pipeline ---------
        let query = req.query.clone().unwrap_or_default();
        let filters = serde_json::json!({ "source_scope": req.constraints.caller_scope });
        let mut candidates: Vec<RetrievalResult> = Vec::new();
        let mut truncated = false;
        if req.corpora.is_empty() {
            let resp = self.query_reference(QueryReferenceRequest {
                canonical_id: None,
                query,
                mode: Some(mode.clone()),
                top_k: cap,
                version: None,
                filters,
            })?;
            truncated = resp.truncated;
            query_id = resp.query_id;
            mode = resp.mode;
            candidates = resp.results;
        } else {
            for corpus in &req.corpora {
                let resp = self.query_reference(QueryReferenceRequest {
                    canonical_id: Some(corpus.clone()),
                    query: query.clone(),
                    mode: Some(mode.clone()),
                    top_k: cap,
                    version: None,
                    filters: filters.clone(),
                })?;
                if query_id == generated_query_id {
                    query_id = resp.query_id.clone();
                    mode = resp.mode.clone();
                }
                if resp.truncated {
                    truncated = true;
                }
                for r in resp.results {
                    if !candidates.iter().any(|c| c.result_id == r.result_id) {
                        candidates.push(r);
                    }
                }
            }
            candidates.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }


        // --- 3. Freshness gate (declared freshness requirement) ------------
        let mut omissions: Vec<RetrievalOmission> = Vec::new();
        let mut had_candidates = false;
        let mut dropped_stale = false;
        if let Some(max_age) = req.freshness.max_age_secs {
            let any_before = !candidates.is_empty();
            let mut fresh = Vec::with_capacity(candidates.len());
            for r in candidates.drain(..) {
                let stale = self.result_is_stale(&r, max_age)?;
                if stale {
                    dropped_stale = true;
                } else {
                    fresh.push(r);
                }
            }
            candidates = fresh;
            if dropped_stale {
                omissions.push(RetrievalOmission::code(OMISSION_STALE_SOURCE));
            }
            had_candidates = any_before;
        }

        // --- 4. Budget application (result cap + token ceiling) ------------
        let mut kept: Vec<RetrievalResult> = Vec::new();
        let mut consumed_tokens: i64 = 0;
        let mut more = truncated;
        let mut budget_hit = false;
        for r in candidates {
            if i64::try_from(kept.len()).unwrap_or(i64::MAX) >= cap {
                more = true;
                break;
            }
            let cost = estimate_result_tokens(&r);
            if let Some(max_tokens) = token_budget {
                if consumed_tokens + cost > max_tokens {
                    budget_hit = true;
                    break;
                }
            }
            consumed_tokens += cost;
            kept.push(r);
        }
        if budget_hit {
            omissions.push(RetrievalOmission::code(OMISSION_TOKEN_BUDGET));
        }
        if more {
            omissions.push(RetrievalOmission::code(OMISSION_MAX_RESULTS));
        }

        // --- 5. Requested corpora availability ------------------------------
        for corpus in &req.corpora {
            if self.storage.get_corpus(corpus)?.is_none() {
                omissions.push(RetrievalOmission::with_detail(
                    NoContextReason::CorpusNotFound.as_str(),
                    corpus.clone(),
                ));
            }
        }

        // --- 6. Required semantic distinctions ------------------------------
        if kept.is_empty() {
            let reason = if budget_hit {
                NoContextReason::BudgetExhausted
            } else if dropped_stale && had_candidates {
                NoContextReason::StaleContent
            } else if req
                .corpora
                .iter()
                .any(|c| self.storage.get_corpus(c).ok().flatten().is_none())
            {
                NoContextReason::CorpusNotFound
            } else {
                NoContextReason::NoMatchingContent
            };
            let reason_code = reason.as_str();
            if !omissions.iter().any(|o| o.code == reason_code) {
                omissions.push(RetrievalOmission::code(reason_code));
            }
            return Ok(ContextResolution {
                query_id,
                mode,
                results: Vec::new(),
                access: ContextAccess::Accessible,
                completeness: RetrievalCompleteness::None,
                omissions,
                no_context_reason: Some(reason),
                budget: BudgetUsage {
                    requested_results: cap,
                    consumed_results: 0,
                    requested_tokens,
                    consumed_tokens,
                    more_available: more,
                },
                corpora: searched,
            });
        }
        for concept in &req.required_concepts {
            let wanted = concept.to_lowercase();
            let present = kept.iter().any(|r| {
                let blob = format!(
                    "{} {} {} {}",
                    r.title.as_deref().unwrap_or(""),
                    r.heading.as_deref().unwrap_or(""),
                    r.excerpt,
                    r.citation
                )
                .to_lowercase();
                blob.contains(&wanted)
            });
            if !present {
                omissions.push(RetrievalOmission::with_detail(
                    OMISSION_REQUIRED_CONCEPT,
                    concept.clone(),
                ));
            }
        }

        // --- 7. Prospective calibration (settlement-backed only) ------------
        if let Some(store) = &self.context_effects {
            match store.summary(&req.world_ref) {
                Ok(summary) => {
                    let mut adjusted = false;
                    for r in &mut kept {
                        let source_id = r
                            .metadata
                            .get("source_id")
                            .and_then(|v| v.as_str())
                            .map(str::to_string);
                        if let Some(source_id) = source_id.clone() {
                            let penalty = summary.penalty_for(&source_id);
                            if penalty > 0.0 {
                                r.score *= 1.0 - penalty;
                                if let Some(obj) = r.score_breakdown.as_object_mut() {
                                    obj.insert(
                                        "calibration".to_string(),
                                        serde_json::json!({
                                            "penalty": penalty,
                                            "effects": summary.effect_count_for(&source_id),
                                        }),
                                    );
                                }
                                adjusted = true;
                            }
                        }
                    }
                    if adjusted {
                        kept.sort_by(|a, b| {
                            b.score
                                .partial_cmp(&a.score)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "context effect calibration unreadable; proceeding uncalibrated"
                    );
                }
            }
        }

        // --- 8. Completeness verdict ----------------------------------------
        let completeness = if omissions.is_empty() {
            RetrievalCompleteness::Complete
        } else {
            RetrievalCompleteness::Partial
        };

        let consumed_results = kept.len();

        Ok(ContextResolution {
            query_id,
            mode,
            results: kept,
            access: ContextAccess::Accessible,
            completeness,
            omissions,
            no_context_reason: None,
            budget: BudgetUsage {
                requested_results: cap,
                consumed_results,
                requested_tokens,
                consumed_tokens,
                more_available: more,
            },
            corpora: searched,
        })
    }

    /// Whether a result's backing source is older than the declared freshness
    /// window. A result with no attributable source timestamp is NOT treated
    /// as stale (unknown is never silently converted into a verdict).
    fn result_is_stale(&self, r: &RetrievalResult, max_age_secs: u64) -> Result<bool> {
        let Some(source_id) = r.metadata.get("source_id").and_then(|v| v.as_str()) else {
            return Ok(false);
        };
        let Some(source) = self.storage.get_source(source_id)? else {
            return Ok(false);
        };
        let Some(ts) = source
            .last_refresh_success_at
            .as_deref()
            .filter(|s| !s.is_empty())
            .or(source.first_indexed_at.as_deref().filter(|s| !s.is_empty()))
        else {
            return Ok(false);
        };
        let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts) else {
            return Ok(false);
        };
        let age = Utc::now()
            .signed_duration_since(parsed.with_timezone(&Utc))
            .num_seconds()
            .max(0) as u64;
        Ok(age > max_age_secs)
    }
}

/// Rough token cost of one retrieval result (whitespace word count).
fn estimate_result_tokens(r: &RetrievalResult) -> i64 {
    (r.excerpt.split_whitespace().count()
        + r.title.as_deref().unwrap_or("").split_whitespace().count())
        as i64
}

impl AppServices {
    /// Ingest settlement-backed context effects for prospective retrieval
    /// calibration. This is the ONLY durable calibration path: each effect
    /// must cite a settlement/evidence artifact (self-reports are rejected),
    /// the ledger is append-only, and historical bundles are never rewritten.
    /// Effects influence FUTURE retrieval of the same world only.
    pub fn ingest_context_effects(
        &self,
        effects: &[crate::context_effect::ContextEffect],
    ) -> Result<crate::context_effect::ContextEffectIngestReport> {
        let Some(store) = &self.context_effects else {
            anyhow::bail!("context effect ledger not configured");
        };
        let mut report = crate::context_effect::ContextEffectIngestReport::default();
        for (index, effect) in effects.iter().enumerate() {
            match store.append(effect) {
                Ok(true) => report.accepted += 1,
                Ok(false) => report.skipped_duplicates += 1,
                Err(e) => report.rejected.push((index, e.to_string())),
            }
        }
        Ok(report)
    }
}

