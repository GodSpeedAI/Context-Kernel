//! Settlement-backed Context Effect calibration and the MCP tool boundary.
//!
//! Calibration is prospective retrieval tuning only: settlement-supported
//! evidence in, future retrieval influence out, history immutable, and an
//! agent's self-report is never evidence.

mod common;

use std::fs;

use ck_app::{ContextEffect, ContextEffectKind, ContextEffectStore};
use ck_core::KernelStatus;
use ck_mcp::agentic_capability_loop::AclContextAgent;
use ck_mcp::cep_bundle::{validate_context_bundle, verify_envelope_integrity};
use ck_mcp::McpService;
use common::test_kernel;
use serde_json::json;

const WORLD: &str =
    "world:test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn effect(kind: ContextEffectKind, source: &str) -> ContextEffect {
    ContextEffect {
        effect_id: format!("eff-{source}-{}", kind as u64),
        recorded_at: "2026-10-07T12:00:00Z".into(),
        kind,
        world_ref: WORLD.into(),
        work_request_ref: "wr_test".into(),
        bundle_envelope_id: "env-ctx_01TEST".into(),
        bundle_content_hash: format!("sha256:{}", "c".repeat(64)),
        affected_sources: vec![source.to_string()],
        evidence_ref: "settlement:e6_operational_settlement-001".into(),
        detail: Some("settlement showed the supplied context was contradicted".into()),
    }
}

fn seeded_kernel() -> (tempfile::TempDir, tempfile::TempDir, ck_app::AppServices) {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let docs = corpus.path().join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("a.md"), "deploy rotation runbook").unwrap();
    let app = test_kernel(kernel.path());
    app.ingest_source(ck_app::IngestSourceRequest {
        source_id: "ctxfs:docs".into(),
        source_kind: "filesystem_tree".into(),
        adapter_config_json: json!({ "path": docs.to_str().unwrap() }).to_string(),
        corpus_id: Some("docs".into()),
    })
    .unwrap();
    (corpus, kernel, app)
}

fn requirement_with_world() -> ck_app::ContextRequirement {
    serde_json::from_value::<ck_app::ContextRequirement>(json!({}))
        .ok();
    ck_app::ContextRequirement {
        identity: ck_app::RequirementIdentity {
            work_request_ref: "wr_test".into(),
            context_requirement_id: "cr_test".into(),
            task_ref: None,
            route_ref: None,
        },
        world_ref: WORLD.into(),
        purpose: None,
        query: Some("rotation".into()),
        mode: None,
        required_concepts: Vec::new(),
        corpora: vec!["docs".into()],
        constraints: Default::default(),
        freshness: Default::default(),
        max_results: 10,
        budget: None,
        expected_refs: Vec::new(),
        risk_classification: None,
        trace: Default::default(),
    }
}

#[test]
fn settlement_backed_effects_demote_future_retrieval_prospectively() {
    let (corpus, kernel, app) = seeded_kernel();
    let ledger = kernel.path().join("context-effects.jsonl");
    let app = app.with_context_effect_store(ContextEffectStore::new(&ledger));

    // BEFORE calibration: normal score.
    let before = app.resolve_context(&requirement_with_world()).unwrap();
    let before_score = before.results[0].score;

    // Ingest a settlement-backed effect against the source.
    let report = app
        .ingest_context_effects(&[effect(ContextEffectKind::SourceContradicted, "ctxfs:docs")])
        .unwrap();
    assert_eq!(report.accepted, 1);
    assert!(report.rejected.is_empty());

    // AFTER: the source is demoted — FUTURE retrieval only.
    let after = app.resolve_context(&requirement_with_world()).unwrap();
    let r = &after.results[0];
    assert!(
        r.score < before_score,
        "calibration demotes the flagged source: {before_score} -> {}",
        r.score
    );
    assert!(
        r.score_breakdown.get("calibration").is_some(),
        "the demotion is explained: {:?}",
        r.score_breakdown
    );

    // The historical bundle identity is recorded, never rewritten.
    let effects = ContextEffectStore::new(&ledger).load().unwrap();
    assert_eq!(effects[0].bundle_envelope_id, "env-ctx_01TEST");
    let _ = corpus;
}

#[test]
fn self_reports_are_never_evidence() {
    let (_corpus, kernel, app) = seeded_kernel();
    let app = app.with_context_effect_store(ContextEffectStore::new(kernel.path().join("effects.jsonl")));

    let mut bare = effect(ContextEffectKind::ContextItemUnused, "ctxfs:docs");
    bare.evidence_ref = "".into(); // "this context was useful" is not evidence
    let mut bogus = effect(ContextEffectKind::DistinctionMaterialToSettlement, "ctxfs:docs");
    bogus.evidence_ref = "agent-self-report:useful".into();

    let report = app.ingest_context_effects(&[bare, bogus]).unwrap();
    assert_eq!(report.accepted, 0);
    assert_eq!(report.rejected.len(), 2);
}

#[test]
fn effects_never_cross_worlds_and_reingest_is_idempotent() {
    let (_corpus, kernel, app) = seeded_kernel();
    let ledger = kernel.path().join("effects.jsonl");
    let app = app.with_context_effect_store(ContextEffectStore::new(&ledger));

    let mut other_world = effect(ContextEffectKind::SourceContradicted, "ctxfs:docs");
    other_world.world_ref = "world:other@sha256:".to_string() + &"d".repeat(64);
    app.ingest_context_effects(&[other_world]).unwrap();
    let res = app.resolve_context(&requirement_with_world()).unwrap();
    assert!(
        res.results[0].score_breakdown.get("calibration").is_none(),
        "another world's effects never touch this world's retrieval"
    );

    // Idempotent: re-ingesting the same effect_id appends nothing.
    let e = effect(ContextEffectKind::SourceStale, "ctxfs:docs");
    let first = app.ingest_context_effects(&[e.clone()]).unwrap();
    let second = app.ingest_context_effects(&[e]).unwrap();
    assert_eq!(first.accepted, 1);
    assert_eq!(second.skipped_duplicates, 1);
    assert_eq!(ContextEffectStore::new(&ledger).load().unwrap().len(), 2);
}

#[test]
fn effect_ingestion_refuses_unverifiable_identities() {
    let (_corpus, kernel, app) = seeded_kernel();
    let app = app.with_context_effect_store(ContextEffectStore::new(kernel.path().join("effects.jsonl")));
    let mut bad_world = effect(ContextEffectKind::SourceContradicted, "ctxfs:docs");
    bad_world.world_ref = "world:alias".into();
    let mut bad_hash = effect(ContextEffectKind::SourceContradicted, "ctxfs:docs");
    bad_hash.bundle_content_hash = "not-a-hash".into();
    let report = app.ingest_context_effects(&[bad_world, bad_hash]).unwrap();
    assert_eq!(report.accepted, 0);
    assert_eq!(report.rejected.len(), 2);
}

/// The full GodSpeed tool path: canonical CEP bundle out, legacy dual-read
/// fields retained, and CONTEXT ACCESS (never action authority) reported.
#[test]
fn tool_response_carries_the_canonical_bundle_and_access_semantics() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let docs = corpus.path().join("runbooks");
    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("blue.md"), "Blue-green rotation: canary, verify, promote.").unwrap();
    let app = test_kernel(kernel.path());
    let agent = AclContextAgent {
        corpus_root: corpus.path().to_path_buf(),
        learning_proposals_ledger: None,
    };
    let service = McpService::with_app(KernelStatus::placeholder(), app).with_context_agent(agent);

    let request = json!({
        "schema_version": "v1",
        "event_id": "11111111-2222-3333-4444-555555555555",
        "source_agent": "swe-seed",
        "event_type": "ContextRequired",
        "occurred_at": "2026-10-07T00:00:00Z",
        "payload": {
            "domain_model_hash": "a".repeat(64),
            "world_ref": WORLD,
            "work_request_id": "wr-tool",
            "context_requirement_id": "cr-tool",
            "corpus_id": "runbooks",
            "query": "rotation",
            "max_results": 5,
        },
        "required": true,
    });
    let resp = service.call_tool("context_required", request).unwrap();

    // Canonical artifact: a structurally valid, integrity-stamped CEP bundle.
    let bundle = &resp["context_bundle"];
    validate_context_bundle(bundle).expect("bundle satisfies the profile rules");
    verify_envelope_integrity(bundle).expect("bundle integrity self-verifies");
    assert_eq!(bundle["scope"]["world_ref"], WORLD);
    assert_eq!(bundle["envelope_kind"], "context_bundle");
    assert_eq!(
        bundle["extensions"]["cep.profile"]["profile_id"],
        "godspeed.context_bundle"
    );

    // Context ACCESS, not action authority: accessible/inaccessible/requires_scope.
    assert_eq!(resp["access"]["result"], "accessible");
    assert_eq!(resp["outcome"], "cited");
    // Legacy dual-read fields remain during the bounded migration window.
    assert!(resp["context_envelope"].is_object());
    assert!(resp["context_packet"].is_object());
}

#[test]
fn tool_reports_requires_scope_for_private_corpora_without_leaking() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let docs = corpus.path().join("secrets");
    fs::create_dir_all(&docs).unwrap();
    fs::write(docs.join("s.md"), "classified deployment notes").unwrap();
    let app = test_kernel(kernel.path());
    use ck_core::ports::StoragePort;
    app.ingest_source(ck_app::IngestSourceRequest {
        source_id: "ctxfs:secrets".into(),
        source_kind: "filesystem_tree".into(),
        adapter_config_json: json!({ "path": docs.to_str().unwrap() }).to_string(),
        corpus_id: Some("secrets".into()),
    })
    .unwrap();
    // Register the corpus PRIVATE through real storage state.
    let store = ck_storage_sqlite::SqliteStore::open(kernel.path().join("ck.db")).unwrap();
    let mut corp = store.get_corpus("secrets").unwrap().unwrap();
    corp.visibility = "private".into();
    store.upsert_corpus(&corp).unwrap();
    // Re-ingestion must NEVER open a registered private corpus.
    app.ingest_source(ck_app::IngestSourceRequest {
        source_id: "ctxfs:secrets".into(),
        source_kind: "filesystem_tree".into(),
        adapter_config_json: json!({ "path": docs.to_str().unwrap() }).to_string(),
        corpus_id: Some("secrets".into()),
    })
    .unwrap();
    assert_eq!(
        store.get_corpus("secrets").unwrap().unwrap().visibility,
        "private",
        "re-ingestion must not open a private corpus"
    );

    let agent = AclContextAgent {
        corpus_root: corpus.path().to_path_buf(),
        learning_proposals_ledger: None,
    };
    let service = McpService::with_app(KernelStatus::placeholder(), app).with_context_agent(agent);
    let request = json!({
        "schema_version": "v1",
        "event_id": "22222222-2222-3333-4444-555555555555",
        "source_agent": "swe-seed",
        "event_type": "ContextRequired",
        "occurred_at": "2026-10-07T00:00:00Z",
        "payload": {
            "domain_model_hash": "a".repeat(64),
            "world_ref": WORLD,
            "work_request_id": "wr-priv",
            "context_requirement_id": "cr-priv",
            "corpus_id": "secrets",
            "query": "classified",
            "max_results": 5,
        },
        "required": true,
    });
    let resp = service.call_tool("context_required", request).unwrap();

    assert_eq!(resp["access"]["result"], "requires_scope");
    assert_eq!(resp["outcome"], "no_context_governed");
    let bundle = &resp["context_bundle"];
    validate_context_bundle(bundle).expect("a none bundle is still a valid, honest bundle");
    assert_eq!(bundle["completeness_status"], "minimal");
    assert!(bundle.get("references").is_none(), "no content leaks");
}
