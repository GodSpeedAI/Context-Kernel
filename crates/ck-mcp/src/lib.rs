use std::sync::Arc;

use anyhow::Result;
use base64::Engine;
use ck_app::{
    AppServices, IngestSourceRequest, QueryReferenceRequest, RecallContextRequest,
    RefreshSourceRequest, ResolveReferenceRequest, StoreContextRequest,
};
use ck_core::{ports::AuthPort, KernelStatus};
use serde_json::Value;
use thiserror::Error;

pub mod agentic_capability_loop;
pub use agentic_capability_loop::{AclContextAgent, ContextAgent};

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

#[derive(Debug, Clone)]
pub struct ToolTelemetry {
    pub tool_name: String,
    pub outcome: String,
    pub duration_ms: u64,
}

impl ToolTelemetry {
    pub fn success(tool_name: &str, duration_ms: u64) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            outcome: "success".to_string(),
            duration_ms,
        }
    }

    pub fn failure(tool_name: &str, duration_ms: u64) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            outcome: "failure".to_string(),
            duration_ms,
        }
    }
}

pub struct McpService {
    status: KernelStatus,
    app: Option<AppServices>,
    auth: Option<Arc<dyn AuthPort>>,
    /// WP-5 (F-05): the agentic-capability-loop context agent. When present,
    /// the `context_required` MCP tool is served (cited ContextPacketCreated).
    acl_context: Option<Arc<AclContextAgent>>,
}

impl McpService {
    pub fn new(status: KernelStatus) -> Self {
        Self {
            status,
            app: None,
            auth: None,
            acl_context: None,
        }
    }

    pub fn with_app(status: KernelStatus, app: AppServices) -> Self {
        Self {
            status,
            app: Some(app),
            auth: None,
            acl_context: None,
        }
    }

    pub fn with_app_and_auth(
        status: KernelStatus,
        app: AppServices,
        auth: Arc<dyn AuthPort>,
    ) -> Self {
        Self {
            status,
            app: Some(app),
            auth: Some(auth),
            acl_context: None,
        }
    }

    /// WP-5 (F-05): attach the ACL context agent so the `context_required`
    /// tool is served through the real binary (stdio/SSE).
    pub fn with_context_agent(mut self, agent: AclContextAgent) -> Self {
        self.acl_context = Some(Arc::new(agent));
        self
    }

    pub fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        self.dispatch_tool(name, arguments)
    }

    pub fn call_tool_authorized(
        &self,
        token: Option<&str>,
        name: &str,
        arguments: Value,
    ) -> Result<Value, McpError> {
        self.authorize(token, name, &arguments)?;
        self.dispatch_tool(name, arguments)
    }

    fn dispatch_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        let start = std::time::Instant::now();
        let result = match name {
            "get_status" => Ok(serde_json::to_value(&self.status).expect("status serializes")),
            "store_context" => self.handle_store_context(arguments),
            "recall_context" => self.handle_recall_context(arguments),
            "ingest_source" => self.handle_ingest_source(arguments),
            "refresh_source" => self.handle_refresh_source(arguments),
            "query_reference" | "get-library-docs" | "query-docs" => {
                self.handle_query_reference(arguments)
            }
            "resolve_reference" | "resolve-library-id" => self.handle_resolve_reference(arguments),
            // WP-5 (F-05): the loop-facing context agent, served over MCP.
            "context_required" => self.handle_context_required(arguments),
            other => Err(McpError::UnknownTool(other.to_string())),
        };
        let elapsed = start.elapsed().as_millis() as u64;
        match &result {
            Ok(_) => tracing::info!(
                tool_name = name,
                duration_ms = elapsed,
                outcome = "success",
                "tool call completed"
            ),
            Err(e) => {
                tracing::warn!(tool_name = name, duration_ms = elapsed, outcome = "error", error = %e, "tool call failed")
            }
        }
        result
    }

    fn authorize(
        &self,
        token: Option<&str>,
        name: &str,
        arguments: &Value,
    ) -> Result<(), McpError> {
        let Some(auth) = &self.auth else {
            return Ok(());
        };

        if !auth.authorize_tool(token, name) {
            return Err(McpError::InvalidArguments(format!(
                "unauthorized tool: {name}"
            )));
        }

        for corpus_id in corpus_ids_from_arguments(name, arguments) {
            if !auth.authorize_corpus(token, &corpus_id) {
                return Err(McpError::InvalidArguments(format!(
                    "unauthorized corpus: {corpus_id}"
                )));
            }
        }

        Ok(())
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

    /// WP-5 (F-05): serve the loop-facing `context_required` tool by delegating
    /// to the (previously unwired) `AclContextAgent`. Returns the
    /// `ContextPacketCreated` payload with citations + the POL-ACL-002
    /// authorization result, so SWE_SEED's context stage can satisfy
    /// POL-ACL-001/003 with a CK-sourced packet.
    fn handle_context_required(&self, arguments: Value) -> Result<Value, McpError> {
        let agent = self.acl_context.as_ref().ok_or_else(|| {
            McpError::Internal(
                "context_required tool not configured (no ACL context agent)".to_string(),
            )
        })?;

        // Build the ContextRequired envelope the agent consumes.
        let payload = serde_json::json!({
            "domain_model_hash": arguments["domain_model_hash"].as_str().unwrap_or(""),
            "work_request_id": arguments["work_request_id"].as_str().unwrap_or(""),
            "context_requirement_id": arguments["context_requirement_id"]
                .as_str()
                .unwrap_or(""),
            "corpus_id": arguments["corpus_id"].as_str().unwrap_or(""),
            "query": arguments["query"].as_str(),
            "max_results": arguments["max_results"].as_u64().unwrap_or(10) as u32,
            "is_private": arguments["is_private"].as_bool().unwrap_or(false),
            "scope_claim": arguments["scope_claim"].as_str(),
        });
        let envelope = serde_json::json!({
            "event_id": format!("evt_{}", chrono::Utc::now().timestamp_micros()),
            "event_type": "ContextRequired",
            "namespace": "agentic_capability_loop",
            "occurred_at": chrono::Utc::now().to_rfc3339(),
            "payload": payload,
        });
        let event: agentic_capability_loop::EventEnvelope =
            serde_json::from_value(envelope).map_err(|e| McpError::InvalidArguments(e.to_string()))?;

        let (packet, auth) = agent
            .handle_context_required(&event)
            .map_err(|e| McpError::Internal(format!("{e:#}")))?;

        Ok(serde_json::json!({
            "context_packet": packet.payload,
            "authorization": auth.payload,
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

fn corpus_ids_from_arguments(name: &str, arguments: &Value) -> Vec<String> {
    match name {
        "ingest_source" => arguments
            .get("source")
            .and_then(|s| s.get("corpus_id"))
            .and_then(Value::as_str)
            .map(|s| vec![s.to_string()])
            .unwrap_or_default(),
        "query_reference" | "get-library-docs" | "query-docs" => arguments
            .get("canonical_id")
            .or_else(|| arguments.get("context7CompatibleLibraryID"))
            .and_then(Value::as_str)
            .filter(|s| !s.starts_with('/'))
            .map(|s| vec![s.to_string()])
            .unwrap_or_default(),
        "resolve_reference" | "resolve-library-id" => arguments
            .get("source_scope")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ck_core::ports::AuthPort;
    use serde_json::json;

    struct TestAuth;

    impl AuthPort for TestAuth {
        fn validate_token(&self, token: &str) -> bool {
            token == "allowed"
        }

        fn is_local_trust_mode(&self) -> bool {
            false
        }

        fn authorize_tool(&self, token: Option<&str>, tool_name: &str) -> bool {
            token == Some("allowed") && matches!(tool_name, "get_status" | "query_reference")
        }

        fn authorize_corpus(&self, token: Option<&str>, corpus_id: &str) -> bool {
            token == Some("allowed") && corpus_id == "public_docs"
        }
    }

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

    #[test]
    fn context_required_unconfigured_reports_error() {
        // WP-5 (F-05): with no agent attached, the tool is reachable but reports
        // itself unconfigured (NOT unknown — the dispatch arm exists).
        let status = KernelStatus::placeholder();
        let service = McpService::new(status);
        let result = service.call_tool("context_required", json!({"corpus_id": "docs"}));
        match result {
            Err(McpError::Internal(msg)) => assert!(msg.contains("not configured")),
            Err(McpError::UnknownTool(_)) => panic!("context_required must be a known tool"),
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn context_required_returns_cited_packet_through_dispatch() {
        // F-05 teeth: the previously-dead AclContextAgent is reachable through
        // the served dispatch path and returns a cited ContextPacketCreated.
        use std::fs;
        let corpus_root = std::env::temp_dir().join(format!(
            "ck-ctx-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let docs = corpus_root.join("docs");
        fs::create_dir_all(&docs).unwrap();
        fs::write(docs.join("a.md"), "# Plan A\nDeploy via blue-green").unwrap();

        let status = KernelStatus::placeholder();
        let cleanup = corpus_root.clone();
        let agent = AclContextAgent {
            corpus_root,
            learning_proposals_ledger: None,
        };
        let service = McpService::new(status).with_context_agent(agent);

        let result = service.call_tool(
            "context_required",
            json!({
                "work_request_id": "wr-1",
                "context_requirement_id": "cr-1",
                "corpus_id": "docs",
                "query": "blue-green",
                "max_results": 5,
                "is_private": false,
            }),
        );
        let resp = result.expect("context_required should succeed with a corpus");
        let packet = &resp["context_packet"];
        assert_eq!(packet["work_request_id"], "wr-1");
        let citations = packet["citations"].as_array().expect("citations array");
        assert!(!citations.is_empty(), "packet must carry >=1 citation");
        assert_eq!(packet["authorized"], true);
        assert_eq!(resp["authorization"]["result"], "allow");
        fs::remove_dir_all(&cleanup).ok();
    }

    #[test]
    fn context_required_enforces_private_corpus_scope() {
        // POL-ACL-002: a private corpus without a scope_claim is denied and
        // returns ZERO citations (the packet is authorized:false).
        let status = KernelStatus::placeholder();
        let agent = AclContextAgent {
            corpus_root: std::env::temp_dir().join("nonexistent-ck-ctx"),
            learning_proposals_ledger: None,
        };
        let service = McpService::new(status).with_context_agent(agent);
        let result = service.call_tool(
            "context_required",
            json!({
                "corpus_id": "private_docs",
                "is_private": true,
                // no scope_claim
            }),
        );
        let resp = result.expect("private corpus without scope still responds");
        assert_eq!(resp["context_packet"]["authorized"], false);
        assert_eq!(resp["context_packet"]["citations"].as_array().unwrap().len(), 0);
        assert_eq!(resp["authorization"]["result"], "deny");
    }

    #[test]
    fn authorized_call_rejects_disallowed_tool() {
        let status = KernelStatus::placeholder();
        let service = McpService {
            status,
            app: None,
            auth: Some(Arc::new(TestAuth)),
            acl_context: None,
        };

        let result = service.call_tool_authorized(Some("allowed"), "ingest_source", json!({}));

        match result {
            Err(McpError::InvalidArguments(msg)) => {
                assert!(msg.contains("unauthorized tool"));
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn authorized_call_rejects_disallowed_corpus() {
        let status = KernelStatus::placeholder();
        let service = McpService {
            status,
            app: None,
            auth: Some(Arc::new(TestAuth)),
            acl_context: None,
        };

        let result = service.call_tool_authorized(
            Some("allowed"),
            "query_reference",
            json!({"canonical_id":"private_docs"}),
        );

        match result {
            Err(McpError::InvalidArguments(msg)) => {
                assert!(msg.contains("unauthorized corpus"));
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn tool_telemetry_records_name_and_outcome() {
        let event = ToolTelemetry::success("get_status", 12);
        assert_eq!(event.tool_name, "get_status");
        assert_eq!(event.outcome, "success");
        assert_eq!(event.duration_ms, 12);
    }

    #[test]
    fn tool_telemetry_failure_records_outcome() {
        let event = ToolTelemetry::failure("ingest_source", 500);
        assert_eq!(event.outcome, "failure");
        assert_eq!(event.duration_ms, 500);
    }
}
