use serde_json::{Value, json};

use crate::cache::decoder::ETH_CALL;
use crate::jsonrpc_types::{JsonRpcCall, RequestType};

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

    pub fn mock_eth_call(to: &str, data: &str) -> Self {
        Self::mock_with_params(1, ETH_CALL, json!([{ "to": to, "data": data }, "latest"]))
    }
}

impl RequestType {
    pub fn mock_jsonrpc(method: &str) -> Self {
        Self::from_request("POST", "/".to_string(), serde_json::to_vec(&json!({ "jsonrpc": "2.0", "method": method, "params": [], "id": 1 })).unwrap())
    }

    pub fn mock_regular(path: &str, method: &str, body: &[u8]) -> Self {
        Self::Regular {
            path: path.to_string(),
            method: method.to_string(),
            body: body.to_vec(),
        }
    }
}
