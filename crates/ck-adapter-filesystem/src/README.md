# ck-adapter-filesystem src

## Overview

Filesystem-side source adapters for Context Kernel ingestion, implementing the `ck-core` `SourceAdapter`, `Normalizer`, and `Chunker` ports. `FilesystemSourceAdapter` discovers and reads supported files from a local directory tree, `MarkdownNormalizer` turns raw bytes into a single `Document`, and `HeadingChunker` splits documents on markdown headings. `LearningProposalsAdapter` is a second source adapter that reads a learning-proposals JSONL ledger instead of a directory tree. All of these types are composed into the `ck-bin` binary at bootstrap (imported in `ck-bin/src/main.rs`).

## Interface

- [lib.rs:FilesystemSourceAdapter] - `pub struct`; implements `SourceAdapter` with kind `"filesystem_tree"`; `discover_units` recursively walks a configured `path` and returns `SourceUnit`s, `fetch_content` reads the referenced file into `RawContent`.
- [lib.rs:MarkdownNormalizer] - `pub struct`; implements `Normalizer`; converts one `RawContent` into one `Document` with a `doc_` ULID id, SHA-256 checksum, and title from the first markdown heading or the file stem.
- [lib.rs:HeadingChunker] - `pub struct`; implements `Chunker`; splits document text on markdown headings into ordered `Chunk`s with heading anchors and checksums.
- [lib.rs:learning_proposals] - `pub mod`, re-exported at the crate root via `pub use learning_proposals::LearningProposalsAdapter`.
- [learning_proposals.rs:LearningProposalsAdapter] - `pub struct`; implements `SourceAdapter` with kind `"learning_proposals_jsonl"`; also exposes inherent `query_by_domain_area` to filter ledger records by `payload.domain_area`.

## Invariants

- Only files with extensions `md`, `txt`, `toml`, `rs`, and `json` are discovered [lib.rs:SUPPORTED_EXTENSIONS]
- The directory walk does not descend into symlinked directories [lib.rs:walk_dir_recursive]
- `discover_units` requires config JSON with a `path` key that is an existing directory, and uses the file path relative to that root as `external_id` [lib.rs:FilesystemSourceAdapter::discover_units]
- Chunks are capped at 800 estimated tokens (whitespace-split words); larger sections are split on paragraph boundaries [lib.rs:HeadingChunker::split_large_section]
- Markdown content is titled from its first `#` heading; all other content types use the file stem [lib.rs:first_heading]
- The learning-proposals adapter requires a `path` config pointing to a JSONL ledger; blank lines are skipped and malformed rows are hard errors [learning_proposals.rs:read_records]
- Learning-proposal units use `payload.learning_proposal_id` as `external_id` with a `learning-proposals://{id}` URI [learning_proposals.rs:unit_from_record]

## Side Effects

- Reads the local filesystem: recursive directory traversal and file reads [lib.rs:walk_dir_recursive]
- Reads the configured JSONL ledger from disk on every discover, fetch, and query call [learning_proposals.rs:read_records]
- Uses current wall-clock time (`Utc::now`) for document `created_at`/`updated_at` [lib.rs:MarkdownNormalizer]
- Uses current wall-clock time (`Utc::now`) for chunk `created_at`/`updated_at` [lib.rs:HeadingChunker]

## Verification

- Run: `cargo test -p ck-adapter-filesystem`
