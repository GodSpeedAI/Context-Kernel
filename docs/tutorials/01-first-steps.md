# Tutorial: First Steps with Context Kernel

This tutorial guides you through building Context Kernel from source, initializing the SQLite database, verifying system health with the `doctor` command, and interacting with the service over standard input using the Model Context Protocol (MCP).

---

## Prerequisites
* **Rust Toolchain**: Rust 1.75+ (2021 edition).
* **Terminal**: Bash, Zsh, or PowerShell.

---

## 1. Build the Release Binary

Compile the `context-kernal` binary from the workspace:

```bash
cargo build --release -p context-kernel
```

The compiled executable will be located at:
* Linux / macOS: `target/release/context-kernal`
* Windows: `target\release\context-kernal.exe`

Verify that the binary runs:

```bash
./target/release/context-kernal --help
```

Expected output:
```text
Context Kernel MCP service

Usage: context-kernal <COMMAND>

Commands:
  serve          
  migrate        
  doctor         
  export-schema  
  rebuild-index  
  help           Print this message or the help of the given subcommand(s)
```

---

## 2. Initialize the Database

Context Kernel uses an embedded SQLite database. Run the `migrate` command to create the database schema:

```bash
mkdir -p data
./target/release/context-kernal migrate --database ./data/kernel.db
```

Expected output:
```text
migrated ./data/kernel.db
```

---

## 3. Verify Health with `doctor`

Inspect the storage status and confirm that the SQLite FTS5 extension is active:

```bash
./target/release/context-kernal doctor --database ./data/kernel.db
```

Expected output:
```text
storage=sqlite path=./data/kernel.db fts5_enabled=true
```

---

## 4. Run the Service & Make Your First MCP Call

Context Kernel serves MCP over stdio by default. Start the server in local mode:

```bash
./target/release/context-kernal serve \
  --database ./data/kernel.db \
  --data-dir ./data
```

The process will block, waiting for JSON-RPC 2.0 lines on standard input.

### Check Service Status (`get_status`)
Paste the following JSON line into the terminal and press **Enter**:

```json
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_status","arguments":{}}}
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "service": {
      "name": "context-kernel",
      "version": "0.1.0",
      "healthy": true
    },
    "storage": {
      "kind": "sqlite",
      "path": "./data/kernel.db",
      "fts5_enabled": true
    },
    "sources": { "total": 0, "enabled": 0, "degraded": 0 },
    "sessions": { "active": 0 },
    "jobs": { "queued": 0, "running": 0 },
    "retrieval": { "default_mode": "hybrid", "semantic_enabled": false }
  }
}
```

---

## 5. Store and Recall Session Memory

### Store a Decision
Paste the following `store_context` tool call to record an architectural decision in session `"sess-01"`:

```json
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"store_context","arguments":{"session_id":"sess-01","kind":"decision","text":"We chose SQLite with WAL mode for local concurrency"}}}
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "event": {
      "event_id": "evt_...",
      "session_id": "sess-01",
      "kind": "decision",
      "stored_as": "inline",
      "artifact_id": null
    }
  }
}
```

### Recall the Decision
Now query session memory to recall the decision you just stored:

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"recall_context","arguments":{"session_id":"sess-01","query":"concurrency"}}}
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "events": [
      {
        "event_id": "evt_...",
        "session_id": "sess-01",
        "kind": "decision",
        "text": "We chose SQLite with WAL mode for local concurrency",
        "pinned": false
      }
    ],
    "total_hits": 1
  }
}
```

Press `Ctrl+C` or `Ctrl+D` to terminate the server.

---

## Next Steps
Congratulations! You have verified that Context Kernel builds, migrates, and serves MCP over stdio.
* Proceed to [Tutorial 2: Local Documentation Indexing](02-local-documentation-indexing.md) to index Markdown files and query references with citations.
* Read the [System Mental Model](../mental-model.md) to understand state ownership and boundaries.
