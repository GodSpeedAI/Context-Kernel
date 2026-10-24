# How-To: Configure Semantic Retrieval & Embeddings

This guide explains how to configure Context Kernel to enable hybrid lexical and semantic search using OpenAI-compatible embedding models and SQLite brute-force vector search.

---

## Goal
Enhance documentation search so that queries can find conceptually relevant documentation chunks even when the exact keyword tokens do not match the source text.

---

## 1. Configure the `[embeddings]` Section

Add the `[embeddings]` section to your `context-kernal.toml` file:

```toml
[storage]
database_path = "./data/kernel.db"

[retrieval]
default_mode = "hybrid"

[embeddings]
enabled = true
provider = "openai"
api_base = "https://api.openai.com/v1"
api_key = "$OPENAI_API_KEY"
model = "text-embedding-3-small"
dimensions = 1536
```

### Configuration Keys

| Key | Type | Description |
| :--- | :--- | :--- |
| `enabled` | boolean | Set to `true` to activate semantic embeddings |
| `provider` | string | Provider kind: `"openai"` or `"none"` |
| `api_base` | string | API base URL (e.g. `https://api.openai.com/v1` or local proxy) |
| `api_key` | string | API key (use `$ENV_VAR` or `${ENV_VAR}` syntax) |
| `model` | string | Embedding model name (default: `text-embedding-3-small`) |
| `dimensions` | integer | Vector dimensions (e.g. 1536 for small, 3072 for large) |

---

## 2. Export Your API Key & Launch the Server

Set your OpenAI API key in your shell:

```bash
export OPENAI_API_KEY="sk-proj-..."

./target/release/context-kernal serve --config context-kernal.toml
```

Expected log output:
```text
INFO ck_bin: semantic retrieval configured provider="openai" model="text-embedding-3-small" dimensions=1536
```

---

## 3. Verify Semantic Availability via `get_status`

Send a `get_status` tool call:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}
```

Verify that `retrieval.semantic_enabled` is reported as `true`:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "retrieval": {
      "default_mode": "hybrid",
      "semantic_enabled": true
    }
  }
}
```

---

## 4. How Semantic Ranking Operates

When semantic retrieval is active:
1. When documents are ingested, `ck-embeddings` generates vectors for each chunk and stores them in SQLite's `chunk_embeddings` table.
2. When an agent queries documentation via `query_reference`, `ck-embeddings` generates a query vector.
3. `ck-vector` calculates cosine similarities across all candidate vectors in SQLite.
4. `ck-ranker` applies the `QueryDocs` or `SearchRefs` weight profile:
   - `QueryDocs`: 60% FTS Lexical, 15% Trust, 15% Freshness, 10% Semantic.
   - `SearchRefs`: 50% FTS Lexical, 10% Trust, 20% Freshness, 20% Semantic.
5. If the OpenAI API experiences a network timeout or temporary outage, Context Kernel logs a warning and falls back gracefully to pure lexical ranking without failing the client query.
