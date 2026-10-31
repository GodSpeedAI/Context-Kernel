#!/usr/bin/env bash
# Functional smoke test for an INSTALLED Context Kernel (no source checkout, no
# cargo). Uses only disposable files under a temporary directory that is removed
# on exit. Every MCP response is parsed as JSON by python3 -I; no raw string guessing.
#
# Covers: ingest a corpus, cited retrieval, memory store + recall after a process
# restart, canonical context bundle verification, and refusal without authority.
set -euo pipefail

BIN="${CK_BIN:-context-kernel}"
command -v "$BIN" >/dev/null || { echo "error: $BIN not on PATH (set CK_BIN)" >&2; exit 1; }
command -v python3 >/dev/null || { echo "error: python3 is required by this test harness only" >&2; exit 1; }

work="$(mktemp -d /tmp/context-kernel-smoke.XXXXXX)"
trap 'rm -rf "$work"' EXIT
db="$work/kernel.db"
data="$work/data"
corpus="$work/corpus/loop-docs"
mkdir -p "$corpus" "$data"
printf '# POL-SMOKE-001\nHigh-risk work needs a cited context packet before authority.\n' \
  > "$corpus/policy.md"

# Tiny JSON assertion helper, kept outside the work directory.
checker="$work/check.py"
cat > "$checker" <<'PY'
import json, sys
mode, path = sys.argv[1], sys.argv[2]
lines = [json.loads(l) for l in open(path) if l.strip()]
def result(i):
    r = lines[i]
    assert "error" not in r, f"MCP error: {r}"
    return r["result"]
if mode == "ingest":
    assert result(0)["job"]["status"] == "completed"
elif mode == "cited":
    hits = [h for h in result(1)["results"] if h["citation"]["uri"].endswith("policy.md")]
    assert hits, "no citation to policy.md"
    assert "cited context packet" in hits[0]["excerpt"], hits[0]
    assert hits[0]["citation"]["anchor"] == "pol-smoke-001", hits[0]["citation"]
    print("citation:", hits[0]["citation"]["uri"], "anchor", hits[0]["citation"]["anchor"])
elif mode == "recall":
    hits = [e for e in result(0)["results"] if "SMOKE" in e.get("text", "")]
    assert hits and hits[0]["kind"] == "decision", result(0)
    print("recalled after restart:", hits[0]["text"])
elif mode == "bundle":
    r = result(0)
    b = r["context_bundle"]
    assert r["access"]["result"] == "accessible", r["access"]
    assert b["extensions"]["cep.profile"]["profile_id"] == "godspeed.context_bundle"
    assert b["extensions"]["cep.profile"]["profile_version"] == "1.0.0"
    assert b["integrity"]["verification_status"] == "verified"
    assert b["completeness_status"] == "complete_for_context"
    assert "policy.md" in json.dumps(b)
    print("bundle:", b["extensions"]["cep.profile"]["profile_id"], b["completeness_status"], b["integrity"]["verification_status"])
elif mode == "refusal":
    r = lines[0]
    assert "error" in r and "result" not in r, f"expected refusal, got {r}"
    print("refused:", r["error"]["message"][:120])
PY

rpc() { python3 -I -c 'import json,sys; print(json.dumps({"jsonrpc":"2.0","id":int(sys.argv[1]),"method":"tools/call","params":{"name":sys.argv[2],"arguments":json.loads(sys.argv[3])}}))' "$1" "$2" "$3"; }

echo "== 1. ingest + cited retrieval"
{
  rpc 1 ingest_source "{\"source\":{\"source_id\":\"smoke_docs\",\"kind\":\"filesystem_tree\",\"path\":\"$work/corpus\"}}"
  rpc 2 query_reference '{"query":"cited context packet","top_k":3}'
} | "$BIN" serve --database "$db" --data-dir "$data" > "$work/out1.jsonl"
python3 -I "$checker" ingest "$work/out1.jsonl"
python3 -I "$checker" cited "$work/out1.jsonl"

echo "== 2. memory store, then recall in a NEW process (restart)"
rpc 3 store_context '{"session_id":"smoke-sess","kind":"decision","text":"SMOKE decision: keep the lexical index in SQLite FTS5","metadata":{}}' \
  | "$BIN" serve --database "$db" --data-dir "$data" > "$work/out2.jsonl"
rpc 4 recall_context '{"session_id":"smoke-sess","query":"lexical index SQLite","top_k":5}' \
  | "$BIN" serve --database "$db" --data-dir "$data" > "$work/out3.jsonl"
python3 -I "$checker" recall "$work/out3.jsonl"

echo "== 3. canonical context bundle (corpus configured via CK_CONTEXT_CORPUS_ROOT)"
hex_a="$(printf 'a%.0s' $(seq 64))"
hex_b="$(printf 'b%.0s' $(seq 64))"
ctx_args='{"schema_version":"v1","event_id":"11111111-2222-3333-4444-555555555555","source_agent":"swe-seed","event_type":"ContextRequired","occurred_at":"2026-08-25T00:00:00Z","payload":{"domain_model_hash":"'"$hex_a"'","world_ref":"world:smoke@sha256:'"$hex_b"'","namespace":"agentic_capability_loop","work_request_id":"wr-smoke","context_requirement_id":"cr-smoke","corpus_id":"loop-docs","query":"context packet","max_results":5}}'
rpc 5 context_required "$ctx_args" \
  | CK_CONTEXT_CORPUS_ROOT="$work/corpus" "$BIN" serve --database "$db" --data-dir "$data" > "$work/out4.jsonl"
python3 -I "$checker" bundle "$work/out4.jsonl"

echo "== 4. refusal without authority (wrong source_agent must be refused)"
bad_args="${ctx_args/swe-seed/someone-else}"
rpc 6 context_required "$bad_args" \
  | CK_CONTEXT_CORPUS_ROOT="$work/corpus" "$BIN" serve --database "$db" --data-dir "$data" > "$work/out5.jsonl" || true
python3 -I "$checker" refusal "$work/out5.jsonl"

echo "== 5. database integrity after restarts"
"$BIN" doctor --database "$db"
"$BIN" rebuild-index --database "$db"
test -s "$db" && echo "database persisted: $(wc -c < "$db") bytes"

echo "PASS: functional smoke test completed against the installed binary."
