# EGM Drift Report — 2026-10-30

Target: all 48 contract files generated/corrected by the `egm-bootstrap` run of
2026-10-30 (root `README.md`, `crates/README.md`, 20 × `crates/ck-*/README.md`,
20 × `crates/ck-*/src/README.md`, 3 × `crates/ck-core/src/{entities,ports,types}/README.md`,
`integration/README.md`, `integration/agentic_capability_loop/README.md`,
`scripts/README.md`).

Method (per `egm-drift-report` skill, run sequentially, no compilation):
1. Link resolution: every relative link target verified to exist on disk.
2. Section compliance: Peanut (Overview/Interface/Invariants/Side Effects/Verification)
   vs Hay (Overview/Subdirectories & Boundaries) vs root template.
3. Citation grounding: every `[file:Symbol]` citation grepped in the cited file
   (48 files, zero misses).
4. Claim verification: 53 specific numeric/flag/wire-shape claims grepped against
   source (all verified, including `1.0 / (1.0 + (-fts_score).exp())` sigmoid,
   weights 0.20/0.10, cap 0.5, ports/defaults, blocked IP ranges, tool names,
   route names, env var names, CLI subcommands).
5. Missing-coverage scan: every `pub struct|enum|trait` in each crate's `src/`
   checked for a mention in that crate's contract tree.

## README.md (root, Repository Blueprint)

- Initial check flagged a missing `## System Overview` heading (template
  compliance); fixed during the run by adding the heading above the existing
  overview prose. No code drift.

## crates/README.md

No drift detected.

## crates/ck-adapter-filesystem/README.md

No drift detected.

## crates/ck-adapter-filesystem/src/README.md

No drift detected.

## crates/ck-adapter-github/README.md

No drift detected.

## crates/ck-adapter-github/src/README.md

No drift detected.

## crates/ck-adapter-web/README.md

No drift detected.

## crates/ck-adapter-web/src/README.md

No drift detected.

## crates/ck-app/README.md

No drift detected.

## crates/ck-app/src/README.md

No drift detected.

## crates/ck-artifacts/README.md

No drift detected.

## crates/ck-artifacts/src/README.md

No drift detected.

## crates/ck-auth/README.md

No drift detected.

## crates/ck-auth/src/README.md

No drift detected.

## crates/ck-bin/README.md

No drift detected.

## crates/ck-bin/src/README.md

No drift detected.

## crates/ck-config/README.md

No drift detected.

## crates/ck-config/src/README.md

No drift detected.

## crates/ck-core/README.md

No drift detected.

## crates/ck-core/src/README.md

No drift detected.

## crates/ck-core/src/entities/README.md

No drift detected.

## crates/ck-core/src/ports/README.md

No drift detected.

## crates/ck-core/src/types/README.md

No drift detected.

## crates/ck-embeddings/README.md

No drift detected.

## crates/ck-embeddings/src/README.md

No drift detected.

## crates/ck-memory/README.md

No drift detected.

## crates/ck-memory/src/README.md

No drift detected.

## crates/ck-mcp/README.md

No drift detected.

## crates/ck-mcp/src/README.md

- Missing coverage (doc-to-code): public types `DomainIdentityError`
  (`Malformed`, `Placeholder` — agentic_capability_loop.rs:26) and
  `EventEnvelope` (agentic_capability_loop.rs:74) existed in source but were
  not referenced by the contract. FIXED during this drift pass: both added to
  the Interface section. Re-scan clean.

## crates/ck-observability/README.md

No drift detected.

## crates/ck-observability/src/README.md

No drift detected.

## crates/ck-ranker/README.md

No drift detected.

## crates/ck-ranker/src/README.md

No drift detected.

## crates/ck-scheduler/README.md

No drift detected.

## crates/ck-scheduler/src/README.md

No drift detected.

## crates/ck-search-fts/README.md

No drift detected.

## crates/ck-search-fts/src/README.md

No drift detected.

## crates/ck-storage-sqlite/README.md

No drift detected.

## crates/ck-storage-sqlite/src/README.md

No drift detected.

## crates/ck-transport-sse/README.md

No drift detected.

## crates/ck-transport-sse/src/README.md

No drift detected.

## crates/ck-transport-stdio/README.md

No drift detected.

## crates/ck-transport-stdio/src/README.md

No drift detected.

## crates/ck-vector/README.md

No drift detected.

## crates/ck-vector/src/README.md

No drift detected.

## integration/README.md

No drift detected.

## integration/agentic_capability_loop/README.md

- Section note (not code drift): no `## Verification` section — intentional;
  the Python shim has no test runner and the skill mandates omitting the
  section when no verification command exists.

## scripts/README.md

No drift detected.

## Summary

- 48 contracts; 1 real drift finding (ck-mcp/src: two undocumented public
  types) — fixed in-place and re-verified.
- 1 template-compliance fix (root `## System Overview` heading).
- 0 broken links, 0 unbalanced fences, 0 ungrounded citations,
  0 unverified specific claims, 0 remaining uncovered public symbols.
