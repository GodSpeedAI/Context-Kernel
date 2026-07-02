use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use ck_core::entities::{RawContent, SourceUnit};
use ck_core::ports::SourceAdapter;
use serde_json::Value;

const MIME_TYPE: &str = "application/json";

pub struct LearningProposalsAdapter;

impl LearningProposalsAdapter {
    pub fn query_by_domain_area(
        &self,
        config_json: &str,
        domain_area: &str,
    ) -> Result<Vec<SourceUnit>> {
        let records = read_records(config_json)?;
        let mut units = Vec::new();
        for record in records {
            let payload = record.get("payload").and_then(Value::as_object);
            let Some(payload) = payload else {
                continue;
            };
            if payload.get("domain_area").and_then(Value::as_str) == Some(domain_area) {
                if let Some(unit) = unit_from_record(&record) {
                    units.push(unit);
                }
            }
        }
        Ok(units)
    }
}

impl SourceAdapter for LearningProposalsAdapter {
    fn kind(&self) -> &str {
        "learning_proposals_jsonl"
    }

    fn discover_units(&self, _source_id: &str, config_json: &str) -> Result<Vec<SourceUnit>> {
        let records = read_records(config_json)?;
        Ok(records.iter().filter_map(unit_from_record).collect())
    }

    fn fetch_content(
        &self,
        _source_id: &str,
        unit: &SourceUnit,
        config_json: &str,
    ) -> Result<RawContent> {
        let records = read_records(config_json)?;
        let record = records
            .into_iter()
            .find(|record| learning_proposal_id(record) == Some(unit.external_id.as_str()))
            .with_context(|| format!("learning proposal not found: {}", unit.external_id))?;
        let bytes = serde_json::to_vec(&record).context("failed to serialize learning proposal")?;
        Ok(RawContent {
            external_id: unit.external_id.clone(),
            uri: unit.uri.clone(),
            mime_type: MIME_TYPE.to_string(),
            bytes,
            metadata_json: "{}".to_string(),
        })
    }
}

fn ledger_path(config_json: &str) -> Result<PathBuf> {
    let config: Value = serde_json::from_str(config_json).context("invalid config JSON")?;
    let path = config
        .get("path")
        .and_then(Value::as_str)
        .context("missing 'path' in config")?;
    Ok(PathBuf::from(path))
}

fn read_records(config_json: &str) -> Result<Vec<Value>> {
    let path = ledger_path(config_json)?;
    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "failed to read learning proposal ledger: {}",
            path.display()
        )
    })?;
    let mut records = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let record =
            serde_json::from_str(trimmed).context("invalid learning proposal JSONL row")?;
        records.push(record);
    }
    Ok(records)
}

fn learning_proposal_id(record: &Value) -> Option<&str> {
    record
        .get("payload")
        .and_then(Value::as_object)?
        .get("learning_proposal_id")
        .and_then(Value::as_str)
}

fn unit_from_record(record: &Value) -> Option<SourceUnit> {
    let learning_proposal_id = learning_proposal_id(record)?;
    Some(SourceUnit {
        external_id: learning_proposal_id.to_string(),
        uri: Some(format!("learning-proposals://{learning_proposal_id}")),
        mime_type_hint: Some(MIME_TYPE.to_string()),
    })
}
