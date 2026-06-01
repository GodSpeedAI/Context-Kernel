use std::path::Path;
use std::{env, fs};

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KernelConfig {
    pub service: ServiceConfig,
    pub storage: StorageConfig,
    pub retrieval: RetrievalConfig,
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
    }
}
