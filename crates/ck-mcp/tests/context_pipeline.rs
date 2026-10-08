//! Retrieval-behavior conformance for the GodSpeed context path: the real
//! application retrieval pipeline (lexical + semantic-when-configured), corpus
//! filtering, private-source scope enforcement, budgets, freshness,
//! deterministic references, preserved identity, and truthful
//! complete/partial/none accounting.

mod common;

use std::fs;

use ck_app::{ContextAccess, ContextRequirement, RequirementIdentity, SourceConstraints};
use ck_core::entities::{Budget, Corpus};
use ck_core::ports::StoragePort;
use ck_core::types::{NoContextReason, RetrievalCompleteness};
use common::{test_kernel_with_store, StubEmbedding, StubVectorSearch};
use serde_json::json;

const WORLD: &str =
    "world:test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn requirement(corpus_id: &str, query: Option<&str>) -> ContextRequirement {
    ContextRequirement {
        identity: RequirementIdentity {
            work_request_ref: "wr_test".into(),
            context_requirement_id: "cr_test".into(),
            task_ref: None,
            route_ref: None,
        },
        world_ref: WORLD.into(),
        purpose: Some("test retrieval".into()),
        query: query.map(str::to_string),
        mode: None,
        required_concepts: Vec::new(),
        corpora: vec![corpus_id.into()],
        constraints: SourceConstraints::default(),
        freshness: Default::default(),
        max_results: 10,
        budget: None,
        expected_refs: Vec::new(),
        risk_classification: None,
        trace: Default::default(),
    }
}


/// Index a corpus through the real ingestion path (the GodSpeed agent
/// auto-ingests before resolving; these tests call the pipeline directly).
fn ingest(app: &ck_app::AppServices, root: &std::path::Path, corpus_id: &str) {
    let dir = root.join(corpus_id);
    app.ingest_source(ck_app::IngestSourceRequest {
        source_id: format!("ctxfs:{corpus_id}"),
        source_kind: "filesystem_tree".into(),
        adapter_config_json: json!({ "path": dir.to_str().unwrap() }).to_string(),
        corpus_id: Some(corpus_id.into()),
    })
    .unwrap();
}

fn seed_corpus(root: &std::path::Path, corpus_id: &str, files: &[(&str, &str)]) {
    let dir = root.join(corpus_id);
    fs::create_dir_all(&dir).unwrap();
    for (name, text) in files {
        fs::write(dir.join(name), *text).unwrap();
    }
}

#[test]
fn lexical_retrieval_preserves_identity_and_ranking_metadata() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "docs", &[("policy.md", "# Policy\nBlue-green rotation policy")]);
    let (app, _store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");

    let res = app
        .resolve_context(&requirement("docs", Some("rotation")))
        .unwrap();

    assert_eq!(res.completeness, RetrievalCompleteness::Complete);
    assert_eq!(res.results.len(), 1);
    let r = &res.results[0];
    assert!(r.chunk_id.is_some(), "chunk identity preserved");
    assert!(r.document_id.is_some(), "document identity preserved");
    let metadata = r.metadata.as_object().unwrap();
    assert_eq!(metadata["source_id"], json!("ctxfs:docs"));
    assert_eq!(metadata["corpus_id"], json!("docs"));
    assert!(
        r.score_breakdown.get("trust_score").is_some()
            || r.score_breakdown.get("freshness").is_some(),
        "ranking metadata carries trust/freshness evaluation: {:?}",
        r.score_breakdown
    );
    assert!(r.citation.get("uri").is_some(), "citation locator survives");
}

#[test]
fn semantic_retrieval_merges_when_configured() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "docs", &[("a.md", "deploy rotation runbook")]);
    let (app, store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");
    let chunk_id = store.list_chunks_by_corpus("docs").unwrap()[0].chunk_id.clone();
    let app = app.with_semantic(
        Box::new(StubEmbedding),
        Box::new(StubVectorSearch {
            items: vec![ck_core::entities::ScoredItem {
                id: chunk_id.clone(),
                score: 0.42,
                score_breakdown: json!({}),
            }],
        }),
    );

    let res = app
        .resolve_context(&requirement("docs", Some("rotation")))
        .unwrap();
    let r = res
        .results
        .iter()
        .find(|r| r.chunk_id.as_deref() == Some(chunk_id.as_str()))
        .expect("semantic hit reaches the resolution");
    assert!(
        r.score_breakdown.get("semantic").is_some(),
        "semantic score lands in the breakdown: {:?}",
        r.score_breakdown
    );
}

#[test]
fn private_corpus_without_scope_is_requires_scope_and_retrieves_nothing() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "private_docs", &[("s.md", "classified deployment notes")]);
    let (app, store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "private_docs");
    let mut c = store.get_corpus("private_docs").unwrap().unwrap();
    c.visibility = "private".into();
    store.upsert_corpus(&c).unwrap();

    let res = app
        .resolve_context(&requirement("private_docs", Some("classified")))
        .unwrap();
    assert_eq!(
        res.access,
        ContextAccess::RequiresScope {
            corpus_id: "private_docs".into()
        }
    );
    assert_eq!(res.completeness, RetrievalCompleteness::None);
    assert_eq!(res.results.len(), 0, "no content leaks without scope");
    assert_eq!(
        res.no_context_reason,
        Some(NoContextReason::ScopeUnresolved)
    );
}

#[test]
fn private_corpus_with_explicit_scope_retrieves() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "private_docs", &[("s.md", "classified deployment notes")]);
    let (app, store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "private_docs");
    let mut c = store.get_corpus("private_docs").unwrap().unwrap();
    c.visibility = "private".into();
    store.upsert_corpus(&c).unwrap();

    let mut req = requirement("private_docs", Some("classified"));
    req.constraints.caller_scope = vec!["private_docs".into()];
    let res = app.resolve_context(&req).unwrap();
    assert_eq!(res.access, ContextAccess::Accessible);
    assert_eq!(res.results.len(), 1);
}

#[test]
fn query_reference_rejects_private_corpus_without_scope_too() {
    // POL-ACL-002 at the query_reference gate: a missing scope never grants.
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "private_docs", &[("s.md", "classified notes")]);
    let (app, store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "private_docs");
    let mut c = store.get_corpus("private_docs").unwrap().unwrap();
    c.visibility = "private".into();
    store.upsert_corpus(&c).unwrap();
    assert!(matches!(store.get_corpus("private_docs").unwrap(), Some(Corpus { visibility, .. }) if visibility == "private"));

    let err = app.query_reference(ck_app::QueryReferenceRequest {
        canonical_id: Some("private_docs".into()),
        query: "classified".into(),
        mode: None,
        top_k: 5,
        version: None,
        filters: json!({}),
    });
    assert!(err.is_err(), "private corpus must be refused without scope");
}

#[test]
fn stale_sources_are_excluded_when_freshness_is_required() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "docs", &[("a.md", "old rotation notes")]);
    let (app, store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");
    // Age the source well past any freshness window.
    let mut source = store.get_source("ctxfs:docs").unwrap().unwrap();
    source.last_refresh_success_at = Some("2020-01-01T00:00:00Z".into());
    store.upsert_source(&source).unwrap();

    // Without a freshness requirement the retrieval still happens (staleness
    // is evaluated and surfaced, not hidden).
    let res = app
        .resolve_context(&requirement("docs", Some("rotation")))
        .unwrap();
    assert_eq!(res.results.len(), 1);
    assert!(res.results[0].score_breakdown.get("freshness").is_some());

    // With a freshness requirement, the stale source is excluded and the
    // absence is explicit.
    let mut req = requirement("docs", Some("rotation"));
    req.freshness.max_age_secs = Some(3600);
    let res = app.resolve_context(&req).unwrap();
    assert_eq!(res.completeness, RetrievalCompleteness::None);
    assert_eq!(res.no_context_reason, Some(NoContextReason::StaleContent));
    assert!(res
        .omissions
        .iter()
        .any(|o| o.code == "stale_source_excluded"));
}

#[test]
fn budgets_are_enforced_with_explicit_omissions() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(
        corpus.path(),
        "docs",
        &[("a.md", "one two three four five shared"), ("b.md", "six seven eight nine ten shared")],
    );
    let (app, _store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");

    // Token ceiling covering exactly one result.
    let mut req = requirement("docs", Some("shared"));
    req.budget = Some(Budget {
        max_tokens: 8,
        max_chunks: 10,
    });
    let res = app.resolve_context(&req).unwrap();
    assert_eq!(res.results.len(), 1, "token budget stops the second result");
    assert_eq!(res.completeness, RetrievalCompleteness::Partial);
    assert!(res
        .omissions
        .iter()
        .any(|o| o.code == "token_budget_exhausted"));
    assert_eq!(res.budget.requested_tokens, Some(8));
    assert!(res.budget.consumed_tokens > 0);

    // A budget that cannot cover even one result yields NOTHING, explicitly.
    let mut req = requirement("docs", Some("shared"));
    req.budget = Some(Budget {
        max_tokens: 1,
        max_chunks: 10,
    });
    let res = app.resolve_context(&req).unwrap();
    assert_eq!(res.completeness, RetrievalCompleteness::None);
    assert_eq!(res.no_context_reason, Some(NoContextReason::BudgetExhausted));
}

#[test]
fn chunk_and_result_limits_cap_the_retrieval() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(
        corpus.path(),
        "docs",
        &[("a.md", "alpha shared"), ("b.md", "beta shared"), ("c.md", "gamma shared")],
    );
    let (app, _store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");
    let mut req = requirement("docs", Some("shared"));
    req.max_results = 1;
    let res = app.resolve_context(&req).unwrap();
    assert_eq!(res.results.len(), 1);
    assert!(res.budget.more_available);
    assert_eq!(res.completeness, RetrievalCompleteness::Partial);
    assert!(res.omissions.iter().any(|o| o.code == "max_results_reached"));
}

#[test]
fn references_are_deterministic_for_identical_retrievals() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "docs", &[("a.md", "alpha shared"), ("b.md", "beta shared")]);
    let (app, _store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");

    let ids: Vec<Vec<String>> = (0..2)
        .map(|_| {
            app.resolve_context(&requirement("docs", Some("shared")))
                .unwrap()
                .results
                .iter()
                .map(|r| r.chunk_id.clone().unwrap_or_else(|| r.result_id.clone()))
                .collect()
        })
        .collect();
    assert_eq!(ids[0], ids[1], "identical retrievals yield identical references");
}

#[test]
fn absent_required_concepts_become_omissions_never_silently_dropped() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "docs", &[("a.md", "deploy rotation runbook")]);
    let (app, _store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");

    let mut req = requirement("docs", Some("rotation"));
    req.required_concepts = vec!["rollback window".into()];
    let res = app.resolve_context(&req).unwrap();
    assert_eq!(res.completeness, RetrievalCompleteness::Partial);
    assert!(res.omissions.iter().any(|o| o.code == "required_distinction_absent"
        && o.detail.as_deref() == Some("rollback window")));
}

#[test]
fn empty_retrieval_is_none_and_never_complete() {
    let _corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    let (app, _store) = test_kernel_with_store(kernel.path());
    let res = app
        .resolve_context(&requirement("missing-corpus", Some("anything")))
        .unwrap();
    assert_eq!(res.completeness, RetrievalCompleteness::None);
    assert!(res.no_context_reason.is_some());
    assert_eq!(res.results.len(), 0);
}

#[test]
fn an_empty_query_browses_the_corpus_within_budget() {
    let corpus = tempfile::tempdir().unwrap();
    let kernel = tempfile::tempdir().unwrap();
    seed_corpus(corpus.path(), "docs", &[("a.md", "alpha content here"), ("b.md", "beta content here")]);
    let (app, _store) = test_kernel_with_store(kernel.path());
    ingest(&app, corpus.path(), "docs");

    let res = app.resolve_context(&requirement("docs", None)).unwrap();
    assert_eq!(res.results.len(), 2, "no query = the corpus, bounded by budget");
    assert_eq!(res.completeness, RetrievalCompleteness::Complete);
}
