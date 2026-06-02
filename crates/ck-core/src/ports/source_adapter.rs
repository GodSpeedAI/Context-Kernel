use anyhow::Result;

use crate::entities::{RawContent, SourceUnit};

/// Port for source adapters that discover and fetch content.
pub trait SourceAdapter: Send + Sync {
    fn kind(&self) -> &str;
    fn discover_units(&self, source_id: &str, config_json: &str) -> Result<Vec<SourceUnit>>;
    fn fetch_content(
        &self,
        source_id: &str,
        unit: &SourceUnit,
        config_json: &str,
    ) -> Result<RawContent>;
}
