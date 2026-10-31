# ck-config

## Overview

`ck-config` is the configuration crate of the Context Kernel workspace: it defines the typed TOML configuration tree (`KernelConfig` and its sub-configs), canonical defaults, and the loader that reads a config file, expands `${VAR}` environment references, and parses TOML. Its runtime dependency set is limited to `anyhow`, `serde`, and `toml` ([Cargo.toml]); `tempfile` is a dev-dependency used only by tests.

## Subdirectories & Boundaries

- src: ./src/README.md - the entire crate surface: `KernelConfig` tree with serde defaults, `default_config()`, and `load_config()` with `${VAR}` environment expansion.

## Verification

- Run: `cargo test -p ck-config`
