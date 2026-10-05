//! Integration tests for AclContextAgent — Context Kernel's retrieval-only
//! adapter. CK resolves citations and returns a packet with authority
//! references; it does NOT create authority decisions.

use std::fs;

use ck_mcp::agentic_capability_loop::{
    context_packet_envelope, verify_world_ref, AclContextAgent, ContextAcquisition,
    ContextRequiredPayload, RetrievalCompleteness,
};
use serde_json::json;

const WORLD: &str =
    "world:test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn make_payload(corpus_id: &str) -> ContextRequiredPayload {
    ContextRequiredPayload {
        domain_model_hash: "deadbeef".to_string(),
        world_ref: WORLD.to_string(),
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
        world_ref: WORLD.to_string(),
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

fn agent_over(dir: &tempfile::TempDir) -> AclContextAgent {
    AclContextAgent {
        corpus_root: dir.path().to_path_buf(),
        learning_proposals_ledger: None,
    }
}

fn corpus_with(dir: &tempfile::TempDir, files: &[(&str, String)]) {
    let corpus_dir = dir.path().join("c");
    fs::create_dir_all(&corpus_dir).unwrap();
    for (name, body) in files {
        fs::write(corpus_dir.join(name), body).unwrap();
    }
}

fn cited(acq: ContextAcquisition) -> ck_mcp::agentic_capability_loop::ContextPacketCreatedPayload {
    match acq {
        ContextAcquisition::Cited(p) => p,
        ContextAcquisition::NoContext(..) => panic!("expected cited"),
    }
}

#[test]
fn world_ref_is_echoed_unchanged_and_in_the_envelope() {
    let dir = tempfile::tempdir().unwrap();
    corpus_with(&dir, &[("a.md", "alpha".into())]);
    let packet = cited(
        agent_over(&dir)
            .acquire_context(&make_payload("c"), None, None)
            .unwrap(),
    );
    assert_eq!(packet.world_ref, WORLD);
    let env = context_packet_envelope(Some("evt_req"), &packet);
    assert_eq!(env["payload"]["world_ref"], WORLD);
    assert_eq!(env["payload"]["retrieval_completeness"], "complete");
}

#[test]
fn full_corpus_within_cap_is_complete() {
    let dir = tempfile::tempdir().unwrap();
    corpus_with(&dir, &[("a.md", "alpha".into()), ("b.md", "beta".into())]);
    let packet = cited(
        agent_over(&dir)
            .acquire_context(&make_payload("c"), None, None)
            .unwrap(),
    );
    assert_eq!(
        packet.retrieval_completeness,
        RetrievalCompleteness::Complete
    );
    assert!(packet.omissions.is_empty());
}

#[test]
fn hitting_the_cap_with_more_matches_is_partial_not_complete() {
    let dir = tempfile::tempdir().unwrap();
    corpus_with(
        &dir,
        &[
            ("a.md", "alpha".into()),
            ("b.md", "beta".into()),
            ("c.md", "gamma".into()),
        ],
    );
    let mut payload = make_payload("c");
    payload.max_results = 2;
    let packet = cited(
        agent_over(&dir)
            .acquire_context(&payload, None, None)
            .unwrap(),
    );
    assert_eq!(packet.citations.len(), 2);
    assert_eq!(
        packet.retrieval_completeness,
        RetrievalCompleteness::Partial
    );
    assert_eq!(packet.omissions, vec!["max_results_reached"]);
}

#[test]
fn cap_exactly_met_with_no_further_match_stays_complete() {
    let dir = tempfile::tempdir().unwrap();
    corpus_with(&dir, &[("a.md", "alpha".into()), ("b.md", "beta".into())]);
    let mut payload = make_payload("c");
    payload.max_results = 2;
    let packet = cited(
        agent_over(&dir)
            .acquire_context(&payload, None, None)
            .unwrap(),
    );
    assert_eq!(
        packet.retrieval_completeness,
        RetrievalCompleteness::Complete
    );
}

#[test]
fn truncated_citation_content_is_partial() {
    let dir = tempfile::tempdir().unwrap();
    corpus_with(&dir, &[("long.md", "x".repeat(900))]);
    let packet = cited(
        agent_over(&dir)
            .acquire_context(&make_payload("c"), None, None)
            .unwrap(),
    );
    assert!(packet.citations[0].truncated);
    assert_eq!(
        packet.retrieval_completeness,
        RetrievalCompleteness::Partial
    );
    assert_eq!(packet.omissions, vec!["citation_content_truncated"]);
}

#[test]
fn no_citations_is_none() {
    let dir = tempfile::tempdir().unwrap();
    let acq = agent_over(&dir)
        .acquire_context(&make_payload("missing"), None, None)
        .unwrap();
    match acq {
        ContextAcquisition::NoContext(p, _) => {
            assert_eq!(p.retrieval_completeness, RetrievalCompleteness::None)
        }
        ContextAcquisition::Cited(_) => panic!("expected no context"),
    }
}

#[test]
fn unpinned_world_refs_are_refused_before_any_retrieval() {
    let dir = tempfile::tempdir().unwrap();
    corpus_with(&dir, &[("a.md", "alpha".into())]);
    for bad in [
        "",
        "world:test",
        "world:test@sha256:abc",
        "test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "world:Test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "world:test@sha256:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
    ] {
        assert!(verify_world_ref(bad).is_err(), "accepted {bad:?}");
        let mut payload = make_payload("c");
        payload.world_ref = bad.to_string();
        assert!(agent_over(&dir)
            .acquire_context(&payload, None, None)
            .is_err());
    }
}

#[test]
fn world_ref_vectors_match_the_cep_pattern() {
    let d = "b".repeat(64);
    for ok in [
        format!("world:a@sha256:{d}"),
        format!("world:acme.core-v2_x@sha256:{d}"),
        format!("world:{}@sha256:{d}", "a".repeat(64)),
    ] {
        assert!(verify_world_ref(&ok).is_ok(), "rejected {ok:?}");
    }
    for bad in [
        format!("world:{}@sha256:{d}", "a".repeat(65)),
        format!("world:a-@sha256:{d}"),
        format!("world:a..b@sha256:{d}"),
        format!("world:1a@sha256:{d}"),
        format!("world:a@sha256:{d}0"),
    ] {
        assert!(verify_world_ref(&bad).is_err(), "accepted {bad:?}");
    }
}
