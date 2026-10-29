# Context Kernel

### Give the agent the context this task needs, not everything it can reach.

[![CI](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/ci.yml)
[![Release](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/release.yml/badge.svg)](https://github.com/GodSpeedAI/Context-Kernel/actions/workflows/release.yml)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-stdio%20%2B%20SSE-blue.svg)](docs/mcp-contracts.md)

A coding agent can search your whole repository.

It can fetch documentation from the web.

It can hold a very large context window.

It can still confidently use the wrong API because the relevant version was buried under newer results, forget a decision made yesterday, or spend half its context window reading files that have nothing to do with the task.

More accessible information does not automatically create better context.

**Context Kernel is an evidence-bearing context assembly service for agents. It turns task-relevant context needs into bounded, cited, provenance-preserving material so an agent can work from a smaller local world instead of repeatedly reconstructing the domain from everything it can access.**

Plainly, Context Kernel answers:

> **What does this agent actually need to know right now, and where did that information come from?**

Its basic shape is:

```text
context need
     ↓
resolve the relevant source
     ↓
search the bounded corpus
     ↓
retrieve compact evidence
     ↓
citations + provenance
     ↓
agent context
```

Context Kernel runs as a single Rust binary with SQLite, FTS5, filesystem artifacts, optional embeddings, source adapters, and MCP interfaces.

It can run entirely inside your own deployment.

---

## The problem is not access

Give an agent access to:

```text
repository
documentation
tickets
session history
GitHub
internal notes
web pages
API references
```

and you have increased what it can search.

You have not answered:

```text
Which source applies?

Which version applies?

Which three files matter?

Which decision is still current?

Which memory is durable?

Which result is stale?

Which source can be cited?

Which private corpus may this agent read?

How much of this belongs in the model's context at once?
```

Without that structure, agent context becomes a repeated reconstruction exercise.

A new session starts.

The model searches again.

It reads a different subset.

It makes slightly different assumptions.

Eventually the repository contains several technically plausible interpretations of the same thing.

Context Kernel makes retrieval and memory explicit infrastructure instead of leaving them as incidental prompt behavior.

---

## Bigger context windows do not remove context selection

Suppose an agent is fixing code that uses a third-party library.

It can read:

```text
100 documentation pages
20 source files
the entire conversation history
```

The useful answer may live in:

```text
one API page for the installed version
two repository files
one prior architectural decision
```

Loading more material increases what the model can attend to.

It does not guarantee that attention lands on the consequential distinction.

Context Kernel's default retrieval shape is deliberately narrower:

```text
resolve identity
      ↓
search focused chunks
      ↓
read compact excerpts
      ↓
cite the source
```

Search before reading.

Resolve before guessing.

Return the few useful excerpts instead of making the model carry the entire library manual.

---

## Start with the mistake you are tired of seeing twice

Good first uses are ordinary:

* an agent repeatedly uses an outdated library API
* architectural decisions disappear between sessions
* the same documentation gets fetched differently every run
* a large repository overwhelms agent attention
* private and public references need different access
* important context lives in large artifacts that should not sit inline in every prompt

Context Kernel exposes MCP tools for these jobs.

Canonical tools include:

```text
get_status
store_context
recall_context
ingest_source
refresh_source
query_reference
resolve_reference
context_required
ingest_context_effects
```

For compatibility with documentation-oriented MCP clients:

```text
resolve-library-id → resolve_reference
get-library-docs   → query_reference
query-docs         → query_reference
```

The useful first experiment is small:

> Put one documentation corpus or recurring decision behind Context Kernel and see whether the next agent can retrieve the right evidence without rediscovering it from scratch.

---

## Three kinds of context fail differently

Context Kernel currently brings together three related jobs.

### Reference context

This is information that already exists somewhere else:

```text
library documentation
repository documentation
GitHub source
web references
local files
```

The problem is retrieval.

Which source?

Which version?

Which chunk?

Which citation?

---

### Session context

This is information produced during the work:

```text
decisions
constraints
requirements
notes
important observations
continuation state
```

The problem is continuity.

Does it survive the conversation?

Can it be recalled later?

Is the useful decision distinguishable from thousands of transcript tokens?

---

### Large context artifacts

Some useful material should not be repeatedly inserted into prompts:

```text
large payloads
generated reports
reference files
structured outputs
```

The problem is storage and addressability.

Context Kernel stores the artifact and lets session records refer to it instead of copying the whole payload into every context.

These jobs share infrastructure, but they remain distinguishable.

---

## Context should arrive with receipts

An agent saying:

> According to the documentation...

is stronger when you can ask:

```text
Which documentation?

From which source?

Which version?

Which chunk?

When was it indexed?

Which corpus did it come from?
```

Context Kernel returns compact reference material with source citations and metadata.

The goal is not citation formatting for its own sake.

The citation allows a downstream agent or person to distinguish:

```text
retrieved evidence
```

from:

```text
model recollection
```

Those can agree.

They should not be treated as the same source.

---

## Resolve identity before retrieving text

Library names are often ambiguous.

Consider:

```text
react
requests
tokio
openai
langchain
```

A free-text query should not have to rediscover which corpus or library identity you meant on every request.

Context Kernel supports a resolution step:

```text
human/library name
      ↓
stable reference identity
      ↓
scoped retrieval
```

This is why the retrieval flow begins with:

```text
resolve_reference
```

and compatibility clients can use:

```text
resolve-library-id
```

Once identity is resolved, later retrieval can stay inside the intended corpus rather than searching every indexed source as one undifferentiated pile.

---

## Search first, read second

A common agent retrieval pattern is:

```text
find promising URL
      ↓
load entire page
      ↓
put page in context
```

Repeat that several times and the model now has a large amount of technically related material competing for attention.

Context Kernel instead favors:

```text
query
  ↓
rank chunks
  ↓
deduplicate documents
  ↓
return compact excerpts
  ↓
follow the relevant source when needed
```

Retrieval currently uses:

* SQLite FTS5
* BM25 lexical search
* intent-aware ranking
* trust signals
* freshness signals
* document-level result deduplication
* optional embeddings
* scoped vector metadata

For small corpora, optional semantic retrieval uses brute-force cosine search.

That is an implementation choice, not a claim that vector search should replace lexical retrieval.

---

## Lexical and semantic retrieval have different strengths

Exact terms matter in software work.

If the agent needs:

```text
ProviderManager
HTTP_429
tokio::select!
v1.4.2
```

lexical search is valuable.

If the task asks:

> Where do we handle retryable provider failures?

the useful source may not contain those exact words.

Optional embeddings can help produce candidates for that kind of query.

Context Kernel can use both.

The important rule is scope first.

Semantic similarity does not grant a result permission to cross corpus boundaries or become relevant simply because the vectors are close.

---

## Durable memory should not become transcript landfill

A conversation transcript contains many things:

```text
questions
false starts
temporary guesses
tool output
decisions
corrections
casual discussion
durable constraints
```

Saving everything forever does not solve memory.

It creates a larger search problem.

Context Kernel stores session events in SQLite and applies explicit memory and retention behavior through the `ck-memory` layer.

Important events can be pinned.

Retention cleanup can remove:

```text
old unpinned session events
inactive sessions
finished refresh jobs
old artifact payloads
disabled sources
```

Pinned events survive cleanup.

The goal is not infinite recollection.

It is durable access to context worth carrying forward.

---

## Large artifacts belong outside the prompt

Suppose a tool produces a large report.

One approach is to paste the entire report into the conversation every time it may matter.

That spends model context repeatedly.

Context Kernel can store the payload in its filesystem artifact store and place a reference in session state.

Conceptually:

```text
large payload
     ↓
artifact store
     ↓
stable reference
     ↓
session event
     ↓
retrieve when needed
```

The model carries the reference until the contents actually become relevant.

Storage becomes cheaper than attention.

---

## Context is task-relative

There is no universally correct context packet.

The same repository can require very different context for:

```text
fix checkout bug

review authentication architecture

update API documentation

investigate deployment failure
```

The most useful context is the smallest set that preserves the distinctions required by the current work.

Context Kernel provides the primitives for that assembly through:

```text
session state
reference identity
scoped corpora
search
ranking
citations
artifacts
```

Today these capabilities are exposed through MCP tools such as `recall_context`, `resolve_reference`, and `query_reference`.

A richer task-specific packet may be assembled by an upstream harness using those tools.

That keeps the implementation claim narrow: Context Kernel owns retrieval and context evidence. It does not need to pretend that every task has one magical automatic context window.

---

## Context Kernel supplies context; it does not decide the task

A retrieved source can say:

```text
production deployments require approval
```

Context Kernel can return that statement with its source.

It does not therefore decide:

```text
deny deployment
```

That is an authority decision.

Likewise, Context Kernel can retrieve a testing requirement.

It does not decide whether the resulting task has settled.

It supplies representation and evidence to systems that make those decisions.

The distinction is important:

```text
retrieval
≠
authority

context
≠
execution

memory
≠
capability
```

Knowing the rule and obeying the rule are different system responsibilities.

---

## Local-first means private context can stay local

Some of the context most valuable to an agent is exactly the material you do not want sent to another SaaS memory layer:

```text
internal documentation
private repositories
architecture decisions
session history
operational notes
```

Context Kernel is designed to run inside your own environment.

Its ordinary storage uses:

```text
SQLite
filesystem artifacts
local indexes
```

The service does not require an external model provider for its core retrieval path.

Optional embeddings can use an OpenAI-compatible endpoint when configured.

If you do not need semantic embeddings, FTS5 works without them.

---

## Sources are explicit

Context Kernel currently supports ingestion from:

### Local filesystem trees

Useful for:

```text
repository documentation
internal references
generated docs
local knowledge bases
```

### GitHub repositories

Useful when the reference source lives in GitHub.

The adapter detects truncated GitHub tree API responses rather than silently treating an incomplete tree as the complete repository.

### Websites

Useful for public documentation and reference material.

The web adapter includes protections against:

```text
private IP targets
localhost-style hosts
non-HTTP schemes
DNS names resolving to private addresses
```

That reduces SSRF exposure when agents can ask the service to ingest remote material.

PDF ingestion is not part of the current release.

Neither is a bundled external documentation dataset.

Those are future capabilities, not hidden assumptions in the README.

---

## Public and private context should not quietly mix

Context Kernel supports corpus-scoped authorization.

For SSE deployments, bearer authentication can be required:

```toml
[mcp.sse]
require_auth = true

[auth]
tokens = ["${CONTEXT_KERNEL_AUTH_TOKEN}"]
```

Tool access and corpus access can be configured separately:

```toml
[authorization]
allow_tools = ["get_status", "query_reference", "resolve_reference"]
allow_corpora = ["public_docs"]
```

This lets an operator distinguish:

```text
may call query_reference
```

from:

```text
may query private_architecture
```

Those are different permissions.

Stdio mode is intended for local-trust workflows where the process boundary is the trust boundary.

---

## Stdio and SSE serve different deployment shapes

For a local editor or agent:

```text
agent process
     ↓ stdio
Context Kernel
     ↓
local storage
```

For remote clients:

```text
agent / service
     ↓ authenticated SSE
Context Kernel
     ↓
shared deployment storage
```

Both surfaces expose the same core application rather than creating separate retrieval implementations.

The transport should not determine the context semantics.

---

## Quick start

Build the binary:

```bash
cargo build --release -p ck-bin
```

Initialize storage:

```bash
target/release/context-kernal migrate \
  --database ./data/kernel.db
```

Check health:

```bash
target/release/context-kernal doctor \
  --database ./data/kernel.db
```

Run locally over MCP stdio:

```bash
target/release/context-kernal serve \
  --database ./data/kernel.db \
  --data-dir ./data
```

Context Kernel is now available as a local MCP service.

---

## Run over SSE

Generate a bearer token:

```bash
export CONTEXT_KERNEL_AUTH_TOKEN="$(openssl rand -hex 32)"
```

Start the service:

```bash
target/release/context-kernal serve \
  --config config/context-kernal.sse.example.toml \
  --sse 127.0.0.1:8765
```

Use SSE when the context service needs to serve processes outside the local stdio relationship.

Configure corpus and tool authorization before exposing private sources.

---

## Inspect the storage schema

Print the embedded SQLite schema:

```bash
target/release/context-kernal export-schema
```

For setup details, see:

* [Install](docs/install.md)
* [Configuration](docs/configuration.md)
* [Operations](docs/operations.md)

---

## Core MCP tools

### `get_status`

Inspect service status.

### `store_context`

Persist session context that should survive the immediate tool call.

### `recall_context`

Retrieve relevant session context.

### `ingest_source`

Add a source corpus.

### `refresh_source`

Refresh previously ingested material.

### `query_reference`

Search reference material and return relevant excerpts.

### `resolve_reference`

Resolve a user-facing library or source name into the stable identity used for retrieval.

### `context_required`

Answer a frozen SWE_SEED context requirement with a sealed, hash-verified context bundle. The response reports retrieval completeness, omissions, a governed no-context reason when nothing qualifies, and access status. It never grants action authority.

### `ingest_context_effects`

Record settlement-backed effects (contradicted source, stale source, missing distinction) into the per-world ledger. Later retrievals in that world demote affected sources, capped at 0.50 per source.

The compatibility aliases exist so clients designed around common documentation MCP conventions can use Context Kernel without changing its canonical internal vocabulary.

See:

* [MCP contracts](docs/mcp-contracts.md)

---

## Freshness matters because documentation moves

Training data has a cutoff.

Documentation changes after that cutoff.

Local repository code may change after the documentation was indexed.

A cached reference can become stale.

Context Kernel therefore treats source refresh and freshness as part of retrieval rather than assuming that once-ingested information remains correct forever.

Sources can be refreshed explicitly:

```text
refresh_source
```

and freshness can participate in ranking.

This still does not guarantee the source itself is correct.

It makes the age and origin of the retrieved material more inspectable.

---

## Trust is a ranking signal, not truth

Some sources deserve more confidence than others.

For example:

```text
official library documentation

repository source code

third-party tutorial

old forum post
```

may not deserve equal rank for an API question.

Context Kernel can incorporate trust signals into ranking.

That does not make a high-trust source infallible.

It means source quality can influence candidate ordering without being erased into one undifferentiated relevance score.

---

## One canonical context service, several consumers

Context Kernel is exposed through MCP so different agents and harnesses can consume the same underlying context infrastructure.

Conceptually:

```text
Claude Code ──┐
Codex ────────┤
Gauntlet ─────┤
SWE_SEED ─────┼──> Context Kernel
other agents ─┤       ↓
              │    shared retrieval
              │    shared memory
              │    shared provenance
              └──  shared corpus boundaries
```

That is more useful than teaching every agent host to independently maintain:

```text
its own session memory
its own docs crawler
its own vector store
its own library resolver
its own source trust rules
```

The consumer can change.

The context layer remains stable.

---

## SWE_SEED integration

SWE_SEED defines the context required by a software-work route.

For example:

```text
bugfix route
      ↓
requires:
  AGENTS.md
  active specification
  relevant implementation
  nearby tests
```

Context Kernel can supply stronger retrieval behind that requirement.

The division becomes:

```text
SWE_SEED
asks:
"What context does this work contract require?"
        ↓
Context Kernel
answers:
"Here is the bounded source material
and where it came from."
```

SWE_SEED should not need to become a second retrieval engine.

Context Kernel should not need to decide the software route.

The wire boundary is the `context_required` tool: SWE_SEED sends a versioned requirement with a pinned `world_ref`, and Context Kernel returns a `context_bundle` whose `integrity.content_hash` can be re-verified. Walkthrough: [Context Requirement Resolution](docs/workflows/context-requirement-resolution.md). Interactive version: [Context Requirement Explorer](docs/explainers/context-requirement-explorer.html).

---

## Gauntlet integration

Long-horizon execution repeatedly creates new context needs.

An executor may discover:

```text
a new dependency
an unfamiliar subsystem
a prior decision
a missing API reference
```

Gauntlet can request additional context as those distinctions become relevant.

The useful pattern is:

```text
bounded execution
      ↓
new uncertainty
      ↓
context request
      ↓
Context Kernel
      ↓
cited context
      ↓
execution continues
```

That lets context expand because the work earned the expansion instead of preloading every potentially related artifact.

---

## DomainForge integration

DomainForge gives a system an explicit semantic model.

Context Kernel gives agents the contextual evidence needed to work with that system.

Those are related but different.

```text
DomainForge
answers:
"What does this domain mean?"

Context Kernel
answers:
"What relevant source material
does this agent need right now?"
```

A `.sea` model can itself become a high-value reference source for context assembly.

But retrieval does not replace semantic compilation.

---

## RealityTrace integration

RealityTrace binds expectations to observations with evidence and provenance.

Context Kernel can help retrieve the sources needed to form or interpret those expectations.

RealityTrace should preserve the observation chain.

Context Kernel should preserve the retrieval chain.

Those are complementary provenance problems.

---

## Architecture

Context Kernel follows a ports-and-adapters architecture.

| Layer       | Crates                                                               | Purpose                                             |
| ----------- | -------------------------------------------------------------------- | --------------------------------------------------- |
| Domain      | `ck-core`                                                            | entities, enums, port traits                        |
| Application | `ck-app`, `ck-memory`                                                | use cases, memory policy, orchestration             |
| Storage     | `ck-storage-sqlite`, `ck-artifacts`, `ck-vector`                     | SQLite persistence, artifacts, vector metadata      |
| Retrieval   | `ck-search-fts`, `ck-ranker`, `ck-embeddings`                        | lexical search, ranking, optional embeddings        |
| Adapters    | `ck-adapter-filesystem`, `ck-adapter-github`, `ck-adapter-web`       | source discovery and normalization                  |
| MCP         | `ck-mcp`                                                             | tool contracts, aliases, errors                     |
| Transports  | `ck-transport-stdio`, `ck-transport-sse`                             | local and remote serving                            |
| Runtime     | `ck-bin`, `ck-config`, `ck-auth`, `ck-observability`, `ck-scheduler` | CLI, configuration, authorization, tracing, refresh |

The architectural boundary is strict:

> SQL, HTTP, and transport-specific behavior do not belong in `ck-core`.

That keeps the context semantics independent from the storage or transport implementation.

---

## Configuration

Production-oriented examples live under:

```text
config/
```

including:

* [`context-kernal.example.toml`](config/context-kernal.example.toml)
* [`context-kernal.local.example.toml`](config/context-kernal.local.example.toml)
* [`context-kernal.sse.example.toml`](config/context-kernal.sse.example.toml)

Important sections:

| Section           | Purpose                     |
| ----------------- | --------------------------- |
| `[storage]`       | SQLite database             |
| `[mcp]`           | stdio and SSE               |
| `[auth]`          | bearer authentication       |
| `[authorization]` | tool and corpus allowlists  |
| `[scheduler]`     | source refresh              |
| `[embeddings]`    | optional semantic retrieval |
| `[reload]`        | configuration reload        |
| `[retention]`     | memory and artifact cleanup |
| `[observability]` | text or JSON tracing        |

Config reload updates application state and SSE authorization state.

Changing the listening bind address still requires a restart.

---

## Observability

Context Kernel emits structured tracing to stderr.

For machine-readable logs:

```toml
[observability]
log_format = "json"
```

The configuration accepts:

```text
otel_endpoint
```

and the `ck-observability` crate detects it.

OpenTelemetry exporter and metrics backend wiring are **not implemented yet**.

That distinction stays explicit because configuration support is not the same thing as an operating exporter.

---

## Retention is part of context quality

Keeping everything forever is easy.

Knowing what should still influence future work is harder.

Context Kernel retention can remove:

* old unpinned session events
* inactive sessions
* completed refresh jobs
* artifact metadata and payloads
* disabled old sources

Pinned context survives.

Retention is therefore not only a storage concern.

It prevents stale incidental history from accumulating indefinitely beside durable context.

---

## Current status

### Implemented

* single Rust binary
* SQLite schema and migrations
* MCP over stdio
* MCP over SSE
* bearer authentication
* tool authorization
* corpus authorization
* filesystem source adapter
* GitHub source adapter
* web source adapter
* FTS5 retrieval
* BM25 ranking
* intent-aware ranking
* trust and freshness signals
* document deduplication
* optional OpenAI-compatible embeddings
* scoped vector metadata
* brute-force vector search
* session memory
* filesystem artifact storage
* retention GC
* source refresh
* configuration templates
* release workflow
* deployment smoke tests

### Known limits

* OpenTelemetry exporter wiring is not implemented
* metrics backend wiring is not implemented
* PDF ingestion is not implemented
* bundled external documentation datasets are not implemented
* vector search is intended for small corpora
* SSE configuration reload does not rebind the HTTP listener

The distinction between implemented and planned behavior should remain visible.

A context system that invents its own implementation status would be missing the point.

---

## Development

Read first:

* [AGENTS.md](AGENTS.md)
* [.github/copilot-instructions.md](.github/copilot-instructions.md)
* [.agents/project_state.md](.agents/project_state.md)
* [.agents/evidence.md](.agents/evidence.md)

Development rules include:

```text
trust code and tests over documentation intent

preserve ports-and-adapters boundaries

use focused tests for behavioral changes

update durable project memory after meaningful work

do not report implementation status
without repository evidence
```

Run the release gate:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p ck-bin
```

---

## Release

Version tags trigger the release workflow.

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds the Linux binary and publishes a SHA-256 checksum.

See:

* [Release process](docs/release.md)

---

## What Context Kernel is not

### Not a vector database

It may use vectors.

Vector search is one retrieval mechanism inside the service.

### Not a hosted memory product

Context Kernel is designed to run in your own environment.

### Not a transcript archive

Persistent memory exists to preserve useful context, not to make every conversation immortal.

### Not a prompt stuffing engine

The objective is bounded relevant context, not maximum tokens.

### Not a coding agent

Context Kernel supplies context to an executor.

It does not perform the software task.

### Not an authority layer

A retrieved policy can inform an authority decision.

Retrieving the policy does not authorize the action.

### Not a source of capability

An agent recalling the correct documentation does not establish that it can reliably use that documentation under variation.

### Not DomainForge

DomainForge compiles canonical domain meaning.

Context Kernel retrieves relevant source material.

### Not RealityTrace

RealityTrace preserves expected-versus-observed evidence.

Context Kernel preserves retrieval and memory provenance.

---

## Where it fits

Context Kernel works standalone as a local MCP context and reference service.

A minimal deployment is:

```text
coding agent
     ↓
Context Kernel
     ↓
local docs + memory + references
```

Within the broader GodSpeed architecture:

```text
.sea
represents consequential domain meaning
        ↓
DomainForge
compiles and validates that meaning
        ↓
Context Kernel
supplies bounded relevant context
        ↓
SWE_SEED
defines the software-work contract
        ↓
SEA-Forge
authorizes consequential action
        ↓
Gauntlet
executes long-horizon work when needed
        ↓
RealityTrace
binds expectation to observation
        ↓
GodSpeed-Agent
updates developmental memory and capability
```

These components remain independently useful.

Context Kernel's boundary is intentionally narrow:

> **Supply the smallest useful body of context, with enough provenance that another system can inspect where it came from.**

---

## When Context Kernel is probably worth trying

Context Kernel starts to earn its cost when:

* agents repeatedly rediscover the same documentation
* API mistakes come from stale model knowledge
* important decisions disappear between sessions
* context windows are filling with large amounts of weakly relevant material
* several agent hosts need access to the same reference layer
* private and public corpora need different access rules
* large artifacts need durable storage outside the conversation
* retrieval results need citations and provenance
* documentation freshness matters
* you want local-first context infrastructure rather than another hosted memory dependency

If a small repository and one short-lived agent session can keep all relevant context directly in view, Context Kernel may be unnecessary.

Use the smallest context system that makes the work reliable.

---

## The short version

Do not give the agent everything and call that context.

Resolve the source.

Search the bounded corpus.

Retrieve the few useful excerpts.

Keep the citation.

Store durable decisions separately from conversation noise.

Keep large artifacts outside the prompt until they matter.

Respect private corpus boundaries.

Refresh changing sources.

Let the task determine how much context it earns.

Context Kernel gives agents a place to look before they guess.

---

## License

Context Kernel is licensed under the Apache License 2.0.

See [LICENSE](LICENSE).
