# agentic_capability_loop

## Overview
Python package providing a Python-level adapter for integration tests against Context-Kernel's `agentic_capability_loop` domain. `__init__.py` is a docstring-only package marker; all behavior lives in `adapters.py`, whose module docstring marks it LEGACY and DEPRECATED: the production implementation is the Rust `crates/ck-mcp` (the GodSpeed path through the real CK retrieval pipeline), and this module is kept only as a bounded dual-read compatibility shim for old integration tests. On the wire it consumes `ContextRequired` (from SWE_SEED) and emits `ContextPacketCreated` (legacy wire, deprecated) plus `ContextAccessEvaluated`; the canonical cross-system context artifact is the CEP-0008 `godspeed.context_bundle` envelope, which postdates this module. Event construction is pure in-memory; the only I/O is reading the SEA domain manifest at import time and an optional JSONL ledger at request time.

## Interface
- [adapters.py:NAMESPACE] - Module constant `"agentic_capability_loop"`; the `namespace` field of every emitted event envelope.
- [adapters.py:DOMAIN_MODEL_HASH] - Module constant resolved at import time by `_load_hash()` from the SEA domain manifest's `meta.sea_file_hash`; injected into every event payload.
- [adapters.py:POL_PRIVATE_CORPUS_SCOPE] - Policy-code constant `"POL-ACL-002"` (PrivateCorpusRequiresExplicitScope, from `agentic_capability_loop.sea`).
- [adapters.py:handle_context_required] - Public handler for a `ContextRequired` event envelope (optional `available_documents` list for testing); returns a `(ContextPacketCreated, ContextAccessEvaluated)` tuple and stores the access decision under the payload key `accessible`. Raises `ValueError` on any other `event_type`. Routes `corpus_id` values starting with `learning-proposals` to the JSONL ledger reader.
- [adapters.py:emit_context_packet_created] - Public helper that wraps already-retrieved citations in a canonical `ContextPacketCreated` envelope; the `authorized` keyword (default `True`) is stored under the payload key `authorized` (note: a different key than the `accessible` key used by `handle_context_required`).
- [adapters.py:_load_hash] - Private: reads the SEA domain manifest at `_Path(__file__).resolve().parents[3] / "SEA" / "docs" / "specs" / "domains" / "agentic_capability_loop" / "agentic_capability_loop.manifest.json"`; from this module's location that resolves to a `SEA` tree that is a sibling of the repository root, not a directory inside it.
- [adapters.py:_event] - Private canonical envelope builder: `event_id` (uuid4), `event_type`, `namespace`, `occurred_at` (UTC ISO-8601), and a payload with `domain_model_hash` merged in.
- [adapters.py:_handle_learning_proposals_corpus] - Private ledger reader: parses the JSONL file named by `payload["learning_proposals_ledger_path"]`, skips malformed lines (`json.JSONDecodeError`/`KeyError`), filters proposals by case-insensitive query substring, and builds citations with `cit-<event_id>` ids and `learning-proposals://<learning_proposal_id>` sources.

## Invariants
- [adapters.py:module docstring] - DEPRECATED/legacy compatibility shim only: production context retrieval is `crates/ck-mcp`; the canonical cross-system artifact is the CEP-0008 `godspeed.context_bundle` envelope, and this module predates it.
- [adapters.py:module docstring] - `ContextAccessEvaluated` results are context access decisions (accessible / inaccessible / requires_scope vocabulary), never action authority; SEA-Forge owns action authority.
- [adapters.py:handle_context_required] - Raises `ValueError` unless the input event's `event_type` is exactly `"ContextRequired"`; `ContextRequired` (from SWE_SEED) is the only consumed event type.
- [adapters.py:handle_context_required] - POL-ACL-002: a private corpus (`is_private`, defaulting to `corpus_id.startswith("private:")`) without a `scope_claim` yields result `requires_scope` with access denied; with a scope claim it yields `accessible`. Citations are emitted only when access is authorized, and when authorized with no documents supplied a single stub citation is generated for integration testing.
- [adapters.py:_handle_learning_proposals_corpus] - The `learning-proposals` corpus is always treated as public: result `accessible`, `authorized: True`, `scope_claim: None`.
- [adapters.py:_event] - Every emitted envelope carries `namespace: "agentic_capability_loop"` and a payload-level `domain_model_hash`.
- [adapters.py:_load_hash] - When the SEA manifest or its `meta.sea_file_hash` is missing, a `warnings.warn("SEA manifest not found; using fallback hash")` is emitted and the hash falls back to `sha256(NAMESPACE)`; import never fails on a missing manifest.

## Side Effects
- [adapters.py:_load_hash] - Filesystem read at import time: the SEA domain manifest JSON under a `SEA` tree located three parent directories above this module (a sibling of the repository root in the current checkout layout).
- [adapters.py:_load_hash] - Emits a Python `warnings.warn` diagnostic when the SEA manifest or its hash is absent.
- [adapters.py:_handle_learning_proposals_corpus] - Filesystem read of the JSONL ledger at `payload["learning_proposals_ledger_path"]`, only when that payload key is provided.
- Otherwise event envelopes are built purely in memory: no network calls, no subprocesses, no filesystem writes.
