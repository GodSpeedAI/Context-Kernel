# ck-artifacts

## Overview

`ck-artifacts` is a small adapter crate that persists artifact payloads (binary content plus JSON metadata) on the local filesystem behind the `ck-core` `ArtifactPort`. Artifact IDs are validated to a safe character set before any filesystem path is constructed, blocking path traversal. The store is wired into the `ck-bin` binary at bootstrap.

## Subdirectories & Boundaries

- `src`: [`./src/README.md`](./src/README.md) - Component contract for `FileArtifactStore` (the filesystem `ArtifactPort` implementation).

## Verification

- Run: `cargo test -p ck-artifacts`
