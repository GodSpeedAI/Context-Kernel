# Tutorial: Local Documentation Indexing

In this tutorial, you will configure a local documentation corpus, ingest a directory of Markdown files into Context Kernel, resolve library identities, and execute reference queries to receive token-efficient excerpts with citations.

---

## Prerequisites
* Completed [Tutorial 1: First Steps](01-first-steps.md).
* A built binary at `./target/release/context-kernal`.

---

## 1. Prepare Sample Documentation

Create a sample documentation folder containing two Markdown files:

```bash
mkdir -p ./sample_docs/auth
mkdir -p ./sample_docs/storage

cat <<'EOF' > ./sample_docs/auth/tokens.md
# Bearer Token Authentication

Context Kernel supports bearer token authentication over the HTTP SSE transport.
Tokens are validated using constant-time comparison to prevent timing side-channel attacks.

## Configuring Tokens

Set the `tokens` array in the `[auth]` section of your `context-kernal.toml` file:

```toml
[auth]
tokens = ["secret-bearer-token-xyz"]
```
EOF

cat <<'EOF' > ./sample_docs/storage/sqlite.md
# SQLite Persistence Engine

Context Kernel uses SQLite as its primary relational store.
All migrations are embedded within the binary in `SCHEMA_V1`.

## Write-Ahead Logging (WAL)

The database connection enables WAL mode by executing `PRAGMA journal_mode = WAL;`.
This allows multiple readers to query full-text search indexes concurrently without blocking writers.
EOF
```

---

## 2. Start the Service

Start Context Kernel pointing to your database and data directory:

```bash
./target/release/context-kernal serve \
  --database ./data/kernel.db \
  --data-dir ./data
```

The process will wait for input on stdin.

---

## 3. Ingest the Documentation Directory

Send the `ingest_source` tool call. Pass the absolute or relative path to `./sample_docs`:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"ingest_source","arguments":{"corpus_id":"local_docs","kind":"filesystem","path":"./sample_docs","display_name":"Sample Project Docs"}}}
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "source_id": "src_...",
    "corpus_id": "local_docs",
    "documents_indexed": 2,
    "chunks_indexed": 4,
    "status": "active"
  }
}
```

Behind the scenes:
1. `FilesystemSourceAdapter` discovered `tokens.md` and `sqlite.md`.
2. `MarkdownNormalizer` parsed headings and titles.
3. `HeadingChunker` created chunks along section headings.
4. `ck-storage-sqlite` inserted the records into `documents` and `chunks`.
5. `ck-search-fts` populated the `chunks_fts` virtual table.

---

## 4. Query Documentation with Citations

Now ask Context Kernel for information on WAL mode using `query_reference`:

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query_reference","arguments":{"query":"WAL mode concurrent readers","intent":"query_docs","top_k":3}}}
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "query_id": "qry_...",
    "mode": "lexical",
    "matches": [
      {
        "chunk_id": "chk_...",
        "document_id": "doc_...",
        "title": "SQLite Persistence Engine",
        "heading": "Write-Ahead Logging (WAL)",
        "uri": "./sample_docs/storage/sqlite.md",
        "text": "The database connection enables WAL mode by executing `PRAGMA journal_mode = WAL;`.\nThis allows multiple readers to query full-text search indexes concurrently without blocking writers.",
        "score": 0.78,
        "score_breakdown": {
          "fts": 0.92,
          "freshness": 1.0,
          "trust": 0.0
        }
      }
    ],
    "total_hits": 1
  }
}
```

Notice the properties of this output:
* **Token-Efficient**: Returns only the relevant section, not the entire file.
* **Citation-Bearing**: Contains exact URI, title, and section heading.
* **Inspectable Score Breakdown**: Shows exact weights assigned to FTS lexical matching and freshness.

---

## 5. Refreshing Documentation

Update one of the files:

```bash
echo -e "\n## Checkpoints\nWAL checkpoints occur automatically under SQLite normal operations." >> ./sample_docs/storage/sqlite.md
```

Trigger `refresh_source` using the `source_id` returned during ingestion:

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"refresh_source","arguments":{"source_id":"<YOUR_SOURCE_ID>"}}}
```

Context Kernel re-indexes the source, purges old document chunks, and indexes the updated content with zero duplicate chunk accumulation.

---

## Summary
You have successfully:
* Ingested a local Markdown documentation tree.
* Observed heading-based chunking and FTS5 indexing.
* Executed an intent-weighted reference query that returned structured citations.
* Verified non-accumulating refresh behavior.
