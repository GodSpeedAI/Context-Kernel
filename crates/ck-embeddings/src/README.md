# ck-embeddings Source

## Overview
This crate implements the embedding-provider stage of Context Kernel as adapters over the `EmbeddingPort` defined in `ck-core`. Two providers exist: `NoopEmbeddingProvider`, a permanently unavailable provider that yields no embeddings, and `OpenAIEmbeddingProvider`, which calls an OpenAI-compatible `/embeddings` HTTP API through a `ureq` agent. Availability is determined purely by configuration (a non-empty API key); an unavailable provider degrades to `Ok(None)` results rather than errors, letting the pipeline run without semantic embeddings. Both providers are composed into the runtime by `ck-bin` (`use ck_embeddings::{NoopEmbeddingProvider, OpenAIEmbeddingProvider}` in `crates/ck-bin/src/main.rs`).

## Interface
- [lib.rs:NoopEmbeddingProvider] - Public unit struct implementing `EmbeddingPort`: `embed`/`embed_batch` always return `Ok(None)`, `model_id()` is `"none"`, `dimensions()` is `0`, `is_available()` is `false`.
- [lib.rs:OpenAIEmbeddingProvider] - Public struct implementing `EmbeddingPort` over an OpenAI-compatible embeddings API; holds `api_base`, `api_key`, `model`, `dimensions`, and a `ureq::Agent`.
- [lib.rs:OpenAIEmbeddingProvider::new] - Builds the provider and its HTTP agent (30-second read timeout, 10-second write timeout).
- Request/response plumbing is private: [lib.rs:OpenAIEmbeddingProvider::build_request_body] assembles `{"model", "input"[, "dimensions"]}`; [lib.rs:OpenAIEmbeddingProvider::post_embedding] performs the authenticated POST; `EmbeddingResponse`/`EmbeddingData` are the serde response DTOs.

## Invariants
- Availability semantics: [lib.rs:OpenAIEmbeddingProvider::is_available] is true if and only if `api_key` is non-empty. When unavailable, `embed` and `embed_batch` return `Ok(None)` entries instead of errors ([lib.rs:OpenAIEmbeddingProvider::embed], [lib.rs:OpenAIEmbeddingProvider::embed_batch]).
- Request shape: [lib.rs:OpenAIEmbeddingProvider::post_embedding] POSTs to `{api_base}/embeddings` (trailing slash trimmed) with an `Authorization: Bearer {api_key}` header; [lib.rs:OpenAIEmbeddingProvider::build_request_body] includes `"dimensions"` in the body only when the configured dimension count is greater than 0.
- Batch alignment: [lib.rs:OpenAIEmbeddingProvider::embed_batch] sends the whole input array in a single request and maps response entries to result positions in order; response entries beyond `texts.len()` are dropped and any position without a response entry stays `None`.
- Empty-response contract: [lib.rs:OpenAIEmbeddingProvider::embed] errors with `no embedding returned in response` when the response `data` array is empty.

## Side Effects
- Outbound network calls: HTTP POST requests to the configured `{api_base}/embeddings` endpoint via the `ureq` agent, carrying the API key in the `Authorization` header ([lib.rs:OpenAIEmbeddingProvider::post_embedding]).
- The HTTP agent enforces a 30-second read timeout and 10-second write timeout ([lib.rs:OpenAIEmbeddingProvider::new]).
- No filesystem, database, lock, thread, or global-state side effects.

## Verification
- Run: `cargo test -p ck-embeddings`
