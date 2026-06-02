# Context Kernel Operations Guide

## Commands

The `context-kernal` binary supports these commands:

### `serve`
Starts the MCP service. Supports stdio (default) and SSE transports.

```bash
# Local stdio mode
context-kernal serve --database ./data/kernel.db --data-dir ./data

# With SSE remote access
context-kernal serve --database ./data/kernel.db --data-dir ./data --sse 127.0.0.1:8765

# With config file
context-kernal serve --config context-kernal.toml
```

### `migrate`
Runs database migrations.

```bash
context-kernal migrate --database ./data/kernel.db
```

### `doctor`
Checks database health and configuration.

```bash
context-kernal doctor --database ./data/kernel.db
```

Output includes storage kind, path, and FTS5 status.

### `export-schema`
Prints the embedded SQLite schema SQL to stdout.

```bash
context-kernal export-schema
```

## Retention GC

When `[retention].enabled = true`, the service runs garbage collection on startup:

- **Session events**: Deletes unpinned events older than `session_event_days`
- **Inactive sessions**: Deletes closed/compacted sessions older than `inactive_session_days`
- **Refresh jobs**: Deletes finished jobs older than `refresh_job_days`
- **Artifacts**: Deletes metadata rows and payload files older than `artifact_days`
- **Disabled sources**: Removes disabled sources older than `disabled_source_days`

Pinned session events are always preserved.

## Config Reload

When `[reload].enabled = true`, the stdio serve loop checks the config file before each request. If the file has changed, the service rebuilds:

- Storage connection
- Application services
- MCP service instance
- SSE state (service and auth policy)

## Periodic Refresh

When `[scheduler].enable_periodic_refresh = true`, a background thread checks for stale sources every `refresh_interval_secs` and re-ingests them.

## Backup and Restore

1. Stop the service
2. Copy the SQLite database file
3. Copy the artifacts directory (if artifact storage is used)
4. Restart the service

## Telemetry

Structured tracing is emitted to stderr. Configure `[observability].log_format = "json"` for machine-readable output. The `otel_endpoint` config field is reserved for future OpenTelemetry export.
