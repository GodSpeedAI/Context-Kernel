use std::sync::{Arc, RwLock};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use ck_core::ports::AuthPort;
use ck_mcp::McpService;
use serde_json::{json, Value};
use tracing::{debug, info};

pub struct ReloadableSseState {
    mcp: RwLock<Arc<McpService>>,
    auth: RwLock<Arc<dyn AuthPort>>,
}

impl ReloadableSseState {
    pub fn new(mcp: Arc<McpService>, auth: Arc<dyn AuthPort>) -> Self {
        Self {
            mcp: RwLock::new(mcp),
            auth: RwLock::new(auth),
        }
    }

    pub fn mcp(&self) -> Arc<McpService> {
        self.mcp.read().unwrap().clone()
    }

    pub fn auth(&self) -> Arc<dyn AuthPort> {
        self.auth.read().unwrap().clone()
    }

    pub fn replace_service(&self, service: Arc<McpService>) {
        *self.mcp.write().unwrap() = service;
    }

    pub fn replace_auth(&self, auth: Arc<dyn AuthPort>) {
        *self.auth.write().unwrap() = auth;
    }
}

pub async fn serve_sse(state: Arc<ReloadableSseState>, bind_addr: &str) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/status", get(api_status_reloadable))
        .route("/mcp", post(mcp_handler_reloadable))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    info!(bind_addr, "SSE MCP server starting");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> Json<Value> {
    Json(json!({"status": "ok"}))
}

async fn readyz() -> Json<Value> {
    Json(json!({"ready": true}))
}

async fn api_status_reloadable(State(state): State<Arc<ReloadableSseState>>) -> Json<Value> {
    let mcp = state.mcp();
    match mcp.call_tool("get_status", json!({})) {
        Ok(result) => Json(result),
        Err(_) => Json(json!({"error": "status unavailable"})),
    }
}

async fn mcp_handler_reloadable(
    State(state): State<Arc<ReloadableSseState>>,
    headers: axum::http::HeaderMap,
    body: String,
) -> impl IntoResponse {
    let mcp = state.mcp();
    let auth = state.auth();

    let auth_header = headers.get("authorization").and_then(|v| v.to_str().ok());

    if !auth.is_local_trust_mode() {
        let auth_header = auth_header.unwrap_or("");
        if !auth.validate_token(auth_header) {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": {"code": "auth_required", "message": "Unauthorized", "retryable": false}
                })),
            );
        }
    }

    debug!(body_len = body.len(), "received MCP request");

    let request: Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {"code": "invalid_arguments", "message": e.to_string(), "retryable": false}
                })),
            );
        }
    };

    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if method != "tools/call" {
        return (
            StatusCode::OK,
            Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": "invalid_arguments", "message": format!("unsupported method: {method}"), "retryable": false}
            })),
        );
    }

    let params = request.get("params").cloned().unwrap_or(json!({}));
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

    match mcp.call_tool_authorized(auth_header, name, arguments) {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({"jsonrpc": "2.0", "id": id, "result": result})),
        ),
        Err(err) => (
            StatusCode::OK,
            Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": "internal", "message": err.to_string(), "retryable": false}
            })),
        ),
    }
}

#[cfg(test)]
mod reload_tests {
    use super::*;
    use ck_core::KernelStatus;

    #[test]
    fn reloadable_sse_state_swaps_service() {
        let status1 = KernelStatus::placeholder();
        let status2 = KernelStatus::placeholder();
        let first = Arc::new(McpService::new(status1));
        let second = Arc::new(McpService::new(status2));
        let auth: Arc<dyn AuthPort> = Arc::new(ck_auth::BearerAuth::no_auth());

        let state = Arc::new(ReloadableSseState::new(first.clone(), auth));

        assert!(Arc::ptr_eq(&state.mcp(), &first));

        state.replace_service(second.clone());

        assert!(Arc::ptr_eq(&state.mcp(), &second));
    }
}
