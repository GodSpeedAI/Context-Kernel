use anyhow::Result;
use base64::Engine;
use ck_app::{
    AppServices, IngestSourceRequest, QueryReferenceRequest, RecallContextRequest,
    RefreshSourceRequest, ResolveReferenceRequest, StoreContextRequest,
};
use ck_core::KernelStatus;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
    #[error("invalid_arguments: {0}")]
    InvalidArguments(String),
    #[error("{0}")]
    Internal(String),
}

impl From<anyhow::Error> for McpError {
    fn from(err: anyhow::Error) -> Self {
        McpError::Internal(err.to_string())
    }
}

pub struct McpService {
    status: KernelStatus,
    app: Option<AppServices>,
}

impl McpService {
    pub fn new(status: KernelStatus) -> Self {
        Self { status, app: None }
    }

    pub fn with_app(status: KernelStatus, app: AppServices) -> Self {
        Self {
            status,
            app: Some(app),
        }
    }

    pub fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        match name {
            "get_status" => Ok(serde_json::to_value(&self.status).expect("status serializes")),

            "store_context" => self.handle_store_context(arguments),

            "recall_context" => self.handle_recall_context(arguments),

            "ingest_source" => self.handle_ingest_source(arguments),

            "refresh_source" => self.handle_refresh_source(arguments),

            "query_reference" | "get-library-docs" | "query-docs" => {
                self.handle_query_reference(arguments)
            }

            "resolve_reference" | "resolve-library-id" => self.handle_resolve_reference(arguments),

            other => Err(McpError::UnknownTool(other.to_string())),
        }
    }

    fn require_app(&self) -> Result<&AppServices, McpError> {
        self.app
            .as_ref()
            .ok_or_else(|| McpError::Internal("application services not initialized".into()))
    }

    fn handle_store_context(&self, arguments: Value) -> Result<Value, McpError> {
        let app = self.require_app()?;

        let session_id = arguments["session_id"]
            .as_str()
            .unwrap_or("default")
            .to_string();
        let kind = arguments["kind"].as_str().unwrap_or("note").to_string();
        let text = arguments["text"].as_str().map(|s| s.to_string());
        let metadata = arguments
            .get("metadata")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        // Check for artifact_ref — content may be base64-encoded binary
        let artifact_content = arguments
            .get("artifact_ref")
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
            .map(|s| {
                // Try base64 decode first; fall back to raw UTF-8 bytes
                base64::engine::general_purpose::STANDARD
                    .decode(s)
                    .unwrap_or_else(|_| s.as_bytes().to_vec())
            });
        let artifact_content_type = arguments
            .get("artifact_ref")
            .and_then(|v| v.get("content_type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let req = StoreContextRequest {
            session_id,
            kind,
            text,
            artifact_content,
            artifact_content_type,
            metadata,
        };

        let resp = app.store_context(req)?;

        Ok(serde_json::json!({
            "event": {
                "event_id": resp.event.event_id,
                "session_id": resp.event.session_id,
                "kind": resp.event.kind,
                "stored_as": resp.event.stored_as,
                "artifact_id": resp.event.artifact_id,
                "created_at": resp.event.created_at,
            }
        }))
    }

    fn handle_recall_context(&self, arguments: Value) -> Result<Value, McpError> {
        let app = self.require_app()?;

        let session_id = arguments["session_id"].as_str().map(|s| s.to_string());
        let query = arguments["query"].as_str().unwrap_or("").to_string();
        let top_k = arguments["top_k"].as_i64().unwrap_or(5);
        let scope_str = arguments["scope"].as_str().unwrap_or("current");
        let scope = match scope_str {
            "recent" => ck_core::types::RecallScope::Recent,
            "all" => ck_core::types::RecallScope::All,
            _ => ck_core::types::RecallScope::Current,
        };

        let req = RecallContextRequest {
            session_id,
            query,
            scope,
            top_k,
        };

        let resp = app.recall_context(req)?;

        let results: Vec<Value> = resp
            .results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "event_id": r.event_id,
                    "kind": r.kind,
                    "text": r.text,
                    "artifact_id": r.artifact_id,
                    "score": r.score,
                    "score_breakdown": r.score_breakdown,
                    "created_at": r.created_at,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "results": results,
            "query_id": resp.query_id,
        }))
    }

    fn handle_ingest_source(&self, arguments: Value) -> Result<Value, McpError> {
        let app = self.require_app()?;

        let source = arguments
            .get("source")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        let source_id = source["source_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let source_kind = source["kind"]
            .as_str()
            .unwrap_or("filesystem_tree")
            .to_string();
        let adapter_config_json = serde_json::to_string(&source).unwrap_or_default();
        let corpus_id = source["corpus_id"].as_str().map(|s| s.to_string());

        let req = IngestSourceRequest {
            source_id,
            source_kind,
            adapter_config_json,
            corpus_id,
        };

        let resp = app.ingest_source(req)?;

        Ok(serde_json::json!({
            "job": {
                "job_id": resp.job.job_id,
                "source_id": resp.job.source_id,
                "status": resp.job.status,
                "trigger": resp.job.trigger,
            }
        }))
    }

    fn handle_query_reference(&self, arguments: Value) -> Result<Value, McpError> {
        let app = self.require_app()?;

        // Handle compatibility alias mapping
        let canonical_id = arguments["canonical_id"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| {
                arguments["context7CompatibleLibraryID"]
                    .as_str()
                    .map(|s| s.to_string())
            });
        let query = arguments["query"]
            .as_str()
            .or_else(|| arguments["topic"].as_str())
            .unwrap_or("")
            .to_string();
        let top_k = arguments["top_k"].as_i64().unwrap_or(8);
        let mode = arguments["mode"].as_str().map(|s| s.to_string());
        let version = arguments["version"].as_str().map(|s| s.to_string());
        let filters = arguments
            .get("filters")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let req = QueryReferenceRequest {
            canonical_id,
            query,
            mode,
            top_k,
            version,
            filters,
        };

        let resp = app.query_reference(req)?;

        let results: Vec<Value> = resp
            .results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "result_id": r.result_id,
                    "canonical_id": r.canonical_id,
                    "document_id": r.document_id,
                    "chunk_id": r.chunk_id,
                    "title": r.title,
                    "heading": r.heading,
                    "excerpt": r.excerpt,
                    "citation": r.citation,
                    "score": r.score,
                    "score_breakdown": r.score_breakdown,
                    "metadata": r.metadata,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "results": results,
            "query_id": resp.query_id,
            "mode": resp.mode,
            "truncated": resp.truncated,
        }))
    }

    fn handle_resolve_reference(&self, arguments: Value) -> Result<Value, McpError> {
        let app = self.require_app()?;

        // Handle compatibility alias mapping
        let query = arguments["query"]
            .as_str()
            .or_else(|| arguments["libraryName"].as_str())
            .unwrap_or("")
            .to_string();
        let kind = arguments["kind"].as_str().map(|s| s.to_string());
        let version = arguments["version"].as_str().map(|s| s.to_string());
        let source_scope = arguments["source_scope"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        });
        let limit = arguments["limit"].as_i64().unwrap_or(10);

        let req = ResolveReferenceRequest {
            query,
            kind,
            version,
            source_scope,
            limit,
        };

        let resp = app.resolve_reference(req)?;

        let candidates: Vec<Value> = resp
            .candidates
            .iter()
            .map(|c| {
                serde_json::json!({
                    "canonical_id": c.canonical_id,
                    "provider": c.provider,
                    "name": c.name,
                    "version": c.version,
                    "aliases": c.aliases,
                    "summary": c.summary,
                    "trust_score": c.trust_score,
                    "score": c.score,
                    "score_breakdown": c.score_breakdown,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "candidates": candidates,
            "query_id": resp.query_id,
            "mode": resp.mode,
        }))
    }

    fn handle_refresh_source(&self, arguments: Value) -> Result<Value, McpError> {
        let app = self.require_app()?;

        let source_id = arguments["source_id"].as_str().unwrap_or("").to_string();

        if source_id.is_empty() {
            return Err(McpError::InvalidArguments(
                "source_id is required".to_string(),
            ));
        }

        let req = RefreshSourceRequest { source_id };

        let resp = app.refresh_source(req)?;

        Ok(serde_json::json!({
            "job": {
                "job_id": resp.job.job_id,
                "source_id": resp.job.source_id,
                "status": resp.job.status,
                "trigger": resp.job.trigger,
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_status_returns_stable_contract_shape() {
        let status = KernelStatus::placeholder();
        let service = McpService::new(status);

        let value = service.call_tool("get_status", json!({})).unwrap();

        assert_eq!(value["service"]["name"], "context-kernel");
        assert_eq!(value["storage"]["kind"], "sqlite");
        assert_eq!(value["retrieval"]["default_mode"], "hybrid");
    }

    #[test]
    fn unknown_tool_returns_error() {
        let status = KernelStatus::placeholder();
        let service = McpService::new(status);

        let result = service.call_tool("nonexistent_tool", json!({}));
        assert!(result.is_err());
    }

    #[test]
    fn store_context_without_app_returns_error() {
        let status = KernelStatus::placeholder();
        let service = McpService::new(status);

        let result = service.call_tool(
            "store_context",
            json!({"session_id": "s1", "kind": "note", "text": "hello"}),
        );
        assert!(result.is_err());
    }

    #[test]
    fn compatibility_aliases_resolve_library_id() {
        let status = KernelStatus::placeholder();
        let service = McpService::new(status);

        // Should not return UnknownTool
        let result = service.call_tool("resolve-library-id", json!({"libraryName": "react"}));
        assert!(result.is_err());
        match result {
            Err(McpError::Internal(_)) | Err(McpError::InvalidArguments(_)) => {} // expected - no app services
            Err(McpError::UnknownTool(_)) => panic!("should not be unknown tool"),
            Ok(_) => panic!("should not succeed without app"),
        }
    }

    #[test]
    fn compatibility_aliases_get_library_docs() {
        let status = KernelStatus::placeholder();
        let service = McpService::new(status);

        let result = service.call_tool(
            "get-library-docs",
            json!({"context7CompatibleLibraryID": "/vercel/next.js", "topic": "routing"}),
        );
        assert!(result.is_err());
        match result {
            Err(McpError::Internal(_)) => {}
            Err(McpError::UnknownTool(_)) => panic!("should not be unknown tool"),
            _ => panic!("unexpected"),
        }
    }

    #[test]
    fn refresh_source_without_app_returns_error() {
        let status = KernelStatus::placeholder();
        let service = McpService::new(status);

        let result = service.call_tool("refresh_source", json!({"source_id": "test"}));
        assert!(result.is_err());
        match result {
            Err(McpError::Internal(_)) => {}
            Err(McpError::UnknownTool(_)) => panic!("should not be unknown tool"),
            _ => panic!("unexpected"),
        }
    }
}
