# Canonical Boundary: Context Kernel, SWE_SEED, and SEA-Forge

Treat this document as the controlling responsibility boundary between Context Kernel, SWE_SEED, and SEA-Forge.

Current repository structure may contain temporary adapters, duplicated implementations, incomplete integrations, or historical responsibilities. Do not infer canonical ownership merely from where code currently lives.

## Core Distinction

```text
Context Kernel = contextual authority
SWE_SEED       = software-work orchestration and proof authority
SEA-Forge      = runtime action authority
```

Canonical shorthand:

```text
Context Kernel supplies the grounded working world.
SWE_SEED routes and proves the work.
SEA-Forge governs consequential action.
```

An even shorter formulation:

```text
Know through Context Kernel.
Work through SWE_SEED.
Act through SEA-Forge.
```

## The Three Questions

Every agentic software-work cycle must answer three separate questions.

### 1. What does the agent need to know for this job?

Context Kernel answers this question.

It constructs a bounded, provenance-bearing context packet containing the portion of the wider organizational and repository world relevant to the current task.

A context packet may include:

* the active task or case;
* actor and session identity;
* repository state;
* relevant files and symbols;
* architecture decisions;
* domain concepts;
* prior decisions;
* previous attempts;
* known failures;
* current status;
* open questions;
* applicable constraints;
* available tools;
* active policies;
* required proof conditions;
* citations and provenance;
* freshness and version information.

Context Kernel does not give the agent “all available context.”

It gives the agent the smallest sufficient working world for the current job.

```text
Context is not maximum retrieval.
Context is bounded relevance with provenance.
```

### 2. How should this software job be performed and proven?

SWE_SEED answers this question.

SWE_SEED converts a work request and its context packet into an executable software-work contract.

The work contract includes:

* selected route;
* required context;
* permitted work scope;
* artifact requirements;
* implementation procedure;
* proof commands;
* trace requirements;
* completion criteria;
* failure reporting;
* residue or reflection requirements.

SWE_SEED coordinates the work loop:

```text
request context
→ validate context sufficiency
→ select route
→ establish work contract
→ request authority where required
→ perform work
→ produce artifact
→ run proof
→ preserve trace
→ report observed result
```

SWE_SEED determines whether the software-work contract passed or failed.

It does not grant itself authority, invent missing context, or determine developmental capability.

### 3. May this operation happen under these conditions?

SEA-Forge answers this question.

SEA-Forge evaluates consequential operations against declared authority and policy.

Its canonical decision surface is:

```text
actor
+ operation
+ resource
+ execution context
+ applicable policy
→ allow / deny / escalate
```

SEA-Forge governs actions such as:

* reading protected information;
* crossing a context boundary;
* accessing secrets;
* writing files;
* calling external APIs;
* executing privileged commands;
* creating commits;
* opening or merging pull requests;
* modifying infrastructure;
* changing production systems;
* delegating to other agents;
* publishing data;
* performing high-risk or irreversible operations.

SEA-Forge records the authority decision and the evidence supporting it.

It does not choose the work route, assemble the context packet, or decide whether software proof passed.

## Important Meaning of “Authority”

The word `authority` is used in different senses across these systems.

### Contextual authority

Context Kernel is authoritative about:

```text
which retrieved material belongs in the current working context
where that material came from
what version or time boundary applies
which citations support it
whether the packet satisfies the requested context contract
```

This does not mean Context Kernel decides organizational policy or absolute truth.

A context packet is an authoritative, bounded projection for the task—not a complete representation of reality.

### Work-contract authority

SWE_SEED is authoritative about:

```text
which work route was selected
which artifacts were required
which proof commands were declared
which commands actually ran
whether those commands passed
what execution trace exists
whether the work contract was fulfilled
```

This does not mean SWE_SEED can authorize its own side effects.

### Runtime action authority

SEA-Forge is authoritative about:

```text
whether this actor may perform this operation
on this resource
under this context
according to these policies
```

This does not mean SEA-Forge decides that the context was sufficient or that the code is correct.

## Full Interaction Loop

```text
WorkRequested
    ↓
SWE_SEED identifies context requirements
    ↓
ContextRequired
    ↓
Context Kernel retrieves, scopes, ranks, and cites relevant material
    ↓
ContextPacketCreated
    ↓
SWE_SEED validates packet completeness and selects a route
    ↓
RouteSelected
    ↓
SWE_SEED proposes the first consequential operation
    ↓
ActionProposed
    ↓
SEA-Forge evaluates authority
    ↓
allow / deny / escalate
    ↓
SWE_SEED executes permitted work
    ↓
Additional consequential operation encountered
    ↓
SEA-Forge evaluates that operation
    ↓
SWE_SEED produces the required artifact
    ↓
ProofStarted
    ↓
SWE_SEED runs declared proof commands
    ↓
ProofCompleted
    ↓
Evidence package emitted for downstream settlement processing
```

SEA-Forge may be invoked multiple times during one SWE_SEED work cycle.

Authority is evaluated at consequential crossings, not only once at job startup.

## Responsibility Map

### Context Kernel Owns

Context Kernel owns:

* real-time context assembly;
* durable document and repository retrieval;
* contextual search;
* relevance ranking;
* citation resolution;
* provenance;
* freshness metadata;
* session-specific context;
* context-packet construction;
* context deduplication;
* retrieval boundaries;
* application of already-authorized access scope;
* context sufficiency signals;
* detection of missing or conflicting context.

Context Kernel may say:

```text
These materials are relevant to this task.

These citations support the supplied claims.

This packet was assembled from these sources and versions.

Required architecture context is missing.

Two retrieved sources conflict.

This packet is stale relative to the active repository state.
```

Context Kernel must not say:

```text
This file write is authorized.

This deployment may proceed.

This implementation is correct.

The work contract passed.

This capability is metabolized.
```

### SWE_SEED Owns

SWE_SEED owns:

* work-request intake;
* context-requirement declaration;
* route selection;
* agent instructions;
* task decomposition;
* work-contract formation;
* repository workflow;
* artifact requirements;
* proof commands;
* test, build, lint, and validation execution;
* trace capture;
* completion evidence;
* work-contract pass or failure;
* evidence packaging.

SWE_SEED may say:

```text
This task requires these context classes.

This route is appropriate for the work request.

These files and artifacts are in scope.

These proof commands must pass.

The declared proof commands passed or failed.

This evidence bundle records the completed attempt.
```

SWE_SEED must not say:

```text
I could not retrieve the required context, so I invented it.

The requested operation is probably safe, so I authorized it.

The proof command was inconvenient, so I replaced it silently.

A passing test means the organization has acquired the capability.
```

### SEA-Forge Owns

SEA-Forge owns:

* authenticated actor identity;
* operation classification;
* resource classification;
* execution-context evaluation;
* policy resolution;
* authority checks;
* allow, deny, and escalate decisions;
* approval workflows;
* runtime enforcement;
* protected-boundary interception;
* fail-closed behavior;
* authority evidence;
* policy-decision records;
* authority and policy drift detection.

SEA-Forge may say:

```text
This actor may read this resource for this work request.

This context scope may not include this protected source.

This file modification is allowed under these constraints.

This operation requires human approval.

This production action is prohibited.

This decision was evaluated against these policy versions.
```

SEA-Forge must not say:

```text
These are the most relevant documents for the task.

This route is the best engineering approach.

The implementation satisfies the acceptance criteria.

The tests prove the capability is metabolized.
```

## Context Access and Action Authority

Context retrieval can itself cross an authority boundary.

For example:

```text
SWE_SEED requests context
    ↓
Context Kernel identifies a potentially relevant protected source
    ↓
SEA-Forge evaluates whether the actor may access that source
    ↓
allow
    → Context Kernel retrieves and cites it

deny
    → Context Kernel excludes it and records an unavailable-context condition

escalate
    → Context Kernel waits for an approval result or returns a constrained packet
```

The division is:

```text
Context Kernel determines relevance.

SEA-Forge determines permission.

SWE_SEED determines whether the resulting packet is sufficient to perform the job.
```

Context Kernel may enforce the scope supplied by SEA-Forge, but it must not invent the policy governing that scope.

## Required Contracts

### Context requirement

SWE_SEED should send a context request shaped like:

```yaml
context_requirement:
  work_request_id:
  actor_id:
  task:
  repository:
  active_case:
  required_context_classes:
    - architecture
    - source_code
    - current_status
    - prior_decisions
    - tests
    - policy
  preferred_sources:
  prohibited_sources:
  freshness_requirement:
  token_or_size_budget:
  citation_required: true
  minimum_completeness:
```

### Context packet

Context Kernel should return:

```yaml
context_packet:
  packet_id:
  work_request_id:
  actor_id:
  assembled_at:
  source_state:
  scope:
  materials:
    - source_id:
      source_type:
      version:
      excerpt_or_reference:
      relevance:
      citation:
      access_decision_reference:
  unresolved_conflicts:
  missing_context:
  freshness:
  completeness:
  content_hash:
```

### Route and work contract

SWE_SEED should derive:

```yaml
work_contract:
  work_request_id:
  context_packet_id:
  context_packet_hash:
  route:
  objective:
  files_in_scope:
  files_out_of_scope:
  artifact_contract:
  proof_commands:
  authority_requirements:
  trace_policy:
  expected_result:
  failure_policy:
```

### Authority request

For each consequential action, SWE_SEED should submit:

```yaml
authority_request:
  work_request_id:
  context_packet_id:
  context_packet_hash:
  actor:
  operation:
  resource:
  execution_context:
  route:
  policy_references:
  risk_class:
  requested_duration:
  requested_scope:
```

### Authority decision

SEA-Forge should return:

```yaml
authority_decision:
  decision_id:
  work_request_id:
  actor:
  operation:
  resource:
  decision: allow | deny | escalate
  constraints:
  policy_references:
  approval_reference:
  context_packet_hash:
  expires_at:
  evidence_reference:
```

## Hash and Identity Continuity

The same work cycle must preserve referential continuity across systems.

At minimum, retain:

```text
work_request_id
context_packet_id
context_packet_hash
route_id
authority_decision_id
proof_run_id
trace_id
correlation_id
causation_id
```

This allows downstream systems to prove:

```text
This work used this context packet.

This action was authorized against that context.

This artifact resulted from that route.

This proof evaluated that artifact.

This evidence belongs to the same work request.
```

If the context packet changes materially after authority is granted, the authority decision may no longer be valid.

The system must either:

* reevaluate authority;
* prove that the change is irrelevant to the decision;
* or stop the protected operation.

```text
Authority granted against context A
does not automatically authorize action under materially different context B.
```

## Critical Non-Equivalences

The implementation must preserve these distinctions:

```text
retrieved information ≠ authorized information

authorized information ≠ relevant information

relevant information ≠ sufficient context

sufficient context ≠ correct route

correct route ≠ authorized action

authorized action ≠ correct implementation

passing proof ≠ complete context

large context ≠ good context

citation present ≠ claim supported

repository access ≠ permission to modify

permission to modify ≠ permission to commit

permission to commit ≠ permission to merge
```

## Fail-Closed Rules

### Missing context

When required context is missing:

```text
do not invent it
do not silently reduce the contract
do not treat absence as permission
```

SWE_SEED should:

* request additional context;
* select a safe discovery route;
* narrow the work;
* return a blocked-context result;
* or escalate.

### Missing citation or provenance

When a context claim requires provenance but none exists:

* mark it ungrounded;
* exclude it from authority-sensitive decisions;
* or require human confirmation.

### Denied context access

When SEA-Forge denies access to a source:

* Context Kernel must not retrieve or leak it;
* SWE_SEED must not infer its contents;
* the work contract must record the resulting limitation.

### Stale context

When source state materially changes after packet creation:

* invalidate or mark the packet stale;
* reevaluate the route;
* rerun affected authority decisions;
* rerun proof when the change affects the artifact or environment.

### Authority unavailable

When SEA-Forge cannot evaluate a protected operation:

* block the operation;
* do not degrade into self-authorization;
* preserve the request for retry or escalation.

### Proof unavailable

When SWE_SEED cannot run a required proof:

* do not claim completion;
* record the unverified state;
* preserve the produced artifact separately from a passing result.

## Failure Ownership

Route failures according to the invariant that was violated.

### Context Kernel failure

Examples:

* relevant material was omitted;
* stale information was presented as current;
* a citation points to the wrong source;
* context from another task or tenant leaked into the packet;
* the packet exceeded its budget without improving sufficiency;
* conflicting sources were silently collapsed;
* a protected source was retrieved outside its authorized scope.

### SWE_SEED failure

Examples:

* context requirements were incomplete;
* the context packet was ignored;
* the wrong route was selected;
* work exceeded the declared scope;
* required proof was not run;
* artifact and proof were not linked;
* the completion claim exceeded the evidence.

### SEA-Forge failure

Examples:

* a protected action occurred without a decision;
* the wrong actor, operation, resource, or context was evaluated;
* a denied action proceeded;
* an escalation was bypassed;
* authority was reused after material context changed;
* the system failed open when policy evaluation was unavailable.

## Anti-Duplication Rules

### Context Kernel must not become a work orchestrator

It may recommend that context is missing or contradictory.

It must not choose the implementation route, modify the repository, or run software proof commands.

### SWE_SEED must not become a retrieval system

It may declare what context it needs and consume returned packets.

It must not maintain an independent, competing context-retrieval truth layer or silently bypass Context Kernel with ad hoc retrieval for canonical work.

Temporary local fallback may exist only when explicitly declared, bounded, and unable to masquerade as a canonical Context Kernel packet.

### SWE_SEED must not become an authority system

It may request and enforce SEA-Forge decisions.

It must not infer permission from task intent, tool availability, prior success, or model confidence.

### SEA-Forge must not become a context-ranking system

It may constrain which sources or fields are accessible.

It must not decide which permitted information is most relevant to the task.

### SEA-Forge must not become a proof harness

It may require that proof evidence exists before allowing promotion or downstream action.

It must not independently run or reinterpret SWE_SEED’s software proof contract unless acting through an explicit proof adapter.

## Adapter Rule

Adapters translate contracts. They do not inherit the authority of the systems they connect.

An adapter may:

* validate schemas;
* translate identifiers;
* attach correlation metadata;
* enforce envelope versions;
* forward requests;
* verify signatures;
* check idempotency;
* return the canonical owner’s response.

An adapter must not independently:

* assemble canonical context;
* select the work route;
* authorize protected action;
* declare proof success;
* alter packet content without changing its hash;
* silently substitute one source of truth for another.

```text
Adapter translates.
Owner decides.
Evidence preserves the decision.
```

## Verification Invariants

The architecture is incorrect if any of these conditions can occur:

1. SWE_SEED performs canonical work without a required context packet.
2. A context packet lacks provenance for authority-sensitive material.
3. Context Kernel retrieves protected information without a SEA-Forge decision.
4. SWE_SEED treats task intent as authorization.
5. SEA-Forge authorizes an action without knowing which context packet informed it.
6. An authority decision remains valid after a material context change without reevaluation.
7. Context Kernel selects the engineering route.
8. SEA-Forge declares software proof successful.
9. SWE_SEED silently invents missing context.
10. An adapter performs context, route, policy, or proof decisions itself.
11. A stale context packet is presented as current.
12. An agent claims completion when required proof could not run.

## Boundary Test

For every component, module, function, event, or ledger, ask:

```text
Does it determine what information belongs in the current working world?
→ Context Kernel

Does it determine how the software job should run and how completion is proven?
→ SWE_SEED

Does it determine whether a consequential operation is permitted?
→ SEA-Forge
```

If one component answers more than one question, inspect it for responsibility leakage.

## Canonical Relationship

```text
Context Kernel:
Construct a bounded, cited, task-specific representation of the local world.

SWE_SEED:
Use that representation to route, execute, and prove the software-work contract.

SEA-Forge:
Evaluate and enforce authority whenever that work crosses a consequential boundary.
```

Final formulation:

```text
Context Kernel prevents the agent from inventing its working world.

SEA-Forge prevents the agent from inventing its permissions.

SWE_SEED prevents the agent from inventing its completion.
```
