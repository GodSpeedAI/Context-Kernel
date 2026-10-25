# How-To: Manage Database, Migrations & Index Rebuilding

This operational guide explains how to initialize, diagnose, inspect, and repair Context Kernel's SQLite database and FTS5 search indexes.

---

## 1. Initialize or Upgrade the Database (`migrate`)

Context Kernel embeds its schema migrations inside the binary. To apply all tables, triggers, and virtual tables to a new or existing database file:

```bash
./target/release/context-kernal migrate --database ./data/kernel.db
```

Expected output:
```text
migrated ./data/kernel.db
```

* **Idempotency**: Running `migrate` multiple times is completely safe. The DDL uses `CREATE TABLE IF NOT EXISTS` and `INSERT OR IGNORE INTO schema_migrations`.
* **Foreign Keys**: Migrations enable SQLite foreign key constraints (`PRAGMA foreign_keys = ON;`).
* **Version 2 (stable document IDs)**: a v1 database that holds documents with a NULL `external_id`
  or `uri` cannot be migrated. `migrate` fails with "Re-ingest required". Move the database file
  aside, run `migrate` on a new file, and re-ingest every source. Databases without such documents
  are re-keyed automatically in one transaction.

---

## 2. Check Database Health (`doctor`)

To verify that the database is accessible, writable, and that the SQLite FTS5 extension is properly installed:

```bash
./target/release/context-kernal doctor --database ./data/kernel.db
```

Expected output:
```text
storage=sqlite path=./data/kernel.db fts5_enabled=true
```

If `fts5_enabled=false` or if the command fails, check:
1. File permissions on `./data/kernel.db`.
2. That SQLite was compiled with FTS5 enabled (the workspace builds with `rusqlite` bundled source which includes FTS5 by default).

---

## 3. Export Embedded Schema DDL (`export-schema`)

To inspect the raw SQL DDL embedded in the binary (useful for schema audits, DBA reviews, or generating SQL migration scripts):

```bash
./target/release/context-kernal export-schema
```

This prints `SCHEMA_V1` directly to standard output, including all table definitions, indexes, and virtual tables. You can redirect this to a `.sql` file:

```bash
./target/release/context-kernal export-schema > schema.sql
```

---

## 4. Rebuild Search Indexes (`rebuild-index`)

### When to Use
Use `rebuild-index` if:
* The service experienced an ungraceful shutdown (SIGKILL / power outage) during a large document ingestion.
* Search queries return zero results for documents you know exist in the database.
* The FTS5 index tables (`chunks_fts`, `session_events_fts`, `library_aliases_fts`) have become corrupted or desynchronized from the canonical relational tables.

### Execution
Run the `rebuild-index` command:

```bash
./target/release/context-kernal rebuild-index --database ./data/kernel.db
```

Expected output:
```text
rebuilt FTS index: 142 chunks, 18 session_events, 6 aliases
```

### What This Does Internally
1. Clears existing FTS virtual tables (`DELETE FROM chunks_fts`, etc.).
2. Executes atomic `INSERT ... SELECT` queries in SQLite, repopulating all virtual tables directly from the canonical `chunks`, `documents`, `session_events`, and `library_aliases` tables.

---

## 5. Backup and Restore Procedures

Because Context Kernel uses SQLite WAL mode:

### Creating a Hot Backup
1. Use SQLite's backup API or the command-line `.backup` command to create a consistent snapshot without shutting down the server:
   ```bash
   sqlite3 ./data/kernel.db ".backup './data/kernel.backup.db'"
   ```
2. If artifact storage is used, copy the artifact payload directory:
   ```bash
   cp -r ./data/artifacts ./data/artifacts.backup
   ```

### Restoring from Backup
1. Stop the `context-kernal` service.
2. Replace `kernel.db` with your backup copy.
3. Replace the `artifacts` directory with your backup copy.
4. Run `./target/release/context-kernal doctor --database ./data/kernel.db` to confirm integrity.
5. Restart the service.
