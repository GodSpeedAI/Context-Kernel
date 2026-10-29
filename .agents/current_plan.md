# Current Plan — Cognitive Representation Architecture for docs/ (COMPLETE 2026-10-24)

## Objective
Reconcile the documentation system with the 2026-10 context-pipeline implementation
(typed Context Requirement → resolve_context → CEP godspeed.context_bundle → settlement-
backed Context Effect calibration) and add cognitively economical representations:
verified Mermaid diagrams and one standalone interactive HTML explainer, integrated into
the existing 9-layer knowledge system. No application code, test, config, or CI changes.

## Modification boundary
- Persistent deliverables: `docs/**` only, plus narrowly justified root `README.md` corrections.
- Agent-memory updates in `.agents/**` are required by repo AGENTS.md governance and are
  working memory, not product deliverables.
- Preserve the untracked `.jolli/` directory untouched.

## Verified drift (evidence: crates/ck-mcp/src/lib.rs:123-134, ck-app/src/context.rs,
## ck-app/src/context_effect.rs, ck-mcp/src/cep_bundle.rs, docs grep audit 2026-10-23)
1. `docs/README.md` — System-in-One-Picture routes AclContextAgent directly to Storage,
   contradicting the real path (AclContextAgent → AppServices::resolve_context); concept 8
   still names the deprecated `ContextPacketCreated` as the canonical artifact; effect
   ledger and CEP projection absent.
2. `docs/reference/mcp-tools.md` — missing the `ingest_context_effects` tool (dispatch
   ck-mcp/src/lib.rs:134); context_required spec needs a bundle-first check.
3. `docs/source-map.md` — missing resolve_context / ContextEffectStore / cep_bundle /
   types/context.rs symbols and the four new test suites; stale L449 line ref.
4. `docs/concepts.md` — Context Packet entry predates the CEP bundle; no entries for
   Context Requirement, Context Bundle, Context Effect, world_ref, completeness.
5. `docs/mental-model.md` — invariants/state-ownership predate the context pipeline.
6. `docs/security.md` — POL-ACL-002 resolve_context semantics (requires_scope vs
   out_of_scope, private corpora never auto-ingested) not covered.
7. `docs/subsystems/application-orchestration.md`, `subsystems/mcp-and-protocols.md`,
   `subsystems/retrieval-and-ranking.md` — context resolution / bundle-first egress /
   calibration-penalty stage missing.
8. `docs/explanation/retrieval-and-ranking-design.md` — no settlement-backed calibration
   section. `docs/explanation/known-limits-and-debt.md` — matrix needs currency check.
9. `docs/workflows/agentic-capability-loop.md` — boundary view current (updated 2026-10-05)
   but no internal resolution-pipeline trace; link a dedicated workflow page.
10. Root `README.md` — tool surface / SWE_SEED loop section predate the bundle.

## Deliverables
A. New page `docs/workflows/context-requirement-resolution.md` — end-to-end execution
   trace with Mermaid sequence, data-access decision flowchart, completeness-branch
   diagram, calibration-loop diagram, stage table with file:line evidence.
B. New interactive single-file explainer `docs/explainers/context-requirement-explorer.html`
   — exact model of E2 gates + resolve_context 8 stages + assemble() truthfulness rules,
   real SHA-256 canonical-JSON integrity computed in-page, outcome/reason/access panel,
   illustrative candidate corpus clearly labeled, presets for every terminal branch.
C. Mermaid updates in `docs/README.md` (system picture) and `docs/architecture.md` (only
   diagrams that are actually stale).
D. Surgical reconciliation of items 2-10 above.
E. Navigation: `docs/documentation-map.md` rows for A+B; links from README/workflow pages.
F. Memory updates per AGENTS.md: work_log, evidence, project_state, current_status.yml,
   decisions, open_questions (record the missing-justfile discrepancy).

## Validation steps
- Trace every technical claim in A/B/C/D to source (file:line) — evidence recorded.
- Mermaid: static syntax review (no mmdc available); keep diagram grammar conservative.
- HTML: node syntax check of embedded JS; headless-chromium (ms-playwright cache)
  file:// screenshot + DOM dump at desktop and mobile widths; SHA-256 JS verified
  against node crypto on known vectors; grep for forbidden network refs.
- Idempotence: re-run drift greps after edits; a second reconciliation pass must
  produce no substantive changes; `git status` diff limited to docs/, README.md, .agents/.
- `just context-check` cannot run (no justfile at repo root) — record in open_questions.

## Working notes
- context_required egress is bundle-first: {context_bundle, context_envelope(deprecated),
  context_packet(deprecated), outcome, reason, access} (ck-mcp/src/lib.rs:562-569).
- NoContextReason strings: corpus_root_unconfigured, corpus_not_found, no_matching_content,
  corpus_unavailable, corpus_scope_unresolved, corpus_excluded, budget_exhausted,
  no_fresh_content. Omission codes: max_results_reached, token_budget_exhausted,
  stale_source_excluded, required_distinction_absent, citation_content_truncated (legacy).
- Envelope: completeness_status complete_for_context/partial/minimal; none carries
  questions (never references); complete carries references (never omissions);
  integrity.content_hash = sha256(canonical JSON of envelope minus integrity), floats
  rounded to 6 decimals, -0.0 → 0.0 (ck-mcp/src/cep_bundle.rs:59-108).
- Calibration: source_contradicted 0.20, source_stale 0.10, per-source penalty capped at
  0.5, score *= (1 - penalty), per-world, append-only JSONL, settlement evidence_ref
  mandatory (ck-app/src/context_effect.rs, ck-app/src/context.rs:464-507).

## Outcome (2026-10-24)
All deliverables A-F landed; items 1-10 reconciled. Evidence:
- A. `docs/workflows/context-requirement-resolution.md` — written.
- B. `docs/explainers/context-requirement-explorer.html` — written; JS syntax, SHA-256 vectors,
  branch harness, and desktop/mobile renders verified.
- C. `docs/README.md` system picture rebuilt around resolve_context -> bundle -> effects.
  `docs/architecture.md` required no change (no stale context diagram).
- D. Items 2-10 reconciled; plus `docs/mcp-contracts.md`, `docs/troubleshooting.md`, and
  `docs/explanation/known-limits-and-debt.md` (stale single-adapter claim corrected).
- E. `docs/documentation-map.md` has rows for the workflow page and the explainer; README and
  the capability-loop page link both.
- F. Memory files updated. Recorded discrepancy: no justfile, so `just context-check` cannot run.

Verification: all relative doc links resolve; no `file:///` links remain; docs-only diff.
