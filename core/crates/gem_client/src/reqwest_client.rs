use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use reqwest::header::USER_AGENT;
use reqwest::{Method, RequestBuilder};
use serde::{Serialize, de::DeserializeOwned};

use crate::{Client, ClientError, Response, build_request_url, deserialize_response, encode_request_body, retry_policy};

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    base_url: String,
    client: reqwest::Client,
    default_headers: HashMap<String, String>,
    user_agent: Option<String>,
}

impl ReqwestClient {
    pub fn new(url: String, client: reqwest::Client) -> Self {
        Self {
            base_url: url,
            client,
            default_headers: HashMap::new(),
            user_agent: None,
        }
    }

    pub fn new_with_user_agent(url: String, client: reqwest::Client, user_agent: String) -> Self {
        Self {
            base_url: url,
            client,
            default_headers: HashMap::new(),
            user_agent: (!user_agent.is_empty()).then_some(user_agent),
        }
    }

    pub fn new_with_retry(url: String, timeout_secs: u64, max_retries: u32) -> Self {
        let client = crate::client_config::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .retry(retry_policy(url.clone(), max_retries))
            .build()
            .expect("Failed to build reqwest client with retry");
        Self {
            base_url: url,
            client,
            default_headers: HashMap::new(),
            user_agent: None,
        }
    }

    pub fn with_default_headers(self, default_headers: HashMap<String, String>) -> Self {
        Self { default_headers, ..self }
    }

    pub fn with_base_url(self, base_url: String) -> Self {
        Self { base_url, ..self }
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = build_request_url(&self.base_url, path);
        self.build_request(self.client.request(method, url), HashMap::new())
    }

    pub fn new_test_client(url: String) -> Self {
        Self::new_with_retry(url, 30, 3)
    }

    fn build_request(&self, request: RequestBuilder, headers: HashMap<String, String>) -> RequestBuilder {
        let request = if let Some(ref user_agent) = self.user_agent { request.header(USER_AGENT, user_agent) } else { request };

        let request = self.default_headers.iter().fold(request, |request, (key, value)| request.header(key, value));
        headers.into_iter().fold(request, |request, (key, value)| request.header(&key, &value))
    }

    fn map_reqwest_error(e: reqwest::Error) -> ClientError {
        if e.is_timeout() {
            return ClientError::Timeout;
        }
        let (is_connect, is_builder) = (e.is_connect(), e.is_builder());
        let error = e.without_url();
        match (is_connect, is_builder) {
            (true, _) => ClientError::Network(format!("Connection error: {error}")),
            (_, true) => ClientError::Network(format!("Request builder error: {error}")),
            _ => ClientError::Network(error.to_string()),
        }
    }
}

#[async_trait]
impl Client for ReqwestClient {
    async fn get_with<R>(&self, path: &str, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let url = build_request_url(&self.base_url, path);
        let request = self.build_request(self.client.get(&url), headers);

        let response = request.send().await.map_err(Self::map_reqwest_error)?;
        json_response(response).await
    }

    async fn get_url<R>(&self, url: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let request = self.build_request(self.client.get(url), HashMap::new());
        let response = request.send().await.map_err(Self::map_reqwest_error)?;
        json_response(response).await
    }

    async fn post_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        self.send_body(Method::POST, path, body, headers).await
    }

    async fn patch_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        self.send_body(Method::PATCH, path, body, headers).await
    }
}

impl ReqwestClient {
    async fn send_body<T, R>(&self, method: Method, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = build_request_url(&self.base_url, path);
        let request_body = encode_request_body(&headers, body)?;
        let request = self.build_request(self.client.request(method, &url).body(request_body), headers);
        let response = request.send().await.map_err(Self::map_reqwest_error)?;

        json_response(response).await
    }
}

pub async fn json_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, ClientError> {
    let status = response.status().as_u16();
    let data = response.bytes().await.map_err(|e| ClientError::Network(format!("Failed to read response body: {}", e.without_url())))?.to_vec();
    let response = Response { status: Some(status), data };
    deserialize_response(&response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_user_agent_override_preserves_client_default() {
        let client = ReqwestClient::new_with_user_agent("https://example.com".to_string(), crate::reqwest_client(), String::new());

        assert!(client.user_agent.is_none());
    }

    #[tokio::test]
    async fn test_a_failed_request_never_reports_what_the_url_carried() {
        let client = ReqwestClient::new("http://gem:hunter2@127.0.0.1:1".to_string(), crate::reqwest_client());
        let error = client.get_with::<serde_json::Value>("/v1/wallets/secret?api_key=shhh", HashMap::new()).await.unwrap_err();

        let ClientError::Network(message) = error else {
            panic!("a refused connection is a network error");
        };
        for secret in ["hunter2", "shhh", "api_key", "wallets"] {
            assert!(!message.contains(secret), "{secret} leaked into {message}");
        }
    }
}
