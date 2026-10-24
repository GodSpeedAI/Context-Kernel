//! agentic_capability_loop — Context Kernel MCP adapter
//!
//! Context Kernel's role in the loop is retrieval only: it resolves
//! citations and emits a `ContextPacketCreated` packet. It does NOT create
//! authority decisions — SEA-Forge owns `AuthorityChecked`. The packet
//! carries authority *references* (decision ID + reference string) that
//! link the cited context to a prior verified SEA allow, but CK does not
//! compute the decision itself.
//!
//! Consumed:
//! - `ContextRequired`      — emitted by SWE_SEED
//!
//! Emitted:
//! - `ContextPacketCreated` — after resolving citations

use serde::{Deserialize, Serialize};

/// The namespace-derived standalone pseudo-hash (sha256 of
/// "agentic_capability_loop") that the frozen convergence preregistration
/// forbids masquerading as canonical domain identity (ENV-I2).
pub const FALLBACK_PSEUDO_HASH: &str =
    "dc144cbd71a483431301ca1bf32e95c014af4edba8dbcc525f505310e30a107e";

/// Why a candidate `domain_model_hash` cannot enter the canonical boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainIdentityError {
    /// Not a 64-character lowercase hex digest.
    Malformed,
    /// A known pseudo-identity (standalone fallback constant / all-zero).
    Placeholder,
}

impl std::fmt::Display for DomainIdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Malformed => write!(
                f,
                "domain_model_hash is not a 64-char lowercase sha256 digest"
            ),
            Self::Placeholder => write!(
                f,
                "fallback/placeholder pseudo-hash rejected as domain identity"
            ),
        }
    }
}

/// Canonical identity ingress gate (convergence T02 adopting frozen ENV-I1/I2):
/// a digest is admissible only when it is a real SHA-256 hex form AND not a
/// known pseudo-identity. CK never manufactures an identity.
pub fn verify_domain_identity(hash: &str) -> Result<(), DomainIdentityError> {
    let well_formed = hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    if !well_formed {
        return Err(DomainIdentityError::Malformed);
    }
    if hash == FALLBACK_PSEUDO_HASH || hash.bytes().all(|b| b == b'0') {
        return Err(DomainIdentityError::Placeholder);
    }
    Ok(())
}

/// The immutable semantic-world gate and retrieval-completeness vocabulary
/// live in `ck-core` (shared across the kernel). Re-exported here so the
/// boundary module keeps its public paths.
pub use ck_core::types::{
    verify_world_ref, NoContextReason, RetrievalCompleteness, WorldRefError,
    OMISSION_CONTENT_TRUNCATED, OMISSION_MAX_RESULTS,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub namespace: String,
    pub occurred_at: String,
    pub payload: serde_json::Value,
}

/// The E2 `ContextRequired` wire payload. This is the *wire* shape; the
/// native Context Kernel input model is `ck_app::ContextRequirement`
/// (built by [`ContextRequiredPayload::to_requirement`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRequiredPayload {
    pub domain_model_hash: String,
    /// Immutable semantic-world reference the request is pinned to.
    /// `#[serde(default)]` so a missing field reaches the fail-closed world
    /// gate (""), rather than a bare decode error.
    #[serde(default)]
    pub world_ref: String,
    pub work_request_id: String,
    pub context_requirement_id: String,
    pub corpus_id: String,
    pub query: Option<String>,
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    // --- typed requirement extensions (all optional on the wire) ---------
    /// Retrieval mode: `lexical`, `semantic`, `hybrid`.
    #[serde(default)]
    pub mode: Option<String>,
    /// Purpose / retrieval intent.
    #[serde(default)]
    pub purpose: Option<String>,
    /// Semantic concepts the requirement must represent.
    #[serde(default)]
    pub required_concepts: Vec<String>,
    /// Caller-declared scope claim (POL-ACL-002: required for private corpora).
    #[serde(default)]
    pub scope_claim: Option<String>,
    /// Caller assertion that the corpus is private (storage is authoritative).
    #[serde(default)]
    pub is_private: bool,
    /// Corpora/sources excluded from this retrieval.
    #[serde(default)]
    pub exclusions: Vec<String>,
    /// Maximum accepted source age (seconds).
    #[serde(default)]
    pub freshness_max_age_secs: Option<u64>,
    /// Token ceiling for the retrieved context.
    #[serde(default)]
    pub token_budget: Option<i64>,
    /// Task/route identity.
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub route_id: Option<String>,
    /// Expected artifact/proof references.
    #[serde(default)]
    pub expected_refs: Vec<String>,
    /// Caller-supplied risk classification (informational; SEA-Forge decides
    /// consequential-action authority).
    #[serde(default)]
    pub risk_classification: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub causation_id: Option<String>,
}

impl ContextRequiredPayload {
    /// Project the wire payload onto the native typed context requirement.
    pub fn to_requirement(&self) -> ck_app::ContextRequirement {
        ck_app::ContextRequirement {
            identity: ck_app::RequirementIdentity {
                work_request_ref: self.work_request_id.clone(),
                context_requirement_id: self.context_requirement_id.clone(),
                task_ref: self.task_id.clone(),
                route_ref: self.route_id.clone(),
            },
            world_ref: self.world_ref.clone(),
            purpose: self.purpose.clone(),
            query: self.query.clone(),
            mode: self.mode.clone(),
            required_concepts: self.required_concepts.clone(),
            corpora: if self.corpus_id.is_empty() {
                Vec::new()
            } else {
                vec![self.corpus_id.clone()]
            },
            constraints: ck_app::SourceConstraints {
                caller_scope: self.scope_claim.iter().map(|s| s.to_string()).collect(),
                exclusions: self.exclusions.clone(),
            },
            freshness: ck_app::FreshnessRequirement {
                max_age_secs: self.freshness_max_age_secs,
            },
            max_results: self.max_results,
            budget: self
                .token_budget
                .map(|max_tokens| ck_core::entities::Budget {
                    max_tokens,
                    max_chunks: i64::from(self.max_results.max(1)),
                }),
            expected_refs: self.expected_refs.clone(),
            risk_classification: self.risk_classification.clone(),
            trace: ck_app::TraceRefs {
                correlation_id: self.correlation_id.clone(),
                causation_id: self.causation_id.clone(),
            },
        }
    }
}

fn default_max_results() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub citation_id: String,
    pub source: String,
    pub content: String,
    pub fetched_at: String,
    /// True when `content` is a prefix of the source rather than all of it.
    #[serde(default)]
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacketCreatedPayload {
    pub domain_model_hash: String,
    /// Echo of the request's `world_ref`, unchanged.
    pub world_ref: String,
    pub retrieval_completeness: RetrievalCompleteness,
    #[serde(default)]
    pub omissions: Vec<String>,
    pub context_packet_id: String,
    pub context_requirement_id: String,
    pub work_request_id: String,
    pub corpus_id: String,
    pub citations: Vec<Citation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_decision_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_reference: Option<String>,
}

// ---------------------------------------------------------------------------
// AclContextAgent — concrete implementation
// ---------------------------------------------------------------------------

use std::path::PathBuf;

use chrono::Utc;
use ulid::Ulid;

/// The GodSpeed-facing context acquisition adapter. Thin by design: it
/// validates the boundary, resolves requested corpora into the real kernel
/// (auto-ingesting operator-configured corpus sources on first use), and
/// delegates ALL retrieval to `AppServices::resolve_context` — the same
/// application pipeline as normal Context Kernel operation. There is no
/// degraded, integration-only retrieval path.
///
/// Context Kernel does NOT decide authority — SEA-Forge owns that. This agent
/// only resolves context and returns a packet carrying authority *references*
/// (passed through from the caller, not computed locally).
pub struct AclContextAgent {
    /// Operator-configured filesystem corpus root used for auto-ingest.
    pub corpus_root: PathBuf,
    /// GodSpeed-Agent learning-proposals JSONL ledger (auto-ingested through
    /// the real `LearningProposalsAdapter` source adapter).
    pub learning_proposals_ledger: Option<PathBuf>,
}

/// The outcome of one context acquisition: the legacy packet (dual-read
/// compatibility) plus the full native resolution the CEP projection is
/// built from.
#[derive(Debug)]
pub struct ContextAcquisitionResult {
    pub packet: ContextPacketCreatedPayload,
    pub resolution: ck_app::ContextResolution,
    pub no_context: Option<NoContextReason>,
}

impl ContextAcquisitionResult {
    /// Frozen E3 outcome naming: only `cited` is a successful acquisition;
    /// zero citations are explicit and governed, never silently successful.
    pub fn outcome(&self, required: bool) -> &'static str {
        if self.no_context.is_none() {
            "cited"
        } else if required {
            "no_context_governed"
        } else {
            "empty_optional"
        }
    }

    pub fn reason(&self) -> Option<&'static str> {
        self.no_context.map(|r| r.as_str())
    }
}

/// Why a corpus could not be made retrievable.
enum EnsureOutcome {
    Ready,
    Blocked(NoContextReason),
}

impl AclContextAgent {
    /// Acquire context with an EXPLICIT outcome: cited success or governed
    /// no-context with a machine-readable reason.
    pub fn acquire_context(
        &self,
        app: &ck_app::AppServices,
        payload: &ContextRequiredPayload,
        authority_decision_id: Option<&str>,
        authority_reference: Option<&str>,
    ) -> Result<ContextAcquisitionResult, anyhow::Error> {
        verify_world_ref(&payload.world_ref)
            .map_err(|e| anyhow::anyhow!("world_ref rejected: {e}"))?;
        let requirement = payload.to_requirement();

        // Resolve requested corpora into the kernel BEFORE retrieval. An
        // operator-configured corpus is auto-ingested exactly once; anything
        // else resolves to an explicit no-context reason, never to fabricated
        // context.
        for corpus_id in &requirement.corpora {
            if app.corpus(corpus_id)?.is_some() {
                continue;
            }
            if payload.is_private {
                // A caller-declared private corpus must already be registered
                // with its visibility — it is never auto-ingested open.
                return Ok(self.no_context_result(
                    payload,
                    NoContextReason::CorpusNotFound,
                    authority_decision_id,
                    authority_reference,
                )?);
            }
            match self.ensure_corpus_ingested(app, corpus_id) {
                Ok(EnsureOutcome::Ready) => {}
                Ok(EnsureOutcome::Blocked(reason)) => {
                    return Ok(self.no_context_result(
                        payload,
                        reason,
                        authority_decision_id,
                        authority_reference,
                    )?);
                }
                Err(e) => return Err(e),
            }
        }

        // The real retrieval pipeline: lexical + semantic-when-configured,
        // corpus filtering, scope enforcement, ranking, trust/freshness,
        // budgets, completeness accounting, calibration.
        let resolution = app.resolve_context(&requirement)?;

        let citations: Vec<Citation> = resolution.results.iter().map(result_to_citation).collect();
        let omissions: Vec<String> = resolution
            .omissions
            .iter()
            .map(|o| o.code.clone())
            .collect();
        let mut packet = self.packet_for(
            payload,
            citations,
            authority_decision_id,
            authority_reference,
        )?;
        packet.retrieval_completeness = resolution.completeness;
        packet.omissions = omissions;
        let no_context = resolution.no_context_reason;
        if no_context.is_some() {
            packet.citations.clear();
        }
        Ok(ContextAcquisitionResult {
            packet,
            resolution,
            no_context,
        })
    }

    /// Explicit no-context outcome with an empty packet and a blocked
    /// resolution — absence is stated, never fabricated.
    fn no_context_result(
        &self,
        payload: &ContextRequiredPayload,
        reason: NoContextReason,
        authority_decision_id: Option<&str>,
        authority_reference: Option<&str>,
    ) -> Result<ContextAcquisitionResult, anyhow::Error> {
        let requirement = payload.to_requirement();
        let mut packet = self.packet_for(
            payload,
            Vec::new(),
            authority_decision_id,
            authority_reference,
        )?;
        packet.retrieval_completeness = RetrievalCompleteness::None;
        packet.omissions = vec![reason.as_str().to_string()];
        let resolution = ck_app::ContextResolution::blocked(
            format!("qry_{}", Ulid::new()),
            requirement
                .mode
                .clone()
                .unwrap_or_else(|| "hybrid".to_string()),
            ck_app::ContextAccess::Accessible,
            reason,
            requirement.corpora.clone(),
            requirement.effective_result_cap(),
            requirement.budget.as_ref().map(|b| b.max_tokens),
        );
        Ok(ContextAcquisitionResult {
            packet,
            resolution,
            no_context: Some(reason),
        })
    }

    fn packet_for(
        &self,
        payload: &ContextRequiredPayload,
        citations: Vec<Citation>,
        authority_decision_id: Option<&str>,
        authority_reference: Option<&str>,
    ) -> Result<ContextPacketCreatedPayload, anyhow::Error> {
        let context_packet_id = format!("ctx_{}", Ulid::new());
        Ok(ContextPacketCreatedPayload {
            domain_model_hash: payload.domain_model_hash.clone(),
            world_ref: payload.world_ref.clone(),
            retrieval_completeness: RetrievalCompleteness::None,
            omissions: Vec::new(),
            context_packet_id,
            context_requirement_id: payload.context_requirement_id.clone(),
            work_request_id: payload.work_request_id.clone(),
            corpus_id: payload.corpus_id.clone(),
            citations,
            authority_decision_id: authority_decision_id.map(str::to_string),
            authority_reference: authority_reference.map(str::to_string),
        })
    }

    /// Make one requested corpus retrievable through the real kernel:
    /// auto-ingest the operator-configured source (filesystem tree under
    /// `corpus_root`, or the learning-proposals ledger) exactly once.
    ///
    /// A corpus_id that escapes `corpus_root` via traversal is a hard refusal,
    /// not a quiet no-context.
    fn ensure_corpus_ingested(
        &self,
        app: &ck_app::AppServices,
        corpus_id: &str,
    ) -> Result<EnsureOutcome, anyhow::Error> {
        if corpus_id.starts_with("learning-proposals") {
            let Some(ledger) = &self.learning_proposals_ledger else {
                return Ok(EnsureOutcome::Blocked(
                    NoContextReason::CorpusRootUnconfigured,
                ));
            };
            if !ledger.exists() {
                return Ok(EnsureOutcome::Blocked(NoContextReason::CorpusNotFound));
            }
            let config = serde_json::json!({ "path": ledger.to_str().unwrap_or("") }).to_string();
            return match app.ingest_source(ck_app::IngestSourceRequest {
                source_id: format!("learning-proposals:{corpus_id}"),
                source_kind: "learning_proposals_jsonl".to_string(),
                adapter_config_json: config,
                corpus_id: Some(corpus_id.to_string()),
            }) {
                Ok(_) => Ok(EnsureOutcome::Ready),
                Err(_) => Ok(EnsureOutcome::Blocked(NoContextReason::CorpusUnavailable)),
            };
        }

        let Ok(canonical_root) = self.corpus_root.canonicalize() else {
            // corpus root not configured / missing — explicit, never silent
            return Ok(EnsureOutcome::Blocked(
                NoContextReason::CorpusRootUnconfigured,
            ));
        };
        let corpus_dir = self.corpus_root.join(corpus_id);
        if !corpus_dir.is_dir() {
            return Ok(EnsureOutcome::Blocked(NoContextReason::CorpusNotFound));
        }
        let Ok(canonical_dir) = corpus_dir.canonicalize() else {
            return Ok(EnsureOutcome::Blocked(NoContextReason::CorpusNotFound));
        };
        if !canonical_dir.starts_with(&canonical_root) {
            anyhow::bail!(
                "corpus_id '{}' escapes the configured corpus root",
                corpus_id
            );
        }
        let config =
            serde_json::json!({ "path": canonical_dir.to_str().unwrap_or("") }).to_string();
        match app.ingest_source(ck_app::IngestSourceRequest {
            source_id: format!("ctxfs:{corpus_id}"),
            source_kind: "filesystem_tree".to_string(),
            adapter_config_json: config,
            corpus_id: Some(corpus_id.to_string()),
        }) {
            Ok(_) => Ok(EnsureOutcome::Ready),
            Err(_) => Ok(EnsureOutcome::Blocked(NoContextReason::CorpusUnavailable)),
        }
    }
}

/// Project one real retrieval result onto the legacy citation wire shape.
/// The bounded representation is the ranked excerpt with a resolvable
/// locator; full content lives at the cited source (see the retrieval
/// contract recorded on the bundle projection).
fn result_to_citation(result: &ck_core::entities::RetrievalResult) -> Citation {
    let uri = result
        .citation
        .get("uri")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| result.result_id.clone());
    let source = match result.citation.get("anchor").and_then(|v| v.as_str()) {
        Some(anchor) => format!("{uri}#{anchor}"),
        None => uri,
    };
    Citation {
        citation_id: format!("cit_{}", Ulid::new()),
        source,
        content: result.excerpt.clone(),
        fetched_at: Utc::now().to_rfc3339(),
        truncated: result.excerpt.ends_with('…'),
    }
}

// ---------------------------------------------------------------------------
// Canonical E3 envelope construction (convergence T02)
// ---------------------------------------------------------------------------

/// The v1 wire schema version label shared with the canonical envelope family.
pub const SCHEMA_VERSION_V1: &str = "v1";
/// CK's canonical producer stamp (exclusive authoritative producer of
/// `ContextPacketCreated` per the frozen edge topology).
pub const CONTEXT_KERNEL_AGENT: &str = "context-kernel";
/// Provenance-chain entry prefix marking a causal parent envelope id.
pub const CAUSALITY_PREFIX: &str = "caused_by:";

fn sorted_without_signature(value: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match value {
        Value::Object(obj) => {
            let mut keys: Vec<_> = obj.keys().collect();
            keys.sort();
            let mut sorted = serde_json::Map::new();
            for key in keys {
                if key == "signature" {
                    continue;
                }
                sorted.insert(key.clone(), sorted_without_signature(&obj[key]));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.iter().map(sorted_without_signature).collect()),
        other => other.clone(),
    }
}

/// Content-derived dedup key matching the canonical family:
/// sha256("{correlation_id}|{event_type}|{canonical_payload_json}").
fn idempotency_key(
    event_type: &str,
    payload: &serde_json::Value,
    correlation_id: Option<&str>,
) -> String {
    use sha2::{Digest, Sha256};
    let payload_json =
        serde_json::to_string(&sorted_without_signature(payload)).unwrap_or_default();
    let content = format!(
        "{}|{}|{}",
        correlation_id.unwrap_or(""),
        event_type,
        payload_json
    );
    format!("{:x}", Sha256::digest(content.as_bytes()))
}

/// Build the canonical v1 `ContextPacketCreated` ENVELOPE (not just the
/// payload) so the consumer can run its composed boundary gates — producer
/// authority, identity, drift, causality — on the real E3 wire object.
///
/// `request_event_id`, when supplied, is recorded as the causal parent
/// (`caused_by:<event_id>`) of this packet envelope.
pub fn context_packet_envelope(
    request_event_id: Option<&str>,
    packet: &ContextPacketCreatedPayload,
) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "domain_model_hash".into(),
        serde_json::Value::String(packet.domain_model_hash.clone()),
    );
    payload.insert(
        "world_ref".into(),
        serde_json::Value::String(packet.world_ref.clone()),
    );
    payload.insert(
        "retrieval_completeness".into(),
        serde_json::to_value(packet.retrieval_completeness).unwrap_or_default(),
    );
    payload.insert(
        "omissions".into(),
        serde_json::to_value(&packet.omissions).unwrap_or_default(),
    );
    payload.insert(
        "namespace".into(),
        serde_json::Value::String("agentic_capability_loop".into()),
    );
    payload.insert(
        "work_request_id".into(),
        serde_json::Value::String(packet.work_request_id.clone()),
    );
    payload.insert(
        "context_requirement_id".into(),
        serde_json::Value::String(packet.context_requirement_id.clone()),
    );
    payload.insert(
        "context_packet_id".into(),
        serde_json::Value::String(packet.context_packet_id.clone()),
    );
    payload.insert(
        "corpus_id".into(),
        serde_json::Value::String(packet.corpus_id.clone()),
    );
    payload.insert(
        "citations".into(),
        serde_json::to_value(&packet.citations).unwrap_or_default(),
    );
    if let Some(a) = &packet.authority_decision_id {
        payload.insert(
            "authority_decision_id".into(),
            serde_json::Value::String(a.clone()),
        );
    }
    if let Some(a) = &packet.authority_reference {
        payload.insert(
            "authority_reference".into(),
            serde_json::Value::String(a.clone()),
        );
    }
    let payload_value = serde_json::Value::Object(payload);

    let mut chain = vec![format!("domain_model_hash:{}", packet.domain_model_hash)];
    if let Some(parent) = request_event_id {
        chain.push(format!("{CAUSALITY_PREFIX}{parent}"));
    }

    serde_json::json!({
        "schema_version": SCHEMA_VERSION_V1,
        "event_id": Ulid::new().to_string(),
        "source_agent": CONTEXT_KERNEL_AGENT,
        "event_type": "ContextPacketCreated",
        "occurred_at": Utc::now().to_rfc3339(),
        "idempotency_key": idempotency_key(
            "ContextPacketCreated",
            &payload_value,
            Some(&packet.work_request_id)
        ),
        "payload": payload_value,
        "provenance": { "origin": CONTEXT_KERNEL_AGENT, "chain": chain },
    })
}
