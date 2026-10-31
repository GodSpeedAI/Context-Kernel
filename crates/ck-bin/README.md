# ck-bin

## Overview

`ck-bin` is the composition/bootstrap crate of the Context Kernel workspace: it contains no library surface and declares the `context-kernal` binary target (`[[bin]]` in `Cargo.toml`, path `src/main.rs`; the clap display name is `context-kernel`). Its only production source file wires concrete adapters from the workspace crates (SQLite storage, FTS search, file artifact store, filesystem and learning-proposals source adapters, embeddings, bearer auth, stdio and SSE transports) into `ck_app::AppServices` and `ck_mcp::McpService`, and exposes the CLI subcommands `serve`, `migrate`, `doctor`, `export-schema`, and `rebuild-index`. Integration tests live under `tests/` and are exercised by the crate verification command.

## Subdirectories & Boundaries

- [src](./src/README.md) - Binary entry point and composition root: clap CLI surface (`Serve`, `Migrate`, `Doctor`, `ExportSchema`, `RebuildIndex`), adapter wiring into `ck_app::AppServices`, stdio/SSE serving, config reload, retention GC, and periodic refresh.

## Verification

- Run: `cargo test -p ck-bin`
