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

/// Why a required-context acquisition resolved with zero citations.
///
/// Frozen edge E3: missing corpus configuration MUST NOT silently settle as
/// successful required-context acquisition; zero-citation outcomes must be
/// explicit and governed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoContextReason {
    /// Corpus root itself was not configured / does not exist.
    CorpusRootUnconfigured,
    /// Requested corpus directory does not exist under the root.
    CorpusNotFound,
    /// Corpus exists but no content matched the query.
    NoMatchingContent,
}

impl NoContextReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CorpusRootUnconfigured => "corpus_root_unconfigured",
            Self::CorpusNotFound => "corpus_not_found",
            Self::NoMatchingContent => "no_matching_content",
        }
    }
}

/// Outcome of one context acquisition against the agent.
#[derive(Debug)]
pub enum ContextAcquisition {
    /// At least one citation resolved.
    Cited(ContextPacketCreatedPayload),
    /// Zero citations; the reason makes the absence explicit (never silent).
    NoContext(ContextPacketCreatedPayload, NoContextReason),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub namespace: String,
    pub occurred_at: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRequiredPayload {
    pub domain_model_hash: String,
    pub work_request_id: String,
    pub context_requirement_id: String,
    pub corpus_id: String,
    pub query: Option<String>,
    #[serde(default = "default_max_results")]
    pub max_results: u32,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacketCreatedPayload {
    pub domain_model_hash: String,
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
use ck_core::ports::SourceAdapter;
use ulid::Ulid;

/// Concrete implementation backed by ck-adapter-filesystem.
///
/// For the `learning-proposals` corpus, reads from a JSONL ledger written by GodSpeed-Agent.
/// For all other corpora, uses FilesystemSourceAdapter to resolve citations from a local path.
///
/// Context Kernel does NOT decide authority — SEA-Forge owns that. This agent
/// only resolves citations and returns a packet carrying authority *references*
/// (passed through from the caller, not computed locally).
pub struct AclContextAgent {
    pub corpus_root: PathBuf,
    pub learning_proposals_ledger: Option<PathBuf>,
}

impl AclContextAgent {
    /// Acquire context with an EXPLICIT outcome: cited success or governed
    /// no-context with a machine-readable reason. This is the frozen E3
    /// boundary — zero citations are never silently successful.
    pub fn acquire_context(
        &self,
        payload: &ContextRequiredPayload,
        authority_decision_id: Option<&str>,
        authority_reference: Option<&str>,
    ) -> Result<ContextAcquisition, anyhow::Error> {
        let (citations, reason) = self.resolve_citations(payload)?;
        if citations.is_empty() {
            let reason = reason.expect("empty citations always carry a reason");
            let mut packet = self.packet_for(
                payload,
                Vec::new(),
                authority_decision_id,
                authority_reference,
            )?;
            packet.citations.clear();
            return Ok(ContextAcquisition::NoContext(packet, reason));
        }
        let packet = self.packet_for(
            payload,
            citations,
            authority_decision_id,
            authority_reference,
        )?;
        Ok(ContextAcquisition::Cited(packet))
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
            context_packet_id,
            context_requirement_id: payload.context_requirement_id.clone(),
            work_request_id: payload.work_request_id.clone(),
            corpus_id: payload.corpus_id.clone(),
            citations,
            authority_decision_id: authority_decision_id.map(|s| s.to_string()),
            authority_reference: authority_reference.map(|s| s.to_string()),
        })
    }

    /// Back-compat entry point: returns the packet regardless of outcome.
    /// Prefer [`Self::acquire_context`] at canonical boundaries so zero-citation
    /// acquisitions stay explicit.
    pub fn handle_context_required(
        &self,
        payload: &ContextRequiredPayload,
        authority_decision_id: Option<&str>,
        authority_reference: Option<&str>,
    ) -> Result<ContextPacketCreatedPayload, anyhow::Error> {
        match self.acquire_context(payload, authority_decision_id, authority_reference)? {
            ContextAcquisition::Cited(p) => Ok(p),
            ContextAcquisition::NoContext(p, _) => Ok(p),
        }
    }

    fn resolve_citations(
        &self,
        payload: &ContextRequiredPayload,
    ) -> Result<(Vec<Citation>, Option<NoContextReason>), anyhow::Error> {
        // Route learning-proposals corpus to JSONL ledger
        if payload.corpus_id.starts_with("learning-proposals") {
            return self.resolve_from_learning_proposals(payload);
        }

        // Resolve from filesystem corpus directory
        let corpus_dir = self.corpus_root.join(&payload.corpus_id);

        // Containment: reject corpus_id that escapes corpus_root via traversal,
        // absolute paths, or symlinks. Canonicalize resolves symlinks.
        let canonical_root = match self.corpus_root.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // corpus root not configured / missing — explicit, never silent
                return Ok((vec![], Some(NoContextReason::CorpusRootUnconfigured)));
            }
        };
        if !corpus_dir.is_dir() {
            return Ok((vec![], Some(NoContextReason::CorpusNotFound)));
        }
        let canonical_dir = match corpus_dir.canonicalize() {
            Ok(p) => p,
            Err(_) => return Ok((vec![], Some(NoContextReason::CorpusNotFound))),
        };
        if !canonical_dir.starts_with(&canonical_root) {
            anyhow::bail!(
                "corpus_id '{}' escapes the configured corpus root",
                payload.corpus_id
            );
        }
        let config =
            serde_json::json!({ "path": canonical_dir.to_str().unwrap_or("") }).to_string();
        let adapter = ck_adapter_filesystem::FilesystemSourceAdapter;
        let units = adapter.discover_units(&payload.corpus_id, &config)?;

        let query = payload.query.as_deref().unwrap_or("");
        let max = payload.max_results as usize;
        let now = Utc::now().to_rfc3339();

        let mut citations = Vec::new();
        for unit in units.iter() {
            if citations.len() >= max {
                break;
            }
            let raw = adapter.fetch_content(&payload.corpus_id, unit, &config)?;
            let content = String::from_utf8_lossy(&raw.bytes).into_owned();
            if query.is_empty() || content.to_lowercase().contains(&query.to_lowercase()) {
                citations.push(Citation {
                    citation_id: format!("cit_{}", Ulid::new()),
                    source: unit.uri.clone().unwrap_or_else(|| unit.external_id.clone()),
                    content: content.chars().take(500).collect(),
                    fetched_at: now.clone(),
                });
            }
        }
        if citations.is_empty() {
            Ok((vec![], Some(NoContextReason::NoMatchingContent)))
        } else {
            Ok((citations, None))
        }
    }

    fn resolve_from_learning_proposals(
        &self,
        payload: &ContextRequiredPayload,
    ) -> Result<(Vec<Citation>, Option<NoContextReason>), anyhow::Error> {
        use std::io::{BufRead, BufReader};

        let ledger_path = match &self.learning_proposals_ledger {
            Some(p) => p.clone(),
            None => return Ok((vec![], Some(NoContextReason::CorpusRootUnconfigured))),
        };
        if !ledger_path.exists() {
            return Ok((vec![], Some(NoContextReason::CorpusNotFound)));
        }

        let file = std::fs::File::open(&ledger_path)?;
        let reader = BufReader::new(file);
        let query = payload.query.as_deref().unwrap_or("").to_lowercase();
        let max = payload.max_results as usize;
        let now = Utc::now().to_rfc3339();

        let mut citations = Vec::new();
        for line in reader.lines() {
            if citations.len() >= max {
                break;
            }
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let raw: serde_json::Value = serde_json::from_str(line)?;
            // Handle both raw proposal envelopes and LedgerStore-wrapped envelopes:
            // - raw: {event_id, payload: {proposal, domain_area, ...}}
            // - wrapped: {record: {event_id, payload: ...}, ledger, checksum, ...}
            // - developmental_events developmental variant where proposal may be backfilled
            let event = if raw.get("record").is_some() && raw["record"].is_object() {
                raw.get("record").unwrap().clone()
            } else {
                raw.clone()
            };
            // Proposal text may be at payload.proposal (canonical) or payload.status fallback;
            // if still empty but domain_area present, synthesize a citable stub (mirrors GSA export helper).
            let mut proposal_text = event["payload"]["proposal"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let domain_area = event["payload"]["domain_area"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if proposal_text.is_empty() && !domain_area.is_empty() {
                let status = event["payload"]["status"].as_str().unwrap_or("open");
                proposal_text = format!("Learning proposal in {domain_area}: {status}");
            }
            // Also support the developmental ledger's LearningProposalCreated where proposal
            // may be synthesized from domain_area/status above — still citable via learning-proposals://

            if query.is_empty()
                || proposal_text.to_lowercase().contains(&query)
                || domain_area.to_lowercase().contains(&query)
            {
                // Skip entries with no citable proposal text and no domain_area (degenerate, not provenance)
                if proposal_text.is_empty() && domain_area.is_empty() {
                    continue;
                }
                citations.push(Citation {
                    citation_id: format!("cit_{}", Ulid::new()),
                    source: format!(
                        "learning-proposals://{}",
                        event["payload"]["learning_proposal_id"]
                            .as_str()
                            .or_else(|| event["payload"]["learning_proposal_id"].as_str())
                            .or_else(|| event["payload"]["coherence_break_id"].as_str())
                            .unwrap_or("unknown")
                    ),
                    content: proposal_text,
                    fetched_at: now.clone(),
                });
            }
        }
        if citations.is_empty() {
            Ok((vec![], Some(NoContextReason::NoMatchingContent)))
        } else {
            Ok((citations, None))
        }
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
