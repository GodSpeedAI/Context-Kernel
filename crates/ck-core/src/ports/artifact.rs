use anyhow::Result;

use crate::entities::Artifact;

/// Port for artifact persistence.
pub trait ArtifactPort: Send + Sync {
    fn store_artifact(
        &self,
        artifact_id: &str,
        session_id: Option<&str>,
        kind: &str,
        mime_type: Option<&str>,
        content: &[u8],
    ) -> Result<Artifact>;

    fn get_artifact(&self, artifact_id: &str) -> Result<Option<Artifact>>;
    fn read_artifact_content(&self, artifact_id: &str) -> Result<Option<Vec<u8>>>;
}
