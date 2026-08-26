# AGENTS.md

## Normative Language

The key words MUST, MUST NOT, REQUIRED, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in RFC 2119.

Implementation-defined means the behavior is part of the implementation contract, but this specification does not prescribe one universal policy. Implementations MUST document the selected behavior.

## Purpose

This file defines how coding agents MUST operate in this repository.

The repository implements **Context Kernal**, a Rust, ports-and-adapters, single-binary MCP service
that unifies:
- Context7-style documentation retrieval
- Context Mode-style context persistence and continuity
- Ref-style reference retrieval

Agents working in this repository MUST follow the rules in this file.

---

## Core Operating Principles

1. **Codebase evidence over documentation**
   - Treat the repository source code as the primary source of truth.
   - Verify implementation claims using code, tests, migration files, config files, and actual
     repository contents.
   - Do **not** claim something is implemented merely because:
     - it appears in `SPEC.md`
     - it appears in `README.md`
     - it appears in issue text
     - it appears in comments or TODOs
   - Documentation may describe intent; only code and tests prove behavior.

2. **File-based memory is required**
   - Agents MUST maintain persistent project state in repository files.
   - Agents MUST read existing project memory files before starting work.
   - Agents MUST update project memory files after meaningful changes.
   - Memory files are the shared continuity mechanism across agent sessions.

3. **Ports-and-adapters discipline**
   - Preserve separation between domain, application, ports, and adapters.
   - Do not introduce infrastructure details into domain/core crates.
   - Do not bypass ports just because a direct implementation is faster.

4. **Small, verifiable changes**
   - Prefer incremental, testable changes.
   - Every significant change MUST leave evidence in code, tests, or snapshots.
   - If behavior changes, update or add tests to prove it.

5. **No fabricated completion claims**
   - Do not say work is complete unless code, tests, and repository state support the claim.
   - If something is partially implemented, say exactly what exists and what is missing.

---

## Required Project Memory Files

Agents MUST use the `.agents/` directory for persistent, file-based working memory.

Required files:

- `.agents/project_state.md`
- `.agents/current_plan.md`
- `.agents/work_log.md`
- `.agents/decisions.md`
- `.agents/open_questions.md`
- `.agents/evidence.md`
- `.agents/OBSERVED_DEBT.md`
- `.agents/LESSONS.md`
- `.agents/current_status.yml`

If the directory or files do not exist, the agent MUST create them.

### File purposes

#### `.agents/project_state.md`
Canonical snapshot of current implementation status.

Must include:
- current architecture status by crate/module
- implemented features
- partially implemented features
- known gaps
- current migration/schema status
- current MCP tools implemented
- current adapters implemented
- current test coverage summary

This file should describe **actual repository state**, not aspirational state.

#### `.agents/current_plan.md`
The active task plan.

Must include:
- objective
- scope
- constraints
- ordered implementation steps
- explicit files/crates expected to change
- validation plan

Update this before coding if the task is non-trivial.

#### `.agents/work_log.md`
Chronological log of work performed.

Each entry should include:
- date/time
- task performed
- files changed
- tests run
- results
- blockers if any

#### `.agents/decisions.md`
Architecture and implementation decisions.

Each decision entry should include:
- title
- date
- context
- decision
- alternatives considered
- evidence in codebase
- consequences

#### `.agents/open_questions.md`
Outstanding uncertainties that require resolution.

Must distinguish:
- blocked questions
- non-blocking follow-ups
- assumptions currently in use

#### `.agents/evidence.md`
Repository-grounded evidence for important claims.

For each claim, record:
- claim
- evidence type (`code`, `test`, `migration`, `config`, `snapshot`)
- file path(s)
- line references if available
- confidence
- notes

This file is especially important when summarizing project status.

#### `.agents/OBSERVED_DEBT.md`
Concrete out-of-scope debt, gaps, risks, or defects found while working but
outside the current task. Each entry needs evidence, impact, and a next move.
Remove an entry when resolved; do not use this file as a backlog of ideas.
Update existing entries instead of duplicating them.

#### `.agents/LESSONS.md`
Verified, reusable project lessons specific to this repository.

Each entry should include:
- lesson statement
- date observed
- evidence (file, test, command, or incident)
- scope where the lesson applies

Record only lessons verified against repository evidence; update existing
entries instead of duplicating them.

#### `.agents/current_status.yml`
Resumable handoff state so any agent (or session) can continue cold.

Must include:
- objective
- worktree state
- changed files
- completed work
- verification performed and results
- remaining steps
- blockers
- decisions taken

---

## Workflow and Permissions

* Investigate before asking. Use `rg --files`, then `rg`, and read narrow source,
  test, spec, and history ranges until evidence is sufficient.
* Use `$understand-chat`, `$understand-explain`, or `$understand-diff` only when
  relationships or blast radius would otherwise require broad reading. Verify
  graph results against source and tests.
* Prefer `rust-analyzer` semantics for Rust structural questions before
  grep-and-recompile loops.
* Read every file before editing it; inspect one nearby pattern and its tests.
* Keep changes small and milestone-scoped. Preserve unrelated worktree changes
  and avoid unrelated formatting.
* Ask before adding or upgrading dependencies; changing persisted schemas, ID
  grammar, policy precedence, exit codes, public interfaces, CI, or deployment;
  deleting files; or expanding the current milestone.
* Update a spec or ADR when changing a public contract, persisted schema,
  architecture boundary, proof level, or implementation-defined behavior.
* Review diffs for correctness, security, compatibility, test strength,
  needless complexity, and architectural coherence.
* Keep secrets, credentials, private keys, `.env` contents, and sensitive
  payloads out of code, fixtures, logs, traces, evidence, and instructions.
* Never bypass authority, weaken a sandbox, allow unknown operations, or
  silently fall back when policy or jail infrastructure is unavailable.
* Make small focused commits when the task calls for commits. Do not push, open
  a pull request, deploy, or publish unless the user asks.

Before non-trivial work, update `.agents/current_plan.md` with explicit
validation steps and the files/crates expected to change.

After meaningful work, update the project memory files listed in
[Required Project Memory Files](#required-project-memory-files).

---

## Handoff

Keep `.agents/current_status.yml` resumable: objective, worktree state, changed
files, completed work, verification, remaining steps, blockers, and decisions.

Record only concrete out-of-scope debt in `.agents/OBSERVED_DEBT.md`, verified
reusable project lessons in `.agents/LESSONS.md`, and unresolved owner choices
in `.agents/open_questions.md`. Update existing entries instead of duplicating
them.

Run `just context-check` before handoff.

### Build-lock and artifact discipline

Only one agent may perform compile-heavy work against the shared build directory
at a time. Do not launch overlapping Cargo builds merely to reduce wall-clock
time.

Preserve hot incremental build state. Do not use routine `cargo clean` as a
response to slow compilation or ordinary test failure.

Watch build-artifact growth when compile-heavy work is active:

```sh
du -sh target
```

If `target/` exceeds the project's documented build-artifact ceiling, first
determine what is consuming space and whether an active build owns those
artifacts. Remove stale artifacts before destroying useful active build state.

Use a full `cargo clean` only when:

* the project's documented build-cache policy explicitly requires it,
* stale/corrupt artifacts are supported by evidence,
* a true clean rebuild is part of the requested verification, or
* the configured artifact ceiling has been exceeded and narrower cleanup cannot
  safely recover sufficient space.

Never clean a target directory while another agent or process is compiling into
it.

Do not increase Cargo build jobs or test concurrency to compensate for slow
builds without evidence that memory and scheduling permit it. If a build or
test is OOM-killed, diagnose memory pressure and the failing process rather than
retrying with greater parallelism.

Global `sccache` and linker acceleration may reduce repeated build cost, but
they do not weaken the single-writer build-lock rule or any SEA Forge
verification requirement.

---

## Implementation Verification Rules

### Priority order of evidence

When inspecting the repository, weigh evidence in this order:

1. source code
2. tests
3. migrations/schema
4. config/examples
5. snapshots/golden files
6. documentation

### Evidence standard

A feature is only considered implemented if there is repository evidence such as:

- executable code path
- test coverage
- migration/schema support
- configuration wiring
- registered MCP tool
- adapter wiring in bootstrap/runtime

Strong evidence examples:
- a trait implementation wired into runtime
- a CLI command that is registered and tested
- a migration file plus repository code using the resulting tables
- an MCP handler registered in code with tests exercising it

Weak evidence examples:
- comments
- TODO markers
- unused structs
- dead code
- unreferenced modules
- documentation only

### Required verification checklist

Before reporting implementation status for any feature, check:

- Is the code present?
- Is it reachable from runtime/bootstrap?
- Is it covered by at least one test or integration path?
- Is required schema/config support present?
- Is there evidence the feature is actually invoked or invocable?

If any answer is “no”, describe the feature as partial or incomplete.

### Do not cite docs as proof

Agents MUST NOT use the following as sole evidence:
- `README.md`
- `SPEC.md`
- issue text
- PR description
- comments
- external documentation

These may be used for intent/context, but not as proof of implementation.

---

## Required Status Language

When summarizing status, agents MUST use precise categories:

- `implemented`
- `partially implemented`
- `stubbed`
- `not implemented`
- `unclear; needs code inspection`

Avoid vague phrases like:
- “should work”
- “appears complete”
- “basically done”

unless followed by specific repository evidence.

---

## Coding Standards

### Architecture

Preserve the intended layering:

- `ck-core`: domain types and ports only
- `ck-app`: use cases and orchestration
- adapters: infrastructure-specific implementations
- `ck-bin`: composition/bootstrap only

Rules:
- no SQL in `ck-core`
- no transport code in `ck-core`
- no HTTP framework types in domain logic
- no MCP protocol details inside storage abstractions
- no direct adapter coupling across unrelated crates if a port should exist

### Changes to interfaces

If a port/trait changes:
- update all implementations
- update tests
- record the decision in `.agents/decisions.md`

### Schema changes

If SQLite schema changes:
- add a migration
- update schema-related tests
- update `.agents/project_state.md`
- note compatibility implications in `.agents/decisions.md`

### MCP changes

If MCP tools or JSON contracts change:
- update handlers
- update DTOs
- update tests/golden outputs
- update `.agents/evidence.md` with exact handler registration evidence

---

## Testing Expectations

Agents MUST verify changes with the strongest reasonable validation available.

Preferred order:
1. unit tests
2. integration tests
3. snapshot/golden tests
4. manual runtime verification
5. static inspection only, if nothing else is possible

### Minimum expectations by change type

#### Domain logic
- unit tests required

#### Storage/schema
- migration/integration tests required

#### MCP handlers/contracts
- handler tests or integration tests required

#### Transport/bootstrap wiring
- integration or startup tests preferred

#### Bug fixes
- add a regression test when practical

### If tests cannot be run

If the environment prevents running tests:
- say so explicitly
- explain why
- provide the exact commands that should be run
- do not claim verification beyond static inspection

---

## File-Based Memory Templates

### `.agents/project_state.md`

```markdown
# Project State

## Last Updated
- Date:
- By:

## Current Repository Status

### Crates
- ck-core:
- ck-config:
- ck-app:
- ck-storage-sqlite:
- ck-search-fts:
- ck-ranker:
- ck-memory:
- ck-artifacts:
- ck-mcp:
- ck-transport-stdio:
- ck-transport-sse:
- ck-auth:
- ck-observability:
- ck-scheduler:
- ck-bin:

### Implemented Features
-

### Partially Implemented Features
-

### Not Yet Implemented
-

### Database / Migrations
-

### MCP Tools
-

### Source Adapters
-

### Verification Summary
-
```

### `.agents/current_plan.md`

```markdown
# Current Plan

## Objective
-

## Scope
-

## Constraints
-

## Files / Crates Expected To Change
-

## Steps
1.
2.
3.

## Validation Plan
-
```

### `.agents/work_log.md`

```markdown
# Work Log

## YYYY-MM-DD HH:MM
- Task:
- Files changed:
- Tests run:
- Results:
- Notes:
```

### `.agents/decisions.md`

```markdown
# Decisions

## Decision: <title>
- Date:
- Context:
- Decision:
- Alternatives considered:
- Evidence in codebase:
- Consequences:
```

### `.agents/open_questions.md`

```markdown
# Open Questions

## Blocking
-

## Non-Blocking
-

## Active Assumptions
-
```

### `.agents/evidence.md`

```markdown
# Evidence Log

## Claim
- Claim:
- Status: implemented | partial | stubbed | not implemented | unclear
- Evidence type:
- Files:
- Lines:
- Notes:
- Confidence:
```

### `.agents/OBSERVED_DEBT.md`

```markdown
# Observed Debt

Record concrete debt, gaps, risks, or defects found while working but outside
the current task. Each entry needs evidence, impact, and a next move. Remove an
entry when resolved; do not use this file as a backlog of ideas.

<!-- Entry format:
## Open: Short problem statement
- Observed: YYYY-MM-DD
- Evidence: file, line, command, or failing proof
- Impact: concrete cost or risk
- Next move: smallest credible resolution
- Scope: why it was not fixed in the discovering task
-->
```

### `.agents/LESSONS.md`

```markdown
# Lessons

Verified, reusable project lessons only. Each entry needs evidence and scope.
Remove or rewrite an entry when superseded; update entries instead of
duplicating them.

<!-- Entry format:
## Lesson: <short statement>
- Observed: YYYY-MM-DD
- Evidence: file, test, command, or incident
- Scope: where this lesson applies
- Confidence: high | medium | low
-->
```

### `.agents/current_status.yml`

```yaml
# Resumable handoff state. Keep current enough for any agent to resume cold.
objective:
worktree_state:
changed_files: []
completed_work: []
verification:
remaining_steps: []
blockers: []
decisions: []
```

---

## Change Reporting Requirements

When reporting work to the user, agents SHOULD include:

1. what changed
2. what evidence supports the change
3. what tests were run
4. what remains incomplete
5. what project memory files were updated

If project memory files were not updated, explain why.

---

## Forbidden Behaviors

Agents MUST NOT:

- invent implementation details
- report documentation intent as implemented behavior
- skip updating memory files after meaningful work
- silently change architecture boundaries
- leave schema changes undocumented
- claim a feature works without checking code paths
- rely on external docs as proof of implementation

---

## Preferred Completion Checklist

Before concluding a meaningful task, the agent SHOULD verify:

- [ ] Relevant code inspected directly
- [ ] `.agents/current_plan.md` updated
- [ ] Implementation completed or partial status clearly stated
- [ ] Tests run or inability documented
- [ ] `.agents/work_log.md` updated
- [ ] `.agents/project_state.md` updated
- [ ] `.agents/evidence.md` updated
- [ ] `.agents/decisions.md` updated if needed
- [ ] `.agents/open_questions.md` updated if needed
- [ ] `.agents/OBSERVED_DEBT.md` / `.agents/LESSONS.md` updated if new debt or verified lessons were found
- [ ] `.agents/current_status.yml` updated (resumable handoff state)
- [ ] `just context-check` run before handoff

---

## Final Rule

If there is a conflict between repository documentation and repository code, **trust the code** and
record the discrepancy in `.agents/open_questions.md` or `.agents/evidence.md`.
