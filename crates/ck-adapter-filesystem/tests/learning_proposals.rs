use ck_adapter_filesystem::LearningProposalsAdapter;
use ck_core::ports::SourceAdapter;
use serde_json::json;

#[test]
fn learning_proposals_adapter_exposes_jsonl_records_as_source_units() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join("agentic_learning_proposals.jsonl");
    let proposal_one = json!({
        "event_id": "evt_1",
        "event_type": "LearningProposalCreated",
        "payload": {
            "learning_proposal_id": "lp_1",
            "domain_area": "agentic_capability_loop",
            "proposal": "Investigate missing context packet",
            "status": "open"
        }
    });
    let proposal_two = json!({
        "event_id": "evt_2",
        "event_type": "LearningProposalCreated",
        "payload": {
            "learning_proposal_id": "lp_2",
            "domain_area": "agentic_capability_loop",
            "proposal": "Review stale proof completion policy",
            "status": "open"
        }
    });
    std::fs::write(
        &ledger_path,
        format!("{}\n{}\n", proposal_one, proposal_two),
    )
    .unwrap();

    let adapter = LearningProposalsAdapter;
    let config = json!({ "path": ledger_path }).to_string();

    let units = adapter
        .discover_units("learning-proposals", &config)
        .unwrap();

    assert_eq!(units.len(), 2);
    assert_eq!(units[0].external_id, "lp_1");
    assert!(units[0]
        .uri
        .as_deref()
        .unwrap()
        .contains("learning-proposals://lp_1"));
}

#[test]
fn learning_proposals_adapter_fetches_proposal_as_json_content() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join("agentic_learning_proposals.jsonl");
    let proposal = json!({
        "event_id": "evt_1",
        "event_type": "LearningProposalCreated",
        "payload": {
            "learning_proposal_id": "lp_1",
            "domain_area": "agentic_capability_loop",
            "proposal": "Investigate missing context packet",
            "status": "open"
        }
    });
    std::fs::write(&ledger_path, format!("{}\n", proposal)).unwrap();

    let adapter = LearningProposalsAdapter;
    let config = json!({ "path": ledger_path }).to_string();
    let unit = adapter
        .discover_units("learning-proposals", &config)
        .unwrap()
        .remove(0);

    let raw = adapter
        .fetch_content("learning-proposals", &unit, &config)
        .unwrap();
    let raw_json: serde_json::Value = serde_json::from_slice(&raw.bytes).unwrap();

    assert_eq!(raw.external_id, "lp_1");
    assert_eq!(raw.mime_type, "application/json");
    assert_eq!(
        raw_json["payload"]["proposal"],
        "Investigate missing context packet"
    );
}

#[test]
fn learning_proposals_adapter_queries_by_domain_area() {
    let dir = tempfile::tempdir().unwrap();
    let ledger_path = dir.path().join("agentic_learning_proposals.jsonl");
    let proposal_one = json!({
        "event_id": "evt_1",
        "event_type": "LearningProposalCreated",
        "payload": {
            "learning_proposal_id": "lp_1",
            "domain_area": "agentic_capability_loop",
            "proposal": "Investigate missing context packet",
            "status": "open"
        }
    });
    let proposal_two = json!({
        "event_id": "evt_2",
        "event_type": "LearningProposalCreated",
        "payload": {
            "learning_proposal_id": "lp_2",
            "domain_area": "agentic_capability_loop",
            "proposal": "Review stale proof completion policy",
            "status": "open"
        }
    });
    let unrelated = json!({
        "event_id": "evt_3",
        "event_type": "LearningProposalCreated",
        "payload": {
            "learning_proposal_id": "lp_3",
            "domain_area": "other",
            "proposal": "Other domain",
            "status": "open"
        }
    });
    std::fs::write(
        &ledger_path,
        format!("{}\n{}\n{}\n", proposal_one, proposal_two, unrelated),
    )
    .unwrap();

    let adapter = LearningProposalsAdapter;
    let config = json!({ "path": ledger_path }).to_string();

    let units = adapter
        .query_by_domain_area(&config, "agentic_capability_loop")
        .unwrap();

    assert_eq!(units.len(), 2);
    assert_eq!(units[0].external_id, "lp_1");
    assert_eq!(units[1].external_id, "lp_2");
}
