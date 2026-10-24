//! Integration tests for AclContextAgent — Context Kernel's thin GodSpeed
//! adapter. Every acquisition routes through the real application retrieval
//! pipeline (`AppServices::resolve_context`); the agent only resolves corpora,
//! enforces the boundary, and reports access — it does NOT create authority
//! decisions (SEA-Forge owns those).

mod common;

use std::fs;

use ck_mcp::agentic_capability_loop::{
    verify_world_ref, AclContextAgent, ContextRequiredPayload, RetrievalCompleteness,
};
use common::test_kernel;
use serde_json::json;

fn payload(corpus_id: &str) -> ContextRequiredPayload {
    serde_json::from_value(common::required_payload(corpus_id, None)).unwrap()
}

fn agent_over(corpus_root: &std::path::Path) -> AclContextAgent {
    AclContextAgent {
        corpus_root: corpus_root.to_path_buf(),
        learning_proposals_ledger: None,
    }
}

fn corpus_with(corpus_root: &std::path::Path, corpus_id: &str, files: &[(&str, &str)]) {
    let root = corpus_root.join(corpus_id);
    fs::create_dir_all(&root).unwrap();
    for (name, text) in files {
        fs::write(root.join(name), *text).unwrap();
    }
}

fn cited(acq: &ck_mcp::agentic_capability_loop::ContextAcquisitionResult) -> bool {
    acq.no_context.is_none()
}

#[test]
fn acl_agent_resolves_citations_through_the_real_pipeline() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    corpus_with(
        corpus.path(),
        "test-corpus",
        &[("doc1.md", "# Doc1\nHello world from doc1")],
    );
    let app = test_kernel(kernel.path());
    let agent = agent_over(corpus.path());

    let acq = agent
        .acquire_context(&app, &payload("test-corpus"), None, None)
        .unwrap();

    assert!(
        cited(&acq),
        "the corpus has matching content: {:?}",
        acq.no_context
    );
    assert_eq!(acq.packet.citations.len(), 1);
    assert!(acq.packet.citations[0].content.contains("Hello world"));
    // Real pipeline identity: the resolution carries chunk/document identity.
    assert!(acq.resolution.results[0].chunk_id.is_some());
    assert!(acq.resolution.results[0].document_id.is_some());
    assert_eq!(acq.resolution.completeness, RetrievalCompleteness::Complete);
}

#[test]
fn acl_agent_passes_authority_references_through() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let app = test_kernel(kernel.path());
    let agent = agent_over(corpus.path());

    let acq = agent
        .acquire_context(
            &app,
            &payload("nonexistent"),
            Some("sea-decision-001"),
            Some("AuthorityChecked#evt_abc"),
        )
        .unwrap();

    assert_eq!(
        acq.packet.authority_decision_id,
        Some("sea-decision-001".into())
    );
    assert_eq!(
        acq.packet.authority_reference,
        Some("AuthorityChecked#evt_abc".into())
    );
    assert!(!cited(&acq));
}

#[test]
fn acl_agent_resolves_citations_from_learning_proposals_ledger() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let ledger_path = corpus.path().join("agentic_learning_proposals.jsonl");

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
    fs::write(&ledger_path, serde_json::to_string(&ledger_entry).unwrap()).unwrap();

    let app = test_kernel(kernel.path());
    let agent = AclContextAgent {
        corpus_root: corpus.path().to_path_buf(),
        learning_proposals_ledger: Some(ledger_path),
    };
    let mut p = payload("learning-proposals");
    p.query = Some("table-driven tests".to_string());

    let acq = agent.acquire_context(&app, &p, None, None).unwrap();

    assert!(
        cited(&acq),
        "the ledger has a matching proposal: {:?}",
        acq.no_context
    );
    assert!(
        acq.packet.citations[0]
            .source
            .starts_with("learning-proposals://lp_001"),
        "citation keeps the ledger identity: {:?}",
        acq.packet.citations[0].source
    );
}

#[test]
fn result_cap_marks_partial_with_a_machine_readable_omission() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    corpus_with(
        corpus.path(),
        "c",
        &[
            ("a.md", "alpha shared"),
            ("b.md", "beta shared"),
            ("c.md", "gamma shared"),
        ],
    );
    let app = test_kernel(kernel.path());
    let mut p = payload("c");
    p.query = Some("shared".into());
    p.max_results = 2;

    let acq = agent_over(corpus.path())
        .acquire_context(&app, &p, None, None)
        .unwrap();

    assert!(cited(&acq));
    assert_eq!(acq.resolution.results.len(), 2);
    assert_eq!(acq.resolution.completeness, RetrievalCompleteness::Partial);
    assert_eq!(acq.packet.omissions, vec!["max_results_reached"]);
}

#[test]
fn cap_exactly_met_with_no_further_match_stays_complete() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    corpus_with(
        corpus.path(),
        "c",
        &[("a.md", "alpha shared"), ("b.md", "beta shared")],
    );
    let app = test_kernel(kernel.path());
    let mut p = payload("c");
    p.query = Some("shared".into());
    p.max_results = 2;

    let acq = agent_over(corpus.path())
        .acquire_context(&app, &p, None, None)
        .unwrap();

    assert_eq!(acq.resolution.completeness, RetrievalCompleteness::Complete);
}

#[test]
fn excerpt_truncation_is_a_reference_limitation_not_a_retrieval_omission() {
    // The declared retrieval contract is ranked excerpts with resolvable
    // locators: a bounded excerpt is surfaced as `content_truncated` on the
    // reference (and the legacy citation flag), never silently dropped — and
    // it does not by itself make the retrieval incomplete.
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    corpus_with(corpus.path(), "c", &[("long.md", &"x".repeat(900))]);
    let app = test_kernel(kernel.path());

    let acq = agent_over(corpus.path())
        .acquire_context(&app, &payload("c"), None, None)
        .unwrap();

    assert!(acq.packet.citations[0].truncated);
    assert_eq!(acq.resolution.completeness, RetrievalCompleteness::Complete);
}

#[test]
fn no_citations_is_none_with_an_explicit_reason() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let app = test_kernel(kernel.path());

    let acq = agent_over(corpus.path())
        .acquire_context(&app, &payload("missing"), None, None)
        .unwrap();

    assert!(!cited(&acq));
    assert_eq!(acq.resolution.completeness, RetrievalCompleteness::None);
    assert!(acq.resolution.no_context_reason.is_some());
    assert_eq!(acq.packet.citations.len(), 0);
    assert!(
        !acq.packet.omissions.is_empty(),
        "absence is always explained"
    );
}

#[test]
fn unpinned_world_refs_are_refused_before_any_retrieval() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    corpus_with(corpus.path(), "c", &[("a.md", "alpha")]);
    let app = test_kernel(kernel.path());
    for bad in [
        "",
        "world:test",
        "world:test@sha256:abc",
        "test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "world:Test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "world:test@sha256:BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
    ] {
        assert!(verify_world_ref(bad).is_err(), "accepted {bad:?}");
        let mut p = payload("c");
        p.world_ref = bad.to_string();
        assert!(agent_over(corpus.path())
            .acquire_context(&app, &p, None, None)
            .is_err());
    }
}

#[test]
fn corpus_id_traversal_is_a_hard_refusal() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let corpus_root = corpus.path().join("root");
    fs::create_dir_all(&corpus_root).unwrap();
    // Sibling dir OUTSIDE corpus_root containing a secret file.
    let sibling = corpus.path().join("sibling");
    fs::create_dir_all(&sibling).unwrap();
    fs::write(sibling.join("secret.md"), "TOP SECRET CONTENT").unwrap();
    let app = test_kernel(kernel.path());

    let mut p = payload("c");
    p.corpus_id = "../sibling".into();
    let err = agent_over(&corpus_root).acquire_context(&app, &p, None, None);
    assert!(err.is_err(), "traversal corpus_id must be refused: {err:?}");
}

#[test]
fn a_declared_private_corpus_is_never_auto_ingested_open() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    corpus_with(corpus.path(), "secrets", &[("s.md", "private content")]);
    let app = test_kernel(kernel.path());
    let mut p = payload("secrets");
    p.is_private = true;

    let acq = agent_over(corpus.path())
        .acquire_context(&app, &p, None, None)
        .unwrap();
    assert!(!cited(&acq));
    assert!(
        app.corpus("secrets").unwrap().is_none(),
        "must not be registered"
    );
}
