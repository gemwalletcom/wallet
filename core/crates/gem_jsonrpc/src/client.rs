use crate::types::{ERROR_CLIENT_ERROR, ERROR_INTERNAL_ERROR, JsonRpcError, JsonRpcRequest, JsonRpcResult, JsonRpcResults, ToJsonRpcRequest};
use gem_client::{Client, ClientError, ClientExt};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, time::SystemTime};

#[derive(Clone, Debug)]
pub struct JsonRpcClient<C: Client + Clone> {
    client: C,
}

impl From<ClientError> for JsonRpcError {
    fn from(value: ClientError) -> Self {
        JsonRpcError {
            code: ERROR_CLIENT_ERROR,
            message: value.to_string(),
            cause: None,
        }
    }
}

impl<C: Client + Clone> JsonRpcClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn request<U: DeserializeOwned + Send, T: ToJsonRpcRequest>(&self, request: T) -> Result<U, JsonRpcError> {
        self.request_with_cache(&request, None).await?.take()
    }

    pub async fn request_with_cache<U: DeserializeOwned + Send, T: ToJsonRpcRequest>(&self, request: &T, ttl: Option<u64>) -> Result<JsonRpcResult<U>, JsonRpcError> {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        let request = request.to_jsonrpc_request(timestamp);
        self.send_request(request, ttl).await
    }

    pub async fn batch_request<U: DeserializeOwned + Send, T: ToJsonRpcRequest>(&self, requests: Vec<T>) -> Result<JsonRpcResults<U>, JsonRpcError> {
        let requests: Vec<JsonRpcRequest> = requests.iter().enumerate().map(|(index, request)| request.to_jsonrpc_request(index as u64 + 1)).collect();
        if requests.is_empty() {
            return Ok(Default::default());
        }

        let mut results: Vec<JsonRpcResult<U>> = self.client.post("", &requests).await?;
        results.sort_by_key(JsonRpcResult::id);

        if results.len() != requests.len() || results.iter().enumerate().any(|(index, result)| result.id() != Some(index as u64 + 1)) {
            return Err(JsonRpcError {
                message: "Invalid batch response IDs".into(),
                code: ERROR_INTERNAL_ERROR,
                cause: None,
            });
        }

        Ok(JsonRpcResults(results))
    }

    async fn send_request<T: DeserializeOwned + Send>(&self, request: JsonRpcRequest, ttl: Option<u64>) -> Result<JsonRpcResult<T>, JsonRpcError> {
        let mut headers = HashMap::new();
        if let Some(ttl_seconds) = ttl {
            headers.insert("Cache-Control".to_string(), format!("max-age={}", ttl_seconds));
        }

        let result: JsonRpcResult<T> = self.client.post_with_headers("", &request, headers).await?;
        Ok(result)
    }
}

#[cfg(feature = "reqwest")]
impl JsonRpcClient<gem_client::ReqwestClient> {
    pub fn new_reqwest(url: String) -> Self {
        use gem_client::ReqwestClient;
        let reqwest_client = gem_client::builder().build().expect("Failed to build reqwest client");
        let client = ReqwestClient::new(url, reqwest_client);
        Self { client }
    }
}
