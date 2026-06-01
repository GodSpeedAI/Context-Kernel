use ck_core::KernelStatus;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpError {
    #[error("unknown tool: {0}")]
    UnknownTool(String),
}

pub struct McpService {
    status: KernelStatus,
}

impl McpService {
    pub fn new(status: KernelStatus) -> Self {
        Self { status }
    }

    pub fn call_tool(&self, name: &str, _arguments: Value) -> Result<Value, McpError> {
        match name {
            "get_status" => Ok(serde_json::to_value(&self.status).expect("status serializes")),
            other => Err(McpError::UnknownTool(other.to_string())),
        }
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

        assert_eq!(value["service"]["name"], "context-kernal");
        assert_eq!(value["storage"]["kind"], "sqlite");
        assert_eq!(value["retrieval"]["default_mode"], "hybrid");
    }
}
