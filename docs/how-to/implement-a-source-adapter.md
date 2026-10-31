# How-To: Implement a New Source Adapter

This guide explains how to implement a custom documentation ingestion adapter in Context Kernel by implementing the `SourceAdapter` port trait from `ck-core`.

---

## Goal
Add a new documentation source adapter (e.g. for an S3 bucket, an internal Wiki, or a package repository) that integrates with Context Kernel's normalization, chunking, and indexing pipeline.

---

## 1. Understand the `SourceAdapter` Trait

The trait is defined in [`crates/ck-core/src/ports/source_adapter.rs`](../../crates/ck-core/src/ports/source_adapter.rs):

```rust
use anyhow::Result;
use crate::entities::{RawContent, SourceUnit};

pub trait SourceAdapter: Send + Sync {
    /// Returns the unique string identifier for this source kind (e.g., "s3_bucket").
    fn kind(&self) -> &str;

    /// Discovers all units available in the source based on the JSON configuration string.
    fn discover_units(&self, source_id: &str, config_json: &str) -> Result<Vec<SourceUnit>>;

    /// Fetches the raw text or binary content and revision tracking string for a unit.
    fn fetch_content(
        &self,
        source_id: &str,
        unit: &SourceUnit,
        config_json: &str,
    ) -> Result<RawContent>;
}
```

---

## 2. Step-by-Step Implementation Example

Here is a minimal implementation for an in-memory dictionary or mock adapter:

```rust
use anyhow::Result;
use ck_core::entities::{RawContent, SourceUnit};
use ck_core::ports::SourceAdapter;
use serde::Deserialize;

#[derive(Deserialize)]
struct MemoryConfig {
    topics: Vec<String>,
}

pub struct MemorySourceAdapter;

impl SourceAdapter for MemorySourceAdapter {
    fn kind(&self) -> &str {
        "memory_docs"
    }

    fn discover_units(&self, _source_id: &str, config_json: &str) -> Result<Vec<SourceUnit>> {
        let config: MemoryConfig = serde_json::from_str(config_json)?;
        let mut units = Vec::new();
        for topic in config.topics {
            units.push(SourceUnit {
                external_id: topic.clone(),
                uri: Some(format!("memory://{}", topic)),
                mime_type_hint: Some("text/markdown".to_string()),
            });
        }
        Ok(units)
    }

    fn fetch_content(
        &self,
        _source_id: &str,
        unit: &SourceUnit,
        _config_json: &str,
    ) -> Result<RawContent> {
        let content_string = format!("# Documentation for {}\n\nThis is synthetic content.", unit.external_id);
        Ok(RawContent {
            content: content_string.into_bytes(),
            mime_type: Some("text/markdown".to_string()),
            revision: Some("v1".to_string()),
        })
    }
}
```

---

## 3. Wiring into `AppServices`

In `crates/ck-bin/src/main.rs::wire_app`, pass your adapter instance into `AppServices::new`:

```rust
let mut app = AppServices::new(
    Box::new(store.clone()),
    Box::new(store.clone()),
    Box::new(FileArtifactStore::new(PathBuf::from(data_dir))),
    Box::new(FtsSearch::open(db_path.clone())?),
    Box::new(LexicalRanker),
    Box::new(MemorySourceAdapter), // <-- Your new adapter here
    Box::new(MarkdownNormalizer),
    Box::new(HeadingChunker),
);
```

---

## 4. Ingesting Through MCP

Once wired, callers can ingest sources using your new kind:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "ingest_source",
    "arguments": {
      "corpus_id": "synthetic",
      "kind": "memory_docs",
      "adapter_config_json": "{\"topics\":[\"rust\",\"async\"]}",
      "display_name": "Synthetic Memory Docs"
    }
  }
}
```

Context Kernel will automatically handle title extraction (`Normalizer`), heading chunking (`Chunker`), SQLite relational insertion, and FTS5 indexing.
