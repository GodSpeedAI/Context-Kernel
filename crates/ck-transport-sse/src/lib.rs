use std::sync::Arc;

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

pub struct SseState {
    pub mcp: Arc<McpService>,
    pub auth: Arc<dyn AuthPort>,
}

pub async fn serve_sse(state: SseState, bind_addr: &str) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/api/v1/status", get(api_status))
        .route("/mcp", post(mcp_handler))
        .with_state(Arc::new(state));

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

async fn api_status(State(state): State<Arc<SseState>>) -> Json<Value> {
    match state.mcp.call_tool("get_status", json!({})) {
        Ok(result) => Json(result),
        Err(_) => Json(json!({"error": "status unavailable"})),
    }
}

async fn mcp_handler(
    State(state): State<Arc<SseState>>,
    headers: axum::http::HeaderMap,
    body: String,
) -> impl IntoResponse {
    if !state.auth.is_local_trust_mode() {
        let auth_header = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if !state.auth.validate_token(auth_header) {
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

    match state.mcp.call_tool(name, arguments) {
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
