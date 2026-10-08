//! Shared context-boundary types: the retrieval-completeness vocabulary, the
//! explicit no-context reasons, machine-readable omission codes, and the
//! immutable semantic-world reference gate (CEP-0008 `world:<name>@sha256:<64hex>`).
//!
//! These are the native Context Kernel spellings of facts that later project
//! onto the CEP `godspeed.context_bundle` profile. They are deliberately free
//! of any CEP/transport structure: the boundary adapter does the projection.

use serde::{Deserialize, Serialize};

/// How much of the matching context a resolution carries. Relative to the
/// query and corpus; never a claim about reality. Only `Complete` may be read
/// as "everything that matched within the declared retrieval contract is
/// here". Partial must never masquerade as complete, and absence must never
/// masquerade as anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalCompleteness {
    Complete,
    Partial,
    None,
}

/// Why a required-context acquisition resolved with zero usable results.
///
/// Zero-citation outcomes MUST NOT silently settle as successful required
/// context acquisition; absence is always explicit and machine-readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoContextReason {
    /// Corpus root itself was not configured / does not exist.
    CorpusRootUnconfigured,
    /// Requested corpus is not indexed and could not be discovered.
    CorpusNotFound,
    /// Corpus exists but no content matched the query.
    NoMatchingContent,
    /// The corpus exists but ingestion of the requested source failed, so no
    /// content could be searched.
    CorpusUnavailable,
    /// The corpus is private and the caller declared no (or the wrong) scope,
    /// so retrieval was refused without fabricating context.
    ScopeUnresolved,
    /// The corpus exists but the caller's own exclusions barred it.
    CorpusExcluded,
    /// The declared token budget could not cover even one result.
    BudgetExhausted,
    /// Content matched, but every candidate violated the declared freshness
    /// requirement and was excluded.
    StaleContent,
}

impl NoContextReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CorpusRootUnconfigured => "corpus_root_unconfigured",
            Self::CorpusNotFound => "corpus_not_found",
            Self::NoMatchingContent => "no_matching_content",
            Self::CorpusUnavailable => "corpus_unavailable",
            Self::ScopeUnresolved => "corpus_scope_unresolved",
            Self::CorpusExcluded => "corpus_excluded",
            Self::BudgetExhausted => "budget_exhausted",
            Self::StaleContent => "no_fresh_content",
        }
    }
}

/// Machine-readable omission codes recorded on a non-complete resolution.

/// Why a candidate `world_ref` cannot enter a protected boundary
/// (CEP-0008 / GodSpeed profile).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldRefError {
    /// Not `world:<name>@sha256:<64 lowercase hex>` with a valid name.
    Malformed,
}

impl std::fmt::Display for WorldRefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "world_ref is not world:<name>@sha256:<64 lowercase hex digest>"
        )
    }
}

impl std::error::Error for WorldRefError {}

/// Syntax-only gate for a `world_ref`. Context Kernel neither recomputes nor
/// verifies the digest (DomainForge owns the world; SEA-Forge re-verifies it);
/// it only refuses anything that is not a pinned, well-formed reference, so a
/// mutable alias or label can never stand in for a world revision.
pub fn verify_world_ref(world_ref: &str) -> Result<(), WorldRefError> {
    let (name_part, digest) = world_ref
        .split_once("@sha256:")
        .ok_or(WorldRefError::Malformed)?;
    let name = name_part
        .strip_prefix("world:")
        .ok_or(WorldRefError::Malformed)?;
    let digest_ok = digest.len() == 64
        && digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    // name: [a-z][a-z0-9]*(?:[._-][a-z0-9]+)*, at most 64 chars
    let mut chars = name.chars().peekable();
    let name_ok = name.len() <= 64
        && chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && {
            let mut prev_sep = false;
            let mut ok = true;
            for c in chars.by_ref() {
                if c.is_ascii_lowercase() || c.is_ascii_digit() {
                    prev_sep = false;
                } else if matches!(c, '.' | '_' | '-') && !prev_sep {
                    prev_sep = true;
                } else {
                    ok = false;
                    break;
                }
            }
            ok && !prev_sep
        };
    if digest_ok && name_ok {
        Ok(())
    } else {
        Err(WorldRefError::Malformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const D: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn world_ref_vectors_match_the_cep_pattern() {
        for ok in [
            format!("world:a@sha256:{D}"),
            format!("world:acme.core-v2_x@sha256:{D}"),
            format!("world:{}@sha256:{D}", "a".repeat(64)),
        ] {
            assert!(verify_world_ref(&ok).is_ok(), "rejected {ok:?}");
        }
        for bad in [
            "".to_string(),
            "world:test".to_string(),
            "world:test@sha256:abc".to_string(),
            format!("world:{}@sha256:{D}", "a".repeat(65)),
            format!("world:a-@sha256:{D}"),
            format!("world:a..b@sha256:{D}"),
            format!("world:1a@sha256:{D}"),
            format!("world:a@sha256:{}0", "b".repeat(64)),
            format!("world:Test@sha256:{D}"),
            format!("world:a@sha256:{}", "B".repeat(64)),
        ] {
            assert!(verify_world_ref(&bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn completeness_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&RetrievalCompleteness::None).unwrap(),
            "\"none\""
        );
        assert_eq!(
            serde_json::to_string(&RetrievalCompleteness::Complete).unwrap(),
            "\"complete\""
        );
    }
}

/// These become CEP `omissions` entries at the boundary.
pub const OMISSION_MAX_RESULTS: &str = "max_results_reached";
/// Legacy code: packet-carried citation content was cut short. Kept for wire
/// compatibility with historical packets.
pub const OMISSION_CONTENT_TRUNCATED: &str = "citation_content_truncated";
pub const OMISSION_TOKEN_BUDGET: &str = "token_budget_exhausted";
pub const OMISSION_STALE_SOURCE: &str = "stale_source_excluded";
pub const OMISSION_REQUIRED_CONCEPT: &str = "required_distinction_absent";
