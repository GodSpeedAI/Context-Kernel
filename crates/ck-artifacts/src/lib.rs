use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use ck_core::entities::Artifact;
use ck_core::ports::ArtifactPort;
use sha2::{Digest, Sha256};

pub struct FileArtifactStore {
    data_dir: PathBuf,
}

impl FileArtifactStore {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn artifacts_dir(&self) -> PathBuf {
        self.data_dir.join("artifacts")
    }

    fn bin_path(&self, artifact_id: &str) -> PathBuf {
        self.artifacts_dir().join(format!("{artifact_id}.bin"))
    }

    fn meta_path(&self, artifact_id: &str) -> PathBuf {
        self.artifacts_dir()
            .join(format!("{artifact_id}.meta.json"))
    }
}

/// Validates that an artifact_id contains only safe characters
/// (ASCII alphanumeric, '-', and '_') and is non-empty.
fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() {
        anyhow::bail!("artifact_id must not be empty");
    }
    if let Some(bad) = id
        .chars()
        .find(|c| !c.is_ascii_alphanumeric() && *c != '-' && *c != '_')
    {
        anyhow::bail!("artifact_id contains invalid character: '{}'", bad);
    }
    Ok(())
}

impl ArtifactPort for FileArtifactStore {
    fn store_artifact(
        &self,
        artifact_id: &str,
        session_id: Option<&str>,
        kind: &str,
        mime_type: Option<&str>,
        content: &[u8],
    ) -> Result<Artifact> {
        validate_id(artifact_id)?;
        let dir = self.artifacts_dir();
        fs::create_dir_all(&dir)?;

        let bin_path = self.bin_path(artifact_id);
        fs::write(&bin_path, content)?;

        let mut hasher = Sha256::new();
        hasher.update(content);
        let checksum = hex::encode(hasher.finalize());

        let artifact = Artifact {
            artifact_id: artifact_id.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            kind: kind.to_string(),
            mime_type: mime_type.map(|m| m.to_string()),
            storage_uri: bin_path.to_string_lossy().to_string(),
            size_bytes: content.len() as i64,
            summary: None,
            checksum,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let meta_path = self.meta_path(artifact_id);
        let json = serde_json::to_string_pretty(&artifact)?;
        fs::write(&meta_path, json)?;

        tracing::debug!(artifact_id, size = content.len(), "stored artifact");

        Ok(artifact)
    }

    fn get_artifact(&self, artifact_id: &str) -> Result<Option<Artifact>> {
        validate_id(artifact_id)?;
        let meta_path = self.meta_path(artifact_id);
        if !meta_path.exists() {
            return Ok(None);
        }
        let json = fs::read_to_string(&meta_path)?;
        let artifact: Artifact = serde_json::from_str(&json)?;
        Ok(Some(artifact))
    }

    fn read_artifact_content(&self, artifact_id: &str) -> Result<Option<Vec<u8>>> {
        validate_id(artifact_id)?;
        let bin_path = self.bin_path(artifact_id);
        if !bin_path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&bin_path)?;
        Ok(Some(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, FileArtifactStore) {
        let dir = TempDir::new().unwrap();
        let store = FileArtifactStore::new(dir.path().to_path_buf());
        (dir, store)
    }

    #[test]
    fn store_and_read_back() {
        let (_tmp, store) = make_store();
        let content = b"hello world";
        let artifact = store
            .store_artifact("a1", Some("s1"), "text", Some("text/plain"), content)
            .unwrap();

        assert_eq!(artifact.artifact_id, "a1");
        assert_eq!(artifact.session_id, Some("s1".to_string()));
        assert_eq!(artifact.kind, "text");
        assert_eq!(artifact.mime_type, Some("text/plain".to_string()));
        assert_eq!(artifact.size_bytes, content.len() as i64);

        let fetched = store.get_artifact("a1").unwrap().unwrap();
        assert_eq!(fetched.artifact_id, "a1");
        assert_eq!(fetched.checksum, artifact.checksum);

        let bytes = store.read_artifact_content("a1").unwrap().unwrap();
        assert_eq!(bytes, content);
    }

    #[test]
    fn store_verifies_checksum() {
        let (_tmp, store) = make_store();
        let content = b"test checksum content";
        let artifact = store
            .store_artifact("a2", None, "binary", None, content)
            .unwrap();

        let mut hasher = Sha256::new();
        hasher.update(content);
        let expected = hex::encode(hasher.finalize());
        assert_eq!(artifact.checksum, expected);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let (_tmp, store) = make_store();
        assert!(store.get_artifact("nope").unwrap().is_none());
        assert!(store.read_artifact_content("nope").unwrap().is_none());
    }

    #[test]
    fn validate_id_rejects_traversal() {
        assert!(validate_id("").is_err());
        assert!(validate_id("../etc/passwd").is_err());
        assert!(validate_id("foo/bar").is_err());
        assert!(validate_id("foo\\bar").is_err());
        assert!(validate_id("a..b").is_err());
        assert!(validate_id("valid-id_123").is_ok());
        assert!(validate_id("abc").is_ok());
    }
}
