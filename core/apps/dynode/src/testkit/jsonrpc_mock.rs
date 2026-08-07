use serde_json::{Value, json};

use crate::jsonrpc_types::JsonRpcCall;

impl JsonRpcCall {
    pub fn mock(id: u64, method: &str) -> Self {
        Self::mock_with_params(id, method, json!([]))
    }

    pub fn mock_with_params(id: u64, method: &str, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        }
    }
}
