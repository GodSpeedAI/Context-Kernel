# ck-artifacts src

## Overview

Artifact payload storage adapter implementing the `ck-core` `ArtifactPort` on the local filesystem. `FileArtifactStore` writes each artifact payload as a `.bin` file and its metadata as a sibling `.meta.json` file under `<data_dir>/artifacts/`. Artifact IDs are strictly validated before any path is constructed, preventing path traversal. The store is composed into the `ck-bin` binary at bootstrap (`ck-bin/src/main.rs` constructs `FileArtifactStore`).

## Interface

- [lib.rs:FileArtifactStore] - `pub struct` with `new(data_dir: PathBuf)`; implements `ArtifactPort` via `store_artifact`, `get_artifact`, `read_artifact_content`, and `delete_artifact_payload`.

## Invariants

- Artifact IDs must be non-empty and contain only ASCII alphanumerics, `-`, and `_`; all other characters (including `/`, `\`, and `..` sequences) are rejected before any path is built [lib.rs:validate_id]
- Payload bytes are stored at `<data_dir>/artifacts/<artifact_id>.bin` [lib.rs:bin_path]
- Metadata JSON is stored at `<data_dir>/artifacts/<artifact_id>.meta.json` [lib.rs:meta_path]
- The returned `Artifact` carries the SHA-256 checksum of the stored content and the storage URI of the payload file [lib.rs:FileArtifactStore::store_artifact]
- `get_artifact` returns `Ok(None)` when the metadata file does not exist [lib.rs:get_artifact]
- `read_artifact_content` returns `Ok(None)` when the payload file does not exist [lib.rs:read_artifact_content]
- `delete_artifact_payload` reports whether any file was actually removed [lib.rs:delete_artifact_payload]

## Side Effects

- Creates the `<data_dir>/artifacts` directory and writes the payload and metadata files [lib.rs:FileArtifactStore::store_artifact]
- Reads artifact metadata JSON from disk [lib.rs:get_artifact]
- Reads artifact payload bytes from disk [lib.rs:read_artifact_content]
- Deletes payload and metadata files from disk [lib.rs:delete_artifact_payload]
- Uses current wall-clock time (`Utc::now`) for the stored `created_at` timestamp [lib.rs:FileArtifactStore::store_artifact]
- Emits a `tracing::debug` event when an artifact is stored [lib.rs:FileArtifactStore::store_artifact]

## Verification

- Run: `cargo test -p ck-artifacts`
