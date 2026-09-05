use std::{collections::HashMap, fmt::Debug, time::Duration};

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

mod content_type;
mod multipart;
mod provider_config;
mod request;
mod target;
mod types;

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

#[cfg(feature = "reqwest")]
mod reqwest_client;

#[cfg(feature = "reqwest")]
pub mod retry;

#[cfg(feature = "reqwest")]
pub mod client_config;

pub mod query;

pub use content_type::{CONTENT_TYPE, ContentType};
pub use multipart::{MULTIPART_FORM_DATA, MultipartForm};
pub use provider_config::RemoteProviderConfig;
pub use query::{build_path_with_query, build_request_url};
use request::BodyMethod;
pub use request::{GetRequest, PostRequest};
pub use target::Target;
use target::body_headers;
pub use types::{ClientError, Response, decode_json_byte_array, deserialize_response, encode_request_body};

#[cfg(feature = "reqwest")]
pub use reqwest_client::{ReqwestClient, json_response};

#[cfg(feature = "reqwest")]
pub use retry::{default_should_retry, retry, retry_policy};

#[cfg(feature = "reqwest")]
pub use client_config::{builder, reqwest_client};

pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[async_trait]
pub trait Client: Send + Sync + Debug {
    async fn get_with<R>(&self, path: &str, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        R: DeserializeOwned;

    async fn get_url<R>(&self, url: &str) -> Result<R, ClientError>
    where
        R: DeserializeOwned;

    async fn post_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;

    async fn patch_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;
}

#[async_trait]
pub trait ClientExt: Client {
    fn get<R>(&self, target: impl Target) -> GetRequest<'_, Self, R> {
        GetRequest::new(self, target.path(), target.headers())
    }

    fn post<'a, T, R>(&'a self, target: impl Target, body: &'a T) -> PostRequest<'a, Self, T, R>
    where
        T: Serialize + Send + Sync,
    {
        PostRequest::new(self, BodyMethod::Post, target.path(), body_headers(&target), body)
    }

    fn patch<'a, T, R>(&'a self, target: impl Target, body: &'a T) -> PostRequest<'a, Self, T, R>
    where
        T: Serialize + Send + Sync,
    {
        PostRequest::new(self, BodyMethod::Patch, target.path(), body_headers(&target), body)
    }

    async fn get_or_error<R, E>(&self, target: impl Target + Send) -> Result<R, ClientError<Option<E>>>
    where
        R: DeserializeOwned + Send,
        E: DeserializeOwned + Send,
    {
        self.get(target).await.map_err(ClientError::decode_body)
    }

    async fn post_or_error<T, R, E>(&self, target: impl Target + Send, body: &T) -> Result<R, ClientError<Option<E>>>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned + Send,
        E: DeserializeOwned + Send,
    {
        self.post(target, body).await.map_err(ClientError::decode_body)
    }
}

impl<T: Client + ?Sized> ClientExt for T {}
