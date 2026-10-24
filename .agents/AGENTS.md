The following files are used to maintain project memory and handoff state. Agents should update these files as part of their work, and include them in their change reports.

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
