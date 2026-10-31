# ck-adapter-github

## Overview

`ck-adapter-github` is an adapter crate that ingests documentation from GitHub repositories over the REST API; its source adapter and normalizer live in a single `src` module. It depends on `ck-core` for ports and entities and uses `ureq` for HTTP access to `api.github.com`. The adapter is implemented and unit-tested at the crate level, but it is not composed into the `ck-bin` binary: `ck-bin/Cargo.toml` does not list it as a dependency.

## Subdirectories & Boundaries

- `src`: [`./src/README.md`](./src/README.md) - Component contract for `GitHubSourceAdapter` and `GitHubNormalizer`.

## Verification

- Run: `cargo test -p ck-adapter-github`
