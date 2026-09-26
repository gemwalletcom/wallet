use gem_client::{ClientError, testkit::MockClient};
use serde_json::Value;

use crate::client::JsonRpcClient;

pub fn mock_jsonrpc_client<F>(handler: F) -> JsonRpcClient<MockClient>
where
    F: Fn(&str, &Value) -> Result<Value, ClientError> + Send + Sync + 'static,
{
    JsonRpcClient::new(mock_jsonrpc_transport(handler))
}

pub fn mock_jsonrpc_transport<F>(handler: F) -> MockClient
where
    F: Fn(&str, &Value) -> Result<Value, ClientError> + Send + Sync + 'static,
{
    MockClient::new().with_post(move |_, body| {
        let request: Value = serde_json::from_slice(body).map_err(|error| ClientError::Serialization(error.to_string()))?;
        let method = request.get("method").and_then(Value::as_str).ok_or_else(|| ClientError::Serialization("missing method".to_string()))?;
        let params = request.get("params").cloned().unwrap_or(Value::Null);
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let result = handler(method, &params)?;

        serde_json::to_vec(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result,
        }))
        .map_err(|error| ClientError::Serialization(error.to_string()))
    })
}
