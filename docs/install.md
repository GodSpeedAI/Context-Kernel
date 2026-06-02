# Install

Build a single release binary:

```bash
cargo build --release -p ck-bin
```

The executable is:

```bash
target/release/context-kernal
```

Install it into a local bin directory:

```bash
install -m 0755 target/release/context-kernal ~/.local/bin/context-kernal
```

Initialize storage:

```bash
context-kernal migrate --database ./kernel.db
```

Serve over stdio:

```bash
context-kernal serve --database ./kernel.db --data-dir ./data
```

Serve with SSE:

```bash
context-kernal serve --database ./kernel.db --data-dir ./data --sse 127.0.0.1:8765
```

Useful operational commands:

```bash
context-kernal doctor --database ./kernel.db
context-kernal export-schema
```

Milestone 10 hardening config keys are `[reload]`, `[retention]`, `[authorization]`, and `[observability]`. Config reload is checked during the stdio serve loop when `[reload].enabled = true` and `--config <path>` is supplied.
