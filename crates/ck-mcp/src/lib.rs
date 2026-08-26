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
pub use agentic_capability_loop::AclContextAgent;

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

    /// Serve the frozen E2/E3 boundary (convergence T02).
    ///
    /// Ingress: the arguments ARE the canonical E2 envelope produced by the
    /// exclusive authoritative producer (`swe_seed` for `ContextRequired`).
    /// CK rejects wrong-producer, non-v1, placeholder-identity, or
    /// correlation-less requests fail-closed.
    ///
    /// Egress: the `ContextPacketCreated` payload inside a canonical v1
    /// envelope stamped by `context-kernel`, recording the E2 request event id
    /// as causal parent (`caused_by:`), with an EXPLICIT acquisition outcome.
    /// Zero-citation results are never silent: when `required=true` the
    /// response carries `outcome="no_context_governed"` plus a machine-readable
    /// reason; only `outcome="cited"` is successful acquisition. Authority
    /// fields are pass-through references — CK never decides authority.
    fn handle_context_required(&self, arguments: Value) -> Result<Value, McpError> {
        let agent = self.acl_context.as_ref().ok_or_else(|| {
            McpError::Internal(
                "context_required tool not configured (no ACL context agent)".to_string(),
            )
        })?;

        // --- E2 envelope ingress gates ------------------------------------
        let schema_version = arguments["schema_version"].as_str().unwrap_or("");
        if schema_version != agentic_capability_loop::SCHEMA_VERSION_V1 {
            return Err(McpError::InvalidArguments(
                "request must be a v1 canonical envelope".to_string(),
            ));
        }
        match arguments["event_type"].as_str() {
            Some("ContextRequired") => {}
            other => {
                return Err(McpError::InvalidArguments(format!(
                    "expected ContextRequired request, got {other:?}"
                )))
            }
        }
        let source_agent = arguments["source_agent"].as_str().unwrap_or("");
        if source_agent != "swe-seed" && source_agent != "swe_seed" {
            return Err(McpError::InvalidArguments(format!(
                "ContextRequired may only be produced by swe_seed (got {source_agent:?})"
            )));
        }
        let request_event_id = arguments["event_id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                McpError::InvalidArguments("request envelope missing event_id".to_string())
            })?
            .to_string();

        let payload_obj = arguments["payload"].as_object().ok_or_else(|| {
            McpError::InvalidArguments("request envelope missing payload".to_string())
        })?;

        let domain_model_hash = payload_obj
            .get("domain_model_hash")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        agentic_capability_loop::verify_domain_identity(&domain_model_hash)
            .map_err(|e| McpError::InvalidArguments(format!("domain identity rejected: {e}")))?;

        let work_request_id = payload_obj
            .get("work_request_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let context_requirement_id = payload_obj
            .get("context_requirement_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if work_request_id.is_empty() || context_requirement_id.is_empty() {
            return Err(McpError::InvalidArguments(
                "work_request_id and context_requirement_id are required correlation".to_string(),
            ));
        }

        let payload = agentic_capability_loop::ContextRequiredPayload {
            domain_model_hash,
            work_request_id,
            context_requirement_id,
            corpus_id: payload_obj
                .get("corpus_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            query: payload_obj
                .get("query")
                .and_then(Value::as_str)
                .map(String::from),
            max_results: payload_obj
                .get("max_results")
                .and_then(Value::as_u64)
                .unwrap_or(10) as u32,
        };

        let authority_decision_id = payload_obj
            .get("authority_decision_id")
            .and_then(Value::as_str);
        let authority_reference = payload_obj
            .get("authority_reference")
            .and_then(Value::as_str);
        // Frozen E3 semantics key off REQUIRED context; the tool name already
        // declares the caller's intent, but an explicit flag lets optional
        // acquisitions stay legitimate.
        let required = arguments["required"].as_bool().unwrap_or(true);

        let acquisition = agent
            .acquire_context(&payload, authority_decision_id, authority_reference)
            .map_err(|e| McpError::Internal(format!("{e:#}")))?;

        let (packet, outcome, reason): (_, &str, Option<&str>) = match acquisition {
            agentic_capability_loop::ContextAcquisition::Cited(p) => (p, "cited", None),
            agentic_capability_loop::ContextAcquisition::NoContext(p, r) => {
                let outcome = if required {
                    "no_context_governed"
                } else {
                    "empty_optional"
                };
                (p, outcome, Some(r.as_str()))
            }
        };

        let envelope =
            agentic_capability_loop::context_packet_envelope(Some(&request_event_id), &packet);

        Ok(serde_json::json!({
            "context_envelope": envelope,
            "context_packet": serde_json::to_value(&packet).unwrap_or_default(),
            "outcome": outcome,
            "reason": reason,
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
        // Canonical E2 requests nest corpus_id inside the envelope payload;
        // legacy flat requests carry it top-level. Both must surface so the
        // corpus allowlist is enforced on every wire shape.
        "context_required" => arguments
            .get("corpus_id")
            .or_else(|| arguments.get("payload").and_then(|p| p.get("corpus_id")))
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(|s| vec![s.to_string()])
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

        let mut req = e2_request("docs", "wr-1", "cr-1", GOOD_DIGEST);
        req["payload"]["query"] = json!("blue-green");
        req["required"] = json!(true);
        let result = service.call_tool("context_required", req);
        let resp = result.expect("context_required should succeed with a corpus");
        assert_eq!(
            resp["outcome"], "cited",
            "cited acquisition must be explicit"
        );
        let packet = &resp["context_packet"];
        assert_eq!(packet["work_request_id"], "wr-1");
        let citations = packet["citations"].as_array().expect("citations array");
        assert!(!citations.is_empty(), "packet must carry >=1 citation");

        // Canonical E3 envelope shape (convergence T02): authoritative producer
        // stamp, identity echo, and the E2 request recorded as causal parent.
        let env = &resp["context_envelope"];
        assert_eq!(env["schema_version"], "v1");
        assert_eq!(env["event_type"], "ContextPacketCreated");
        assert_eq!(env["source_agent"], "context-kernel");
        assert_eq!(env["payload"]["work_request_id"], "wr-1");
        assert_eq!(env["payload"]["domain_model_hash"], GOOD_DIGEST);
        let chain = env["provenance"]["chain"].as_array().expect("chain");
        assert!(
            chain
                .iter()
                .any(|e| e == &json!("caused_by:11111111-2222-3333-4444-555555555555")),
            "E2 request event id must be recorded as causal parent, chain={chain:?}"
        );
        let key = env["idempotency_key"].as_str().expect("idempotency key");
        assert_eq!(key.len(), 64, "idempotency key is sha256-hex");
        fs::remove_dir_all(&cleanup).ok();
    }

    #[test]
    fn context_required_no_longer_emits_authorization() {
        let status = KernelStatus::placeholder();
        let agent = AclContextAgent {
            corpus_root: std::env::temp_dir().join("nonexistent-ck-ctx"),
            learning_proposals_ledger: None,
        };
        let service = McpService::new(status).with_context_agent(agent);
        let mut req = e2_request("docs", "wr-x", "cr-x", GOOD_DIGEST);
        req["required"] = json!(false);
        let result = service.call_tool("context_required", req);
        let resp = result.expect("context_required responds");
        assert!(
            resp.get("authorization").is_none(),
            "CK must not emit authorization decisions"
        );
        assert!(
            resp["context_packet"]["authorized"].as_bool().is_none(),
            "packet must not carry an authorized bool"
        );
    }

    // --- convergence T02 boundary teeth ------------------------------------

    const GOOD_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    /// Build a canonical E2 request envelope (producer: swe_seed).
    fn e2_request(corpus_id: &str, wr: &str, cr: &str, digest: &str) -> Value {
        json!({
            "schema_version": "v1",
            "event_id": "11111111-2222-3333-4444-555555555555",
            "source_agent": "swe-seed",
            "event_type": "ContextRequired",
            "occurred_at": "2026-08-25T00:00:00Z",
            "payload": {
                "domain_model_hash": digest,
                "namespace": "agentic_capability_loop",
                "work_request_id": wr,
                "context_requirement_id": cr,
                "corpus_id": corpus_id,
                "query": null,
                "max_results": 5,
            },
        })
    }

    fn ctx_service_with(corpus_root: std::path::PathBuf) -> McpService {
        let agent = AclContextAgent {
            corpus_root,
            learning_proposals_ledger: None,
        };
        McpService::new(KernelStatus::placeholder()).with_context_agent(agent)
    }

    #[test]
    fn t02_placeholder_identities_fail_closed_at_dispatch() {
        let service = ctx_service_with(std::env::temp_dir().join("t02-placeholder"));
        for bad in [
            "0".repeat(64),
            agentic_capability_loop::FALLBACK_PSEUDO_HASH.to_string(),
        ] {
            let mut req = e2_request("docs", "wr-1", "cr-1", &bad);
            req["payload"]["domain_model_hash"] = json!(bad);
            let result = service.call_tool("context_required", req);
            match result {
                Err(McpError::InvalidArguments(msg)) => {
                    assert!(msg.contains("identity rejected"), "{msg}");
                }
                other => panic!("placeholder identity {bad} must fail closed: {other:?}"),
            }
        }
    }

    #[test]
    fn t02_malformed_identity_fails_closed_at_dispatch() {
        let service = ctx_service_with(std::env::temp_dir().join("t02-malformed"));
        let result = service.call_tool(
            "context_required",
            e2_request("docs", "wr-1", "cr-1", "deadbeef"),
        );
        assert!(matches!(result, Err(McpError::InvalidArguments(_))));
    }

    #[test]
    fn t02_missing_correlation_fails_closed() {
        let service = ctx_service_with(std::env::temp_dir().join("t02-correlation"));
        let mut req = e2_request("docs", "", "", GOOD_DIGEST);
        req["payload"]["work_request_id"] = json!("");
        req["payload"]["context_requirement_id"] = json!("");
        let result = service.call_tool("context_required", req);
        match result {
            Err(McpError::InvalidArguments(msg)) => {
                assert!(msg.contains("correlation"), "{msg}");
            }
            other => panic!("missing correlation must fail closed: {other:?}"),
        }
    }

    #[test]
    fn t02_non_authoritative_producer_fails_closed() {
        // Only swe_seed may emit ContextRequired into CK.
        let service = ctx_service_with(std::env::temp_dir().join("t02-producer"));
        let mut req = e2_request("docs", "wr-p", "cr-p", GOOD_DIGEST);
        req["source_agent"] = json!("realitytrace");
        let result = service.call_tool("context_required", req);
        match result {
            Err(McpError::InvalidArguments(msg)) => {
                assert!(msg.contains("swe_seed"), "{msg}");
            }
            other => panic!("forged producer must fail closed: {other:?}"),
        }
    }

    #[test]
    fn t02_required_zero_citation_is_governed_outcome_never_silent_success() {
        // Missing corpus configuration + required context: frozen E3 forbids
        // this settling as successful acquisition. Root exists, corpus absent.
        let root = std::env::temp_dir().join("t02-no-such-corpus");
        std::fs::create_dir_all(&root).ok();
        let service = ctx_service_with(root);
        let mut req = e2_request("absent-corpus", "wr-9", "cr-9", GOOD_DIGEST);
        req["required"] = json!(true);
        let resp = service
            .call_tool("context_required", req)
            .expect("response must exist");
        assert_eq!(
            resp["outcome"], "no_context_governed",
            "required zero-citation must be an explicit governed outcome"
        );
        assert_eq!(
            resp["reason"], "corpus_not_found",
            "machine-readable reason required"
        );
        assert!(resp["context_packet"]["citations"]
            .as_array()
            .expect("citations")
            .is_empty());
    }

    #[test]
    fn t02_optional_zero_citation_is_explicit_optional_not_governed_failure() {
        // Root itself unconfigured: still explicit, still not silent success.
        let service = ctx_service_with(std::env::temp_dir().join("t02-optional"));
        let mut req = e2_request("absent-corpus", "wr-8", "cr-8", GOOD_DIGEST);
        req["required"] = json!(false);
        let resp = service
            .call_tool("context_required", req)
            .expect("response must exist");
        assert_eq!(resp["outcome"], "empty_optional");
        assert_eq!(resp["reason"], "corpus_root_unconfigured");
    }

    #[test]
    fn context_required_corpus_id_uses_authorization_arm() {
        // Fix 1 (unit-level): corpus_ids_from_arguments must surface the
        // context_required corpus_id so the authorize() allowlist check runs.
        // Without the dedicated arm this falls through to `_ => Vec::new()` and
        // the corpus is never checked.
        let ids = corpus_ids_from_arguments("context_required", &json!({"corpus_id": "x"}));
        assert_eq!(ids, vec!["x".to_string()]);

        // Empty/missing corpus_id yields no ids (no false positives).
        assert!(corpus_ids_from_arguments("context_required", &json!({})).is_empty());
        assert!(
            corpus_ids_from_arguments("context_required", &json!({"corpus_id": ""})).is_empty()
        );
    }

    #[test]
    fn context_required_rejects_path_traversal_outside_corpus_root() {
        // Fix 2 teeth: a corpus_id that escapes corpus_root via traversal must
        // be rejected, NOT return citations read from outside the root.
        use std::fs;
        let nonce = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let corpus_root = std::env::temp_dir().join(format!("ck-ctx-trav-{nonce}"));
        // Sibling dir OUTSIDE corpus_root containing a secret .md file.
        let sibling = std::env::temp_dir().join(format!("ck-ctx-outside-{nonce}"));
        fs::create_dir_all(&corpus_root).unwrap();
        fs::create_dir_all(&sibling).unwrap();
        fs::write(sibling.join("secret.md"), "TOP SECRET CONTENT").unwrap();

        let status = KernelStatus::placeholder();
        let cleanup_root = corpus_root.clone();
        let cleanup_sibling = sibling.clone();
        let agent = AclContextAgent {
            corpus_root,
            learning_proposals_ledger: None,
        };
        let service = McpService::new(status).with_context_agent(agent);

        // corpus_id "../<sibling>" resolves outside corpus_root via traversal.
        let sibling_name = sibling.file_name().unwrap().to_str().unwrap().to_string();
        let escape = format!("../{sibling_name}");
        let result = service.call_tool(
            "context_required",
            json!({
                "corpus_id": escape,
                "query": "SECRET",
            }),
        );
        fs::remove_dir_all(&cleanup_root).ok();
        fs::remove_dir_all(&cleanup_sibling).ok();

        assert!(
            result.is_err(),
            "path-traversal corpus_id must be rejected, got: {result:?}"
        );
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
