use std::path::Path;
use std::{env, fs};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelConfig {
    pub service: ServiceConfig,
    pub storage: StorageConfig,
    pub retrieval: RetrievalConfig,
    #[serde(default)]
    pub mcp: McpConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
    #[serde(default)]
    pub embeddings: EmbeddingsConfig,
    #[serde(default)]
    pub reload: ReloadConfig,
    #[serde(default)]
    pub retention: RetentionConfig,
    #[serde(default)]
    pub authorization: AuthorizationConfig,
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetrievalConfig {
    pub default_mode: String,
    pub semantic_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(default = "default_enable_stdio")]
    pub enable_stdio: bool,
    #[serde(default)]
    pub enable_sse: bool,
    #[serde(default)]
    pub sse: SseConfig,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enable_stdio: true,
            enable_sse: false,
            sse: SseConfig::default(),
        }
    }
}

fn default_enable_stdio() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SseConfig {
    #[serde(default = "default_sse_bind")]
    pub bind: String,
    #[serde(default = "default_sse_base_path")]
    pub base_path: String,
    #[serde(default)]
    pub require_auth: bool,
}

fn default_sse_bind() -> String {
    "127.0.0.1:8765".to_string()
}

fn default_sse_base_path() -> String {
    "/mcp".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    #[serde(default = "default_trust_mode")]
    pub trust_mode: String,
    #[serde(default)]
    pub tokens: Vec<String>,
}

fn default_trust_mode() -> String {
    "internal".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SchedulerConfig {
    #[serde(default)]
    pub enable_periodic_refresh: bool,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,
}

fn default_refresh_interval() -> u64 {
    1800
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_embedding_provider")]
    pub provider: String,
    #[serde(default)]
    pub api_base: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_embedding_model")]
    pub model: String,
    #[serde(default = "default_embedding_dimensions")]
    pub dimensions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReloadConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetentionConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_session_event_retention_days")]
    pub session_event_days: i64,
    #[serde(default = "default_session_retention_days")]
    pub inactive_session_days: i64,
    #[serde(default = "default_refresh_job_retention_days")]
    pub refresh_job_days: i64,
    #[serde(default = "default_artifact_retention_days")]
    pub artifact_days: i64,
    #[serde(default = "default_disabled_source_retention_days")]
    pub disabled_source_days: i64,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            session_event_days: default_session_event_retention_days(),
            inactive_session_days: default_session_retention_days(),
            refresh_job_days: default_refresh_job_retention_days(),
            artifact_days: default_artifact_retention_days(),
            disabled_source_days: default_disabled_source_retention_days(),
        }
    }
}

fn default_session_event_retention_days() -> i64 {
    90
}

fn default_session_retention_days() -> i64 {
    30
}

fn default_refresh_job_retention_days() -> i64 {
    30
}

fn default_artifact_retention_days() -> i64 {
    90
}

fn default_disabled_source_retention_days() -> i64 {
    30
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    #[serde(default = "default_allow_all_tools")]
    pub allow_tools: Vec<String>,
    #[serde(default = "default_allow_all_corpora")]
    pub allow_corpora: Vec<String>,
}

impl Default for AuthorizationConfig {
    fn default() -> Self {
        Self {
            allow_tools: default_allow_all_tools(),
            allow_corpora: default_allow_all_corpora(),
        }
    }
}

fn default_allow_all_tools() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_allow_all_corpora() -> Vec<String> {
    vec!["*".to_string()]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default = "default_log_format")]
    pub log_format: String,
    #[serde(default)]
    pub otel_endpoint: Option<String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_format: default_log_format(),
            otel_endpoint: None,
        }
    }
}

fn default_log_format() -> String {
    "text".to_string()
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_embedding_provider(),
            api_base: String::new(),
            api_key: String::new(),
            model: default_embedding_model(),
            dimensions: default_embedding_dimensions(),
        }
    }
}

fn default_embedding_provider() -> String {
    "none".to_string()
}

fn default_embedding_model() -> String {
    "text-embedding-3-small".to_string()
}

fn default_embedding_dimensions() -> usize {
    1536
}

pub fn load_config(path: impl AsRef<Path>) -> Result<KernelConfig> {
    let raw = fs::read_to_string(path)?;
    let expanded = expand_env_vars(&raw);
    Ok(toml::from_str(&expanded)?)
}

pub fn default_config() -> KernelConfig {
    KernelConfig {
        service: ServiceConfig {
            name: "context-kernal".to_string(),
        },
        storage: StorageConfig {
            database_path: "kernel.db".to_string(),
        },
        retrieval: RetrievalConfig {
            default_mode: "hybrid".to_string(),
            semantic_enabled: false,
        },
        mcp: McpConfig {
            enable_stdio: true,
            enable_sse: false,
            sse: SseConfig::default(),
        },
        auth: AuthConfig::default(),
        scheduler: SchedulerConfig::default(),
        embeddings: EmbeddingsConfig::default(),
        reload: ReloadConfig::default(),
        retention: RetentionConfig::default(),
        authorization: AuthorizationConfig::default(),
        observability: ObservabilityConfig::default(),
    }
}

fn expand_env_vars(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find("${") {
        output.push_str(&rest[..start]);
        let after_marker = &rest[start + 2..];
        if let Some(end) = after_marker.find('}') {
            let key = &after_marker[..end];
            output.push_str(&env::var(key).unwrap_or_default());
            rest = &after_marker[end + 1..];
        } else {
            output.push_str(&rest[start..]);
            rest = "";
        }
    }

    output.push_str(rest);
    output
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn loads_toml_configuration_from_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("context-kernal.toml");
        fs::write(
            &path,
            r#"
[service]
name = "context-kernal"

[storage]
database_path = "/tmp/kernel.db"

[retrieval]
default_mode = "hybrid"
semantic_enabled = false
"#,
        )
        .unwrap();

        let config = load_config(&path).unwrap();

        assert_eq!(config.service.name, "context-kernal");
        assert_eq!(config.storage.database_path, "/tmp/kernel.db");
        assert_eq!(config.retrieval.default_mode, "hybrid");
        assert!(!config.retrieval.semantic_enabled);
        assert!(config.mcp.enable_stdio);
    }

    #[test]
    fn mcp_default_enables_stdio() {
        assert!(McpConfig::default().enable_stdio);
        assert!(default_config().mcp.enable_stdio);
    }

    #[test]
    fn loads_sse_and_auth_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("context-kernal.toml");
        fs::write(
            &path,
            r#"
[service]
name = "context-kernal"

[storage]
database_path = "/tmp/kernel.db"

[retrieval]
default_mode = "hybrid"
semantic_enabled = false

[mcp]
enable_stdio = true
enable_sse = true

[mcp.sse]
bind = "0.0.0.0:9999"
require_auth = true

[auth]
trust_mode = "internal"
tokens = ["secret123"]

[scheduler]
enable_periodic_refresh = true
refresh_interval_secs = 3600
"#,
        )
        .unwrap();

        let config = load_config(&path).unwrap();

        assert!(config.mcp.enable_sse);
        assert!(config.mcp.enable_stdio);
        assert_eq!(config.mcp.sse.bind, "0.0.0.0:9999");
        assert!(config.mcp.sse.require_auth);
        assert_eq!(config.auth.trust_mode, "internal");
        assert_eq!(config.auth.tokens, vec!["secret123"]);
        assert!(config.scheduler.enable_periodic_refresh);
        assert_eq!(config.scheduler.refresh_interval_secs, 3600);
    }

    #[test]
    fn loads_embeddings_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("context-kernal.toml");
        fs::write(
            &path,
            r#"
[service]
name = "context-kernal"

[storage]
database_path = "/tmp/kernel.db"

[retrieval]
default_mode = "hybrid"
semantic_enabled = false

[embeddings]
enabled = true
provider = "openai"
api_base = "https://api.openai.com/v1"
api_key = "$OPENAI_API_KEY"
model = "text-embedding-3-small"
dimensions = 1536
"#,
        )
        .unwrap();

        let config = load_config(&path).unwrap();

        assert!(config.embeddings.enabled);
        assert_eq!(config.embeddings.provider, "openai");
        assert_eq!(config.embeddings.model, "text-embedding-3-small");
        assert_eq!(config.embeddings.dimensions, 1536);
    }

    #[test]
    fn embeddings_default_is_disabled() {
        let config = default_config();
        assert!(!config.embeddings.enabled);
        assert_eq!(config.embeddings.provider, "none");
    }

    #[test]
    fn example_configs_parse() {
        let example_paths = [
            "../../config/context-kernal.example.toml",
            "../../config/context-kernal.sse.example.toml",
            "../../config/context-kernal.local.example.toml",
        ];

        for path in &example_paths {
            if std::path::Path::new(path).exists() {
                let config = load_config(path);
                assert!(
                    config.is_ok(),
                    "failed to parse {}: {:?}",
                    path,
                    config.err()
                );
            }
        }
    }

    #[test]
    fn loads_milestone_10_hardening_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("context-kernal.toml");
        fs::write(
            &path,
            r#"
[service]
name = "context-kernal"

[storage]
database_path = "/tmp/kernel.db"

[retrieval]
default_mode = "hybrid"
semantic_enabled = false

[reload]
enabled = true

[retention]
enabled = true
session_event_days = 14
inactive_session_days = 7
refresh_job_days = 3
artifact_days = 21
disabled_source_days = 10

[authorization]
allow_tools = ["get_status", "query_reference"]
allow_corpora = ["public_docs"]

[observability]
log_format = "json"
otel_endpoint = "http://127.0.0.1:4317"
"#,
        )
        .unwrap();

        let config = load_config(&path).unwrap();

        assert!(config.reload.enabled);
        assert!(config.retention.enabled);
        assert_eq!(config.retention.session_event_days, 14);
        assert_eq!(config.retention.inactive_session_days, 7);
        assert_eq!(config.retention.refresh_job_days, 3);
        assert_eq!(config.retention.artifact_days, 21);
        assert_eq!(config.retention.disabled_source_days, 10);
        assert_eq!(
            config.authorization.allow_tools,
            vec!["get_status".to_string(), "query_reference".to_string()]
        );
        assert_eq!(config.authorization.allow_corpora, vec!["public_docs"]);
        assert_eq!(config.observability.log_format, "json");
        assert_eq!(
            config.observability.otel_endpoint.as_deref(),
            Some("http://127.0.0.1:4317")
        );
    }
}
