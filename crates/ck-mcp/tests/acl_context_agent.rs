//! Integration tests for AclContextAgent — the Context Kernel's concrete
//! ContextAgent implementation.  Covers filesystem citation resolution,
//! POL-ACL-002 enforcement, and learning-proposals JSONL routing.

use std::fs;

use ck_mcp::agentic_capability_loop::{
    AclContextAgent, ContextAgent, ContextRequiredPayload, EventEnvelope,
};
use serde_json::json;

fn make_envelope(payload: ContextRequiredPayload) -> EventEnvelope {
    EventEnvelope {
        event_id: "evt_test_001".to_string(),
        event_type: "ContextRequired".to_string(),
        namespace: "agentic_capability_loop".to_string(),
        occurred_at: "2026-06-10T00:00:00Z".to_string(),
        payload: serde_json::to_value(payload).unwrap(),
    }
}

fn make_payload(
    corpus_id: &str,
    is_private: bool,
    scope_claim: Option<&str>,
) -> ContextRequiredPayload {
    ContextRequiredPayload {
        domain_model_hash: "deadbeef".to_string(),
        work_request_id: "wr_test".to_string(),
        context_requirement_id: "cr_test".to_string(),
        corpus_id: corpus_id.to_string(),
        query: None,
        max_results: 10,
        is_private,
        scope_claim: scope_claim.map(|s| s.to_string()),
    }
}

#[test]
fn acl_agent_resolves_citations_from_filesystem_corpus() {
    let dir = tempfile::tempdir().unwrap();
    let corpus_id = "test-corpus";
    let corpus_dir = dir.path().join(corpus_id);
    fs::create_dir_all(&corpus_dir).unwrap();
    fs::write(corpus_dir.join("doc1.md"), "Hello world from doc1").unwrap();

    let agent = AclContextAgent {
        corpus_root: dir.path().to_path_buf(),
        learning_proposals_ledger: None,
    };

    let payload = make_payload(corpus_id, false, None);
    let envelope = make_envelope(payload);
    let (packet, auth) = agent.handle_context_required(&envelope).unwrap();

    // ContextPacketCreated: authorized, has citations
    let packet_payload = &packet.payload;
    assert!(packet_payload["authorized"].as_bool().unwrap());
    assert_eq!(packet_payload["citations"].as_array().unwrap().len(), 1);
    assert!(packet_payload["citations"][0]["content"]
        .as_str()
        .unwrap()
        .contains("Hello world"));

    // ContextAuthorized: Allow
    assert_eq!(auth.payload["result"].as_str().unwrap(), "allow");
    assert!(auth.payload["reason"].as_str().unwrap().contains("public"));
}

#[test]
fn acl_agent_denies_private_corpus_without_scope_claim() {
    let dir = tempfile::tempdir().unwrap();

    let agent = AclContextAgent {
        corpus_root: dir.path().to_path_buf(),
        learning_proposals_ledger: None,
    };

    let payload = make_payload("secret-corpus", true, None);
    let envelope = make_envelope(payload);
    let (packet, auth) = agent.handle_context_required(&envelope).unwrap();

    // ContextPacketCreated: NOT authorized, no citations
    assert!(!packet.payload["authorized"].as_bool().unwrap());
    assert!(packet.payload["citations"].as_array().unwrap().is_empty());

    // ContextAuthorized: Deny
    assert_eq!(auth.payload["result"].as_str().unwrap(), "deny");
    assert!(auth.payload["reason"]
        .as_str()
        .unwrap()
        .contains("POL-ACL-002"));
}

#[test]
fn acl_agent_allows_private_corpus_with_scope_claim() {
    let dir = tempfile::tempdir().unwrap();
    let corpus_id = "private-corpus";
    let corpus_dir = dir.path().join(corpus_id);
    fs::create_dir_all(&corpus_dir).unwrap();
    fs::write(corpus_dir.join("secret.md"), "classified info").unwrap();

    let agent = AclContextAgent {
        corpus_root: dir.path().to_path_buf(),
        learning_proposals_ledger: None,
    };

    let payload = make_payload(corpus_id, true, Some("scope:confidential"));
    let envelope = make_envelope(payload);
    let (packet, auth) = agent.handle_context_required(&envelope).unwrap();

    // ContextPacketCreated: authorized, has citations
    assert!(packet.payload["authorized"].as_bool().unwrap());
    assert_eq!(packet.payload["citations"].as_array().unwrap().len(), 1);
    assert!(packet.payload["citations"][0]["content"]
        .as_str()
        .unwrap()
        .contains("classified info"));

    // ContextAuthorized: Allow
    assert_eq!(auth.payload["result"].as_str().unwrap(), "allow");
    assert!(auth.payload["reason"]
        .as_str()
        .unwrap()
        .contains("scope claim present"));
}

#[test]
fn acl_agent_resolves_citations_from_learning_proposals_ledger() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join("agentic_learning_proposals.jsonl");

    let ledger_entry = json!({
        "event_id": "evt_lp_001",
        "event_type": "LearningProposalCreated",
        "namespace": "agentic_capability_loop",
        "occurred_at": "2026-06-10T00:00:00Z",
        "payload": {
            "learning_proposal_id": "lp_001",
            "domain_area": "testing",
            "proposal": "Prefer table-driven tests for enum variants"
        }
    });
    let line = serde_json::to_string(&ledger_entry).unwrap();
    fs::write(&ledger_path, line).unwrap();

    let agent = AclContextAgent {
        corpus_root: dir.path().to_path_buf(),
        learning_proposals_ledger: Some(ledger_path),
    };

    let payload = ContextRequiredPayload {
        domain_model_hash: "deadbeef".to_string(),
        work_request_id: "wr_test".to_string(),
        context_requirement_id: "cr_test".to_string(),
        corpus_id: "learning-proposals".to_string(),
        query: Some("testing".to_string()),
        max_results: 10,
        is_private: false,
        scope_claim: None,
    };
    let envelope = make_envelope(payload);
    let (packet, auth) = agent.handle_context_required(&envelope).unwrap();

    // Should resolve from JSONL ledger
    assert!(packet.payload["authorized"].as_bool().unwrap());
    let citations = packet.payload["citations"].as_array().unwrap();
    assert_eq!(citations.len(), 1);
    assert!(citations[0]["content"]
        .as_str()
        .unwrap()
        .contains("table-driven tests"));
    assert!(citations[0]["source"]
        .as_str()
        .unwrap()
        .contains("learning-proposals://lp_001"));

    // Authorized (public corpus)
    assert_eq!(auth.payload["result"].as_str().unwrap(), "allow");
}
