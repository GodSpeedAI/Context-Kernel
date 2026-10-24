# Context Kernel product marketing context

**Status:** Working canonical product-marketing context
**Product:** Context Kernel
**Repository:** `Context_Kernel_Service_MCP`
**Company:** GodSpeed AI
**Primary use:** README copy, website copy, launch materials, developer documentation, product strategy, enterprise explanations, proof demonstrations, and AI-generated marketing content
**Last updated:** July 29, 2026

---

## 1. Source-of-truth hierarchy

When sources disagree, use this order:

1. The current Context Kernel repository, tests, commands, schemas, MCP tools, and license.
2. The current Context Kernel README and architecture decisions.
3. The current GodSpeed system-integration contracts.
4. This product-marketing context.
5. The CognitiveOS Manifesto and representational architecture.
6. The GodSpeed AI Brand Voice Style Guide.
7. Strategic architecture and syntelligent-infrastructure documents.
8. Older language describing Context Kernel primarily as memory, RAG, document search, or a general-purpose knowledge base.

Never let positioning override:

* current implementation;
* retrieval behavior;
* source provenance;
* disclosure boundaries;
* integration maturity;
* security limitations;
* product ownership boundaries;
* licensing.

Separate claims into four levels:

* **Evidence-backed:** supported by current code, tests, commands, MCP behavior, schemas, or repository documentation.
* **Partially proven:** meaningful implementation exists, but full real-time loop integration, production evidence, or trust-boundary proof remains incomplete.
* **Strategic interpretation:** follows from the architecture but should be presented as positioning rather than current implementation.
* **Roadmap:** planned, specified, or intended but not yet proven.

---

## 2. The framing hierarchy

Use these descriptions at different levels.

### Implementation description

Context Kernel is a local-first, single-binary MCP service for durable document and context retrieval.

The current implementation uses SQLite and FTS5 and exposes retrieval capabilities through MCP transports supported by the repository.

### Product description

Context Kernel is the real-time context authority for human-agent work.

It receives a context requirement and constructs a bounded, cited, provenance-bearing context packet for the current:

* task;
* actor;
* question;
* domain;
* route;
* policy environment;
* artifact;
* evidence obligation.

### Category description

Context Kernel is a local-world construction layer.

It reduces a wide field of organizational information into the smallest trustworthy world in which the current task can be understood and performed coherently.

### Strategic description

Context Kernel is the visibility and semantic-continuity layer of syntelligent infrastructure.

It determines what the working system can currently see, why those materials belong in view, which distinctions must survive compression, and what remains outside the boundary.

### Canonical formulation

> Context Kernel constructs the smallest trustworthy world required for the next action.

---

## 3. What “context authority” means

The phrase **context authority** must not be confused with runtime action authority.

Context Kernel is authoritative for:

* constructing the requested context packet;
* resolving relevant source material;
* preserving citations and provenance;
* applying declared source-access and disclosure scope;
* recording why material was included;
* recording what was excluded;
* representing freshness and uncertainty;
* preserving semantic continuity across sessions and agents;
* identifying conflicts and missing context.

Context Kernel is not authoritative for:

* allowing a file write;
* approving a shell command;
* permitting an API call;
* granting a deployment;
* accepting a settlement;
* promoting a capability;
* changing organizational policy.

SEA Forge owns consequential runtime authority.

### Canonical distinction

> Context Kernel decides what belongs in view. SEA Forge decides what may become action.

### Disclosure boundary

Context Kernel may enforce or consume rules about which sources the requesting actor is permitted to receive.

That is context-disclosure scoping.

It is not authority to perform the downstream operation.

---

## 4. Product summary

### Plain-language definition

Context Kernel gives an agent the specific information required for the work in front of it, with citations showing where the information came from and boundaries showing what was not included.

It does not dump an entire repository, document collection, or chat history into the prompt.

It constructs a task-specific context packet containing the relevant:

* problem;
* actor;
* organizational meaning;
* prior decisions;
* current files;
* constraints;
* tools;
* policies;
* evidence requirements;
* unresolved conflicts.

### Product definition

Context Kernel is a real-time context authority that assembles minimal sufficient semantic envelopes for human-agent work.

It retrieves from authorized sources, maps information into canonical organizational meaning where available, preserves citations and provenance, evaluates freshness and relevance, exposes omissions and conflicts, and produces a bounded context packet for downstream routing, governance, execution, proof, and settlement.

### Smallest useful explanation

> Context Kernel gives the agent the right world, not the whole warehouse.

### One-sentence position

> Context Kernel turns scattered organizational information into a bounded, cited, task-specific context packet.

### Technical one-sentence position

> A local-first MCP context service for retrieval, provenance, semantic scoping, continuity, and task-bound context-packet construction.

### Sharp public version

> The whole repository is not context.

### Controlling product truth

> More context is not more understanding.

---

## 5. The local-world thesis

An agent begins with more possible information than it can use.

A repository may contain:

* thousands of files;
* old implementations;
* current implementations;
* generated code;
* architecture decisions;
* contradictory documentation;
* hidden conventions;
* abandoned experiments;
* security boundaries;
* deployment rules;
* unresolved questions.

An organization contains even more:

* policies;
* roles;
* systems;
* relationships;
* histories;
* metrics;
* exceptions;
* agreements;
* observed behavior;
* tacit knowledge.

The task gives that field shape.

Context Kernel determines which part of the larger world belongs in the current working frame.

### Without Context Kernel

```text id="1vy4o0"
large information field
→ prompt stuffing or broad retrieval
→ mixed relevance and authority
→ agent reconstructs the task world
→ hidden assumptions
```

### With Context Kernel

```text id="f54nxz"
context requirement
→ source and scope resolution
→ semantic selection
→ citation and freshness checks
→ bounded context packet
→ downstream work
```

### Canonical analogy

> Without Context Kernel, the agent stands in front of the whole warehouse and calls that context.

The product does not give the agent the warehouse.

It prepares the workbench.

---

## 6. Minimal sufficient context

The product should be described through the phrase **minimal sufficient context**.

Each word matters.

### Minimal

The packet excludes material that does not justify its cost.

Context consumes:

* tokens;
* attention;
* retrieval time;
* model capacity;
* human review;
* privacy exposure;
* conflict resolution;
* inference burden.

Minimal does not mean arbitrarily short.

It means every included item earns its place.

### Sufficient

The packet preserves the distinctions required to:

* understand the task;
* choose a route;
* avoid known contradictions;
* comply with active constraints;
* locate relevant artifacts;
* apply the correct domain meaning;
* identify evidence obligations;
* recognize when more context is required.

A compact packet that omits a decisive constraint is not efficient.

It is incomplete.

### Semantic

The packet should preserve what the included information means inside the organization.

Where DomainForge models exist, context should connect sources to canonical:

* concepts;
* actors;
* resources;
* flows;
* policies;
* metrics;
* relationships;
* versions.

Semantic context is not only similar text.

It is information whose role in the current domain can be identified.

### Envelope

The packet includes more than content.

It carries:

* identity;
* boundary;
* provenance;
* citations;
* freshness;
* source authority;
* completeness;
* uncertainty;
* omissions;
* continuity;
* task relation.

### Canonical formulation

> Minimal sufficient context is the smallest bounded packet that preserves every distinction required for the current work to remain coherent.

---

## 7. The market wedge

Context Kernel’s first market should not be “all organizational knowledge.”

The immediate wedge is:

> Give coding agents bounded, current, cited repository context before they route or modify software work.

The smallest credible use case should include:

* one repository;
* one supported coding agent;
* one work request;
* one context requirement;
* one bounded source scope;
* one context packet;
* citations to the decisive files;
* one explicit exclusion or omission;
* one freshness or version check;
* one downstream route or work decision affected by the packet.

The buyer does not need to accept the entire organizational-context thesis before using Context Kernel.

They need to recognize this failure:

```text id="qz3dn5"
The agent can read the repository.
It still does not know which files matter,
which instructions are current,
or which interpretation is authoritative.
```

### Recommended wedge line

> Give coding agents the context the task requires, with sources they can show.

### Recommended product category

> Cited context service for coding agents.

### Broader product category

> Real-time context authority.

### Market category

> Local-world construction infrastructure.

### GodSpeed category

> The visibility and semantic-continuity layer of syntelligent infrastructure.

Use these categories in sequence.

Do not begin with “local-world construction infrastructure” before showing the concrete repository-context failure.

---

## 8. Why now

AI systems can process increasingly large context windows.

That does not mean organizations know what should enter them.

### Larger windows reduce one constraint

They make it possible to include more:

* source files;
* conversations;
* documentation;
* logs;
* policies;
* examples.

### Larger windows increase another problem

They make it easier to confuse:

* volume with relevance;
* access with understanding;
* retrieval with authority;
* recency with validity;
* similarity with meaning;
* prior conversation with current truth;
* documentation with implementation;
* observed behavior with accepted policy.

### The emerging bottleneck

The bottleneck is no longer only whether an agent can access information.

It is whether the system can construct a trustworthy task world from that information.

### Why Context Kernel matters now

As models become better at consuming context, the competitive advantage moves upstream into:

* context selection;
* source provenance;
* freshness;
* semantic identity;
* omission discipline;
* continuity;
* disclosure scope;
* conflict handling;
* task-specific compression.

> A larger context window makes context architecture more important, not less.

---

## 9. The primary customer problem: context is assembled by accident

Most agentic systems construct context through a mixture of:

* user prompts;
* recent conversation;
* broad file access;
* embedding similarity;
* hard-coded instructions;
* repository search;
* copied documentation;
* tool descriptions;
* agent memory;
* whatever the operator remembers to mention.

The resulting prompt may be large and still omit the decisive fact.

### Common current pattern

```text id="j69fhc"
task
+ recent chat
+ top search results
+ selected files
+ system prompt
= assumed context
```

This packet often cannot answer:

* Why was this source selected?
* What source was excluded?
* Is the document current?
* Does the actor have permission to receive it?
* Which organizational concept does it describe?
* Does it conflict with another source?
* Was the source implementation, policy, observation, or speculation?
* What changed since the last packet?
* Which item is decisive for this task?
* What remains unknown?

### Context Kernel pattern

```text id="16uuok"
context requirement
+ actor and disclosure scope
+ task and route needs
+ domain meaning
+ source provenance
+ freshness and conflict checks
+ token and attention budget
= bounded context packet
```

---

## 10. Primary audiences

### 10.1 Primary technical adopter

**Who they are**

* software engineers using coding agents;
* AI infrastructure engineers;
* platform engineers;
* staff and principal engineers;
* developer-experience teams;
* agent-tooling builders;
* technical founders.

**Current situation**

They already have:

* repositories;
* documentation;
* search;
* agent instruction files;
* vector stores;
* MCP servers;
* chat history;
* code intelligence tools.

They still watch agents:

* edit the wrong file;
* follow stale documentation;
* miss an ADR;
* ignore a repository convention;
* rediscover a solved problem;
* retrieve generic library information when repository-specific truth exists;
* treat generated files as source;
* miss the reason a prior decision was made.

**What they want**

* task-specific context;
* direct citations;
* repository-local evidence;
* current documentation;
* source and version identity;
* explicit omissions;
* context continuity across agents;
* lower context cost;
* less repeated explanation;
* integration with the coding agent they already use.

**What they distrust**

* another chat-memory product;
* a vector database marketed as understanding;
* dumping the whole codebase into a prompt;
* summaries without citations;
* context that cannot explain why it was selected;
* proprietary knowledge stores that become a second repository;
* autonomous memory mutation.

### 10.2 Primary platform buyer

**Who they are**

* Head of Engineering;
* VP of Engineering;
* developer-platform leader;
* AI platform leader;
* enterprise architecture leader;
* head of knowledge systems;
* AI governance leader.

**What they need**

* consistent context across teams and agents;
* lower repeated discovery cost;
* source-aware answers;
* visibility into context exposure;
* separation among policy, observation, and implementation;
* a way to connect organizational meaning to agent work;
* measurable reduction in missing-context failures;
* provider independence;
* local and controlled deployment options.

**Emotional state**

They know their agents are powerful enough to act on the wrong interpretation.

They do not want to solve that problem by copying the entire organization into every prompt.

### 10.3 Domain and knowledge owners

**Who they are**

* domain experts;
* architects;
* policy owners;
* process owners;
* quality professionals;
* technical writers;
* product managers;
* Domain Engineers.

**What they need**

* assurance that their material is represented accurately;
* visibility into where a source is used;
* explicit freshness and ownership;
* conflict detection;
* stable concept references;
* the ability to replace or retire stale sources;
* provenance when agents explain or act.

### 10.4 Security and governance audience

**Who they are**

* security engineers;
* privacy teams;
* compliance leaders;
* data-governance teams;
* access-control owners;
* auditors.

**What they need**

* least-disclosure context;
* source-access enforcement;
* tenant and project isolation;
* citation records;
* handling of sensitive material;
* protection against prompt injection in retrieved text;
* packet expiration;
* reviewable context provenance;
* evidence of what an agent was allowed to see.

---

## 11. Core jobs to be done

### Context-construction jobs

When an agent needs to perform work, help me:

1. turn the work request into a context requirement;
2. identify the actor and disclosure scope;
3. identify the relevant domain;
4. locate current authoritative sources;
5. retrieve repository-specific information before generic information;
6. resolve the current version of a source;
7. connect source material to canonical domain concepts;
8. rank information by task relevance rather than similarity alone;
9. detect contradictory sources;
10. distinguish policy, implementation, observation, history, and speculation;
11. preserve citations;
12. state why each item was included;
13. state what was omitted;
14. state what remains unknown;
15. fit the packet within a context budget without losing decisive distinctions;
16. produce a stable packet another component can consume.

### Continuity jobs

Across sessions, tools, and agents, help me:

1. preserve semantic continuity rather than transcript continuity;
2. identify what changed since the last packet;
3. carry forward unresolved questions;
4. avoid repeating settled discovery work;
5. replace stale material;
6. preserve stable concept and source identities;
7. connect current work to prior decisions and evidence;
8. avoid treating conversational recency as authority.

### Governance-support jobs

Help downstream systems know:

1. what the agent saw;
2. where it came from;
3. how fresh it was;
4. which disclosure scope applied;
5. what policy and evidence requirements were visible;
6. which context was missing;
7. which sources conflicted;
8. whether the packet was altered or superseded.

### Operational jobs

Help me:

1. retrieve context locally;
2. expose it through MCP;
3. use it from several agent hosts;
4. inspect the context packet;
5. reproduce the retrieval;
6. compare packets;
7. measure context quality;
8. fail visibly when required context is unavailable.

---

## 12. Emotional and social jobs

### Emotional jobs

Help me feel:

* less responsible for re-explaining the repository every session;
* confident that the agent saw the decisive constraints;
* able to inspect why an answer was produced;
* less exposed to confident work based on stale documentation;
* comfortable switching models or agent hosts;
* aware of what remains uncertain.

### Social jobs

Help me:

* show reviewers which context informed the work;
* give teams one context protocol rather than individual prompt habits;
* prove that sensitive information was scoped;
* let domain owners see how their material influences agents;
* support agent adoption without surrendering information boundaries;
* distinguish a source-backed answer from a plausible reconstruction.

---

## 13. Core customer problems

### 13.1 The whole repository is treated as context

An agent may have permission to read every file.

That does not mean every file deserves equal attention.

Underlying problem:

> Accessibility has been mistaken for relevance.

### 13.2 Prompt stuffing is treated as grounding

Operators copy:

* design documents;
* source files;
* policies;
* logs;
* issue histories;
* prior chat.

The model receives more text but no explicit account of:

* authority;
* freshness;
* source type;
* conflicts;
* omissions;
* semantic role.

Underlying problem:

> The system has more material but no stronger world model.

### 13.3 Similarity retrieval misses decisive differences

Embedding search may retrieve text that looks semantically similar.

The decisive source may differ through:

* version;
* environment;
* actor;
* resource;
* policy;
* architecture boundary;
* current implementation;
* exception status.

Underlying problem:

> Similar language does not guarantee applicable meaning.

### 13.4 Agents start every session from zero

Relevant history exists in:

* chat logs;
* commits;
* ADRs;
* proof reports;
* incidents;
* prior tasks;
* human memory.

The next agent does not receive the relevant residue.

Underlying problem:

> Work leaves history, but the next task cannot spend it.

### 13.5 Conversation continuity is mistaken for semantic continuity

A long conversation preserves what was said recently.

It may not preserve:

* current repository state;
* valid policy;
* domain identity;
* source freshness;
* accepted decisions;
* superseded assumptions.

Underlying problem:

> Remembering the conversation is not the same as maintaining a current world.

### 13.6 Stale documentation outranks current implementation

A well-written old document may rank above:

* current code;
* current schema;
* current tests;
* newer architecture decisions.

Underlying problem:

> Retrieval confidence is disconnected from source freshness and authority.

### 13.7 Context has no provenance

An agent cites a claim without identifying:

* source;
* section;
* version;
* retrieval time;
* content hash;
* applicable scope.

Underlying problem:

> The claim cannot be independently inspected.

### 13.8 Context hides its omissions

Compression and retrieval necessarily exclude material.

Most systems present the packet as though it were the relevant world.

Underlying problem:

> Invisible omission creates false completeness.

### 13.9 Contradictions are flattened

Two sources disagree.

The retrieval system summarizes them into one smooth answer.

Underlying problem:

> The system removes the difference that should have governed the next move.

### 13.10 Retrieved content can attack the agent

Documents, issues, code comments, websites, and logs may contain instructions aimed at the model.

Underlying problem:

> Retrieved evidence is being treated as executable instruction.

### 13.11 Sensitive context leaks across boundaries

A useful document may be:

* project restricted;
* role restricted;
* customer restricted;
* legally sensitive;
* secret bearing;
* no longer authorized.

Underlying problem:

> Relevance is being allowed to override disclosure authority.

### 13.12 Context systems become competing sources of truth

A retrieval system stores summaries, rewritten facts, and inferred relationships.

Those outputs gradually outrank the repository, policy store, or domain source.

Underlying problem:

> The context layer has silently become an ungoverned knowledge authority.

---

## 14. Status quo and alternatives

Context Kernel competes first with accidental context assembly.

It also complements several established categories.

### 14.1 Large context windows

**Why teams use them**

* simple;
* fewer retrieval decisions;
* broad coverage;
* supported directly by models.

**Where they fail**

* high token and attention cost;
* stale and conflicting sources coexist;
* sensitive data exposure expands;
* important distinctions can be diluted;
* source provenance is weak;
* inclusion remains arbitrary.

**Context Kernel position**

> Context windows hold the packet. Context Kernel determines what the packet should contain.

### 14.2 Prompt stuffing

**Why teams use it**

* immediate;
* operator controlled;
* no infrastructure;
* easy to debug manually.

**Where it fails**

* does not scale;
* depends on operator memory;
* no repeatable source selection;
* weak freshness;
* omissions remain invisible;
* context differs across users.

**Context Kernel position**

> A good prompt cannot repair a missing context protocol.

### 14.3 Vector databases and semantic search

**Why teams use them**

* scalable retrieval;
* similarity search;
* mature tools;
* broad content support.

**Where they fall short**

* similarity is only one relevance signal;
* weak source authority and freshness models;
* limited task and actor scoping;
* no guarantee of semantic continuity;
* retrieval results are not yet a context packet.

**Context Kernel position**

> Vector retrieval may be one mechanism. Context authority is the product.

### 14.4 Keyword and full-text search

**Why teams use them**

* deterministic;
* fast;
* inspectable;
* strong for names, symbols, and exact phrases.

**Where they fall short**

* query wording matters;
* synonyms and conceptual relationships may be missed;
* search results still require selection, conflict handling, and packaging.

**Context Kernel position**

> Full-text search finds candidate sources. Context Kernel constructs the working world from them.

The current SQLite and FTS5 implementation is a valid foundation, not a limitation to hide.

### 14.5 Generic RAG frameworks

**Why teams use them**

* available components;
* ingestion pipelines;
* model integrations;
* configurable retrieval.

**Where they fall short**

* optimize answer generation;
* often return top chunks without explaining the context boundary;
* may not represent actor, route, policies, omissions, and evidence requirements;
* can become tied to one agent application.

**Context Kernel position**

> RAG answers with retrieved text. Context Kernel produces a governed context packet another system can act from.

### 14.6 Agent memory

**Why teams use it**

* continuity;
* personalization;
* retrieval of prior interactions;
* reduced repetition.

**Where it falls short**

* remembered content may be stale;
* memory may lack citations;
* conversation importance may outrank task relevance;
* memory can reinforce prior errors;
* capability and settlement state belong elsewhere.

**Context Kernel position**

> Memory is one possible source. Context Kernel decides whether it belongs in the current packet.

### 14.7 Knowledge graphs

**Why teams use them**

* stable relationships;
* domain queries;
* semantic identity;
* inference.

**Where they fall short**

* may not contain current implementation detail;
* do not automatically produce bounded task packets;
* still require actor, time, route, source, and disclosure scoping.

**Context Kernel position**

> A graph provides semantic structure. Context Kernel selects the relevant subworld for the present task.

### 14.8 Code intelligence and repository maps

**Why teams use them**

* symbol resolution;
* dependency graphs;
* call graphs;
* architecture discovery.

**Where they fall short**

* code is only one context source;
* business policy, history, proof, incidents, and domain meaning may live elsewhere;
* structural relevance is not always task relevance.

**Context Kernel position**

> Code intelligence helps explain the implementation. Context Kernel joins it with the rest of the task world.

### 14.9 Documentation platforms and enterprise search

**Why teams use them**

* broad organizational coverage;
* familiar interfaces;
* permissions;
* mature indexing.

**Where they fall short**

* result lists still require synthesis;
* authority differs by document;
* contextual omissions remain hidden;
* integration with agent work and proof is weak.

**Context Kernel position**

> Enterprise search finds information. Context Kernel turns selected information into an inspectable work contract input.

### 14.10 Model-managed context and summarization

**Why teams use it**

* simple;
* adaptive;
* inexpensive to integrate;
* useful for compression.

**Where it fails**

* the same model may select, compress, and act;
* source distinctions can be merged;
* summaries can invent connective meaning;
* omission is difficult to inspect;
* compression may become irreversible.

**Context Kernel position**

> Models may help rank and compress. The packet contract, provenance, and boundaries must remain outside the model’s unsupported discretion.

---

## 15. Positioning

### Positioning statement

For engineering and organizational teams using AI agents across repositories, documents, policies, and operational history, Context Kernel is the real-time context authority that constructs the smallest cited, current, and semantically coherent world required for each task.

Unlike prompt stuffing, large context windows, generic RAG, vector search, or agent memory used alone, Context Kernel combines task requirements, actor scope, source provenance, freshness, organizational meaning, omissions, conflicts, and continuity into a bounded context packet that downstream systems can inspect and govern.

### Category ladder

Use categories in this order:

1. **Immediate problem:** agents receive too much information and still miss what matters;
2. **Technical category:** local MCP document and context retrieval service;
3. **Product category:** cited task-context service;
4. **Market category:** real-time context authority;
5. **Strategic category:** local-world construction infrastructure;
6. **GodSpeed category:** visibility and continuity layer of syntelligent infrastructure.

### Core contrast

```text id="ndnvit"
Typical context assembly

task
+ recent chat
+ broad search
+ large file dump
→ agent reconstructs the world
```

```text id="f5tc07"
With Context Kernel

context requirement
→ actor and source scope
→ semantic retrieval
→ provenance and freshness
→ conflict and omission record
→ bounded context packet
```

### Product promise

> Give every agent the smallest world in which the task makes sense.

### Strategic value

Context Kernel helps systems move from:

* context access to context construction;
* prompt stuffing to bounded packets;
* similarity results to task relevance;
* untraceable summaries to cited sources;
* chat continuity to semantic continuity;
* hidden omission to explicit boundaries;
* generic retrieval to organizational meaning;
* stale information to freshness-aware context;
* source accumulation to spendable context;
* one-agent memory to portable context authority.

---

## 16. The context requirement

Context construction begins with a context requirement, not a search query.

A context requirement should identify:

* work request;
* actor;
* task or question;
* domain;
* route or expected work type;
* current artifact;
* constraints;
* allowed source scopes;
* prohibited source scopes;
* required policies;
* required evidence;
* freshness expectations;
* context budget;
* continuity reference;
* known uncertainties.

### Search query

A search query asks:

> Which sources match these words or concepts?

### Context requirement

A context requirement asks:

> What must this actor know now for this work to remain coherent, authorized, and provable?

### Canonical distinction

> Search finds candidates. A context requirement defines the world that must be constructed.

---

## 17. The context packet

The context packet is Context Kernel’s primary product output.

It should be versioned, inspectable, reproducible, and suitable for downstream automation.

### Packet identity

A context packet should include, where supported:

* packet ID;
* schema version;
* work-request ID;
* task ID;
* actor ID;
* domain or namespace;
* created time;
* expiration or freshness horizon;
* correlation and causation IDs;
* preceding packet or continuity reference;
* content hash.

### Context boundary

The packet should state:

* included source scopes;
* excluded source scopes;
* time boundary;
* project or tenant boundary;
* domain boundary;
* token or size budget;
* disclosure class;
* packet completeness profile.

### Selected material

Each included item should carry:

* citation;
* source identity;
* source type;
* source owner where known;
* content location;
* version or revision;
* retrieval time;
* content hash where available;
* relevance explanation;
* semantic concepts;
* authority or standing;
* freshness state;
* confidence or uncertainty.

### Working structure

The packet may organize context into:

* task;
* actor;
* domain;
* history;
* current state;
* relevant artifacts;
* constraints;
* tools;
* active policy references;
* evidence requirements;
* unresolved conflicts;
* open questions;
* omitted material.

### Omission record

The packet should explain significant exclusions, such as:

* outside actor scope;
* stale source;
* superseded source;
* low relevance;
* duplicate material;
* budget limit;
* conflicting source requiring review;
* unavailable source;
* disclosure restriction.

### Conflict record

When sources disagree, the packet should preserve:

* both claims;
* source identity;
* date and version;
* type of disagreement;
* current canonical preference if one is established;
* unresolved status if no authority exists.

Do not summarize a consequential disagreement into false consensus.

---

## 18. Context packets and the Semantic Envelope

The context packet should align with the broader Semantic Envelope rather than creating an unrelated transport family.

The packet is best understood as a task-context profile of the Semantic Envelope.

It should preserve:

* identity;
* boundary;
* profile;
* completeness;
* provenance;
* semantic references;
* traceability.

### Canonical relationship

```text id="f43qi5"
Semantic Envelope
  └─ Context Packet profile
       ├─ task boundary
       ├─ actor scope
       ├─ selected context
       ├─ citations
       ├─ omissions
       ├─ conflicts
       └─ continuity
```

Do not claim conformance until the implemented packet and schema have been tested against the current canonical envelope contract.

---

## 19. The core mechanism

Context Kernel’s differentiation is not retrieval alone.

It is the context-construction loop.

### 19.1 Receive the context requirement

A caller such as SWE_SEED requests context for a specific unit of work.

The requirement identifies the actor, task, route, domain, constraints, and evidence needs.

### 19.2 Resolve disclosure scope

Determine which source collections and records the actor may receive.

Context relevance cannot override source authorization.

Where policy is external, Context Kernel consumes the applicable disclosure decision or policy constraints.

It does not invent downstream action authority.

### 19.3 Resolve semantic scope

Use available DomainForge concepts and semantic packs to identify:

* what the task is about;
* which entities and resources matter;
* which policies and flows apply;
* which terms are equivalent;
* which distinctions must remain separate.

### 19.4 Retrieve candidate sources

Use the mechanisms supported by the current deployment, which may include:

* exact search;
* SQLite FTS5;
* source references;
* repository indexes;
* graph queries;
* structured metadata;
* vector retrieval;
* external documentation tools.

Retrieval mechanism is replaceable.

Packet integrity is not.

### 19.5 Rank by applicability

Evaluate more than similarity.

Relevant factors may include:

* task relation;
* domain identity;
* source authority;
* current version;
* actor;
* environment;
* route;
* policy;
* evidence obligation;
* prior decision;
* implementation proximity;
* freshness;
* contradiction.

### 19.6 Detect conflicts and gaps

Identify:

* stale documentation;
* competing definitions;
* missing source;
* incomplete history;
* unresolved authority;
* implementation-policy disagreement;
* unverified claims;
* insufficient context.

A missing decisive source should not be concealed by fluent synthesis.

### 19.7 Compress without erasing distinctions

Reduce duplication and irrelevant detail while preserving:

* source identity;
* causal constraints;
* exceptions;
* policy conditions;
* version boundaries;
* unresolved disagreement;
* evidence requirements.

Model-assisted compression may be useful.

The original citation must remain available.

### 19.8 Construct the packet

Assemble the bounded context packet with:

* selected sources;
* citations;
* semantic references;
* freshness;
* omissions;
* conflicts;
* uncertainty;
* continuity metadata.

### 19.9 Emit `ContextPacketCreated`

The packet enters the agentic capability loop as the response to `ContextRequired`.

Downstream components may now:

* select a route;
* propose an action;
* request authority;
* perform work;
* run proof;
* settle the outcome.

### 19.10 Preserve continuity

Record enough packet identity and lineage to determine:

* what changed;
* what was superseded;
* what remains unresolved;
* which prior packet informed the current one.

### Canonical loop

```text id="41i47h"
Define the need.
Bound the sources.
Preserve the meaning.
Cite the world.
Expose what is missing.
```

---

## 20. Semantic continuity, not prompt continuity

Context Kernel should preserve semantic continuity.

### Prompt continuity

Prompt continuity carries forward:

* recent messages;
* summaries;
* prior responses;
* conversational state.

This is useful but insufficient.

### Semantic continuity

Semantic continuity preserves:

* stable concept identities;
* source revisions;
* accepted decisions;
* current policy references;
* superseded assumptions;
* unresolved conflicts;
* active constraints;
* evidence links;
* packet lineage.

### Example

A prior conversation may say:

> DomainForge generates the domain.

A later conversation may contain an older statement implying SEA Forge generates the domain.

Prompt continuity may retain both.

Semantic continuity should preserve the current canonical distinction and cite the source supporting it.

### Canonical line

> Context Kernel should remember what the organization means, not merely what the conversation last said.

---

## 21. DomainForge relationship

DomainForge and Context Kernel solve adjacent but different problems.

### DomainForge

DomainForge defines organizational source code:

* what exists;
* which distinctions matter;
* how concepts relate;
* which resources flow;
* which roles participate;
* what policies mean;
* what metrics mean;
* which changes preserve identity.

### Context Kernel

Context Kernel decides which part of that wider semantic world belongs in the current task.

### Canonical distinction

> DomainForge defines the world. Context Kernel constructs the local world.

### Semantic packs

DomainForge should own:

* semantic-pack generation;
* semantic validation;
* concept identities;
* adapter merge rules;
* canonical model hashes.

Context Kernel should:

* load relevant semantic packs;
* use them to interpret external sources;
* map retrieved material into declared organizational meaning;
* record unmapped or conflicting information;
* include the relevant semantic subset in the packet.

### External information

External information may enter Context Kernel.

Before it affects:

* policy;
* authority;
* settlement;
* capability;
* organizational source,

it should be mapped to declared semantics or clearly marked as external, unresolved evidence.

Context Kernel must not silently convert retrieved text into organizational truth.

---

## 22. SWE_SEED relationship

SWE_SEED owns the software-work contract.

It requests the context required by the route.

### SWE_SEED supplies

* work request;
* task type;
* route needs;
* expected artifact;
* proof obligations;
* actor;
* repository scope;
* relevant constraints.

### Context Kernel returns

* bounded context packet;
* citations;
* source identity;
* current repository and domain context;
* conflict and omission record;
* evidence-requirement references;
* packet provenance.

### SWE_SEED then owns

* route execution;
* artifact production;
* proof commands;
* trace.

### Canonical distinction

> Context Kernel constructs the work world. SWE_SEED performs the work contract inside it.

### Failure behavior

SWE_SEED should not invent missing context.

If Context Kernel cannot produce required context, the work should:

* stop;
* request clarification;
* select a lower-risk route;
* or escalate.

The exact policy belongs to the work and authority contracts.

---

## 23. SEA Forge relationship

SEA Forge owns runtime action authority.

### Context Kernel supplies

* actor context;
* task context;
* resource context;
* relevant policy references;
* source citations;
* context-packet identity;
* missing or uncertain information.

### SEA Forge decides

* allow;
* deny;
* escalate;
* constrain;
* require approval.

### Canonical distinction

> Context informs authority. It does not replace authority.

### Context access

SEA Forge or another governance system may define which context sources an actor may receive.

Context Kernel applies that disclosure boundary during packet construction.

### Runtime action

The presence of a context packet does not authorize the proposed action.

A well-informed action can still be forbidden.

---

## 24. GodSpeed-Agent relationship

GodSpeed-Agent owns developmental navigation, settlement classification, repetition, and capability change.

### GodSpeed-Agent may supply

* prior settlements;
* capability state;
* repeated failures;
* payment history;
* unresolved developmental questions;
* relevant learning proposals.

### Context Kernel decides

Which of those records belongs in the current packet.

### GodSpeed-Agent later consumes

* settlement evidence;
* context-packet references;
* observed missing-context failures;
* packet quality signals;
* learning evidence.

### Canonical distinction

> GodSpeed-Agent records what prior work earned. Context Kernel decides which earned residue matters now.

Context Kernel must not become the canonical developmental-memory store.

---

## 25. Agent Memory Ledger relationship

Agent Memory Ledger or another approved persistence layer may store:

* prior events;
* governance decisions;
* settlement records;
* memory items;
* twin updates;
* evidence references.

Context Kernel may retrieve from that durable store.

It does not own the store’s canonical memory semantics or lifecycle.

### Canonical distinction

> The ledger remembers. Context Kernel constructs the packet.

### No duplicate truth

Context Kernel should not create an independent durable fact merely because it summarized a ledger record.

The summary remains a packet projection linked to the original source.

---

## 26. Context Kernel is not a second authority plane

Context Kernel should never absorb the responsibilities of the other GodSpeed components.

It should not:

* approve actions;
* settle outcomes;
* promote capability;
* rewrite organizational source automatically;
* classify proof as sufficient;
* become the canonical event ledger;
* choose development strategy;
* run the coding task;
* own the model provider;
* become a universal application runtime.

### Canonical stack formula

> DomainForge defines the domain.
> Context Kernel constructs the local world.
> SEA Forge governs the work.
> SWE_SEED proves the change.
> GodSpeed-Agent compounds the capability.

### Compact public formula

> Know through Context Kernel. Work through SWE_SEED. Act through SEA Forge.

---

## 27. Context quality

Context quality is not measured by packet size.

A useful context packet should perform across several dimensions.

### Relevance

Does the item affect the current task, route, constraint, artifact, or evidence obligation?

### Sufficiency

Does the packet contain the distinctions required to proceed coherently?

### Precision

Does it avoid irrelevant or distracting material?

### Provenance

Can every consequential claim be traced to a source?

### Freshness

Is the source current enough for the task?

### Authority

Does the source have appropriate standing for the claim it supports?

### Semantic coherence

Do the selected sources use compatible concepts and identities?

### Conflict visibility

Are disagreements preserved rather than smoothed over?

### Disclosure integrity

Was the packet restricted to authorized sources?

### Continuity

Does it connect correctly to prior packets, decisions, and unresolved questions?

### Reproducibility

Can another system reconstruct why the packet was produced?

### Cost

Did the packet preserve the needed distinctions at an affordable token, time, and attention burden?

---

## 28. Context sufficiency is task relative

No context packet is universally complete.

A packet can be sufficient for:

* locating a file;
* explaining an architecture;
* writing a test;
* proposing a change;

and insufficient for:

* modifying production policy;
* accessing secrets;
* approving a deployment;
* settling a regulated outcome.

### Canonical rule

> Context completeness must be claimed relative to a task and boundary, never as a universal property.

### Completeness profiles

Where supported, packets may declare profiles such as:

* exploratory;
* explanatory;
* planning;
* implementation;
* security-sensitive;
* governance-sensitive;
* proof-sensitive;
* settlement-sensitive.

The profile changes what counts as sufficient.

Do not market profile names as implemented unless current schemas and behavior support them.

---

## 29. Omission is part of the product

Every bounded context packet excludes information.

The product should make consequential omission visible.

### Good omission

Material is excluded because it is:

* irrelevant;
* duplicated;
* superseded;
* outside scope;
* too stale;
* unauthorized;
* incompatible with the task profile;
* replaceable by a more authoritative source.

### Dangerous omission

Material is excluded because:

* retrieval missed it;
* the query used the wrong term;
* the source was not indexed;
* the token budget truncated it;
* a conflict was inconvenient;
* the agent preferred a simpler story.

### Omission record

The packet should distinguish:

* intentional exclusion;
* unavailable source;
* unresolved missing context;
* budget-driven truncation;
* low-confidence retrieval gap.

### Canonical line

> A trustworthy context packet explains both what it contains and what it could not safely claim to contain.

---

## 30. Conflict is not noise

Context systems often optimize for a coherent answer.

Organizations often contain legitimate disagreement.

Examples:

* code and documentation disagree;
* policy and observed practice disagree;
* two versions of a schema remain active;
* architecture intent and current implementation diverge;
* two domain owners use the same term differently.

Context Kernel should preserve the conflict when the difference affects the work.

### Conflict response

The packet may:

* identify the current canonical source;
* include both sources;
* mark one as superseded;
* request clarification;
* escalate;
* limit the permitted route;
* lower the completeness status.

### Canonical line

> Context that removes a consequential disagreement has made the world easier to read by making it less true.

---

## 31. Prompt injection and hostile context

Retrieved content must be treated as untrusted data unless explicitly authorized as instruction.

Potential sources of hostile instructions include:

* webpages;
* issue comments;
* README files;
* code comments;
* logs;
* emails;
* generated documents;
* tool output;
* prior agent transcripts.

### Required separation

The context packet should distinguish:

* evidence;
* source content;
* policy;
* system instruction;
* tool instruction;
* user request;
* prior agent output.

### Canonical rule

> A document may describe an instruction without receiving authority to issue one.

### Security posture

Context Kernel should support, where implemented:

* source trust classes;
* content sanitization;
* instruction-data separation;
* tenant boundaries;
* secret detection;
* least disclosure;
* packet expiration;
* provenance preservation;
* audit records.

Do not claim universal prompt-injection prevention.

---

## 32. Feature-to-outcome translation

Do not market features without their consequence.

| Capability                | Reader consequence                                                                             |
| ------------------------- | ---------------------------------------------------------------------------------------------- |
| SQLite and FTS5 retrieval | Repository and document context can be searched locally with deterministic full-text behavior. |
| MCP interface             | Existing agent hosts can request context without adopting a new proprietary agent.             |
| Context requirement       | Retrieval begins from the work need rather than an isolated keyword query.                     |
| Bounded packet            | The agent receives a working set instead of the entire information field.                      |
| Citations                 | Reviewers can inspect where consequential claims came from.                                    |
| Provenance                | A source-backed answer can be distinguished from model inference.                              |
| Freshness metadata        | Stale documentation is less likely to outrank current implementation silently.                 |
| Actor and source scope    | Relevant information does not automatically become disclosable information.                    |
| Semantic mapping          | Retrieved material connects to the organization’s canonical concepts.                          |
| Conflict record           | Contradictory sources remain visible.                                                          |
| Omission record           | Compression and retrieval gaps do not masquerade as completeness.                              |
| Packet identity and hash  | Downstream work can refer to the exact context used.                                           |
| Continuity links          | Agents can continue the domain state without relying only on chat history.                     |
| Context diff              | Teams can inspect what changed between runs.                                                   |
| Evidence requirements     | The packet can tell the work system what proof-relevant material applies.                      |
| Local-first operation     | Sensitive repository context can remain close to the work.                                     |
| Context budget            | Token and attention costs can be controlled explicitly.                                        |

Only describe capabilities that exist in the current repository as implemented.

---

## 33. Differentiators

### 33.1 It constructs context rather than returning search results

Search returns candidates.

Context Kernel produces a task-bound packet with provenance, boundaries, and omissions.

### 33.2 It optimizes for sufficiency, not maximum recall

The goal is not to retrieve everything that could be relevant.

The goal is to preserve everything that must remain distinct for the current work.

### 33.3 It makes context inspectable

The packet can show:

* what was included;
* why it was included;
* where it came from;
* how current it is;
* what was excluded;
* what remains uncertain.

### 33.4 It separates semantic continuity from conversation continuity

The system can preserve domain and source identity across agents and sessions without treating the latest transcript as canonical.

### 33.5 It keeps organizational meaning upstream

DomainForge remains the source of declared organizational meaning.

Context Kernel selects and applies that meaning without quietly rewriting it.

### 33.6 It keeps action authority downstream

The packet informs SEA Forge.

It does not permit the action.

### 33.7 It treats omissions and conflicts as first-class

The product does not need to pretend every packet is complete or every source agrees.

### 33.8 It is agent independent

The packet can be consumed by supported agents, tools, and workflows.

Do not claim universal host support without current adapters and tests.

### 33.9 It is retrieval-mechanism independent

SQLite FTS5 is the current foundation.

Additional graph, vector, code-intelligence, or external-document sources may participate without changing the packet contract.

### 33.10 It turns context into a governed artifact

The context used for a consequential action can become a referenced object rather than an invisible prompt state.

---

## 34. Messaging hierarchy

Explain Context Kernel in this order.

### Message 1: The familiar failure

> Agents can read more information than ever and still miss the one distinction that controls the task.

### Message 2: The correction

> The whole repository is not context, and the latest conversation is not organizational memory.

### Message 3: The mechanism

> Context Kernel receives a context requirement and assembles a bounded packet with relevant sources, citations, freshness, semantic meaning, conflicts, and omissions.

### Message 4: The practical result

> The agent begins work inside a task-specific world that another person or system can inspect.

### Message 5: The continuity value

> The next agent can inherit stable meaning and source lineage rather than starting from a prompt summary.

### Message 6: The category

> Context Kernel is the real-time context authority and local-world construction layer of the GodSpeed stack.

Do not begin with Message 6.

---

## 35. Approved message formulations

### Primary headline

> Give the agent the world it needs. Not the whole world.

### Problem-led headline

> The whole repository is not context.

### Sharp headline

> Stop calling a context dump grounding.

### Product-led headline

> Cited, task-specific context for every agent run.

### Primary subhead

> Context Kernel turns repositories, documents, policies, history, and domain models into the smallest cited context packet that preserves what the current task needs.

### Technical subhead

> A local-first MCP service for bounded retrieval, provenance, semantic continuity, freshness, and context-packet construction.

### Plain one-liner

> Context Kernel gives agents the right information for the current task and shows where it came from.

### Developer one-liner

> Ask for the task context. Get the decisive files, decisions, constraints, and citations instead of another repository dump.

### Enterprise one-liner

> Context Kernel gives AI systems a controlled, source-backed view of organizational knowledge for each unit of work.

### Semantic one-liner

> DomainForge defines the wider world. Context Kernel selects the part that matters now.

### Stack one-liner

> Context Kernel constructs the world. SWE_SEED works inside it. SEA Forge governs what leaves it.

### Sharp lines

* More context is not more understanding.
* The whole repository is not context.
* A context window is capacity. It is not judgment.
* Search finds information. Context Kernel constructs a working world.
* If the agent cannot cite it, the organization should not have to trust it.
* The latest document is not always the current truth.
* Conversation continuity is not semantic continuity.
* Relevance does not grant disclosure.
* Similar text can describe a different system.
* A summary without provenance is another model claim.
* The context packet should show its seams.
* What the packet omits can matter more than what it summarizes.
* Context controls visibility. Authority controls action.
* Give the agent a workbench, not the warehouse.

Use sharp lines sparingly.

Do not make every paragraph an aphorism.

---

## 36. Voice and tone

Context Kernel copy should sound:

* precise;
* calm;
* source conscious;
* technically credible;
* anti-hype;
* skeptical of context-window maximalism;
* respectful of specialist retrieval tools;
* honest about missing information;
* serious about privacy and provenance.

The voice should sound like:

> A systems architect and research librarian designing the briefing an expert would demand before allowing an agent to touch consequential work.

### Use Speed Mode for

* context-dump criticism;
* large-window contrasts;
* README openers;
* developer launch content;
* category hooks.

### Use Journey Mode for

* packet schemas;
* provenance;
* semantic continuity;
* disclosure;
* conflict handling;
* freshness;
* implementation maturity;
* integration boundaries.

### Humor boundary

Mock context absurdity, not teams struggling with fragmented information.

Good:

> Giving the agent every file is one way to avoid selecting context. It is also how the selection problem gets assigned to the model.

Bad:

> Teams dumping repositories into prompts do not understand context engineering.

---

## 37. Language rules

### Prefer

* context requirement;
* context packet;
* bounded context;
* minimal sufficient context;
* local world;
* citation;
* provenance;
* source identity;
* freshness;
* disclosure scope;
* omission;
* conflict;
* semantic continuity;
* source authority;
* completeness profile;
* packet lineage;
* context budget;
* selected material;
* relevant distinction;
* current task.

### Explain before relying on

* context authority;
* local-world construction;
* Semantic Envelope;
* semantic pack;
* completeness profile;
* disclosure scope;
* context lineage;
* semantic continuity;
* representational boundary;
* S0A;
* S0B;
* S0C.

### Avoid

* infinite memory;
* knows everything;
* perfect recall;
* complete organizational knowledge;
* universal context;
* second brain;
* context operating system without explanation;
* eliminates hallucinations;
* solves prompt injection;
* always current;
* fully grounded;
* autonomous knowledge management;
* self-updating truth;
* seamless;
* revolutionary;
* unlock;
* supercharge;
* AI-powered search;
* one source of truth when referring to the context store;
* authority without distinguishing disclosure from action.

### Canonical distinctions

Do not cycle among these terms as if they are identical:

* **Source:** original repository, document, record, graph, event, or external resource.
* **Retrieval result:** a candidate item returned by a search mechanism.
* **Context requirement:** the specification of what the work needs.
* **Context item:** a selected and cited source fragment or structured object.
* **Context packet:** the bounded artifact delivered to downstream work.
* **Memory:** durable retained state that may be considered as a source.
* **Continuity:** preservation of relevant meaning and lineage across work.
* **Domain meaning:** canonical concepts and relations supplied by DomainForge.
* **Disclosure scope:** what the actor may receive.
* **Action authority:** what the actor may do, owned by SEA Forge.

---

## 38. Common objections

### “Isn’t this just RAG?”

Response:

RAG may be one implementation mechanism.

Context Kernel’s product object is a bounded context packet containing:

* task scope;
* actor scope;
* citations;
* provenance;
* freshness;
* semantic references;
* conflicts;
* omissions;
* continuity.

Top retrieved chunks are not yet a trustworthy working world.

### “Why not use a larger context window?”

Response:

A larger window increases capacity.

It does not decide:

* which sources are current;
* which source has standing;
* which information is disclosable;
* which disagreement matters;
* what was omitted;
* how the material maps to organizational meaning.

Context Kernel decides what should occupy the window.

### “Why not give the agent the whole repository?”

Response:

The whole repository may contain relevant evidence.

It also contains:

* stale code;
* generated artifacts;
* irrelevant modules;
* conflicting documentation;
* archived experiments;
* sensitive files.

Repository access is useful.

Task-specific context remains a separate problem.

### “Why not let the agent search for itself?”

Response:

The agent should be able to request more context.

It should not be the sole authority deciding:

* what sources are available;
* what is current;
* what is disclosable;
* what was omitted;
* what organizational concept applies.

The actor should not have to invent the world it is being asked to operate inside.

### “Isn’t this just enterprise search?”

Response:

Enterprise search returns candidate documents.

Context Kernel constructs a task-bound artifact that another agent, authority system, or proof harness can inspect and reference.

### “Isn’t this just memory?”

Response:

Memory is one source class.

Context Kernel decides whether prior memory is relevant, current, sufficiently evidenced, and permitted for the present task.

It does not own developmental memory or the durable ledger.

### “Why SQLite and FTS5?”

Response:

SQLite and FTS5 provide a local, inspectable, low-dependency retrieval foundation.

Exact and full-text search remain valuable for:

* symbols;
* names;
* file paths;
* policy IDs;
* version strings;
* error messages.

Additional retrieval methods can be added behind the packet contract when they improve evidence-backed context construction.

### “Will it replace our vector database?”

Response:

Not necessarily.

A vector store may remain one candidate-retrieval mechanism.

Context Kernel’s role is to construct and govern the packet across retrieval methods.

### “Will it replace our knowledge graph?”

Response:

No.

A graph provides structured semantic relationships.

Context Kernel can query the relevant subgraph and combine it with current documents, code, history, policies, and evidence.

### “Does Context Kernel decide what the agent is allowed to do?”

Response:

No.

It may apply disclosure scope to decide what information the actor may receive.

SEA Forge decides whether the consequential action may execute.

### “Can the packet be wrong?”

Response:

Yes.

It can be incomplete, stale, poisoned, mis-scoped, or based on conflicting sources.

That is why packets need:

* citations;
* freshness;
* omissions;
* uncertainty;
* conflict records;
* reproducible construction;
* downstream feedback.

### “Does Context Kernel eliminate hallucinations?”

Response:

No.

It provides stronger source grounding and makes unsupported claims easier to detect.

The model can still misread, ignore, or invent information.

Proof and settlement remain separate.

### “Can it guarantee no sensitive information leaks?”

Response:

No universal guarantee should be made.

The product should enforce supported source scopes, disclosure rules, tenant boundaries, and provenance.

Deployment and integration quality still matter.

### “Is the context packet a source of truth?”

Response:

No.

The packet is a task-specific projection of current sources.

The canonical source remains:

* DomainForge for declared organizational meaning;
* repositories and approved systems for implementation and records;
* ledgers for durable events;
* policy systems for governance.

### “Can Context Kernel automatically update DomainForge?”

Response:

It may identify external information or observed disagreement and propose semantic updates.

A proposal must not become canonical organizational source without review and validation.

### “Does it require the whole GodSpeed stack?”

Response:

No.

The current MCP retrieval service should remain useful standalone.

The full stack adds:

* canonical semantics;
* routed work;
* runtime authority;
* proof;
* settlement;
* capability learning;
* durable ledger integration.

### “Does it require a vector model or remote embedding API?”

Response:

Do not claim that it does unless the current repository requires one.

The current SQLite and FTS5 foundation can operate without making remote embedding infrastructure the core dependency.

---

## 39. Failure modes

### 39.1 Context flooding

The packet includes too much material and transfers selection burden to the model.

**Antidote:** explicit context budget and inclusion rationale.

### 39.2 Context starvation

Compression omits a decisive constraint.

**Antidote:** task-relative sufficiency checks and required-context fields.

### 39.3 Stale-context dominance

Older, polished material outranks current implementation.

**Antidote:** freshness, version, and source-authority metadata.

### 39.4 Citation theater

The packet includes citations that do not support the claims derived from them.

**Antidote:** claim-to-source validation and exact source locations.

### 39.5 Semantic flattening

Two distinct concepts are summarized under one familiar term.

**Antidote:** DomainForge identities and explicit concept mapping.

### 39.6 Conflict smoothing

Contradictory sources are merged into a coherent but unsupported answer.

**Antidote:** conflict records and unresolved status.

### 39.7 Hidden truncation

The packet appears complete after token limits removed relevant material.

**Antidote:** truncation and omission metadata.

### 39.8 Retrieval popularity bias

Frequently repeated claims outrank authoritative but less common sources.

**Antidote:** source standing and version weighting.

### 39.9 Prompt injection

Retrieved content attempts to instruct the agent.

**Antidote:** instruction-data separation and source trust classification.

### 39.10 Cross-scope leakage

Relevant information crosses project, customer, tenant, or role boundaries.

**Antidote:** disclosure scope before retrieval and packet creation.

### 39.11 Memory poisoning

Incorrect prior agent output is recalled as fact.

**Antidote:** provenance, epistemic status, and source hierarchy.

### 39.12 Context replay

An old packet is reused after policy, code, or task state changed.

**Antidote:** expiry, freshness checks, packet supersession, and context diff.

### 39.13 Context monopolization

Context Kernel becomes the only place where organizational knowledge survives.

**Antidote:** packets remain projections linked to canonical sources.

### 39.14 Authority bleed

Context relevance or disclosure logic becomes action authorization.

**Antidote:** explicit SEA Forge boundary.

### 39.15 Continuity drift

Successive summaries gradually alter the meaning of the original source.

**Antidote:** stable citations, source hashes, and rehydration from originals.

---

## 40. Context quality metrics

Do not reduce context quality to retrieval precision alone.

Useful metrics may include:

### Packet relevance

Percentage of included items that materially affected the task.

### Decisive-source recall

Whether the sources later shown to control the outcome were present.

### Citation coverage

Percentage of consequential packet claims linked to inspectable sources.

### Citation entailment

Whether each source actually supports the associated claim.

### Stale-source rate

Percentage of included sources later found to be superseded or too old.

### Irrelevant-context burden

Tokens, items, or review time spent on material that did not affect the work.

### Missing-context failure rate

Work failures attributable to omitted information.

### Conflict-detection rate

Consequential source contradictions surfaced before action.

### Omission transparency

Percentage of known exclusions represented in the packet metadata.

### Disclosure integrity

Unauthorized source exposure incidents.

### Packet reproducibility

Whether the same requirement and source state reproduce an equivalent packet.

### Context continuity

Whether relevant prior decisions and unresolved issues survive across sessions.

### Downstream correction rate

How often SWE_SEED, SEA Forge, reviewers, or settlement systems require additional context after packet creation.

### Context payment

Token, retrieval, latency, compute, and human-review cost.

### Context value

Reduction in:

* irrelevant file reads;
* route changes;
* rework;
* proof failures;
* clarification cycles;
* incorrect modifications.

Do not market unmeasured improvements as established facts.

---

## 41. Evidence and proof inventory

Marketing may point to these surfaces only when present and passing in the current repository.

### Retrieval proof

* SQLite database;
* FTS5 index;
* document ingestion;
* exact and full-text search;
* source filtering;
* result ranking;
* local execution;
* deterministic query tests.

### MCP proof

* supported MCP transport;
* tool registration;
* tool schemas;
* query tools;
* document retrieval tools;
* citation-resolution tools;
* client compatibility tests.

### Context-packet proof

When implemented:

* `ContextRequired` consumption;
* `ContextPacketCreated` emission;
* packet schema;
* citations;
* actor and task identity;
* source scope;
* provenance;
* freshness;
* omissions;
* conflicts;
* packet hash;
* continuity references.

### Disclosure proof

When implemented:

* actor scope;
* source permissions;
* project or tenant boundaries;
* denied-source behavior;
* redaction;
* sensitive-data tests;
* cross-scope leakage tests.

### Semantic proof

When implemented:

* DomainForge concept mapping;
* semantic-pack loading;
* domain-model hash;
* unmapped-term handling;
* semantic conflict detection;
* canonical identity references.

### Continuity proof

When implemented:

* packet lineage;
* packet diff;
* supersession;
* stale-packet rejection;
* unresolved-question carry-forward;
* source-version changes.

### Security proof

When implemented:

* prompt-injection fixtures;
* instruction-data separation;
* hostile document tests;
* secret detection;
* source-trust classes;
* payload limits;
* parser hardening.

### Integration proof

When implemented:

* SWE_SEED `ContextRequired`;
* Context Kernel `ContextPacketCreated`;
* SEA Forge context reference;
* proof record linked to packet;
* settlement record linked to packet;
* durable event persistence;
* end-to-end correlation IDs.

---

## 42. Current implementation and maturity boundary

### Evidence-backed current foundation

The current Context Kernel repository may be described, when verified, as:

* a Rust implementation;
* a single-binary MCP service;
* a local-first durable document-retrieval system;
* built around SQLite and FTS5;
* capable of exposing document and context retrieval through supported MCP transports;
* designed to produce cited context for downstream agent work.

### Current integration gap

The intended role as the live real-time context authority is not yet fully implemented in the canonical Rust loop.

The current audit identified that:

* a `ContextAgent` trait exists;
* no complete Rust implementation of that trait was found;
* live loop behavior uses stand-ins or stubs;
* the context-packet contract is not yet proven end to end across the full capability loop.

### Marketing consequence

Do not say:

* Context Kernel currently governs all real-time agent context;
* every agentic run receives a canonical context packet;
* context-packet continuity is fully implemented;
* the full GodSpeed stack currently uses Context Kernel as its only live context authority;
* all packets conform to the canonical Semantic Envelope.

Say instead:

> Context Kernel currently provides the local MCP retrieval foundation. Its target role is to become the real-time context authority that emits bounded, cited context packets into the GodSpeed capability loop.

### Maturity formula

```text id="r1cf4b"
current proof
= local retrieval service

integration target
= real-time context authority

market category
= local-world construction infrastructure
```

Do not collapse those three layers into one implementation claim.

---

## 43. Non-negotiable integrity gates before strong context-authority claims

The following conditions should be proven before Context Kernel is marketed as the canonical real-time context authority.

### 43.1 One canonical packet schema

The packet contract must define:

* identity;
* task;
* actor;
* boundary;
* sources;
* citations;
* provenance;
* freshness;
* omissions;
* conflicts;
* completeness;
* continuity.

### 43.2 Real loop implementation

The Rust context-agent path must consume real context requirements and emit real packets without relying on test stand-ins.

### 43.3 Citation integrity

Every consequential extracted or summarized claim must remain linked to a supporting source.

### 43.4 Source-scope enforcement

Unauthorized sources must not enter the packet even when highly relevant.

### 43.5 Freshness handling

The system must identify or reject sources outside the task’s freshness requirements.

### 43.6 Conflict preservation

The system must not merge contradictory authoritative sources into one unsupported statement.

### 43.7 Omission transparency

Token limits, inaccessible sources, retrieval gaps, and deliberate exclusions must be represented.

### 43.8 Prompt-injection separation

Retrieved documents must not gain instruction authority merely by entering the packet.

### 43.9 Semantic identity

Where DomainForge models exist, the packet should carry the relevant model identity and concept references.

### 43.10 Packet lineage

A downstream run must be able to identify the exact packet it received and whether that packet was superseded.

### 43.11 End-to-end correlation

The same work request should link:

```text id="gj1u72"
ContextRequired
→ ContextPacketCreated
→ RouteSelected
→ AuthorityChecked
→ ProofCompleted
→ SettlementRecorded
```

### 43.12 Adverse-path proof

Tests must cover:

* missing context;
* stale context;
* conflicting context;
* unauthorized context;
* poisoned context;
* truncated context;
* unavailable sources;
* invalid citations;
* stale packet replay.

### Canonical internal rule

> Context Kernel must not make an agent confidently wrong with better citations.

---

## 44. Licensing

Use the current Context Kernel repository license as the legal source of truth.

Do not infer its license from DomainForge, SEA Forge, or another GodSpeed repository.

Before publishing licensing copy, verify:

* license name;
* modification terms;
* redistribution terms;
* commercial-use terms;
* hosted-service terms;
* bundled dependencies;
* generated packet treatment.

Until verified, approved marketing language is limited to:

> See the repository license for current use, modification, redistribution, and commercial terms.

Do not describe Context Kernel as open source, source available, permissive, copyleft, or commercially restricted unless the current license supports that exact statement.

---

## 45. Adoption path

### Stage 1: Index one repository

Begin with:

* one repository;
* one source scope;
* current documents;
* exact and full-text retrieval;
* one supported agent.

Do not begin with the entire organization.

### Stage 2: Produce one cited packet

Choose one real task.

Create a packet containing:

* task;
* decisive files;
* relevant ADRs;
* current constraints;
* citations;
* one explicit omission.

### Stage 3: Demonstrate a missing-context failure

Run a task without the packet.

Record:

* irrelevant exploration;
* wrong-file selection;
* stale assumption;
* omitted constraint;
* rework.

Run the same or comparable task with the packet.

### Stage 4: Add freshness and source hierarchy

Distinguish:

* current implementation;
* current policy;
* historical decision;
* superseded documentation;
* agent-generated speculation.

### Stage 5: Add DomainForge semantics

Map packet items to canonical:

* concepts;
* resources;
* policies;
* flows;
* model hashes.

### Stage 6: Integrate SWE_SEED

Have the work route emit `ContextRequired` and consume the packet.

Require missing context to fail visibly.

### Stage 7: Add disclosure scoping

Apply:

* actor;
* project;
* tenant;
* role;
* sensitivity;
* source collection.

### Stage 8: Integrate SEA Forge

Reference the context packet during authority decisions without allowing it to become the decision owner.

### Stage 9: Add continuity

Support:

* packet lineage;
* packet diff;
* supersession;
* unresolved questions;
* prior-settlement references.

### Stage 10: Add durable organizational sources

Retrieve from approved:

* ledgers;
* policy repositories;
* knowledge graphs;
* external documentation;
* operational records.

### Stage 11: Add adaptive retrieval carefully

Only after strong packet and citation proof should the system add:

* vector retrieval;
* learned ranking;
* compression models;
* source-quality prediction;
* adaptive context budgeting.

Learned selection remains inspectable and regression tested.

---

## 46. Primary use cases

### Coding-agent repository context

Provide:

* relevant modules;
* architecture boundaries;
* current instructions;
* tests;
* prior decisions;
* proof requirements;
* repository conventions.

### Current API and library context

Retrieve current documentation when version-specific behavior matters.

Keep external documentation separate from repository-specific truth.

### Architecture decision continuity

Carry relevant ADRs into implementation work and identify superseded decisions.

### Incident and debugging context

Construct a packet from:

* current code;
* logs;
* traces;
* recent changes;
* known incidents;
* environment;
* operational constraints.

### Regulated work context

Provide:

* current procedures;
* applicable policies;
* source authority;
* evidence requirements;
* approved standards;
* disclosure boundaries.

### Multi-agent handoff

Give a second agent the same semantic work world without relying on a prose handoff from the first agent.

### Cross-session continuity

Preserve current task meaning, unresolved questions, source versions, and decisions after the conversation ends.

### Domain-aware retrieval

Use DomainForge concepts to retrieve information relevant to the organization’s actual entities, flows, policies, and resources.

### Context for governed actions

Supply SEA Forge with a cited context reference while preserving the separate authority decision.

### Context for proof

Supply SWE_SEED with:

* required tests;
* expected artifacts;
* architecture constraints;
* prior failures;
* proof criteria.

### Context for settlement

Supply GodSpeed-Agent with the relevant context-packet identity so later settlement can be interpreted against what the actor knew.

### Forward-deployed continuity

Preserve customer-specific discoveries without allowing them to leak into another customer’s packet or silently become general policy.

---

## 47. The Context Packet Evidence Pack

A strong market artifact is a **Context Packet Evidence Pack**.

It should contain:

### Context requirement

* work request;
* actor;
* task;
* domain;
* expected route;
* evidence requirements;
* source and disclosure scope;
* context budget.

### Packet

* packet identity;
* selected sources;
* citations;
* provenance;
* freshness;
* semantic references;
* conflict record;
* omission record;
* uncertainty;
* packet hash.

### Downstream work evidence

* route selected;
* artifact produced;
* proof results;
* context follow-up requests;
* missing-context failures;
* final settlement reference.

### Comparative run

Compare:

* unstructured prompt context;
* broad repository dump;
* Context Kernel packet.

Measure:

* irrelevant files read;
* decisive-source recall;
* clarification cycles;
* wrong assumptions;
* rework;
* token burden;
* proof failures;
* citation coverage.

### Acceptance question

The evidence pack should answer:

> Did the packet make the next action more coherent at lower context cost without hiding a decisive omission?

---

## 48. Minimum credible proof

The strongest first demonstration is not a search box.

It is one task whose outcome changes because the context boundary improved.

### Baseline run

Give the agent:

* repository access;
* ordinary instructions;
* no constructed packet.

Record:

* files inspected;
* assumptions;
* missed constraints;
* token use;
* proposed route;
* artifact quality;
* proof result.

### Context Kernel run

Give another comparable run:

* explicit context requirement;
* bounded packet;
* citations;
* freshness;
* omissions;
* relevant domain concepts.

### Evidence of value

Show:

* fewer irrelevant reads;
* correct identification of the decisive source;
* fewer clarification cycles;
* preserved architecture;
* fewer changes outside scope;
* stronger proof completion;
* visible uncertainty;
* lower or justified context cost.

### Failure proof

Also show that Context Kernel refuses or degrades visibly when:

* the decisive source is missing;
* sources conflict;
* the source is stale;
* disclosure prevents access;
* the packet budget is insufficient.

A product that only produces good packets in ideal conditions has not proved context authority.

---

## 49. README-specific guidance

The README must not begin with:

* SQLite;
* FTS5;
* MCP;
* crate architecture;
* a feature inventory;
* “RAG” terminology.

Its upper section should answer:

1. What is wrong with ordinary agent context?
2. Why is the whole repository not context?
3. What does Context Kernel construct?
4. What does a context packet contain?
5. How is it different from search or memory?
6. What can the reader run now?
7. What remains an integration target?

### Recommended upper-page sequence

1. problem-led headline;
2. plain-language product explanation;
3. warehouse-versus-workbench analogy;
4. context requirement to packet flow;
5. smallest local MCP retrieval demonstration;
6. example packet with citations;
7. current implementation versus target architecture;
8. integration with SWE_SEED;
9. boundaries with DomainForge, SEA Forge, and GodSpeed-Agent;
10. technical reference.

### Ten-second outcome

The reader should understand:

> Context Kernel gives an agent the smallest cited set of information required for the current task.

### Sixty-second outcome

The reader should understand:

* more context is not automatically better;
* the whole repository is not a task world;
* a context requirement is more than a search query;
* Context Kernel produces a bounded packet;
* the packet includes citations, freshness, conflicts, and omissions;
* DomainForge supplies semantic meaning;
* SWE_SEED requests and uses the packet;
* SEA Forge owns action authority;
* GodSpeed-Agent owns settlement and capability;
* the current product is a real MCP retrieval foundation;
* full loop context authority remains an integration target.

### Primary developer action

> Index one repository, request context for one real task, and inspect the sources and omissions in the resulting packet.

### Stronger evaluation action

> Run the same task with and without the packet and compare irrelevant exploration, missed constraints, proof failures, and token burden.

---

## 50. Content acceptance tests

Before publishing Context Kernel copy, verify that the reader can answer:

1. What problem does Context Kernel solve?
2. Why is a large context window insufficient?
3. Why is the whole repository not context?
4. What is a context requirement?
5. What is a context packet?
6. What makes context minimal?
7. What makes context sufficient?
8. What makes it semantic?
9. Why are citations necessary?
10. What is provenance?
11. How is freshness represented?
12. What does the packet say about omissions?
13. What happens when sources conflict?
14. What is semantic continuity?
15. How does Context Kernel differ from RAG?
16. How does it differ from memory?
17. How does it relate to DomainForge?
18. How does it relate to SWE_SEED?
19. How does it relate to SEA Forge?
20. How does it relate to GodSpeed-Agent?
21. What authority does it have?
22. What authority does it not have?
23. What is implemented now?
24. What remains an integration target?
25. What should the reader run next?

Apply six final tests.

### So what?

Does each capability connect to better task understanding, lower context burden, stronger provenance, safer disclosure, or less rework?

### Prove it

Can each current implementation claim be reproduced from the repository?

### Boundary

Does the copy distinguish context authority from action authority, settlement, memory, and semantic source ownership?

### Omission integrity

Does the packet expose what it could not include or verify?

### Continuity integrity

Can the system preserve meaning without making prior summaries canonical?

### Compression

Can a technically competent reader explain Context Kernel in one sentence after reading the opening?

---

## 51. Final strategic summary

Context Kernel should not be marketed primarily as:

* RAG;
* vector search;
* a document database;
* an agent memory;
* a second brain;
* prompt management;
* a universal knowledge system.

Those descriptions name mechanisms or adjacent categories.

The immediate problem is:

```text id="4ji0gg"
agent receives task
→ broad information access
→ arbitrary retrieval and prompt assembly
→ hidden omissions
→ agent reconstructs the world
```

Context Kernel changes the relationship:

```text id="4qzpp3"
context requirement
→ actor and disclosure scope
→ semantic retrieval
→ provenance and freshness
→ conflict and omission handling
→ bounded context packet
→ downstream work
```

Its smallest useful promise is:

> Give the agent the decisive context for the current task and show where every important claim came from.

Its product category is:

> Real-time context authority.

Its stronger strategic category is:

> Local-world construction infrastructure.

Its current implementation is:

> A local-first, single-binary MCP retrieval foundation built around SQLite and FTS5.

Its target role in the GodSpeed stack is:

> DomainForge defines the domain.
> Context Kernel constructs the local world.
> SEA Forge governs the work.
> SWE_SEED proves the change.
> GodSpeed-Agent compounds the capability.

Its controlling public idea is:

> Give the agent the world it needs. Not the whole world.

Its controlling internal discipline is:

> Never hide missing, stale, conflicting, or unauthorized information behind a fluent context summary.

Bound the world.

Preserve the distinctions.

Cite the sources.

Expose the gaps.
