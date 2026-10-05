//! Project a Context Kernel packet onto the CEP `godspeed.context_bundle` profile.
//!
//! The legacy `ContextPacketCreated` payload stays the native wire shape. This
//! is the one place its meaning is restated as a CEP-0008 envelope, so a
//! consumer that speaks CEP sees the same facts: the pinned world, what was
//! cited, and, above all, what was NOT retrieved. A partial or empty packet is
//! never projected as complete, and a packet that claims to be incomplete
//! without saying why is refused rather than invented an omission for.

use serde_json::{json, Value};

use crate::agentic_capability_loop::{
    ContextPacketCreatedPayload, NoContextReason, RetrievalCompleteness,
};

const CREATED_BY: &str = "context-kernel";

/// Why a packet could not be projected. Nothing partial is ever returned.
#[derive(Debug, PartialEq, Eq)]
pub enum BundleError {
    /// The packet is not complete but names no omission, so the profile's
    /// "absence is never silent" rule cannot be met honestly.
    UnexplainedIncompleteness(RetrievalCompleteness),
    /// A packet claiming `complete` carries omissions, or a cited packet
    /// claims `none` (or the reverse): the packet contradicts itself.
    Inconsistent(&'static str),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexplainedIncompleteness(c) => {
                write!(f, "packet is {c:?} but names no omission or reason")
            }
            Self::Inconsistent(why) => write!(f, "inconsistent packet: {why}"),
        }
    }
}

impl std::error::Error for BundleError {}

/// Build the `context_bundle` envelope for `packet`.
///
/// `no_context` is the reason an acquisition resolved with zero citations
/// (`ContextAcquisition::NoContext`); it becomes the omission for a `none`
/// packet. `created_at` is an RFC 3339 timestamp supplied by the caller.
pub fn to_cep_context_bundle(
    packet: &ContextPacketCreatedPayload,
    no_context: Option<NoContextReason>,
    created_at: &str,
) -> Result<Value, BundleError> {
    use RetrievalCompleteness::{Complete, None as Nothing, Partial};

    let level = packet.retrieval_completeness;
    match (level, packet.citations.is_empty()) {
        (Complete, true) => return Err(BundleError::Inconsistent("complete with no citations")),
        (Partial, true) => return Err(BundleError::Inconsistent("partial with no citations")),
        (Nothing, false) => return Err(BundleError::Inconsistent("none with citations")),
        _ => {}
    }
    if level == Complete && !packet.omissions.is_empty() {
        return Err(BundleError::Inconsistent("complete with omissions"));
    }

    let mut reasons: Vec<String> = packet.omissions.clone();
    if let Some(reason) = no_context {
        reasons.push(reason.as_str().to_string());
    }
    if level != Complete && reasons.is_empty() {
        return Err(BundleError::UnexplainedIncompleteness(level));
    }

    let (level_name, status) = match level {
        Complete => ("complete", "complete_for_context"),
        Partial => ("partial", "partial"),
        Nothing => ("none", "minimal"),
    };

    let id = &packet.context_packet_id;
    let mut env = json!({
        "envelope_id": format!("env-{id}"),
        "cep_version": "1.0.0",
        "envelope_version": "1.0",
        "envelope_kind": "context_bundle",
        "created_at": created_at,
        "created_by": CREATED_BY,
        "scope": {
            "world_ref": packet.world_ref,
            "evaluation_context": format!("context-kernel:{}", packet.corpus_id),
        },
        "boundary_record": {
            "scope": format!("context for {}", packet.work_request_id),
            "included_sections": [],
            "excluded_sections": [],
            "known_omissions": reasons,
            "unknowns": [],
            "redactions": [],
            "compression_notes": [],
            "out_of_scope_entities": [],
            "limitations": [],
        },
        "completeness_status": status,
        "omission_status": if reasons.is_empty() { "none_known" } else { "marked" },
        "provenance_refs": [format!("prov-{id}")],
        "validation_status": "valid",
        "lineage_refs": [],
        "provenance": [{
            "provenance_id": format!("prov-{id}"),
            "source_system_refs": [format!("corpus:{}", packet.corpus_id)],
            "producer_refs": [CREATED_BY],
            "production_method": "retrieval",
            "created_at": created_at,
        }],
        "extensions": {
            "cep.profile": {
                "profile_id": "godspeed.context_bundle",
                "profile_version": "1.0.0",
            },
            "godspeed.context_bundle": {
                "work_request_ref": packet.work_request_id,
                "retrieval_completeness": level_name,
                "context_packet_id": id,
                "context_requirement_id": packet.context_requirement_id,
                "domain_model_hash": packet.domain_model_hash,
            },
        },
    });

    let obj = env.as_object_mut().expect("envelope is an object");
    let mut included = Vec::new();
    if packet.citations.is_empty() {
        // Nothing retrieved: the requirement stands as an unanswered question,
        // which is the envelope's entity section.
        obj.insert(
            "questions".into(),
            json!([{
                "question_id": packet.context_requirement_id,
                "question_form": format!(
                    "What context applies to work request {}?", packet.work_request_id
                ),
            }]),
        );
        included.push("questions");
    } else {
        let refs: Vec<Value> = packet
            .citations
            .iter()
            .map(|c| {
                let mut r = json!({
                    "ref_id": c.citation_id,
                    "ref_type": "source_excerpt",
                    "target_uri_or_id": c.source,
                    "availability_status": "available",
                    "integrity_status": "unverified",
                });
                if c.truncated {
                    r["limitations"] = json!(["content_truncated"]);
                }
                r
            })
            .collect();
        obj.insert("references".into(), Value::Array(refs));
        included.push("references");
    }
    if !reasons.is_empty() {
        let omissions: Vec<Value> = reasons
            .iter()
            .enumerate()
            .map(|(i, r)| {
                json!({
                    "omission_id": format!("om-{}", i + 1),
                    "omission_type": r,
                    "reason": r,
                })
            })
            .collect();
        obj.insert("omissions".into(), Value::Array(omissions));
        included.push("omissions");
    }
    env["boundary_record"]["included_sections"] = json!(included);
    Ok(env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agentic_capability_loop::{Citation, OMISSION_MAX_RESULTS};

    const WORLD: &str =
        "world:demo@sha256:1111111111111111111111111111111111111111111111111111111111111111";

    fn packet(
        level: RetrievalCompleteness,
        citations: usize,
        omissions: &[&str],
    ) -> ContextPacketCreatedPayload {
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
                .map(|i| Citation {
                    citation_id: format!("c{i}"),
                    source: format!("docs/{i}.md"),
                    content: "x".into(),
                    fetched_at: "2026-10-05T00:00:00Z".into(),
                    truncated: i == 0,
                })
                .collect(),
            authority_decision_id: None,
            authority_reference: None,
        }
    }

    const NOW: &str = "2026-10-05T12:00:00Z";

    #[test]
    fn complete_is_complete_for_context_with_references_and_no_omissions() {
        let env =
            to_cep_context_bundle(&packet(RetrievalCompleteness::Complete, 2, &[]), None, NOW)
                .unwrap();
        assert_eq!(env["completeness_status"], "complete_for_context");
        assert_eq!(env["scope"]["world_ref"], WORLD);
        assert_eq!(env["references"].as_array().unwrap().len(), 2);
        assert!(env.get("omissions").is_none());
    }

    #[test]
    fn partial_keeps_its_omissions_and_never_reads_as_complete() {
        let env = to_cep_context_bundle(
            &packet(RetrievalCompleteness::Partial, 1, &[OMISSION_MAX_RESULTS]),
            None,
            NOW,
        )
        .unwrap();
        assert_eq!(env["completeness_status"], "partial");
        assert_eq!(env["omissions"][0]["omission_type"], OMISSION_MAX_RESULTS);
        assert_eq!(env["references"][0]["limitations"][0], "content_truncated");
    }

    #[test]
    fn none_carries_the_reason_and_the_unanswered_question() {
        let env = to_cep_context_bundle(
            &packet(RetrievalCompleteness::None, 0, &[]),
            Some(NoContextReason::NoMatchingContent),
            NOW,
        )
        .unwrap();
        assert_eq!(env["completeness_status"], "minimal");
        assert_eq!(env["omissions"][0]["reason"], "no_matching_content");
        assert_eq!(env["questions"][0]["question_id"], "req-1");
        assert!(env.get("references").is_none());
    }

    #[test]
    fn an_incomplete_packet_that_cannot_say_why_is_refused() {
        assert_eq!(
            to_cep_context_bundle(&packet(RetrievalCompleteness::None, 0, &[]), None, NOW),
            Err(BundleError::UnexplainedIncompleteness(
                RetrievalCompleteness::None
            ))
        );
        assert_eq!(
            to_cep_context_bundle(&packet(RetrievalCompleteness::Partial, 1, &[]), None, NOW),
            Err(BundleError::UnexplainedIncompleteness(
                RetrievalCompleteness::Partial
            ))
        );
    }

    #[test]
    fn a_self_contradicting_packet_is_refused() {
        for p in [
            packet(RetrievalCompleteness::Complete, 0, &[]),
            packet(RetrievalCompleteness::Complete, 1, &[OMISSION_MAX_RESULTS]),
            packet(RetrievalCompleteness::None, 1, &[]),
            packet(RetrievalCompleteness::Partial, 0, &[OMISSION_MAX_RESULTS]),
        ] {
            assert!(
                matches!(
                    to_cep_context_bundle(&p, None, NOW),
                    Err(BundleError::Inconsistent(_))
                ),
                "{p:?}"
            );
        }
    }

    /// Validate with the cep reference validator (Python) when `CEP_REPO` names a
    /// checkout. Skipped otherwise: this crate does not depend on cep.
    #[test]
    fn projections_satisfy_the_cep_context_bundle_profile() {
        let Some(repo) = std::env::var_os("CEP_REPO") else {
            eprintln!("SKIP: CEP_REPO not set");
            return;
        };
        let cases = [
            to_cep_context_bundle(&packet(RetrievalCompleteness::Complete, 2, &[]), None, NOW)
                .unwrap(),
            to_cep_context_bundle(
                &packet(RetrievalCompleteness::Partial, 1, &[OMISSION_MAX_RESULTS]),
                None,
                NOW,
            )
            .unwrap(),
            to_cep_context_bundle(
                &packet(RetrievalCompleteness::None, 0, &[]),
                Some(NoContextReason::CorpusNotFound),
                NOW,
            )
            .unwrap(),
        ];
        let script =
            "import json,sys\nfrom cep.godspeed import validate_profile, verify_semantics\n\
            env=json.load(sys.stdin)\n\
            errs=validate_profile(env,'context_bundle')+verify_semantics(env,'context_bundle')\n\
            print(json.dumps(errs))";
        for env in cases {
            use std::io::Write;
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
}
