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

Agents MUST use the `.agent/` directory for persistent, file-based working memory.

Required files:

- `.agent/project_state.md`
- `.agent/current_plan.md`
- `.agent/work_log.md`
- `.agent/decisions.md`
- `.agent/open_questions.md`
- `.agent/evidence.md`

If the directory or files do not exist, the agent MUST create them.

### File purposes

#### `.agent/project_state.md`
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

#### `.agent/current_plan.md`
The active task plan.

Must include:
- objective
- scope
- constraints
- ordered implementation steps
- explicit files/crates expected to change
- validation plan

Update this before coding if the task is non-trivial.

#### `.agent/work_log.md`
Chronological log of work performed.

Each entry should include:
- date/time
- task performed
- files changed
- tests run
- results
- blockers if any

#### `.agent/decisions.md`
Architecture and implementation decisions.

Each decision entry should include:
- title
- date
- context
- decision
- alternatives considered
- evidence in codebase
- consequences

#### `.agent/open_questions.md`
Outstanding uncertainties that require resolution.

Must distinguish:
- blocked questions
- non-blocking follow-ups
- assumptions currently in use

#### `.agent/evidence.md`
Repository-grounded evidence for important claims.

For each claim, record:
- claim
- evidence type (`code`, `test`, `migration`, `config`, `snapshot`)
- file path(s)
- line references if available
- confidence
- notes

This file is especially important when summarizing project status.

---

## Agent Workflow

### 1. Start by reading memory files

Before making changes, agents MUST read:

- `.agent/project_state.md`
- `.agent/current_plan.md`
- `.agent/decisions.md`
- `.agent/open_questions.md`
- `.agent/evidence.md`

If they are missing, create them with minimal initial structure.

### 2. Inspect the codebase directly

Before claiming what exists, inspect relevant files directly.

Priority order of evidence:
1. source code
2. tests
3. migrations/schema
4. config/examples
5. snapshots/golden files
6. documentation

### 3. Update the plan

Before non-trivial work:
- update `.agent/current_plan.md`
- write explicit validation steps
- identify which files/crates are expected to change

### 4. Implement incrementally

Make changes in small steps that preserve buildability when possible.

### 5. Verify with repository evidence

After changes:
- run relevant tests
- inspect generated or changed artifacts
- confirm behavior from code paths, not only comments
- record evidence in `.agent/evidence.md`

### 6. Update memory files

After meaningful work, update:
- `.agent/project_state.md`
- `.agent/work_log.md`
- `.agent/evidence.md`
- `.agent/decisions.md` if architectural choices were made
- `.agent/open_questions.md` if new unknowns arose

---

## Implementation Verification Rules

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
- record the decision in `.agent/decisions.md`

### Schema changes

If SQLite schema changes:
- add a migration
- update schema-related tests
- update `.agent/project_state.md`
- note compatibility implications in `.agent/decisions.md`

### MCP changes

If MCP tools or JSON contracts change:
- update handlers
- update DTOs
- update tests/golden outputs
- update `.agent/evidence.md` with exact handler registration evidence

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

### `.agent/project_state.md`

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

### `.agent/current_plan.md`

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

### `.agent/work_log.md`

```markdown
# Work Log

## YYYY-MM-DD HH:MM
- Task:
- Files changed:
- Tests run:
- Results:
- Notes:
```

### `.agent/decisions.md`

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

### `.agent/open_questions.md`

```markdown
# Open Questions

## Blocking
-

## Non-Blocking
-

## Active Assumptions
-
```

### `.agent/evidence.md`

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
- [ ] `.agent/current_plan.md` updated
- [ ] Implementation completed or partial status clearly stated
- [ ] Tests run or inability documented
- [ ] `.agent/work_log.md` updated
- [ ] `.agent/project_state.md` updated
- [ ] `.agent/evidence.md` updated
- [ ] `.agent/decisions.md` updated if needed
- [ ] `.agent/open_questions.md` updated if needed

---

## Final Rule

If there is a conflict between repository documentation and repository code, **trust the code** and
record the discrepancy in `.agent/open_questions.md` or `.agent/evidence.md`.
