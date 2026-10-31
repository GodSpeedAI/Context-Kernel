# ck-embeddings

## Overview
`ck-embeddings` (package `ck-embeddings`) is a ports-and-adapters adapter crate providing text-embedding providers for Context Kernel. It implements the `EmbeddingPort` from `ck-core` with two providers: a no-op provider that is permanently unavailable, and `OpenAIEmbeddingProvider`, which calls an OpenAI-compatible `/embeddings` API and degrades to `Ok(None)` results when no API key is configured. The crate is composed into the runtime by `ck-bin` and has no direct source files outside its `src` module.

## Subdirectories & Boundaries
- `src`: [./src/README.md](./src/README.md) - Component contract for `NoopEmbeddingProvider` and `OpenAIEmbeddingProvider`: availability gating by API key, request/response shapes for the `/embeddings` endpoint, batch alignment rules, and HTTP timeout configuration.

## Verification
- Run: `cargo test -p ck-embeddings`
