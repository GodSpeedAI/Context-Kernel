# Current Plan — EGM Bootstrap (contract READMEs across the repo tree)

## Objective
Apply the `egm-bootstrap` skill (/home/sprime01/projects/elephant-goldfish-model/skills/egm-bootstrap/SKILL.md)
to this repository: generate/correct EGM-compliant `README.md` contract files
(Interface / Invariants / Side Effects / Verification) for every pruned directory
in the tree. Documentation only — no code, schema, config, or CI changes.

## Safety gate
Discovery found 34+ directories (>15). User approved full-tree bootstrapping on
2026-10-30 with the explicit constraint: **sequential execution, one agent at a
time, no compilation** (RAM-constrained machine: 7.8GiB total, ~1.9GiB free at
start; swap in use).

## Modification boundary
- Create/update ONLY `README.md` contract files inside: `crates/**`, `integration/**`,
  `scripts/`, `crates/README.md`, and root `README.md` (Correction Policy: preserve
  existing title/overview, merge EGM sections).
- `docs/**`, `config/**`, `.agents/**` (hidden), `target/` are pruned/ignored — no
  contract files written there. `docs/` and `config/` contain no source-code
  extensions per the skill's pruning rules.
- No git commits, no dependency changes, no compilation (`cargo`/`rustc` forbidden —
  build-lock + RAM discipline). Verification commands are recorded in contracts, not run.

## Discovery map (pruned, 48 contract directories)
- Root (Hay, Correction Policy on existing README.md)
- `crates/` (Hay)
- 20 × `crates/ck-*` (Hay → links `./src/README.md`) and 20 × `crates/ck-*/src` (Peanut)
- `crates/ck-core/src/{entities,ports,types}` (3 leaf Peanuts; only ck-core has nested src)
- `integration/` (Hay) → `integration/agentic_capability_loop` (Peanut; legacy deprecated
  dual-read shim, Python)
- `scripts/` (Peanut; run-exact-test.sh)
- VerificationCommand: workspace `cargo test --workspace`; per-crate `cargo test -p <pkg>`
  (package names match directory names). integration/scripts: no test runner → omit.

## Execution strategy (sequential workers, one at a time)
1. W1: ck-core (+src tree), ck-config
2. W2: ck-app
3. W3: ck-mcp, ck-transport-stdio
4. W4: ck-storage-sqlite, ck-search-fts, ck-memory
5. W5: ck-adapter-filesystem, ck-adapter-github, ck-adapter-web, ck-artifacts, ck-auth
6. W6: ck-ranker, ck-vector, ck-scheduler, ck-embeddings, ck-observability, ck-transport-sse
7. W7 (fallback): integration/, scripts/
8. Root compile: `crates/README.md`, `integration/README.md` (Hay), root `README.md`
   (Correction Policy) from worker JSON payloads.
9. Final verification: read every created/modified contract file; check EGM sections,
   code grounding, markdown syntax, and that every relative link resolves on disk.
10. Drift check (user pre-authorized "may do the drift check after") via egm-drift-report
    skill on modified contracts.
11. Memory updates: work_log, evidence, project_state, current_status.yml (+decisions if
    warranted). `just context-check` cannot run (no justfile — already tracked in
    OBSERVED_DEBT; update existing entry, don't duplicate).

## Worker contract
Each worker: reads skill SKILL.md + templates.md first; inline scouting (no nested
agents); reads only non-test source; cites [file:symbol] for every entry; writes only
README.md in its partition; NO cargo/git/code edits; returns JSON subtree_index payload
(worker-return.schema.json) as its final message.

## Validation steps
- Every Interface/Invariant/Side-Effects entry traceable to a real symbol (scout cites).
- All `./child/README.md` links resolve on disk (script check + manual read).
- No speculative symbols: spot-check each contract against source with rg.
- `git status` limited to README.md files + `.agents/`.
- RAM monitored (`free -h`) before/after each worker; never two agents at once.

## Outcome (2026-10-30)
Complete. 48 contract files written (W1-W8 sequential workers + root compile:
`crates/README.md`, root `README.md` Correction-Policy merge incl. `## System Overview`
heading). Final verification: all 48 files read; link/section/fence script checks pass;
every `[file:Symbol]` citation grepped against the cited file (zero misses); 53/53
specific numeric/flag claims verified; pub-symbol coverage scan found 2 missing types in
`crates/ck-mcp/src/README.md` (`DomainIdentityError`, `EventEnvelope`) — fixed and
re-scanned clean. Drift report: `.agents/reports/2026-10-30_EGM_DRIFT_REPORT.md`.
Deviation from defaults: sequential workers (RAM), verification commands documented not
run, `integration/agentic_capability_loop` has no Verification section (no test runner).
`just context-check` cannot run (no justfile; tracked in OBSERVED_DEBT).
