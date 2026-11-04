use crate::types::{ERROR_CLIENT_ERROR, ERROR_INTERNAL_ERROR, JsonRpcError, JsonRpcRequest, JsonRpcResult, JsonRpcResults, ToJsonRpcRequest};
use gem_client::{Client, ClientError, ClientExt};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct JsonRpcClient<C: Client + Clone> {
    client: C,
}

impl From<ClientError> for JsonRpcError {
    fn from(value: ClientError) -> Self {
        decode_client_error(value)
    }
}

fn decode_client_error<E>(value: ClientError) -> E
where
    E: DeserializeOwned + From<JsonRpcError> + AsMut<JsonRpcError> + Send,
{
    match &value {
        ClientError::Http { body, .. } | ClientError::Response { body, .. } => {
            if let Ok(JsonRpcResult::Error(response)) = serde_json::from_slice::<JsonRpcResult<Value, E>>(body) {
                return response.error;
            }
            if let Ok(mut responses) = serde_json::from_slice::<Vec<JsonRpcResult<Value, E>>>(body) {
                responses.sort_by_key(JsonRpcResult::id);
                if let Err(error) = JsonRpcResults(responses).take_all() {
                    return error;
                }
            }
        }
        ClientError::Network(_) | ClientError::Timeout | ClientError::Serialization(_) => {}
    }
    JsonRpcError {
        code: ERROR_CLIENT_ERROR,
        message: value.to_string(),
    }
    .into()
}

impl<C: Client + Clone> JsonRpcClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn request<U: DeserializeOwned + Send, T: ToJsonRpcRequest>(&self, request: T) -> Result<U, JsonRpcError> {
        self.request_with_error(request).await
    }

    pub async fn request_with_error<U, T, E>(&self, request: T) -> Result<U, E>
    where
        U: DeserializeOwned + Send,
        T: ToJsonRpcRequest,
        E: DeserializeOwned + From<JsonRpcError> + AsMut<JsonRpcError> + Send,
    {
        let timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();
        let result: JsonRpcResult<U, E> = self.send_request::<U, E>(&request.to_jsonrpc_request(timestamp)).await?;
        result.take()
    }

    pub async fn requests<U: DeserializeOwned + Send, T: ToJsonRpcRequest>(&self, requests: Vec<T>) -> Result<JsonRpcResults<U>, JsonRpcError> {
        let requests: Vec<JsonRpcRequest> = requests.iter().enumerate().map(|(index, request)| request.to_jsonrpc_request(index as u64 + 1)).collect();
        let mut results: Vec<JsonRpcResult<U>> = match requests.as_slice() {
            [] => return Ok(Default::default()),
            [request] => return Ok(JsonRpcResults(vec![self.send_request(request).await?])),
            requests => self.client.post("", &requests).await?,
        };
        results.sort_by_key(JsonRpcResult::id);

        if results.len() != requests.len() || results.iter().enumerate().any(|(index, result)| result.id() != Some(index as u64 + 1)) {
            return Err(JsonRpcError {
                message: "Invalid batch response IDs".into(),
                code: ERROR_INTERNAL_ERROR,
            });
        }

        Ok(JsonRpcResults(results))
    }

    async fn send_request<T, E>(&self, request: &JsonRpcRequest) -> Result<JsonRpcResult<T, E>, E>
    where
        T: DeserializeOwned + Send,
        E: DeserializeOwned + From<JsonRpcError> + AsMut<JsonRpcError> + Send,
    {
        self.client.post::<_, JsonRpcResult<T, E>>("", request).await.map_err(decode_client_error)
    }
}

#[cfg(feature = "reqwest")]
impl JsonRpcClient<gem_client::ReqwestClient> {
    pub fn new_reqwest(url: String) -> Self {
        use gem_client::ReqwestClient;
        let client = ReqwestClient::new(url, gem_client::reqwest_client());
        Self { client }
    }
}

#[cfg(test)]
mod tests {
    use gem_client::{Response, deserialize_response, testkit::MockClient};
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::types::TypedJsonRpcError;

    #[derive(Debug, Default, Deserialize, Serialize)]
    struct Payload {
        #[serde(skip_serializing_if = "Option::is_none")]
        cause: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<String>,
    }

    type MockRpcError = TypedJsonRpcError<Payload>;

    struct Method(&'static str);

    impl ToJsonRpcRequest for Method {
        fn method(&self) -> &'static str {
            self.0
        }

        fn params(&self) -> Value {
            Value::Array(vec![])
        }
    }

    #[tokio::test]
    async fn test_request_with_error_preserves_transport_failure() {
        let client = JsonRpcClient::new(MockClient::new().with_post(|_, _| Err(ClientError::Timeout)));

        let error = client.request_with_error::<Value, _, MockRpcError>(Method("eth_call")).await.unwrap_err();

        assert_eq!(serde_json::to_value(error).unwrap(), serde_json::json!({"code":ERROR_CLIENT_ERROR,"message":"Timeout error"}));
    }

    #[tokio::test]
    async fn test_requests() {
        let client = |batch: bool| {
            JsonRpcClient::new(MockClient::new().with_post(move |_, body| {
                let body: Value = serde_json::from_slice(body).unwrap();
                assert_eq!(body.is_array(), batch);
                let response = |request: &Value| serde_json::json!({ "jsonrpc": "2.0", "id": request["id"], "result": request["method"] });
                let body = match body {
                    Value::Array(requests) => Value::Array(requests.iter().map(response).collect()),
                    request => response(&request),
                };
                Ok(serde_json::to_vec(&body).unwrap())
            }))
        };

        let single = client(false).requests::<String, _>(vec![Method("a")]).await.unwrap();
        let batch = client(true).requests::<String, _>(vec![Method("a"), Method("b")]).await.unwrap();

        assert_eq!(single.take_all().unwrap(), vec!["a"]);
        assert_eq!(batch.take_all().unwrap(), vec!["a", "b"]);
    }

    #[test]
    fn test_http_status_preserves_jsonrpc_error() {
        let errors = [
            serde_json::json!({"code":-32000,"message":"Account not found","cause":{"name":"UNKNOWN_ACCOUNT"}}),
            serde_json::json!({"code":3,"message":"execution reverted","data":"0x77209fe8"}),
        ];
        for status in [None, Some(200), Some(400), Some(404), Some(500)] {
            for id in [serde_json::json!(1), serde_json::json!("1"), Value::Null] {
                for expected in &errors {
                    let response = Response {
                        status,
                        data: serde_json::to_vec(&serde_json::json!({"jsonrpc":"2.0","id":id,"error":expected})).unwrap(),
                    };
                    let error = deserialize_response::<JsonRpcResult<Value, MockRpcError>>(&response).map_err(decode_client_error).and_then(JsonRpcResult::take).unwrap_err();

                    assert_eq!(&serde_json::to_value(error).unwrap(), expected);
                }
            }
        }
    }

    #[test]
    fn test_http_error_preserves_jsonrpc_batch_error() {
        for status in [400, 404, 500] {
            let response = Response {
                status: Some(status),
                data: br#"[{"id":2,"error":{"code":-32000,"message":"Provider detail","cause":{"name":"X"},"data":"0x01"}},{"id":1,"result":7}]"#.to_vec(),
            };
            let error = deserialize_response::<Vec<JsonRpcResult<Value, MockRpcError>>>(&response).map_err(decode_client_error::<MockRpcError>).unwrap_err();

            assert_eq!(
                serde_json::to_value(error).unwrap(),
                serde_json::json!({"code":-32000,"message":"batch request [1]: Provider detail","cause":{"name":"X"},"data":"0x01"})
            );
        }
    }

    #[test]
    fn test_http_error_rejects_jsonrpc_batch_success() {
        for status in [400, 404, 500] {
            let response = Response {
                status: Some(status),
                data: br#"[{"id":1,"result":7}]"#.to_vec(),
            };
            let error = deserialize_response::<Vec<JsonRpcResult<u64>>>(&response).map_err(JsonRpcError::from).unwrap_err();

            assert_eq!((error.code, error.message), (ERROR_CLIENT_ERROR, format!("HTTP error: status {status}")));
        }
    }

    #[test]
    fn test_http_error_rejects_jsonrpc_success() {
        for status in [400, 404, 500] {
            let response = Response {
                status: Some(status),
                data: br#"{"jsonrpc":"2.0","id":1,"result":7}"#.to_vec(),
            };
            let error = deserialize_response::<JsonRpcResult<u64>>(&response).map_err(JsonRpcError::from).and_then(JsonRpcResult::take).unwrap_err();

            assert_eq!((error.code, error.message), (ERROR_CLIENT_ERROR, format!("HTTP error: status {status}")));
        }
    }

    #[test]
    fn test_http_error_preserves_api_message() {
        let response = Response {
            status: Some(400),
            data: br#"{"error":{"message":"Invalid request"}}"#.to_vec(),
        };
        let error = deserialize_response::<JsonRpcResult<Value>>(&response).map_err(JsonRpcError::from).unwrap_err();

        assert_eq!((error.code, error.message), (ERROR_CLIENT_ERROR, "Invalid request".to_string()));
    }
}
