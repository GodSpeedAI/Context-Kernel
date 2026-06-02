use std::path::Path;

use anyhow::{Context, Result};
use base64::Engine;
use chrono::Utc;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use ulid::Ulid;

use ck_core::entities::{Document, RawContent, SourceUnit};
use ck_core::ports::{Normalizer, SourceAdapter};

const SUPPORTED_EXTENSIONS: &[&str] = &["md", "txt", "mdx"];

#[derive(Debug, Deserialize)]
struct GitHubConfig {
    owner: String,
    repo: String,
    #[serde(default)]
    branch: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    files: Option<Vec<String>>,
}

impl GitHubConfig {
    fn branch(&self) -> &str {
        self.branch.as_deref().unwrap_or("main")
    }

    fn api_base(&self) -> String {
        format!("https://api.github.com/repos/{}/{}", self.owner, self.repo)
    }

    fn web_url(&self, file_path: &str) -> String {
        let encoded = encode_path(file_path);
        format!(
            "https://github.com/{}/{}/blob/{}/{}",
            self.owner,
            self.repo,
            self.branch(),
            encoded
        )
    }

    fn contents_url(&self, file_path: &str) -> String {
        let encoded = encode_path(file_path);
        format!(
            "{}/contents/{}?ref={}",
            self.api_base(),
            encoded,
            self.branch()
        )
    }

    fn tree_url(&self) -> String {
        format!(
            "{}/git/trees/{}?recursive=1",
            self.api_base(),
            self.branch()
        )
    }
}

#[derive(Debug, Deserialize)]
struct TreeResponse {
    tree: Vec<TreeEntry>,
    #[serde(default)]
    truncated: bool,
}

#[derive(Debug, Deserialize)]
struct TreeEntry {
    path: String,
    #[serde(rename = "type")]
    entry_type: String,
}

#[derive(Debug, Deserialize)]
struct ContentsResponse {
    content: Option<String>,
    #[allow(dead_code)]
    encoding: Option<String>,
    #[allow(dead_code)]
    name: String,
    path: String,
    sha: String,
}

/// Characters to percent-encode in URL path segments.
/// Encodes controls, space, and characters that would break URL parsing
/// (#, ?, &, =, +, %) but preserves safe characters like ., -, _, ~, /, @, :, etc.
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'#')
    .add(b'?')
    .add(b'&')
    .add(b'=')
    .add(b'+')
    .add(b'%');

fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| utf8_percent_encode(segment, PATH_SEGMENT_ENCODE_SET).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn file_extension(path: &str) -> Option<String> {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
}

fn extension_to_mime(ext: &str) -> &'static str {
    match ext {
        "md" | "mdx" => "text/markdown",
        "txt" => "text/plain",
        _ => "application/octet-stream",
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
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

fn title_from_filename(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string()
}

fn build_request(config: &GitHubConfig, url: &str) -> ureq::Request {
    let req = ureq::get(url).set("User-Agent", "ck-adapter-github/0.1");
    if let Some(token) = &config.token {
        let token = if let Some(var_name) = token.strip_prefix('$') {
            match std::env::var(var_name) {
                Ok(val) => val,
                Err(_) => {
                    tracing::warn!("env var {} not set, proceeding without auth", var_name);
                    return req;
                }
            }
        } else {
            token.clone()
        };
        req.set("Authorization", &format!("Bearer {}", token))
    } else {
        req
    }
}

pub struct GitHubSourceAdapter;

impl SourceAdapter for GitHubSourceAdapter {
    fn kind(&self) -> &str {
        "github_repo"
    }

    fn discover_units(&self, _source_id: &str, config_json: &str) -> Result<Vec<SourceUnit>> {
        let config: GitHubConfig =
            serde_json::from_str(config_json).context("invalid GitHub config JSON")?;

        let file_paths = if let Some(files) = &config.files {
            files.clone()
        } else {
            discover_via_tree_api(&config)?
        };

        let mut units = Vec::new();
        for file_path in file_paths {
            let ext = file_extension(&file_path);
            let is_supported = ext
                .as_deref()
                .map(|e| SUPPORTED_EXTENSIONS.contains(&e))
                .unwrap_or(false);

            if !is_supported {
                continue;
            }

            let mime_hint = ext.as_deref().map(|e| extension_to_mime(e).to_string());
            let api_url = config.contents_url(&file_path);

            units.push(SourceUnit {
                external_id: file_path,
                uri: Some(api_url),
                mime_type_hint: mime_hint,
            });
        }
        Ok(units)
    }

    fn fetch_content(
        &self,
        _source_id: &str,
        unit: &SourceUnit,
        config_json: &str,
    ) -> Result<RawContent> {
        let config: GitHubConfig =
            serde_json::from_str(config_json).context("invalid GitHub config JSON")?;

        let url = unit
            .uri
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("unit has no URI"))?;

        let response: ContentsResponse = build_request(&config, url)
            .call()
            .context("GitHub API request failed")?
            .into_json()
            .context("failed to parse GitHub contents response")?;

        let bytes = decode_content(&response)?;

        let ext = file_extension(&response.path).unwrap_or_default();
        let mime = extension_to_mime(&ext).to_string();
        let web_url = config.web_url(&response.path);

        let metadata = serde_json::json!({
            "github_sha": response.sha,
            "github_path": response.path,
            "github_owner": config.owner,
            "github_repo": config.repo,
            "github_branch": config.branch(),
        });

        Ok(RawContent {
            external_id: unit.external_id.clone(),
            uri: Some(web_url),
            mime_type: mime,
            bytes,
            metadata_json: metadata.to_string(),
        })
    }
}

fn discover_via_tree_api(config: &GitHubConfig) -> Result<Vec<String>> {
    let url = config.tree_url();
    let response: TreeResponse = build_request(config, &url)
        .call()
        .context("GitHub tree API request failed")?
        .into_json()
        .context("failed to parse GitHub tree response")?;

    if response.truncated {
        tracing::warn!(
            "GitHub tree API returned truncated results for {}/{}; some files may be missing",
            config.owner,
            config.repo
        );
    }

    let prefix = config.path.as_deref().unwrap_or("");

    let mut paths = Vec::new();
    for entry in &response.tree {
        if entry.entry_type != "blob" {
            continue;
        }

        if !prefix.is_empty() && !entry.path.starts_with(prefix) {
            continue;
        }

        paths.push(entry.path.clone());
    }
    paths.sort();
    Ok(paths)
}

fn decode_content(response: &ContentsResponse) -> Result<Vec<u8>> {
    let encoded = response
        .content
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("no content in GitHub response for {}", response.path))?;

    let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(&cleaned)
        .context("failed to decode base64 content")
}

pub struct GitHubNormalizer;

impl Normalizer for GitHubNormalizer {
    fn normalize(
        &self,
        raw: &RawContent,
        source_id: &str,
        corpus_id: &str,
    ) -> Result<Vec<Document>> {
        let content_text =
            String::from_utf8(raw.bytes.clone()).context("content is not valid UTF-8")?;

        let title = if raw.mime_type == "text/markdown" {
            first_heading(&content_text).unwrap_or_else(|| title_from_filename(&raw.external_id))
        } else {
            title_from_filename(&raw.external_id)
        };

        let checksum = sha256_hex(content_text.as_bytes());
        let now = Utc::now().to_rfc3339();

        let mut metadata = serde_json::json!({});
        if !raw.metadata_json.is_empty() && raw.metadata_json != "{}" {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&raw.metadata_json) {
                metadata = parsed;
            }
        }
        metadata["normalized_at"] = serde_json::Value::String(now.clone());

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
            metadata_json: metadata.to_string(),
            checksum,
            superseded_by_document_id: None,
            created_at: now.clone(),
            updated_at: now,
        };
        Ok(vec![doc])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_kind_is_github_repo() {
        let adapter = GitHubSourceAdapter;
        assert_eq!(adapter.kind(), "github_repo");
    }

    #[test]
    fn config_parsing_full() {
        let json = r#"{
            "owner": "vercel",
            "repo": "next.js",
            "path": "docs",
            "branch": "main",
            "token": "ghp_test123"
        }"#;
        let config: GitHubConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.owner, "vercel");
        assert_eq!(config.repo, "next.js");
        assert_eq!(config.path, Some("docs".to_string()));
        assert_eq!(config.branch(), "main");
        assert_eq!(config.token, Some("ghp_test123".to_string()));
    }

    #[test]
    fn config_parsing_minimal() {
        let json = r#"{"owner": "test", "repo": "test"}"#;
        let config: GitHubConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.owner, "test");
        assert_eq!(config.repo, "test");
        assert_eq!(config.branch(), "main");
        assert!(config.path.is_none());
        assert!(config.token.is_none());
        assert!(config.files.is_none());
    }

    #[test]
    fn discover_units_with_static_file_list() {
        let json = r#"{
            "owner": "test",
            "repo": "test",
            "files": ["README.md", "docs/guide.md", "src/main.rs", "data.json", "notes.txt"]
        }"#;

        let adapter = GitHubSourceAdapter;
        let units = adapter.discover_units("src1", json).unwrap();

        let ids: Vec<&str> = units.iter().map(|u| u.external_id.as_str()).collect();
        assert!(ids.contains(&"README.md"));
        assert!(ids.contains(&"docs/guide.md"));
        assert!(ids.contains(&"notes.txt"));
        assert!(!ids.contains(&"src/main.rs"));
        assert!(!ids.contains(&"data.json"));

        assert_eq!(units.len(), 3);

        for unit in &units {
            assert!(unit.uri.is_some());
            assert!(unit.mime_type_hint.is_some());
            assert!(unit.uri.as_ref().unwrap().contains("api.github.com"));
        }
    }

    #[test]
    fn discover_units_filters_unsupported_extensions() {
        let json = r#"{
            "owner": "test",
            "repo": "test",
            "files": ["image.png", "style.css", "readme.md"]
        }"#;

        let adapter = GitHubSourceAdapter;
        let units = adapter.discover_units("src1", json).unwrap();

        assert_eq!(units.len(), 1);
        assert_eq!(units[0].external_id, "readme.md");
    }

    #[test]
    fn discover_units_empty_files() {
        let json = r#"{
            "owner": "test",
            "repo": "test",
            "files": []
        }"#;

        let adapter = GitHubSourceAdapter;
        let units = adapter.discover_units("src1", json).unwrap();
        assert!(units.is_empty());
    }

    #[test]
    fn config_contents_url() {
        let config: GitHubConfig =
            serde_json::from_str(r#"{"owner": "vercel", "repo": "next.js", "branch": "canary"}"#)
                .unwrap();

        let url = config.contents_url("docs/getting-started.md");
        assert_eq!(
            url,
            "https://api.github.com/repos/vercel/next.js/contents/docs/getting-started.md?ref=canary"
        );
    }

    #[test]
    fn config_web_url() {
        let config: GitHubConfig =
            serde_json::from_str(r#"{"owner": "vercel", "repo": "next.js", "branch": "main"}"#)
                .unwrap();

        let url = config.web_url("docs/index.md");
        assert_eq!(
            url,
            "https://github.com/vercel/next.js/blob/main/docs/index.md"
        );
    }

    #[test]
    fn config_tree_url() {
        let config: GitHubConfig =
            serde_json::from_str(r#"{"owner": "vercel", "repo": "next.js", "branch": "main"}"#)
                .unwrap();

        let url = config.tree_url();
        assert_eq!(
            url,
            "https://api.github.com/repos/vercel/next.js/git/trees/main?recursive=1"
        );
    }

    #[test]
    fn normalizer_creates_document_with_github_metadata() {
        let metadata = serde_json::json!({
            "github_sha": "abc123",
            "github_path": "README.md",
            "github_owner": "vercel",
            "github_repo": "next.js",
            "github_branch": "main",
        });

        let raw = RawContent {
            external_id: "README.md".to_string(),
            uri: Some("https://github.com/vercel/next.js/blob/main/README.md".to_string()),
            mime_type: "text/markdown".to_string(),
            bytes: b"# Next.js\nThe React framework.".to_vec(),
            metadata_json: metadata.to_string(),
        };

        let normalizer = GitHubNormalizer;
        let docs = normalizer.normalize(&raw, "src1", "corp1").unwrap();

        assert_eq!(docs.len(), 1);
        let doc = &docs[0];
        assert!(doc.document_id.starts_with("doc_"));
        assert_eq!(doc.source_id, "src1");
        assert_eq!(doc.corpus_id, "corp1");
        assert_eq!(doc.title, "Next.js");
        assert_eq!(
            doc.uri.as_deref(),
            Some("https://github.com/vercel/next.js/blob/main/README.md")
        );
        assert!(!doc.checksum.is_empty());

        let doc_meta: serde_json::Value = serde_json::from_str(&doc.metadata_json).unwrap();
        assert_eq!(doc_meta["github_sha"], "abc123");
        assert_eq!(doc_meta["github_owner"], "vercel");
        assert!(doc_meta["normalized_at"].is_string());
    }

    #[test]
    fn normalizer_uses_filename_for_non_markdown() {
        let raw = RawContent {
            external_id: "notes.txt".to_string(),
            uri: Some("https://github.com/test/test/blob/main/notes.txt".to_string()),
            mime_type: "text/plain".to_string(),
            bytes: b"Just some text".to_vec(),
            metadata_json: "{}".to_string(),
        };

        let normalizer = GitHubNormalizer;
        let docs = normalizer.normalize(&raw, "src1", "corp1").unwrap();

        assert_eq!(docs[0].title, "notes");
    }

    #[test]
    fn extension_to_mime_mapping() {
        assert_eq!(extension_to_mime("md"), "text/markdown");
        assert_eq!(extension_to_mime("mdx"), "text/markdown");
        assert_eq!(extension_to_mime("txt"), "text/plain");
        assert_eq!(extension_to_mime("png"), "application/octet-stream");
    }

    #[test]
    fn file_extension_extraction() {
        assert_eq!(file_extension("README.md"), Some("md".to_string()));
        assert_eq!(file_extension("docs/guide.MDX"), Some("mdx".to_string()));
        assert_eq!(file_extension("Makefile"), None);
    }

    #[test]
    fn first_heading_extraction() {
        assert_eq!(
            first_heading("# Title\nSome text"),
            Some("Title".to_string())
        );
        assert_eq!(
            first_heading("## Sub Title\nText"),
            Some("Sub Title".to_string())
        );
        assert_eq!(first_heading("No heading here"), None);
    }

    #[test]
    fn title_from_filename_works() {
        assert_eq!(
            title_from_filename("docs/getting-started.md"),
            "getting-started"
        );
        assert_eq!(title_from_filename("README.md"), "README");
        assert_eq!(title_from_filename("noext"), "noext");
    }

    #[test]
    fn decode_content_handles_base64() {
        let response = ContentsResponse {
            content: Some("SGVsbG8gV29ybGQ=".to_string()),
            encoding: Some("base64".to_string()),
            name: "test.md".to_string(),
            path: "test.md".to_string(),
            sha: "abc123".to_string(),
        };

        let bytes = decode_content(&response).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "Hello World");
    }

    #[test]
    fn decode_content_strips_whitespace() {
        let response = ContentsResponse {
            content: Some("SGVs\n\nbG8g\nV29ybGQ=".to_string()),
            encoding: Some("base64".to_string()),
            name: "test.md".to_string(),
            path: "test.md".to_string(),
            sha: "abc123".to_string(),
        };

        let bytes = decode_content(&response).unwrap();
        assert_eq!(String::from_utf8(bytes).unwrap(), "Hello World");
    }

    #[test]
    fn fetch_content_returns_error_without_uri() {
        let unit = SourceUnit {
            external_id: "test.md".to_string(),
            uri: None,
            mime_type_hint: Some("text/markdown".to_string()),
        };

        let adapter = GitHubSourceAdapter;
        let result = adapter.fetch_content("src1", &unit, r#"{"owner":"t","repo":"t"}"#);
        assert!(result.is_err());
    }
}
