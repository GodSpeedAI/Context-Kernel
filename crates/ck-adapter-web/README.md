# ck-adapter-web

## Overview

`ck-adapter-web` is an adapter crate that ingests web pages over HTTP(S) with SSRF guardrails: URL validation blocks non-HTTP schemes, private and reserved IP literals, and hosts whose DNS resolution fails, returns empty, or resolves into blocked ranges. The HTTP fetch disables redirects and supports conditional GET via ETag/Last-Modified validators. The crate depends on `ck-core` plus `ureq` and `url`. It is implemented and unit-tested at the crate level, but it is not composed into the `ck-bin` binary: `ck-bin/Cargo.toml` does not list it as a dependency.

## Subdirectories & Boundaries

- `src`: [`./src/README.md`](./src/README.md) - Component contract for `WebSourceAdapter`, `WebNormalizer`, the `DnsResolver` trait, and URL validation.

## Verification

- Run: `cargo test -p ck-adapter-web`
