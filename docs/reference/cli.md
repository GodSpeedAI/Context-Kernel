# Reference: Command-Line Interface (CLI)

The `context-kernal` executable is the single binary entry point for Context Kernel. This document provides an exhaustive reference for all subcommands, arguments, flags, and environment variables.

---

## Synopsis

```bash
context-kernal <COMMAND> [OPTIONS]
```

---

## Global Flags & Environment Variables

| Flag | Environment Variable | Default | Description |
| :--- | :--- | :--- | :--- |
| `-h`, `--help` | — | — | Prints help information |
| `-V`, `--version` | — | — | Prints version information |
| — | `CK_CONTEXT_CORPUS_ROOT` | (empty) | Path to the root directory for filesystem corpus citations in `context_required` |
| — | `CK_LEARNING_PROPOSALS_LEDGER`| (empty) | Path to GodSpeed learning proposals JSON ledger file for `context_required` |
| — | `RUST_LOG` | `info` | Filter directive for tracing output (e.g. `ck_app=debug,info`) |

---

## Subcommands

### 1. `serve`
Starts the MCP service. Operates over standard input/output (stdio JSON-RPC) and optionally exposes an HTTP SSE server.

```bash
context-kernal serve [OPTIONS]
```

#### Options
* `--config <PATH>`: Path to a TOML configuration file (e.g. `config/context-kernal.toml`).
* `--database <PATH>`: Path to the SQLite database file (overrides `storage.database_path` from configuration; default: `kernel.db`).
* `--data-dir <PATH>`: Directory used for content-addressed artifact payload storage (default: parent directory of the database file or `.`).
* `--sse <HOST:PORT>`: Enables the HTTP SSE MCP transport and binds to the specified address (e.g. `127.0.0.1:8765`). Overrides `mcp.sse.bind` from configuration.

#### Examples
```bash
# Serve locally over stdio
context-kernal serve --database ./data/kernel.db --data-dir ./data

# Serve over stdio and SSE with custom config
context-kernal serve --config ./config/context-kernal.toml --sse 127.0.0.1:8765
```

---

### 2. `migrate`
Initializes or updates the SQLite database schema by executing the embedded migration SQL (`SCHEMA_V1`).

```bash
context-kernal migrate [OPTIONS]
```

#### Options
* `--config <PATH>`: Path to configuration file.
* `--database <PATH>`: Path to SQLite database file.

#### Behavior
* Idempotent: safe to run repeatedly against an existing database.
* Applies all tables, foreign keys, indexes, and virtual tables (`chunks_fts`, etc.).
* Outputs: `migrated <path>` on success.

---

### 3. `doctor`
Inspects database connectivity, verifies that foreign keys are enabled, and tests whether the SQLite FTS5 extension is active.

```bash
context-kernal doctor [OPTIONS]
```

#### Options
* `--config <PATH>`: Path to configuration file.
* `--database <PATH>`: Path to SQLite database file.

#### Output Format
```text
storage=sqlite path=<PATH> fts5_enabled=<true|false>
```

#### Exit Codes
* `0`: Database is healthy and accessible.
* `1`: Database file is missing, corrupt, or permissions are insufficient.

---

### 4. `export-schema`
Prints the exact SQL schema embedded in the binary to standard output.

```bash
context-kernal export-schema
```

#### Options
* None.

#### Example
```bash
context-kernal export-schema > schema.sql
```

---

### 5. `rebuild-index`
Clears all FTS5 virtual tables (`chunks_fts`, `session_events_fts`, `library_aliases_fts`) and repopulates them from canonical relational tables.

```bash
context-kernal rebuild-index [OPTIONS]
```

#### Options
* `--config <PATH>`: Path to configuration file.
* `--database <PATH>`: Path to SQLite database file.

#### Output Format
```text
rebuilt FTS index: <chunks> chunks, <events> session_events, <aliases> aliases
```
