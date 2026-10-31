# ck-adapter-filesystem

## Overview

`ck-adapter-filesystem` is an adapter crate providing filesystem-backed ingestion for Context Kernel: a source adapter that walks local directory trees (`FilesystemSourceAdapter`), a second source adapter that reads learning-proposals JSONL ledgers (`LearningProposalsAdapter`), plus a markdown normalizer (`MarkdownNormalizer`) and heading-based chunker (`HeadingChunker`) used by the same pipeline. The crate depends only on `ck-core` ports and entities plus utility crates, keeping infrastructure concerns out of the core. Both source adapters, the normalizer, and the chunker are composed into the `ck-bin` binary at bootstrap. An integration test suite lives under `tests/` and is outside the scope of this contract.

## Subdirectories & Boundaries

- `src`: [`./src/README.md`](./src/README.md) - Component contract for the filesystem and learning-proposals source adapters, the markdown normalizer, and the heading chunker.

## Verification

- Run: `cargo test -p ck-adapter-filesystem`
