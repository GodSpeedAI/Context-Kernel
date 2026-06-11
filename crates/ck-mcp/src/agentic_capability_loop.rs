//! agentic_capability_loop — Context Kernel MCP adapter stub
//!
//! Defines Rust types for the events defined in the agentic_capability_loop
//! SEA domain model:
//!   docs/specs/domains/agentic_capability_loop/agentic_capability_loop.manifest.json (SEA-Forge)
//!
//! This module provides:
//! - Serde-serializable event structs for all relevant events
//! - A ContextAgent trait for implementing the Context Kernel's role in the loop
//!
//! The Python integration adapter at:
//!   integration/agentic_capability_loop/adapters.py
//! provides a Python-level stub for use in integration tests.
//!
//! # Events this module handles
//!
//! Consumed:
//! - `ContextRequired`      — emitted by SWE_SEED
//!
//! Emitted:
//! - `ContextPacketCreated` — after resolving citations
//! - `ContextAuthorized`    — after checking POL-ACL-002
//!
//! # Policy
//!
//! POL-ACL-002: PrivateCorpusRequiresExplicitScope
//! Context Kernel only retrieves private corpus documents when explicitly
//! scoped and authorized.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Shared envelope
// ---------------------------------------------------------------------------

/// Canonical event envelope for all agentic_capability_loop events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: String,
    pub event_type: String,
    pub namespace: String,
    pub occurred_at: String,
    pub payload: serde_json::Value,
}

// ---------------------------------------------------------------------------
// ContextRequired (consumed from SWE_SEED)
// ---------------------------------------------------------------------------

/// Payload for the `ContextRequired` event emitted by SWE_SEED.
///
/// This event requests that Context Kernel fetch relevant documents
/// from the specified corpus and emit a `ContextPacketCreated` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRequiredPayload {
    /// Hash of the SEA domain model used for traceability.
    pub domain_model_hash: String,
    /// ID of the originating work request.
    pub work_request_id: String,
    /// Unique ID for this context requirement.
    pub context_requirement_id: String,
    /// ID of the knowledge corpus to query.
    pub corpus_id: String,
    /// Optional natural-language query string.
    pub query: Option<String>,
    /// Maximum number of citations to return.
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    /// True if the corpus is private-scoped.
    #[serde(default)]
    pub is_private: bool,
    /// Required if `is_private` is true; authorization scope token.
    pub scope_claim: Option<String>,
}

fn default_max_results() -> u32 {
    10
}

// ---------------------------------------------------------------------------
// Citation
// ---------------------------------------------------------------------------

/// A single citation in a context packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub citation_id: String,
    pub source: String,
    pub content: String,
    pub fetched_at: String,
}

// ---------------------------------------------------------------------------
// ContextPacketCreated (emitted by Context Kernel)
// ---------------------------------------------------------------------------

/// Payload for the `ContextPacketCreated` event.
///
/// Emitted after Context Kernel resolves citations from the requested corpus.
/// If the corpus requires authorization and it was denied, `citations` will
/// be empty and `authorized` will be false.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacketCreatedPayload {
    pub domain_model_hash: String,
    pub context_packet_id: String,
    pub context_requirement_id: String,
    pub work_request_id: String,
    pub corpus_id: String,
    pub citations: Vec<Citation>,
    pub authorized: bool,
}

// ---------------------------------------------------------------------------
// ContextAuthorized (emitted by Context Kernel)
// ---------------------------------------------------------------------------

/// Result of the POL-ACL-002 authorization check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationResult {
    Allow,
    Deny,
    Escalate,
}

/// Payload for the `ContextAuthorized` event.
///
/// Emitted alongside `ContextPacketCreated` to communicate the authorization
/// decision to the SEA-Forge authority layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAuthorizedPayload {
    pub domain_model_hash: String,
    pub context_requirement_id: String,
    pub work_request_id: String,
    pub corpus_id: String,
    pub result: AuthorizationResult,
    pub reason: String,
    pub policy_code: String,
    pub scope_claim: Option<String>,
    pub evaluated_at: String,
}

// ---------------------------------------------------------------------------
// ContextAgent trait
// ---------------------------------------------------------------------------

/// Trait for the Context Kernel's role in the agentic capability loop.
///
/// Implementors should resolve citations from the requested corpus,
/// check POL-ACL-002, and emit the appropriate events.
pub trait ContextAgent {
    type Error: std::fmt::Debug;

    /// Handle a `ContextRequired` event.
    ///
    /// Returns a tuple of:
    /// - `ContextPacketCreated` envelope
    /// - `ContextAuthorized` envelope
    fn handle_context_required(
        &self,
        event: &EventEnvelope,
    ) -> Result<(EventEnvelope, EventEnvelope), Self::Error>;
}

// ---------------------------------------------------------------------------
// AclContextAgent — concrete implementation
// ---------------------------------------------------------------------------

use std::path::PathBuf;

use chrono::Utc;
use ck_core::ports::SourceAdapter;
use ulid::Ulid;

/// Concrete implementation of ContextAgent backed by ck-adapter-filesystem.
///
/// For the `learning-proposals` corpus, reads from a JSONL ledger written by GodSpeed-Agent.
/// For all other corpora, uses FilesystemSourceAdapter to resolve citations from a local path.
///
/// # Policy
///
/// Enforces POL-ACL-002: private corpora require an explicit scope_claim.
pub struct AclContextAgent {
    /// Root directory containing corpus subdirectories.
    /// Each corpus_id maps to a subdirectory: `<corpus_root>/<corpus_id>/`
    pub corpus_root: PathBuf,
    /// Optional path to the GodSpeed-Agent learning proposals JSONL ledger.
    pub learning_proposals_ledger: Option<PathBuf>,
}

impl ContextAgent for AclContextAgent {
    type Error = anyhow::Error;

    fn handle_context_required(
        &self,
        event: &EventEnvelope,
    ) -> Result<(EventEnvelope, EventEnvelope), Self::Error> {
        let payload: ContextRequiredPayload =
            serde_json::from_value(event.payload.clone())?;

        // POL-ACL-002: private corpus requires explicit scope_claim
        let (authorized, auth_reason) = if payload.is_private {
            match &payload.scope_claim {
                Some(claim) if !claim.is_empty() => {
                    (AuthorizationResult::Allow, format!("scope claim present: {}", claim))
                }
                _ => (
                    AuthorizationResult::Deny,
                    "POL-ACL-002: private corpus requires explicit scope_claim".to_string(),
                ),
            }
        } else {
            (AuthorizationResult::Allow, "public corpus".to_string())
        };

        let citations = if authorized == AuthorizationResult::Allow {
            self.resolve_citations(&payload)?
        } else {
            vec![]
        };

        let context_packet_id = format!("ctx_{}", Ulid::new());
        let now = Utc::now().to_rfc3339();

        let packet_payload = ContextPacketCreatedPayload {
            domain_model_hash: payload.domain_model_hash.clone(),
            context_packet_id: context_packet_id.clone(),
            context_requirement_id: payload.context_requirement_id.clone(),
            work_request_id: payload.work_request_id.clone(),
            corpus_id: payload.corpus_id.clone(),
            citations,
            authorized: authorized == AuthorizationResult::Allow,
        };

        let auth_payload = ContextAuthorizedPayload {
            domain_model_hash: payload.domain_model_hash,
            context_requirement_id: payload.context_requirement_id,
            work_request_id: payload.work_request_id,
            corpus_id: payload.corpus_id,
            result: authorized,
            reason: auth_reason,
            policy_code: "POL-ACL-002".to_string(),
            scope_claim: payload.scope_claim,
            evaluated_at: now.clone(),
        };

        let packet_envelope = EventEnvelope {
            event_id: format!("evt_{}", Ulid::new()),
            event_type: "ContextPacketCreated".to_string(),
            namespace: "agentic_capability_loop".to_string(),
            occurred_at: now.clone(),
            payload: serde_json::to_value(packet_payload)?,
        };
        let auth_envelope = EventEnvelope {
            event_id: format!("evt_{}", Ulid::new()),
            event_type: "ContextAuthorized".to_string(),
            namespace: "agentic_capability_loop".to_string(),
            occurred_at: now,
            payload: serde_json::to_value(auth_payload)?,
        };

        Ok((packet_envelope, auth_envelope))
    }
}

impl AclContextAgent {
    fn resolve_citations(
        &self,
        payload: &ContextRequiredPayload,
    ) -> Result<Vec<Citation>, anyhow::Error> {
        // Route learning-proposals corpus to JSONL ledger
        if payload.corpus_id.starts_with("learning-proposals") {
            return self.resolve_from_learning_proposals(payload);
        }

        // Resolve from filesystem corpus directory
        let corpus_dir = self.corpus_root.join(&payload.corpus_id);
        if !corpus_dir.is_dir() {
            return Ok(vec![]);
        }
        let config =
            serde_json::json!({ "path": corpus_dir.to_str().unwrap_or("") }).to_string();
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
        Ok(citations)
    }

    fn resolve_from_learning_proposals(
        &self,
        payload: &ContextRequiredPayload,
    ) -> Result<Vec<Citation>, anyhow::Error> {
        use std::io::{BufRead, BufReader};

        let ledger_path = match &self.learning_proposals_ledger {
            Some(p) => p.clone(),
            None => return Ok(vec![]),
        };
        if !ledger_path.exists() {
            return Ok(vec![]);
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
            let event: serde_json::Value = serde_json::from_str(line)?;
            let proposal_text = event["payload"]["proposal"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let domain_area = event["payload"]["domain_area"].as_str().unwrap_or("");

            if query.is_empty()
                || proposal_text.to_lowercase().contains(&query)
                || domain_area.to_lowercase().contains(&query)
            {
                citations.push(Citation {
                    citation_id: format!("cit_{}", Ulid::new()),
                    source: format!(
                        "learning-proposals://{}",
                        event["payload"]["learning_proposal_id"]
                            .as_str()
                            .unwrap_or("unknown")
                    ),
                    content: proposal_text,
                    fetched_at: now.clone(),
                });
            }
        }
        Ok(citations)
    }
}
