"""LEGACY integration adapter for the agentic_capability_loop domain.

DEPRECATED: the production implementation is
``crates/ck-mcp`` (the GodSpeed path through the real CK retrieval
pipeline). The canonical cross-system context artifact is the CEP-0008
``godspeed.context_bundle`` envelope; this module predates it and is kept
only as a bounded dual-read compatibility shim for old integration tests.

Context-Kernel emits here:
  - ContextPacketCreated (legacy wire, deprecated)
  - ContextAccessEvaluated (CONTEXT ACCESS: accessible/inaccessible/
    requires_scope - never action authority; SEA-Forge owns that)

Context-Kernel consumes:
  - ContextRequired (from SWE_SEED)
"""
from __future__ import annotations

import datetime
import hashlib
import json as _json
import uuid
import warnings
from pathlib import Path as _Path
from typing import Any

NAMESPACE = "agentic_capability_loop"


def _load_hash() -> str:
    _manifest = (
        _Path(__file__).resolve().parents[3]
        / "SEA"
        / "docs"
        / "specs"
        / "domains"
        / "agentic_capability_loop"
        / "agentic_capability_loop.manifest.json"
    )
    if _manifest.exists():
        _m = _json.loads(_manifest.read_text(encoding="utf-8"))
        _h = (_m.get("meta") or {}).get("sea_file_hash")
        if _h:
            return _h
    warnings.warn("SEA manifest not found; using fallback hash", stacklevel=2)
    return hashlib.sha256(NAMESPACE.encode("utf-8")).hexdigest()


DOMAIN_MODEL_HASH: str = _load_hash()

# Policy code from agentic_capability_loop.sea
POL_PRIVATE_CORPUS_SCOPE = "POL-ACL-002"


def _now_iso() -> str:
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def _event(event_type: str, payload: dict[str, Any]) -> dict[str, Any]:
    """Build a canonical event envelope."""
    payload = {"domain_model_hash": DOMAIN_MODEL_HASH, **payload}
    return {
        "event_id": str(uuid.uuid4()),
        "event_type": event_type,
        "namespace": NAMESPACE,
        "occurred_at": _now_iso(),
        "payload": payload,
    }


def handle_context_required(
    event: dict[str, Any],
    *,
    available_documents: list[dict[str, Any]] | None = None,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Handle ContextRequired event and emit ContextPacketCreated + ContextAccessEvaluated.
    
    Checks POL-ACL-002 (PrivateCorpusRequiresExplicitScope) and either
    grants or denies corpus access, then builds a ContextPacket.
    
    Args:
        event: ContextRequired canonical event envelope.
        available_documents: Optional list of document dicts for testing.
            Each doc should have: {source, content, fetched_at}.
    
    Returns:
        Tuple of (ContextPacketCreated event, ContextAccessEvaluated event).
    
    Raises:
        ValueError: If event type does not match.
    """
    if event.get("event_type") != "ContextRequired":
        raise ValueError(
            f"Expected ContextRequired event, got '{event.get('event_type')}'"
        )
    payload = event.get("payload", {})
    corpus_id = payload.get("corpus_id", "")

    # Route learning-proposals corpus to JSONL ledger reader
    if corpus_id.startswith("learning-proposals"):
        return _handle_learning_proposals_corpus(event, payload)

    context_requirement_id = payload.get("context_requirement_id", "")
    work_request_id = payload.get("work_request_id", "")
    is_private = payload.get("is_private", corpus_id.startswith("private:"))
    scope_claim = payload.get("scope_claim")
    query = payload.get("query", "")
    
    # POL-ACL-002 is DATA ACCESS policy (private corpus requires explicit
    # scope), not action authority. The result vocabulary is context access:
    # accessible / inaccessible / requires_scope.
    authorized = True
    auth_result = "accessible"
    auth_reason = "Corpus is publicly accessible"
    
    if is_private:
        if not scope_claim:
            authorized = False
            auth_result = "requires_scope"
            auth_reason = (
                f"Private corpus '{corpus_id}' requires an explicit scope claim; "
                "none provided (POL-ACL-002)"
            )
        else:
            auth_result = "accessible"
            auth_reason = f"Private corpus accessible with scope '{scope_claim}'"
    
    # Build citations from available documents (or generate stub citations for testing)
    docs = available_documents or []
    if not docs and authorized:
        # Stub citation for integration testing
        docs = [
            {
                "source": f"{corpus_id}/doc/1",
                "content": f"Relevant documentation for query: {query or corpus_id}",
                "fetched_at": _now_iso(),
            }
        ]
    
    citations = [
        {
            "citation_id": str(uuid.uuid4()),
            "source": doc.get("source", "unknown"),
            "content": doc.get("content", ""),
            "fetched_at": doc.get("fetched_at", _now_iso()),
        }
        for doc in docs
        if authorized
    ]
    
    context_packet_created = _event(
        "ContextPacketCreated",
        {
            "context_packet_id": str(uuid.uuid4()),
            "context_requirement_id": context_requirement_id,
            "work_request_id": work_request_id,
            "corpus_id": corpus_id,
            "citations": citations,
            "accessible": authorized,
        },
    )
    
    context_access = _event(
        "ContextAccessEvaluated",
        {
            "context_requirement_id": context_requirement_id,
            "work_request_id": work_request_id,
            "corpus_id": corpus_id,
            "result": auth_result,
            "reason": auth_reason,
            "policy_code": POL_PRIVATE_CORPUS_SCOPE,
            "scope_claim": scope_claim,
            "evaluated_at": _now_iso(),
        },
    )
    
    return context_packet_created, context_access


def emit_context_packet_created(
    context_requirement_id: str,
    work_request_id: str,
    corpus_id: str,
    citations: list[dict[str, Any]],
    *,
    authorized: bool = True,
) -> dict[str, Any]:
    """Directly emit a ContextPacketCreated event with known citations.
    
    Use this when you have already retrieved documents and want to wrap them
    in a canonical event envelope.
    
    Args:
        context_requirement_id: ID of the originating context requirement.
        work_request_id: ID of the originating work request.
        corpus_id: ID of the knowledge corpus that was queried.
        citations: List of citation dicts (source, content, fetched_at).
        authorized: Whether corpus access was authorized.
    
    Returns:
        Canonical ContextPacketCreated event envelope.
    """
    return _event(
        "ContextPacketCreated",
        {
            "context_packet_id": str(uuid.uuid4()),
            "context_requirement_id": context_requirement_id,
            "work_request_id": work_request_id,
            "corpus_id": corpus_id,
            "citations": citations,
            "authorized": authorized,
        },
    )


def _handle_learning_proposals_corpus(
    event: dict[str, Any], payload: dict[str, Any]
) -> tuple[dict[str, Any], dict[str, Any]]:
    import json
    from pathlib import Path

    ledger_path_str = payload.get("learning_proposals_ledger_path")
    citations: list[dict[str, Any]] = []
    query = (payload.get("query") or "").lower()

    if ledger_path_str:
        ledger_path = Path(ledger_path_str)
        if ledger_path.exists():
            for line in ledger_path.read_text(encoding="utf-8").splitlines():
                line = line.strip()
                if not line:
                    continue
                try:
                    proposal_event = json.loads(line)
                except (json.JSONDecodeError, KeyError):
                    continue
                payload_data = proposal_event.get("payload", {})
                proposal_text = str(payload_data.get("proposal", ""))
                if not query or query in proposal_text.lower():
                    citations.append(
                        {
                            "citation_id": f"cit-{proposal_event.get('event_id', 'unknown')}",
                            "source": f"learning-proposals://{payload_data.get('learning_proposal_id', 'unknown')}",
                            "content": proposal_text,
                            "fetched_at": _now_iso(),
                        }
                    )

    context_packet = _event(
        "ContextPacketCreated",
        {
            "context_packet_id": str(uuid.uuid4()),
            "context_requirement_id": payload.get("context_requirement_id", ""),
            "work_request_id": payload.get("work_request_id", ""),
            "corpus_id": payload.get("corpus_id", ""),
            "citations": citations,
            "authorized": True,
        },
    )
    context_authorized = _event(
        "ContextAccessEvaluated",
        {
            "context_requirement_id": payload.get("context_requirement_id", ""),
            "work_request_id": payload.get("work_request_id", ""),
            "corpus_id": payload.get("corpus_id", ""),
            "result": "accessible",
            "reason": "learning-proposals corpus is always public",
            "policy_code": POL_PRIVATE_CORPUS_SCOPE,
            "scope_claim": None,
            "evaluated_at": _now_iso(),
        },
    )
    return context_packet, context_authorized
