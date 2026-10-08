//! Settlement-backed Context Effect records — retrieval calibration only.
//!
//! This is NOT a developmental-memory or capability system. Its single
//! purpose is prospective retrieval calibration: only effects backed by a
//! settlement/evidence artifact from the GodSpeed loop may be ingested, an
//! agent's self-report ("this context was useful") is never evidence, and
//! historical context bundles are never rewritten. Calibration influences
//! future retrieval decisions only.
//!
//! Durable format: append-only JSONL, one [`ContextEffect`] per line, keyed
//! by `effect_id` (idempotent re-ingest is skipped, never overwritten).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use ck_core::types::verify_world_ref;

/// Why a context effect was recorded. Every variant must be justified by a
/// settlement/evidence reference at ingestion time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextEffectKind {
    /// A required distinction was absent from the supplied context.
    RequiredDistinctionAbsent,
    /// A retrieved source was contradicted by later settlement evidence.
    SourceContradicted,
    /// A retrieved source was stale relative to what settled.
    SourceStale,
    /// A retrieved distinction materially affected successful settlement.
    DistinctionMaterialToSettlement,
    /// A context item was unused — retrieval burden with no effect.
    ContextItemUnused,
    /// Retrieval burden was excessive for the settled outcome.
    ExcessiveRetrievalBurden,
    /// An unresolved omission later proved consequential.
    OmissionConsequential,
}

impl ContextEffectKind {
    /// Prospective score demotion weight for affected sources. Only
    /// settlement-established negative source signals demote; positive or
    /// burden signals are recorded but do not by themselves rescore sources.
    fn demotion_weight(&self) -> f64 {
        match self {
            Self::SourceContradicted => 0.20,
            Self::SourceStale => 0.10,
            _ => 0.0,
        }
    }
}

/// One durable, settlement-backed context effect.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextEffect {
    pub effect_id: String,
    pub recorded_at: String,
    pub kind: ContextEffectKind,
    /// Immutable world the bundle was produced in (validated syntax-only;
    /// calibration is applied per world and never crosses worlds).
    pub world_ref: String,
    pub work_request_ref: String,
    /// CEP `envelope_id` of the historical bundle being calibrated. The
    /// bundle itself is immutable; this record only informs FUTURE retrieval.
    pub bundle_envelope_id: String,
    /// CEP `integrity.content_hash` of that exact bundle
    /// (`sha256:<64 hex>`).
    pub bundle_content_hash: String,
    /// Sources whose future retrieval is affected.
    pub affected_sources: Vec<String>,
    /// `<kind>:<value>` reference to the settlement/evidence artifact that
    /// justifies this effect. REQUIRED — self-reports are rejected.
    pub evidence_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}


/// Why an effect could not be ingested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextEffectError {
    /// No settlement/evidence reference: a bare self-report.
    MissingEvidence,
    /// evidence_ref is not `<kind>:<value>` with an allowed kind.
    MalformedEvidenceRef(String),
    /// world_ref does not name a pinned world revision.
    InvalidWorldRef,
    /// bundle_content_hash is not `sha256:<64 lowercase hex>`.
    InvalidBundleHash,
    Io(String),
}

impl std::fmt::Display for ContextEffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEvidence => write!(
                f,
                "context effect rejected: not settlement-backed (evidence_ref required)"
            ),
            Self::MalformedEvidenceRef(r) => write!(
                f,
                "context effect rejected: evidence_ref {r:?} must be <kind>:<value> with kind in {{settlement,evidence,execution,observation,proof}}"
            ),
            Self::InvalidWorldRef => write!(f, "context effect rejected: malformed world_ref"),
            Self::InvalidBundleHash => write!(
                f,
                "context effect rejected: bundle_content_hash is not sha256:<64 hex>"
            ),
            Self::Io(e) => write!(f, "context effect ledger i/o: {e}"),
        }
    }
}

impl std::error::Error for ContextEffectError {}

const EVIDENCE_KINDS: &[&str] = &[
    "settlement",
    "evidence",
    "execution",
    "observation",
    "proof",
];

impl ContextEffect {
    /// The ingestion gate: settlement-backed, pinned world, content-addressed
    /// bundle identity.
    pub fn validate(&self) -> Result<(), ContextEffectError> {
        if self.effect_id.trim().is_empty()
            || self.work_request_ref.trim().is_empty()
            || self.bundle_envelope_id.trim().is_empty()
        {
            return Err(ContextEffectError::MissingEvidence);
        }
        let ref_ok = match self.evidence_ref.split_once(':') {
            Some((kind, value)) => {
                EVIDENCE_KINDS.contains(&kind)
                    && !value.trim().is_empty()
                    && !value.chars().any(char::is_whitespace)
            }
            None => false,
        };
        if !ref_ok {
            return Err(if self.evidence_ref.trim().is_empty() {
                ContextEffectError::MissingEvidence
            } else {
                ContextEffectError::MalformedEvidenceRef(self.evidence_ref.clone())
            });
        }
        if verify_world_ref(&self.world_ref).is_err() {
            return Err(ContextEffectError::InvalidWorldRef);
        }
        let hash_ok = self
            .bundle_content_hash
            .strip_prefix("sha256:")
            .is_some_and(|d| {
                d.len() == 64
                    && d.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            });
        if !hash_ok {
            return Err(ContextEffectError::InvalidBundleHash);
        }
        Ok(())
    }
}

/// Per-source demotion summary derived from the ledger, scoped to one world.
#[derive(Debug, Default, Clone)]
pub struct CalibrationSummary {
    /// source_id -> (total effects, demotion weight).
    entries: HashMap<String, (usize, f64)>,
}

impl CalibrationSummary {
    pub fn penalty_for(&self, source_id: &str) -> f64 {
        self.entries
            .get(source_id)
            .map(|(_, w)| w.min(0.5))
            .unwrap_or(0.0)
    }
    pub fn effect_count_for(&self, source_id: &str) -> usize {
        self.entries.get(source_id).map(|(n, _)| *n).unwrap_or(0)
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Outcome of one batch ingestion.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContextEffectIngestReport {
    pub accepted: usize,
    pub skipped_duplicates: usize,
    /// (index in the submitted batch, rejection reason).
    pub rejected: Vec<(usize, String)>,
}

/// Append-only JSONL store for settlement-backed context effects.
#[derive(Debug, Clone)]
pub struct ContextEffectStore {
    path: PathBuf,
}

impl ContextEffectStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Validate and append one effect. Existing `effect_id`s are skipped
    /// (idempotent); records are never rewritten.
    pub fn append(
        &self,
        effect: &ContextEffect,
    ) -> Result<bool, ContextEffectError> {
        effect.validate()?;
        let existing = self.load().unwrap_or_default();
        if existing.iter().any(|e| e.effect_id == effect.effect_id) {
            return Ok(false);
        }
        let mut line = serde_json::to_string(effect)
            .map_err(|e| ContextEffectError::Io(e.to_string()))?;
        line.push('\n');
        use std::io::Write;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ContextEffectError::Io(e.to_string()))?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| ContextEffectError::Io(e.to_string()))?;
        file.write_all(line.as_bytes())
            .map_err(|e| ContextEffectError::Io(e.to_string()))?;
        Ok(true)
    }

    /// Load all recorded effects (strict: one malformed line fails the read
    /// so a tampered ledger is never silently half-trusted).
    pub fn load(&self) -> Result<Vec<ContextEffect>, ContextEffectError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let text =
            std::fs::read_to_string(&self.path).map_err(|e| ContextEffectError::Io(e.to_string()))?;
        let mut out = Vec::new();
        for (i, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let effect: ContextEffect = serde_json::from_str(line).map_err(|e| {
                ContextEffectError::Io(format!("malformed ledger line {}: {e}", i + 1))
            })?;
            out.push(effect);
        }
        Ok(out)
    }

    /// Prospective calibration summary for one world. Effects from other
    /// worlds never influence this world's retrieval (lineage stays pinned).
    pub fn summary(&self, world_ref: &str) -> Result<CalibrationSummary, ContextEffectError> {
        let mut summary = CalibrationSummary::default();
        for effect in self.load()? {
            if effect.world_ref != world_ref {
                continue;
            }
            let weight = effect.kind.demotion_weight();
            for source in &effect.affected_sources {
                let entry = summary.entries.entry(source.clone()).or_insert((0, 0.0));
                entry.0 += 1;
                entry.1 += weight;
            }
        }
        Ok(summary)
    }
}
