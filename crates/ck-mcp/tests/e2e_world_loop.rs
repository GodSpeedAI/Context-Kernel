//! End-to-end world loop, Context Kernel's hop (driven by sea-rs
//! `scripts/e2e-world-loop.sh`). Reads the real E1 from `E2E_DIR`, performs a
//! REAL retrieval through the application pipeline, and writes the canonical
//! E3 envelope plus its CEP `context_bundle` projection (integrity-stamped),
//! all pinned to the same `E2E_WORLD_REF`. Skipped unless the driver sets
//! `E2E_DIR`.

mod common;

use std::{fs, path::PathBuf};

use ck_mcp::agentic_capability_loop::{
    context_packet_envelope, AclContextAgent, ContextAcquisitionResult, ContextRequiredPayload,
    RetrievalCompleteness,
};
use ck_mcp::cep_bundle::{
    project_context_bundle, validate_context_bundle, verify_envelope_integrity, BundleInput,
};
use common::test_kernel;
use serde_json::{json, Value};

#[test]
fn hop_0_context_packet_for_the_requested_work() {
    let Some(dir) = std::env::var_os("E2E_DIR").map(PathBuf::from) else {
        eprintln!("SKIP: E2E_DIR not set");
        return;
    };
    let world = std::env::var("E2E_WORLD_REF").expect("E2E_WORLD_REF");
    let hash = std::env::var("E2E_DOMAIN_HASH").expect("E2E_DOMAIN_HASH");

    let e1: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("e1_work_requested.json")).unwrap())
            .unwrap();
    assert_eq!(
        e1["payload"]["world_ref"],
        world.as_str(),
        "E1 must name the pinned world"
    );
    let work_request_id = e1["payload"]["work_request_id"]
        .as_str()
        .unwrap()
        .to_string();
    let e1_event_id = e1["event_id"].as_str().unwrap().to_string();

    // A real corpus on disk, retrieved through the REAL application pipeline.
    let corpus = tempfile::tempdir().unwrap();
    fs::create_dir_all(corpus.path().join("runbooks")).unwrap();
    fs::write(
        corpus.path().join("runbooks/blue-green.md"),
        "Blue-green rotation: canary 10%, verify health, promote.",
    )
    .unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let app = test_kernel(kernel.path());
    let agent = AclContextAgent {
        corpus_root: corpus.path().to_path_buf(),
        learning_proposals_ledger: None,
    };
    let required: ContextRequiredPayload = serde_json::from_value(json!({
        "domain_model_hash": hash,
        "world_ref": world.clone(),
        "work_request_id": work_request_id,
        "context_requirement_id": "cr-e2e-001",
        "corpus_id": "runbooks",
        "max_results": 10,
    }))
    .unwrap();
    let ContextAcquisitionResult {
        packet,
        resolution,
        no_context,
    } = agent.acquire_context(&app, &required, None, None).unwrap();
    assert!(no_context.is_none(), "the corpus has a matching document");
    assert_eq!(
        packet.retrieval_completeness,
        RetrievalCompleteness::Complete
    );
    assert_eq!(packet.world_ref, world);

    // Legacy E3 envelope (dual-read compatibility for existing consumers).
    let e3 = context_packet_envelope(Some(&e1_event_id), &packet);
    assert_eq!(e3["payload"]["world_ref"], world.as_str());
    fs::write(
        dir.join("e3_context_packet.json"),
        serde_json::to_string_pretty(&e3).unwrap() + "\n",
    )
    .unwrap();

    // The canonical cross-system artifact: a CEP godspeed.context_bundle.
    let requirement = required.to_requirement();
    let bundle = project_context_bundle(&BundleInput {
        requirement: &requirement,
        resolution: &resolution,
        packet: &packet,
        request_event_id: Some(&e1_event_id),
        created_at: "2026-10-05T12:00:00Z",
    })
    .unwrap();
    validate_context_bundle(&bundle).expect("the bundle satisfies godspeed.context_bundle");
    verify_envelope_integrity(&bundle).expect("the bundle integrity self-verifies");
    assert_eq!(bundle["scope"]["world_ref"], world.as_str());
    assert_eq!(bundle["completeness_status"], "complete_for_context");
    assert_eq!(
        bundle["extensions"]["godspeed.context_bundle"]["work_request_ref"],
        work_request_id.as_str()
    );
    let refs = bundle["references"].as_array().unwrap();
    assert!(!refs.is_empty(), "what was made available is referenced");
    assert!(refs[0]["target_uri_or_id"].as_str().is_some(), "locator");
    fs::write(
        dir.join("e3_context_bundle.cep.json"),
        serde_json::to_string_pretty(&bundle).unwrap() + "\n",
    )
    .unwrap();
}
