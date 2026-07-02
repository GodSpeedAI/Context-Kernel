use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use ulid::Ulid;

use ck_core::entities::{Chunk, Document, RawContent, SourceUnit};
use ck_core::ports::{Chunker, Normalizer, SourceAdapter};

pub mod learning_proposals;
pub use learning_proposals::LearningProposalsAdapter;

const SUPPORTED_EXTENSIONS: &[&str] = &["md", "txt", "toml", "rs", "json"];
const MAX_TOKENS_PER_CHUNK: usize = 800;

fn extension_to_mime(ext: &str) -> &'static str {
    match ext {
        "md" => "text/markdown",
        "txt" => "text/plain",
        "toml" => "application/toml",
        "rs" => "text/rust",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
}

fn file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn estimate_tokens(text: &str) -> usize {
    text.split_whitespace().count()
}

fn heading_to_anchor(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn title_from_filename(uri: &Option<String>) -> String {
    match uri {
        Some(u) => Path::new(u)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string(),
        None => "untitled".to_string(),
    }
}

fn first_heading(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim();
            if !heading.is_empty() {
                return Some(heading.to_string());
            }
        }
    }
    None
}

fn walk_dir_recursive(root: &Path, results: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(root)
        .with_context(|| format!("failed to read directory: {}", root.display()))?;
    let mut dirs: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.context("failed to read directory entry")?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to get file type: {}", entry.path().display()))?;
        if file_type.is_dir() && !file_type.is_symlink() {
            dirs.push(entry.path());
        } else if file_type.is_file() {
            let path = entry.path();
            if let Some(ext) = file_extension(&path) {
                if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                    results.push(path);
                }
            }
        }
    }
    for dir in dirs {
        walk_dir_recursive(&dir, results)?;
    }
    Ok(())
}

pub struct FilesystemSourceAdapter;

impl SourceAdapter for FilesystemSourceAdapter {
    fn kind(&self) -> &str {
        "filesystem_tree"
    }

    fn discover_units(&self, _source_id: &str, config_json: &str) -> Result<Vec<SourceUnit>> {
        let config: serde_json::Value =
            serde_json::from_str(config_json).context("invalid config JSON")?;
        let path_str = config["path"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing 'path' in config"))?;
        let root = Path::new(path_str);
        if !root.is_dir() {
            anyhow::bail!("path is not a directory: {}", path_str);
        }

        let mut files = Vec::new();
        walk_dir_recursive(root, &mut files)?;
        files.sort();

        let mut units = Vec::new();
        for file_path in files {
            let relative = file_path
                .strip_prefix(root)
                .context("failed to compute relative path")?;
            let external_id = relative
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("non-UTF-8 path: {}", relative.display()))?;
            let ext = file_extension(&file_path);
            let mime_hint = ext.as_deref().map(extension_to_mime);
            units.push(SourceUnit {
                external_id: external_id.to_string(),
                uri: Some(file_path.to_string_lossy().into_owned()),
                mime_type_hint: mime_hint.map(String::from),
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
        let uri = unit
            .uri
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("unit has no URI"))?;
        let bytes = fs::read(uri).with_context(|| format!("failed to read file: {}", uri))?;
        let ext = file_extension(Path::new(uri)).unwrap_or_default();
        let mime = extension_to_mime(&ext).to_string();
        Ok(RawContent {
            external_id: unit.external_id.clone(),
            uri: unit.uri.clone(),
            mime_type: mime,
            bytes,
            metadata_json: "{}".to_string(),
        })
    }
}

pub struct MarkdownNormalizer;

impl Normalizer for MarkdownNormalizer {
    fn normalize(
        &self,
        raw: &RawContent,
        source_id: &str,
        corpus_id: &str,
    ) -> Result<Vec<Document>> {
        let content_text =
            String::from_utf8(raw.bytes.clone()).context("content is not valid UTF-8")?;

        let title = if raw.mime_type == "text/markdown" {
            first_heading(&content_text).unwrap_or_else(|| title_from_filename(&raw.uri))
        } else {
            title_from_filename(&raw.uri)
        };

        let checksum = sha256_hex(content_text.as_bytes());
        let now = Utc::now().to_rfc3339();
        let doc = Document {
            document_id: format!("doc_{}", Ulid::new()),
            corpus_id: corpus_id.to_string(),
            source_id: source_id.to_string(),
            external_id: Some(raw.external_id.clone()),
            library_id: None,
            title,
            uri: raw.uri.clone(),
            version: None,
            mime_type: Some(raw.mime_type.clone()),
            content_text,
            metadata_json: "{}".to_string(),
            checksum,
            superseded_by_document_id: None,
            created_at: now.clone(),
            updated_at: now,
        };
        Ok(vec![doc])
    }
}

pub struct HeadingChunker;

impl HeadingChunker {
    fn parse_sections(text: &str) -> Vec<(Option<String>, String)> {
        let mut sections: Vec<(Option<String>, String)> = Vec::new();
        let mut current_heading: Option<String> = None;
        let mut current_lines: Vec<String> = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let hash_count = trimmed.chars().take_while(|c| *c == '#').count();
                let heading_text = trimmed[hash_count..].trim().to_string();
                if !heading_text.is_empty() {
                    if !current_lines.is_empty() || current_heading.is_some() {
                        let body = current_lines.join("\n").trim().to_string();
                        if !body.is_empty() {
                            sections.push((current_heading.clone(), body));
                        }
                    }
                    current_heading = Some(heading_text);
                    current_lines.clear();
                    continue;
                }
            }
            current_lines.push(line.to_string());
        }

        if !current_lines.is_empty() || current_heading.is_some() {
            let body = current_lines.join("\n").trim().to_string();
            if !body.is_empty() {
                sections.push((current_heading, body));
            }
        }

        sections
    }

    fn split_large_section(heading: &Option<String>, text: &str) -> Vec<(Option<String>, String)> {
        let token_count = estimate_tokens(text);
        if token_count <= MAX_TOKENS_PER_CHUNK {
            return vec![(heading.clone(), text.to_string())];
        }

        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut result: Vec<(Option<String>, String)> = Vec::new();
        let mut current = String::new();
        let mut current_tokens = 0;

        for para in &paragraphs {
            let para_tokens = estimate_tokens(para);
            if !current.is_empty() && current_tokens + para_tokens > MAX_TOKENS_PER_CHUNK {
                result.push((heading.clone(), current.trim().to_string()));
                current.clear();
                current_tokens = 0;
            }
            if !current.is_empty() {
                current.push_str("\n\n");
            }
            current.push_str(para);
            current_tokens += para_tokens;
        }

        if !current.trim().is_empty() {
            result.push((heading.clone(), current.trim().to_string()));
        }

        result
    }
}

impl Chunker for HeadingChunker {
    fn chunk(
        &self,
        document_id: &str,
        corpus_id: &str,
        text: &str,
        _title: &str,
    ) -> Result<Vec<Chunk>> {
        let sections = Self::parse_sections(text);

        if sections.is_empty() {
            return Ok(Vec::new());
        }

        let mut chunks = Vec::new();
        let mut ordinal: i64 = 0;

        for (heading, body) in &sections {
            let sub_sections = Self::split_large_section(heading, body);
            for (_sub_heading, sub_body) in sub_sections {
                let now = Utc::now().to_rfc3339();
                let token_count = estimate_tokens(&sub_body) as i64;
                let anchor = heading.as_deref().map(heading_to_anchor);
                let checksum = sha256_hex(sub_body.as_bytes());

                chunks.push(Chunk {
                    chunk_id: format!("chk_{}", Ulid::new()),
                    document_id: document_id.to_string(),
                    corpus_id: corpus_id.to_string(),
                    ordinal,
                    text: sub_body,
                    token_count_estimate: token_count,
                    heading: heading.clone(),
                    anchor,
                    section_path_json: "[]".to_string(),
                    metadata_json: "{}".to_string(),
                    checksum,
                    created_at: now.clone(),
                    updated_at: now,
                });
                ordinal += 1;
            }
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn filesystem_adapter_discovers_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();

        fs::write(root.join("alpha.md"), "# Alpha\nContent").unwrap();
        fs::write(root.join("beta.txt"), "Plain text").unwrap();
        fs::write(root.join("gamma.json"), "{}").unwrap();
        fs::write(root.join("ignored.png"), "binary").unwrap();

        let sub = root.join("sub");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("delta.rs"), "fn main() {}").unwrap();

        let config = serde_json::json!({ "path": root.to_str().unwrap() }).to_string();
        let adapter = FilesystemSourceAdapter;
        let units = adapter.discover_units("src1", &config).unwrap();

        let ids: Vec<&str> = units.iter().map(|u| u.external_id.as_str()).collect();
        assert!(ids.contains(&"alpha.md"));
        assert!(ids.contains(&"beta.txt"));
        assert!(ids.contains(&"gamma.json"));
        assert!(ids.contains(&"sub/delta.rs"));
        assert!(!ids.contains(&"ignored.png"));

        assert_eq!(units.len(), 4);
        for unit in &units {
            assert!(unit.uri.is_some());
            assert!(unit.mime_type_hint.is_some());
        }
    }

    #[test]
    fn normalizer_creates_document() {
        let raw = RawContent {
            external_id: "notes.md".to_string(),
            uri: Some("/tmp/notes.md".to_string()),
            mime_type: "text/markdown".to_string(),
            bytes: b"# My Notes\nSome content here.".to_vec(),
            metadata_json: "{}".to_string(),
        };

        let normalizer = MarkdownNormalizer;
        let docs = normalizer.normalize(&raw, "src1", "corp1").unwrap();

        assert_eq!(docs.len(), 1);
        let doc = &docs[0];
        assert!(doc.document_id.starts_with("doc_"));
        assert_eq!(doc.source_id, "src1");
        assert_eq!(doc.corpus_id, "corp1");
        assert_eq!(doc.title, "My Notes");
        assert_eq!(doc.content_text, "# My Notes\nSome content here.");
        assert!(!doc.checksum.is_empty());
        assert_eq!(doc.mime_type.as_deref(), Some("text/markdown"));
    }

    #[test]
    fn normalizer_uses_filename_for_non_markdown() {
        let raw = RawContent {
            external_id: "code.rs".to_string(),
            uri: Some("/tmp/code.rs".to_string()),
            mime_type: "text/rust".to_string(),
            bytes: b"fn main() {}".to_vec(),
            metadata_json: "{}".to_string(),
        };

        let normalizer = MarkdownNormalizer;
        let docs = normalizer.normalize(&raw, "src1", "corp1").unwrap();

        assert_eq!(docs[0].title, "code");
    }

    #[test]
    fn chunker_splits_by_headings() {
        let text = "\
# Introduction

This is the intro.

## Details

Some details here.

More details.

### Sub

Subsection content.";

        let chunker = HeadingChunker;
        let chunks = chunker.chunk("doc1", "corp1", text, "Test").unwrap();

        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].heading.as_deref(), Some("Introduction"));
        assert_eq!(chunks[0].anchor.as_deref(), Some("introduction"));
        assert!(chunks[0].text.contains("This is the intro."));
        assert_eq!(chunks[0].ordinal, 0);

        assert_eq!(chunks[1].heading.as_deref(), Some("Details"));
        assert!(chunks[1].text.contains("Some details here."));
        assert_eq!(chunks[1].ordinal, 1);

        assert_eq!(chunks[2].heading.as_deref(), Some("Sub"));
        assert!(chunks[2].text.contains("Subsection content."));
        assert_eq!(chunks[2].ordinal, 2);

        for chunk in &chunks {
            assert!(chunk.chunk_id.starts_with("chk_"));
            assert_eq!(chunk.document_id, "doc1");
            assert_eq!(chunk.corpus_id, "corp1");
            assert!(!chunk.checksum.is_empty());
            assert_eq!(chunk.section_path_json, "[]");
        }
    }

    #[test]
    fn chunker_handles_text_without_headings() {
        let text = "Just some plain text without any headings.\n\nAnother paragraph.";

        let chunker = HeadingChunker;
        let chunks = chunker.chunk("doc1", "corp1", text, "Plain").unwrap();

        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].heading.is_none());
        assert!(chunks[0].anchor.is_none());
        assert!(chunks[0].text.contains("Just some plain text"));
        assert_eq!(chunks[0].ordinal, 0);
    }

    #[test]
    fn fetch_content_reads_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.md");
        fs::write(&file_path, "# Hello\nWorld").unwrap();

        let unit = SourceUnit {
            external_id: "test.md".to_string(),
            uri: Some(file_path.to_str().unwrap().to_string()),
            mime_type_hint: Some("text/markdown".to_string()),
        };

        let adapter = FilesystemSourceAdapter;
        let raw = adapter.fetch_content("src1", &unit, "{}").unwrap();

        assert_eq!(raw.external_id, "test.md");
        assert_eq!(raw.mime_type, "text/markdown");
        assert_eq!(String::from_utf8(raw.bytes).unwrap(), "# Hello\nWorld");
    }
}
