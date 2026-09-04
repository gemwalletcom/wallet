use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    sync::Arc,
};

use async_trait::async_trait;
use gem_client::{Client, ClientError, Response, build_request_url, deserialize_response, encode_request};
use primitives::Chain;
use serde::{Serialize, de::DeserializeOwned};

pub type RpcResponse = Response;

pub trait RpcClientError: Error + Send + Sync + 'static + Display + Sized {
    fn into_client_error(self) -> ClientError {
        ClientError::Network(format!("RPC provider error: {}", self))
    }
}

#[derive(Debug, Clone)]
pub struct Target {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

impl From<HttpMethod> for String {
    fn from(value: HttpMethod) -> Self {
        match value {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Patch => "PATCH",
        }
        .into()
    }
}

#[async_trait]
pub trait RpcProvider: Send + Sync + Debug {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error>;
    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error>;
}

#[derive(Debug, Clone)]
pub struct RpcClient<E> {
    base_url: String,
    provider: Arc<dyn RpcProvider<Error = E>>,
}

impl<E> RpcClient<E>
where
    E: RpcClientError,
{
    pub fn new(base_url: String, provider: Arc<dyn RpcProvider<Error = E>>) -> Self {
        Self { base_url, provider }
    }
}

#[async_trait]
impl<E> Client for RpcClient<E>
where
    E: RpcClientError,
{
    async fn get_with<R>(&self, path: &str, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let url = build_request_url(&self.base_url, path);
        let target = Target {
            url,
            method: HttpMethod::Get,
            headers: if headers.is_empty() { None } else { Some(headers) },
            body: None,
        };

        let response = self.provider.request(target).await.map_err(|e| e.into_client_error())?;
        deserialize_response(&response)
    }

    async fn get_url<R>(&self, url: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned,
    {
        let target = Target {
            url: url.to_string(),
            method: HttpMethod::Get,
            headers: None,
            body: None,
        };
        let response = self.provider.request(target).await.map_err(|e| e.into_client_error())?;
        deserialize_response(&response)
    }

    async fn post_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        self.send_body(HttpMethod::Post, path, body, headers).await
    }

    async fn patch_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        self.send_body(HttpMethod::Patch, path, body, headers).await
    }
}

impl<E> RpcClient<E>
where
    E: RpcClientError,
{
    async fn send_body<T, R>(&self, method: HttpMethod, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let url = build_request_url(&self.base_url, path);
        let (request_headers, data) = encode_request(headers, body)?;

        let target = Target {
            url,
            method,
            headers: Some(request_headers),
            body: Some(data),
        };

        let response = self.provider.request(target).await.map_err(|e| e.into_client_error())?;
        deserialize_response(&response)
    }
}

#[async_trait]
impl<E> RpcProvider for RpcClient<E>
where
    E: RpcClientError,
{
    type Error = E;

    async fn request(&self, target: Target) -> Result<RpcResponse, Self::Error> {
        self.provider.request(target).await
    }

    fn get_endpoint(&self, chain: Chain) -> Result<String, Self::Error> {
        self.provider.get_endpoint(chain)
    }
}
