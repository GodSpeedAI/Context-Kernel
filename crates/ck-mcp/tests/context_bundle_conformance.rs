//! CEP `godspeed.context_bundle` envelope conformance: the three truthful
//! completeness states, the negative/falsification cases (wrong kind, wrong
//! profile, wrong version, missing/malformed world_ref, falsely-complete,
//! missing omissions, tampered integrity), and — when `CEP_REPO` names a CEP
//! checkout — the canonical reference validator.

mod common;

use ck_mcp::agentic_capability_loop::{ContextPacketCreatedPayload, NoContextReason, RetrievalCompleteness};
use ck_mcp::cep_bundle::{
    content_hash, to_cep_context_bundle, validate_context_bundle, verify_envelope_integrity,
};
use serde_json::json;

const WORLD: &str =
    "world:test@sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const NOW: &str = "2026-10-07T12:00:00Z";

fn packet(level: RetrievalCompleteness, citations: usize, omissions: &[&str]) -> ContextPacketCreatedPayload {
    ContextPacketCreatedPayload {
        domain_model_hash: "a".repeat(64),
        world_ref: WORLD.into(),
        retrieval_completeness: level,
        omissions: omissions.iter().map(|s| s.to_string()).collect(),
        context_packet_id: "ctx_01TEST".into(),
        context_requirement_id: "req-1".into(),
        work_request_id: "wr-1".into(),
        corpus_id: "docs".into(),
        citations: (0..citations)
            .map(|i| ck_mcp::agentic_capability_loop::Citation {
                citation_id: format!("c{i}"),
                source: format!("docs/{i}.md"),
                content: "x".into(),
                fetched_at: NOW.into(),
                truncated: false,
            })
            .collect(),
        authority_decision_id: None,
        authority_reference: None,
    }
}

fn complete() -> serde_json::Value {
    to_cep_context_bundle(&packet(RetrievalCompleteness::Complete, 2, &[]), None, NOW).unwrap()
}

fn partial() -> serde_json::Value {
    to_cep_context_bundle(
        &packet(RetrievalCompleteness::Partial, 1, &["max_results_reached"]),
        None,
        NOW,
    )
    .unwrap()
}

fn none() -> serde_json::Value {
    to_cep_context_bundle(
        &packet(RetrievalCompleteness::None, 0, &[]),
        Some(NoContextReason::NoMatchingContent),
        NOW,
    )
    .unwrap()
}

#[test]
fn the_three_states_are_valid_and_truthful() {
    for (env, status) in [
        (complete(), "complete_for_context"),
        (partial(), "partial"),
        (none(), "minimal"),
    ] {
        validate_context_bundle(&env).unwrap_or_else(|e| panic!("{status}: {e:?}"));
        verify_envelope_integrity(&env).unwrap_or_else(|e| panic!("{status}: {e}"));
        assert_eq!(env["completeness_status"], status);
        assert_eq!(env["scope"]["world_ref"], WORLD);
        assert_eq!(env["envelope_kind"], "context_bundle");
    }
    // partial and none keep visible what is missing
    assert_eq!(partial()["omissions"][0]["omission_type"], "max_results_reached");
    assert_eq!(none()["questions"][0]["question_id"], "req-1");
    assert!(none().get("references").is_none(), "absence is never dressed up");
}

#[test]
fn wrong_envelope_kind_is_refused() {
    let mut env = complete();
    env["envelope_kind"] = json!("evidence_packet");
    let errs = validate_context_bundle(&env).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("wrong envelope kind")), "{errs:?}");
}

#[test]
fn wrong_profile_id_and_version_are_refused() {
    let mut env = complete();
    env["extensions"]["cep.profile"]["profile_id"] = json!("godspeed.authority_request");
    let errs = validate_context_bundle(&env).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("wrong profile id")), "{errs:?}");

    let mut env = complete();
    env["extensions"]["cep.profile"]["profile_version"] = json!("2.0.0");
    let errs = validate_context_bundle(&env).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("wrong profile version")), "{errs:?}");
}

#[test]
fn missing_or_malformed_world_ref_is_refused() {
    let mut env = complete();
    env["scope"].as_object_mut().unwrap().remove("world_ref");
    let errs = validate_context_bundle(&env).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("missing world_ref")), "{errs:?}");

    let mut env = complete();
    env["scope"]["world_ref"] = json!("world:corporate"); // mutable alias, not identity
    let errs = validate_context_bundle(&env).unwrap_err();
    assert!(errs.iter().any(|e| e.contains("malformed world_ref")), "{errs:?}");
}

#[test]
fn a_falsely_complete_bundle_is_refused() {
    // A partial retrieval promoted to complete must fail both the projector's
    // consistency rules and the structural validator.
    let mut env = partial();
    env["completeness_status"] = json!("complete_for_context");
    env["extensions"]["godspeed.context_bundle"]["retrieval_completeness"] = json!("complete");
    // recompute integrity so ONLY the truthfulness violation remains
    env["integrity"]["content_hash"] = json!(content_hash(&env));
    let errs = validate_context_bundle(&env).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("complete retrieval must not carry omissions")),
        "{errs:?}"
    );
}

#[test]
fn missing_required_omissions_are_refused() {
    let mut env = partial();
    env.as_object_mut().unwrap().remove("omissions");
    env["boundary_record"]["known_omissions"] = json!([]);
    env["omission_status"] = json!("none_known");
    env["integrity"]["content_hash"] = json!(content_hash(&env));
    let errs = validate_context_bundle(&env).unwrap_err();
    assert!(
        errs.iter().any(|e| e.contains("must mark omissions")),
        "{errs:?}"
    );
}

#[test]
fn integrity_tampering_is_detected() {
    let env = complete();
    let mut tampered = env.clone();
    tampered["references"][0]["target_uri_or_id"] = json!("docs/forged.md");
    let err = verify_envelope_integrity(&tampered).unwrap_err();
    assert!(err.to_string().contains("integrity mismatch"), "{err}");
    // and the structural gate refuses it too
    assert!(validate_context_bundle(&tampered).is_err());

    // a removed integrity block is also refused
    let mut bare = env.clone();
    bare.as_object_mut().unwrap().remove("integrity");
    assert!(verify_envelope_integrity(&bare).is_err());
}

#[test]
fn a_forged_bundle_without_honest_omissions_cannot_be_projected() {
    use ck_mcp::cep_bundle::BundleError;
    // complete with zero citations is a self-contradiction
    assert!(matches!(
        to_cep_context_bundle(&packet(RetrievalCompleteness::Complete, 0, &[]), None, NOW),
        Err(BundleError::Inconsistent(_))
    ));
    // an incomplete retrieval that cannot say why is refused, not invented
    assert!(matches!(
        to_cep_context_bundle(&packet(RetrievalCompleteness::Partial, 1, &[]), None, NOW),
        Err(BundleError::UnexplainedIncompleteness(_))
    ));
}

/// Validate the real projections against the CEP reference validator when
/// `CEP_REPO` names a checkout. Skipped otherwise: this crate does not depend
/// on cep.
#[test]
fn projections_satisfy_the_cep_context_bundle_profile() {
    let Some(repo) = std::env::var_os("CEP_REPO") else {
        eprintln!("SKIP: CEP_REPO not set");
        return;
    };
    use std::io::Write;
    let script = "import json,sys\nfrom cep.godspeed import validate_profile, verify_semantics\n\
        env=json.load(sys.stdin)\n\
        errs=validate_profile(env,'context_bundle')+verify_semantics(env,'context_bundle')\n\
        print(json.dumps(errs))";
    for env in [complete(), partial(), none()] {
        let mut child = std::process::Command::new("uv")
            .args(["run", "--quiet", "--project"])
            .arg(&repo)
            .args(["python", "-c", script])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn uv");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(env.to_string().as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        let errs: Vec<String> =
            serde_json::from_slice(&out.stdout).expect("validator prints a JSON list");
        assert!(errs.is_empty(), "{}: {errs:?}", env["completeness_status"]);
    }
}
