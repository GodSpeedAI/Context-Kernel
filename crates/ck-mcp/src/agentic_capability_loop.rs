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
    pub fn handle_context_required(
        &self,
        payload: &ContextRequiredPayload,
        authority_decision_id: Option<&str>,
        authority_reference: Option<&str>,
    ) -> Result<ContextPacketCreatedPayload, anyhow::Error> {
        let citations = self.resolve_citations(payload)?;

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

        // Containment: reject corpus_id that escapes corpus_root via traversal,
        // absolute paths, or symlinks. Canonicalize resolves symlinks.
        let canonical_root = match self.corpus_root.canonicalize() {
            Ok(p) => p,
            Err(_) => return Ok(vec![]), // corpus root not configured / missing
        };
        if !corpus_dir.is_dir() {
            return Ok(vec![]); // missing corpus, no citations (existing behavior)
        }
        let canonical_dir = match corpus_dir.canonicalize() {
            Ok(p) => p,
            Err(_) => return Ok(vec![]),
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
