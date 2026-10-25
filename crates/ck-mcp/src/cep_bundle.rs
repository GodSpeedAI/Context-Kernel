//! Project a native Context Kernel resolution onto the CEP-0008
//! `godspeed.context_bundle` profile (`cep.profile` 1.0.0).
//!
//! This is the ONE place Context Kernel's bounded observer representation is
//! restated as a CEP envelope, so a consumer that speaks CEP sees the same
//! facts: the pinned world (`scope.world_ref`), what was retrieved (references
//! with source/document/chunk identity, revisions, content digests and
//! locators), and — above all — what was NOT retrieved. A partial or empty
//! retrieval is never projected as complete, and a retrieval that claims to
//! be incomplete without saying why is refused rather than invented an
//! omission for.
//!
//! ## Integrity
//!
//! The envelope carries CEP `integrity.content_hash = "sha256:<hex>"`, the
//! SHA-256 of the canonical JSON of the envelope with the `integrity` block
//! removed. Canonical JSON: recursively sorted object keys, compact
//! separators, UTF-8 unescaped. Occurrence identity is `envelope_id`; exact
//! content identity is this hash; semantic-world identity is `scope.world_ref`.
//! The three are deliberately kept separate and are never collapsed into one
//! ambiguous digest.

use serde_json::{json, Map, Value};

use ck_app::{ContextRequirement, ContextResolution};
use ck_core::types::{verify_world_ref, RetrievalCompleteness};

use crate::agentic_capability_loop::{ContextPacketCreatedPayload, NoContextReason};

const CREATED_BY: &str = "context-kernel";
const PROFILE_ID: &str = "godspeed.context_bundle";
const PROFILE_VERSION: &str = "1.0.0";
const RETRIEVAL_POLICY: &str = "ck-app.resolve_context/v1";

/// Why a retrieval could not be projected. Nothing partial is ever returned.
#[derive(Debug, PartialEq, Eq)]
pub enum BundleError {
    /// The retrieval is not complete but names no omission, so the profile's
    /// "absence is never silent" rule cannot be met honestly.
    UnexplainedIncompleteness(RetrievalCompleteness),
    /// The retrieval contradicts itself (complete with omissions, none with
    /// references, ...).
    Inconsistent(&'static str),
    /// The envelope holds a number that cannot be canonicalized exactly.
    Canonicalization(CanonicalError),
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexplainedIncompleteness(c) => {
                write!(f, "retrieval is {c:?} but names no omission or reason")
            }
            Self::Inconsistent(why) => write!(f, "inconsistent retrieval: {why}"),
            Self::Canonicalization(e) => write!(f, "cannot canonicalize bundle: {e}"),
        }
    }
}

impl std::error::Error for BundleError {}

/// Largest integer that every JSON consumer (including IEEE-754 double based
/// ones such as JavaScript) represents exactly: 2^53 - 1.
pub const MAX_SAFE_INTEGER: i64 = (1 << 53) - 1;

/// Why a value cannot be canonicalized. Canonicalization fails closed rather
/// than silently changing a number that a consumer would hash differently.
#[derive(Debug, PartialEq, Eq)]
pub enum CanonicalError {
    /// An integer outside ±(2^53 - 1): not exactly representable in every
    /// consumer's number type, so the same bundle would hash differently there.
    UnsafeInteger(String),
    /// NaN or infinity (not representable in JSON; defensive).
    NonFiniteNumber,
}

impl std::fmt::Display for CanonicalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsafeInteger(v) => write!(
                f,
                "integer {v} is outside ±(2^53-1) and cannot be canonicalized exactly"
            ),
            Self::NonFiniteNumber => write!(f, "non-finite number cannot be canonicalized"),
        }
    }
}

impl std::error::Error for CanonicalError {}

/// Canonical JSON for hashing: recursively sorted object keys, compact
/// separators, UTF-8 unescaped (the CEP reference `json.dumps(sort_keys=True,
/// separators=(",", ":"), ensure_ascii=False)` convention). Numbers:
///
/// - Integers are written exactly and MUST lie within ±(2^53 - 1); anything
///   larger fails with [`CanonicalError::UnsafeInteger`] rather than being
///   rounded or silently changed.
/// - Floats are rounded to 6 decimal places and negative zero is written as
///   `0.0`. This is the rule Context Kernel and SWE_SEED
///   (`federation::context_bundle::bundle_content_hash`) both apply, so the
///   two sides agree on float-bearing envelopes. The CEP specification text
///   does not mandate this rounding (its reference canonicalizer does not
///   round); changing it requires changing both implementations together.
///
/// Rounding is needed for hash stability: serde_json's default float parser
/// can drift 1 ULP on long spellings (`0.44999999999999996` parses to `0.45`),
/// so hashing raw floats would change after a parse round-trip.
pub fn canonical_json(value: &Value) -> Result<String, CanonicalError> {
    fn normalize(value: &Value) -> Result<Value, CanonicalError> {
        Ok(match value {
            Value::Object(obj) => {
                let mut keys: Vec<&String> = obj.keys().collect();
                keys.sort();
                let mut out = Map::new();
                for k in keys {
                    out.insert(k.clone(), normalize(&obj[k])?);
                }
                Value::Object(out)
            }
            Value::Array(items) => {
                Value::Array(items.iter().map(normalize).collect::<Result<Vec<_>, _>>()?)
            }
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    if i.unsigned_abs() > MAX_SAFE_INTEGER as u64 {
                        return Err(CanonicalError::UnsafeInteger(i.to_string()));
                    }
                    value.clone()
                } else if let Some(u) = n.as_u64() {
                    if u > MAX_SAFE_INTEGER as u64 {
                        return Err(CanonicalError::UnsafeInteger(u.to_string()));
                    }
                    value.clone()
                } else {
                    match n.as_f64() {
                        Some(f) if f.is_finite() => {
                            let rounded = format!("{:.6}", f).parse::<f64>().unwrap_or(f);
                            let rounded = if rounded == 0.0 { 0.0 } else { rounded };
                            Value::Number(
                                serde_json::Number::from_f64(rounded)
                                    .ok_or(CanonicalError::NonFiniteNumber)?,
                            )
                        }
                        _ => return Err(CanonicalError::NonFiniteNumber),
                    }
                }
            }
            other => other.clone(),
        })
    }
    Ok(normalize(value)?.to_string())
}

/// `integrity.content_hash` of an envelope: sha256 over the canonical JSON of
/// the envelope with the `integrity` block removed.
pub fn content_hash(envelope: &Value) -> Result<String, CanonicalError> {
    use sha2::{Digest, Sha256};
    let mut bare = envelope.clone();
    if let Some(obj) = bare.as_object_mut() {
        obj.remove("integrity");
    }
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(canonical_json(&bare)?.as_bytes())
    ))
}

/// Why an envelope's integrity does not hold.
#[derive(Debug, PartialEq, Eq)]
pub enum IntegrityError {
    Missing,
    Malformed,
    Mismatch { declared: String, computed: String },
    Canonicalization(CanonicalError),
}

impl std::fmt::Display for IntegrityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => write!(f, "envelope carries no integrity block"),
            Self::Malformed => write!(f, "integrity.content_hash is malformed"),
            Self::Canonicalization(e) => write!(f, "cannot canonicalize envelope: {e}"),
            Self::Mismatch { declared, computed } => {
                write!(
                    f,
                    "integrity mismatch: declared {declared}, computed {computed}"
                )
            }
        }
    }
}

impl std::error::Error for IntegrityError {}

/// Verify the envelope against its own CEP integrity record. Detects any
/// post-emission modification of the bundle content.
pub fn verify_envelope_integrity(envelope: &Value) -> Result<(), IntegrityError> {
    let obj = envelope.as_object().ok_or(IntegrityError::Malformed)?;
    let integrity = obj.get("integrity").ok_or(IntegrityError::Missing)?;
    let declared = integrity
        .get("content_hash")
        .and_then(Value::as_str)
        .ok_or(IntegrityError::Malformed)?;
    if !declared.starts_with("sha256:") {
        return Err(IntegrityError::Malformed);
    }
    let computed = content_hash(envelope).map_err(IntegrityError::Canonicalization)?;
    if declared == computed {
        Ok(())
    } else {
        Err(IntegrityError::Mismatch {
            declared: declared.to_string(),
            computed,
        })
    }
}

/// Everything the projection knows about one bounded retrieval.
struct BundleCore {
    world_ref: String,
    work_request_ref: String,
    context_requirement_id: String,
    context_packet_id: String,
    domain_model_hash: String,
    evaluation_context: String,
    created_at: String,
    level: RetrievalCompleteness,
    omissions: Vec<(String, Option<String>)>,
    reason_no_context: Option<String>,
    references: Vec<Value>,
    evaluations: Vec<Value>,
    provenance_sources: Vec<String>,
    caused_by: Option<String>,
    correlation_id: Option<String>,
    query_mode: Option<String>,
    budget: Option<Value>,
    expected_refs: Vec<String>,
}

/// Assemble the envelope with the profile's truthfulness rules enforced and
/// the CEP integrity record stamped last.
fn assemble(core: BundleCore) -> Result<Value, BundleError> {
    use RetrievalCompleteness::{Complete, None as Nothing, Partial};

    let level = core.level;
    match (level, core.references.is_empty()) {
        (Complete, true) => return Err(BundleError::Inconsistent("complete with no references")),
        (Partial, true) => return Err(BundleError::Inconsistent("partial with no references")),
        (Nothing, false) => return Err(BundleError::Inconsistent("none with references")),
        _ => {}
    }
    if level == Complete && !core.omissions.is_empty() {
        return Err(BundleError::Inconsistent("complete with omissions"));
    }

    let mut omissions = core.omissions;
    if let Some(reason) = core.reason_no_context {
        if !omissions.iter().any(|(code, _)| *code == reason) {
            omissions.push((reason, None));
        }
    }
    if level != Complete && omissions.is_empty() {
        return Err(BundleError::UnexplainedIncompleteness(level));
    }

    let (level_name, status) = match level {
        Complete => ("complete", "complete_for_context"),
        Partial => ("partial", "partial"),
        Nothing => ("none", "minimal"),
    };

    let id = &core.context_packet_id;
    let known_omissions: Vec<String> = omissions.iter().map(|(c, _)| c.clone()).collect();
    let mut env = json!({
        "envelope_id": format!("env-{id}"),
        "cep_version": "1.0.0",
        "envelope_version": "1.0",
        "envelope_kind": "context_bundle",
        "created_at": &core.created_at,
        "created_by": CREATED_BY,
        "scope": {
            "world_ref": &core.world_ref,
            "evaluation_context": &core.evaluation_context,
        },
        "boundary_record": {
            "scope": format!("context for {}", &core.work_request_ref),
            "included_sections": [],
            "excluded_sections": [],
            "known_omissions": known_omissions,
            "unknowns": [],
            "redactions": [],
            "compression_notes": [],
            "out_of_scope_entities": [],
            "limitations": [],
        },
        "completeness_status": status,
        "omission_status": if omissions.is_empty() { "none_known" } else { "marked" },
        "provenance_refs": [format!("prov-{id}")],
        "validation_status": "valid",
        "lineage_refs": [],
        "provenance": [{
            "provenance_id": format!("prov-{id}"),
            "source_system_refs": &core.provenance_sources,
            "producer_refs": [CREATED_BY],
            "production_method": "retrieval",
            "created_at": &core.created_at,
        }],
        "extensions": {
            "cep.profile": {
                "profile_id": PROFILE_ID,
                "profile_version": PROFILE_VERSION,
            },
            "godspeed.context_bundle": {
                "work_request_ref": &core.work_request_ref,
                "retrieval_completeness": level_name,
                "context_packet_id": id,
                "context_requirement_id": &core.context_requirement_id,
                "domain_model_hash": &core.domain_model_hash,
            },
        },
    });

    let mut included: Vec<&str> = Vec::new();
    if core.references.is_empty() {
        // Nothing retrieved: the requirement stands as an unanswered question,
        // which is the envelope's entity section.
        env["questions"] = json!([{
            "question_id": &core.context_requirement_id,
            "question_form": format!(
                "What context applies to work request {}?",
                &core.work_request_ref
            ),
        }]);
        included.push("questions");
    } else {
        env["references"] = Value::Array(core.references);
        included.push("references");
    }
    if !omissions.is_empty() {
        let entries: Vec<Value> = omissions
            .iter()
            .enumerate()
            .map(|(i, (code, detail))| {
                json!({
                    "omission_id": format!("om-{}", i + 1),
                    "omission_type": code,
                    "reason": code,
                    "description": detail.clone().unwrap_or_else(|| code.clone()),
                })
            })
            .collect();
        env["omissions"] = Value::Array(entries);
        included.push("omissions");
    }
    env["boundary_record"]["included_sections"] = json!(included);

    // CK-specific facts ride the governed extension namespace, minimal.
    let ext = env["extensions"]["godspeed.context_bundle"]
        .as_object_mut()
        .expect("extension is an object");
    ext.insert(
        "retrieval_policy".into(),
        Value::String(RETRIEVAL_POLICY.to_string()),
    );
    if let Some(mode) = core.query_mode {
        ext.insert("query_mode".into(), Value::String(mode));
    }
    if let Some(caused_by) = core.caused_by {
        ext.insert("caused_by".into(), Value::String(caused_by));
    }
    if let Some(correlation_id) = core.correlation_id {
        ext.insert("correlation_id".into(), Value::String(correlation_id));
    }
    if let Some(budget) = core.budget {
        ext.insert("budget".into(), budget);
    }
    if !core.evaluations.is_empty() {
        ext.insert("evaluations".into(), Value::Array(core.evaluations));
    }
    if !core.expected_refs.is_empty() {
        ext.insert("expected_refs".into(), json!(core.expected_refs));
    }

    // CEP integrity: exact content identity, separate from envelope_id
    // (occurrence) and world_ref (semantic world).
    let hash = content_hash(&env).map_err(BundleError::Canonicalization)?;
    env["integrity"] = json!({
        "content_hash": hash,
        "verification_method": "sha256-canonical-json-v1",
        "verification_status": "verified",
        "tamper_status": "intact",
    });
    Ok(env)
}

/// Everything the canonical projection needs about one bounded retrieval.
pub struct BundleInput<'a> {
    pub requirement: &'a ContextRequirement,
    pub resolution: &'a ContextResolution,
    pub packet: &'a ContextPacketCreatedPayload,
    /// The causal parent request envelope id (`caused_by:` lineage).
    pub request_event_id: Option<&'a str>,
    pub created_at: &'a str,
}

/// The canonical projection: native CK resolution -> CEP-0008
/// `godspeed.context_bundle`.
pub fn project_context_bundle(input: &BundleInput<'_>) -> Result<Value, BundleError> {
    let mut references = Vec::new();
    let mut evaluations = Vec::new();
    let mut provenance_sources: Vec<String> = Vec::new();
    for corpus in &input.resolution.corpora {
        let entry = format!("corpus:{corpus}");
        if !provenance_sources.contains(&entry) {
            provenance_sources.push(entry);
        }
    }
    for r in &input.resolution.results {
        // Deterministic reference identity: one reference per retrieved
        // chunk (or document), stable across identical retrievals.
        let base = r
            .chunk_id
            .clone()
            .or_else(|| r.document_id.clone())
            .unwrap_or_else(|| r.result_id.clone());
        let ref_id = format!("ref-{base}");
        let mut target = r
            .citation
            .get("uri")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| r.result_id.clone());
        if let Some(anchor) = r.citation.get("anchor").and_then(Value::as_str) {
            target = format!("{target}#{anchor}");
        }
        let checksum = r
            .metadata
            .get("checksum")
            .and_then(Value::as_str)
            .filter(|c| {
                c.len() == 64
                    && c.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            });
        let mut reference = json!({
            "ref_id": ref_id,
            "ref_type": if r.chunk_id.is_some() { "chunk" } else { "document" },
            "target_uri_or_id": target,
            "availability_status": "available",
            "integrity_status": if checksum.is_some() { "verifiable" } else { "unverified" },
        });
        let revision = r
            .metadata
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_string)
            .or_else(|| {
                r.metadata
                    .get("upstream_revision")
                    .and_then(Value::as_str)
                    .map(str::to_string)
            });
        if let Some(v) = revision {
            reference["target_version"] = Value::String(v);
        }
        if let Some(c) = checksum {
            reference["target_hash"] = Value::String(format!("sha256:{c}"));
        }
        if let Some(corpus) = r.metadata.get("corpus_id").and_then(Value::as_str) {
            reference["scope"] = Value::String(corpus.to_string());
        }
        if r.excerpt.ends_with('…') {
            reference["limitations"] = json!(["content_truncated"]);
        }
        if let Some(source_id) = r.metadata.get("source_id").and_then(Value::as_str) {
            let entry = format!("source:{source_id}");
            if !provenance_sources.contains(&entry) {
                provenance_sources.push(entry);
            }
        }
        references.push(reference);
        evaluations.push(json!({
            "ref_id": ref_id,
            "score": r.score,
            "score_breakdown": r.score_breakdown,
        }));
    }
    if provenance_sources.is_empty() {
        provenance_sources.push(format!("corpus:{}", input.packet.corpus_id));
    }

    assemble(BundleCore {
        world_ref: input.requirement.world_ref.clone(),
        work_request_ref: input.requirement.identity.work_request_ref.clone(),
        context_requirement_id: input.requirement.identity.context_requirement_id.clone(),
        context_packet_id: input.packet.context_packet_id.clone(),
        domain_model_hash: input.packet.domain_model_hash.clone(),
        evaluation_context: format!("context-kernel:{}", input.packet.corpus_id),
        created_at: input.created_at.to_string(),
        level: input.resolution.completeness,
        omissions: input
            .resolution
            .omissions
            .iter()
            .map(|o| (o.code.clone(), o.detail.clone()))
            .collect(),
        reason_no_context: input
            .resolution
            .no_context_reason
            .map(|r| r.as_str().to_string()),
        references,
        evaluations,
        provenance_sources,
        caused_by: input.request_event_id.map(str::to_string),
        correlation_id: input.requirement.trace.correlation_id.clone(),
        query_mode: Some(input.resolution.mode.clone()),
        budget: Some(json!({
            "requested_results": input.resolution.budget.requested_results,
            "consumed_results": input.resolution.budget.consumed_results,
            "requested_tokens": input.resolution.budget.requested_tokens,
            "consumed_tokens": input.resolution.budget.consumed_tokens,
        })),
        expected_refs: input.requirement.expected_refs.clone(),
    })
}

/// DEPRECATED dual-read compatibility: project the legacy
/// `ContextPacketCreated` payload alone. The canonical projection is
/// [`project_context_bundle`]; this exists so bounded legacy consumers keep
/// seeing one and the same truth until migration completes.
pub fn to_cep_context_bundle(
    packet: &ContextPacketCreatedPayload,
    no_context: Option<NoContextReason>,
    created_at: &str,
) -> Result<Value, BundleError> {
    let references = packet
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
    assemble(BundleCore {
        world_ref: packet.world_ref.clone(),
        work_request_ref: packet.work_request_id.clone(),
        context_requirement_id: packet.context_requirement_id.clone(),
        context_packet_id: packet.context_packet_id.clone(),
        domain_model_hash: packet.domain_model_hash.clone(),
        evaluation_context: format!("context-kernel:{}", packet.corpus_id),
        created_at: created_at.to_string(),
        level: packet.retrieval_completeness,
        omissions: packet.omissions.iter().map(|o| (o.clone(), None)).collect(),
        reason_no_context: no_context.map(|r| r.as_str().to_string()),
        references,
        evaluations: Vec::new(),
        provenance_sources: vec![format!("corpus:{}", packet.corpus_id)],
        caused_by: None,
        correlation_id: None,
        query_mode: None,
        budget: None,
        expected_refs: Vec::new(),
    })
}

/// Local fail-closed structural mirror of the CEP `context_bundle.v1` profile
/// rules (kind, profile declaration, world pin, provenance, the three
/// completeness branches, and integrity). The authoritative validator is the
/// CEP reference implementation (`cep.godspeed.validate_profile` +
/// `verify_semantics`); this mirror exists so tests and consumers can refuse
/// a bad envelope without a CEP checkout.
pub fn validate_context_bundle(envelope: &Value) -> Result<(), Vec<String>> {
    let mut errors: Vec<String> = Vec::new();
    let Some(obj) = envelope.as_object() else {
        return Err(vec!["envelope must be an object".to_string()]);
    };
    for field in [
        "envelope_id",
        "cep_version",
        "envelope_version",
        "created_at",
        "created_by",
    ] {
        if obj
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or("")
            .is_empty()
        {
            errors.push(format!("missing {field}"));
        }
    }
    if obj.get("envelope_kind").and_then(Value::as_str) != Some("context_bundle") {
        errors.push("wrong envelope kind".to_string());
    }
    let profile = obj.get("extensions").and_then(|e| e.get("cep.profile"));
    if profile
        .and_then(|p| p.get("profile_id"))
        .and_then(Value::as_str)
        != Some(PROFILE_ID)
    {
        errors.push("wrong profile id".to_string());
    }
    if profile
        .and_then(|p| p.get("profile_version"))
        .and_then(Value::as_str)
        != Some(PROFILE_VERSION)
    {
        errors.push("wrong profile version".to_string());
    }
    match obj
        .get("scope")
        .and_then(|s| s.get("world_ref"))
        .and_then(Value::as_str)
    {
        Some(w) if verify_world_ref(w).is_ok() => {}
        Some(_) => errors.push("malformed world_ref".to_string()),
        None => errors.push("missing world_ref".to_string()),
    }
    if obj
        .get("provenance")
        .and_then(Value::as_array)
        .map(|a| a.is_empty())
        .unwrap_or(true)
    {
        errors.push("malformed provenance".to_string());
    }

    let level = obj
        .get("extensions")
        .and_then(|e| e.get("godspeed.context_bundle"))
        .and_then(|b| b.get("retrieval_completeness"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let status = obj
        .get("completeness_status")
        .and_then(Value::as_str)
        .unwrap_or("");
    let has_refs = obj
        .get("references")
        .and_then(Value::as_array)
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    let has_omissions = obj
        .get("omissions")
        .and_then(Value::as_array)
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    let has_questions = obj
        .get("questions")
        .and_then(Value::as_array)
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    match level {
        "complete" => {
            if status != "complete_for_context" {
                errors.push("complete retrieval must read complete_for_context".to_string());
            }
            if !has_refs {
                errors.push("complete retrieval must carry references".to_string());
            }
            if has_omissions {
                errors.push("complete retrieval must not carry omissions".to_string());
            }
        }
        "partial" => {
            if status == "complete_for_context" {
                errors.push("partial retrieval must not read complete".to_string());
            }
            if !has_refs {
                errors.push("partial retrieval must carry references".to_string());
            }
            if !has_omissions {
                errors.push("partial retrieval must mark omissions".to_string());
            }
        }
        "none" => {
            if status == "complete_for_context" {
                errors.push("empty retrieval must not read complete".to_string());
            }
            if has_refs {
                errors.push("empty retrieval must not carry references".to_string());
            }
            if !has_omissions {
                errors.push("empty retrieval must mark omissions".to_string());
            }
            if !has_questions {
                errors.push("empty retrieval must carry its unanswered question".to_string());
            }
        }
        other => errors.push(format!("unknown retrieval_completeness {other:?}")),
    }
    if let Err(e) = verify_envelope_integrity(envelope) {
        errors.push(format!("integrity: {e}"));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
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

    /// Shared cross-implementation vector. The expected canonical string and
    /// digest were produced by the CEP reference canonicalizer
    /// (`cep/src/cep/godspeed.py::_canonical_json` + sha256) from the same
    /// input, so this pins Context Kernel to the same bytes.
    const SHARED_VECTOR_INPUT: &str = r#"{"z": [3, 1.5, {"b": 2, "a": "é"}], "a": 9007199254740991, "m": -9007199254740991, "f": 1.25, "s": "x\"y", "n": null, "t": true}"#;
    const SHARED_VECTOR_CANONICAL: &str = r#"{"a":9007199254740991,"f":1.25,"m":-9007199254740991,"n":null,"s":"x\"y","t":true,"z":[3,1.5,{"a":"é","b":2}]}"#;
    const SHARED_VECTOR_DIGEST: &str =
        "sha256:99582d94bc0cca2453e5057541bf23fc459f3ca10994e75513bc3af8cb5cfd4e";

    #[test]
    fn canonical_json_matches_shared_cross_implementation_vector() {
        use sha2::{Digest, Sha256};
        let value: Value = serde_json::from_str(SHARED_VECTOR_INPUT).unwrap();
        let canonical = canonical_json(&value).unwrap();
        assert_eq!(canonical, SHARED_VECTOR_CANONICAL);
        let digest = format!("sha256:{:x}", Sha256::digest(canonical.as_bytes()));
        assert_eq!(digest, SHARED_VECTOR_DIGEST);
    }

    #[test]
    fn canonical_json_refuses_integers_beyond_two_pow_53_minus_one() {
        let just_over = json!({ "n": 9_007_199_254_740_992_i64 });
        assert_eq!(
            canonical_json(&just_over),
            Err(CanonicalError::UnsafeInteger("9007199254740992".into()))
        );
        let negative = json!({ "n": -9_007_199_254_740_992_i64 });
        assert!(matches!(
            canonical_json(&negative),
            Err(CanonicalError::UnsafeInteger(_))
        ));
        let huge: Value = serde_json::from_str(r#"{"n": 18446744073709551615}"#).unwrap();
        assert!(matches!(
            canonical_json(&huge),
            Err(CanonicalError::UnsafeInteger(_))
        ));
        // The boundary itself is exact and accepted.
        let edge = json!({ "n": 9_007_199_254_740_991_i64 });
        assert_eq!(canonical_json(&edge).unwrap(), r#"{"n":9007199254740991}"#);
    }

    #[test]
    fn refused_integer_fails_the_bundle_hash_instead_of_changing_it() {
        let env = json!({ "k": 9_007_199_254_740_993_i64 });
        assert!(matches!(
            content_hash(&env),
            Err(CanonicalError::UnsafeInteger(_))
        ));
    }

    #[test]
    fn floats_round_to_six_decimals_as_the_cross_implementation_rule() {
        assert_eq!(
            canonical_json(&json!({ "x": 0.1234567 })).unwrap(),
            r#"{"x":0.123457}"#
        );
        assert_eq!(
            canonical_json(&json!({ "x": 1.0000004 })).unwrap(),
            r#"{"x":1.0}"#
        );
        // Values that differ at the 6th decimal stay distinct.
        assert_ne!(
            canonical_json(&json!({ "x": 0.123456 })).unwrap(),
            canonical_json(&json!({ "x": 0.123457 })).unwrap()
        );
    }

    #[test]
    fn negative_zero_is_written_as_zero() {
        let v = json!({ "x": -0.0 });
        assert_eq!(canonical_json(&v).unwrap(), r#"{"x":0.0}"#);
    }
}
