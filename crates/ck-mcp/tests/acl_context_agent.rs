//! Integration tests for AclContextAgent — Context Kernel's retrieval-only
//! adapter. CK resolves citations and returns a packet with authority
//! references; it does NOT create authority decisions.

use std::fs;

use ck_mcp::agentic_capability_loop::{AclContextAgent, ContextRequiredPayload};
use serde_json::json;

fn make_payload(corpus_id: &str) -> ContextRequiredPayload {
    ContextRequiredPayload {
        domain_model_hash: "deadbeef".to_string(),
        work_request_id: "wr_test".to_string(),
        context_requirement_id: "cr_test".to_string(),
        corpus_id: corpus_id.to_string(),
        query: None,
        max_results: 10,
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

    let payload = make_payload(corpus_id);
    let packet = agent.handle_context_required(&payload, None, None).unwrap();

    let packet_val = serde_json::to_value(&packet).unwrap();
    assert_eq!(packet_val["citations"].as_array().unwrap().len(), 1);
    assert!(packet_val["citations"][0]["content"]
        .as_str()
        .unwrap()
        .contains("Hello world"));
}

#[test]
fn acl_agent_passes_authority_references_through() {
    let dir = tempfile::tempdir().unwrap();
    let agent = AclContextAgent {
        corpus_root: dir.path().to_path_buf(),
        learning_proposals_ledger: None,
    };

    let payload = make_payload("nonexistent");
    let packet = agent
        .handle_context_required(
            &payload,
            Some("sea-decision-001"),
            Some("AuthorityChecked#evt_abc"),
        )
        .unwrap();

    assert_eq!(
        packet.authority_decision_id,
        Some("sea-decision-001".into())
    );
    assert_eq!(
        packet.authority_reference,
        Some("AuthorityChecked#evt_abc".into())
    );
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
    };
    let packet = agent.handle_context_required(&payload, None, None).unwrap();

    let packet_val = serde_json::to_value(&packet).unwrap();
    let citations = packet_val["citations"].as_array().unwrap();
    assert_eq!(citations.len(), 1);
    assert!(citations[0]["content"]
        .as_str()
        .unwrap()
        .contains("table-driven tests"));
    assert!(citations[0]["source"]
        .as_str()
        .unwrap()
        .contains("learning-proposals://lp_001"));
}
