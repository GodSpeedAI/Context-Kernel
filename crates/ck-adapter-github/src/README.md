# ck-adapter-github src

## Overview

GitHub repository source adapter for Context Kernel ingestion, implementing the `ck-core` `SourceAdapter` and `Normalizer` ports against the GitHub REST API. `GitHubSourceAdapter` discovers supported files either through the recursive git trees API or a static `files` list in the source config, then fetches file contents from the contents API. `GitHubNormalizer` turns fetched raw content into a single `Document`, preserving GitHub metadata (SHA, path, owner, repo, branch). The crate is implemented and unit-tested at the crate level, but it is not composed into the `ck-bin` binary (`ck-bin/Cargo.toml` does not list it as a dependency).

## Interface

- [lib.rs:GitHubSourceAdapter] - `pub struct`; implements `SourceAdapter` with kind `"github_repo"`; `discover_units` lists supported files (tree API or static `files` list), `fetch_content` downloads and base64-decodes file content.
- [lib.rs:GitHubNormalizer] - `pub struct`; implements `Normalizer`; converts one `RawContent` into one `Document` with a SHA-256 checksum, title from the first markdown heading or file stem, and GitHub metadata passthrough plus `normalized_at`.

## Invariants

- Only files with extensions `md`, `txt`, and `mdx` are indexed [lib.rs:SUPPORTED_EXTENSIONS]
- Truncated git-tree API responses are rejected with an error instead of indexing a partial repository; the source config must fall back to a static `files` list [lib.rs:units_from_tree_response]
- API URLs are built from a hardcoded `https://api.github.com/repos/{owner}/{repo}` base [lib.rs:GitHubConfig::api_base]
- The branch defaults to `main` when not configured [lib.rs:GitHubConfig::branch]
- Auth tokens may be a literal string or a `$ENV_VAR_NAME` reference; if the env var is unset, requests proceed without an Authorization header and a warning is logged [lib.rs:build_request]
- URL path segments are percent-encoded for control characters, space, and `#`, `?`, `&`, `=`, `%` while preserving other safe characters [lib.rs:PATH_SEGMENT_ENCODE_SET]
- Response content is base64-decoded after stripping all whitespace [lib.rs:decode_content]
- Normalization requires valid UTF-8 content [lib.rs:GitHubNormalizer]

## Side Effects

- Outbound HTTPS GET requests to `api.github.com` for recursive tree discovery [lib.rs:discover_via_tree_api]
- Outbound HTTPS GET requests to the `api.github.com` contents endpoint for file fetches [lib.rs:GitHubSourceAdapter::fetch_content]
- Reads environment variables to resolve `$VAR`-style token references [lib.rs:build_request]
- Uses current wall-clock time (`Utc::now`) for document timestamps and `normalized_at` [lib.rs:GitHubNormalizer]

## Verification

- Run: `cargo test -p ck-adapter-github`
