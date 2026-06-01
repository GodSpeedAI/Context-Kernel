use anyhow::Result;
use ck_mcp::McpService;
use serde_json::{json, Value};

pub fn handle_line(service: &McpService, line: &str) -> Result<String> {
    let request: Value = serde_json::from_str(line)?;
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let response = if method == "tools/call" {
        let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        match service.call_tool(name, arguments) {
            Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
            Err(err) => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": "invalid_arguments",
                    "message": err.to_string(),
                    "retryable": false,
                    "details": {}
                }
            }),
        }
    } else {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": "invalid_arguments",
                "message": format!("unsupported method: {method}"),
                "retryable": false,
                "details": {}
            }
        })
    };

    Ok(serde_json::to_string(&response)?)
}
