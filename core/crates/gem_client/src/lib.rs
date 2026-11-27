use std::{collections::HashMap, fmt::Debug, future::Future, time::Duration};

use serde::{Serialize, de::DeserializeOwned};

mod content_type;
mod multipart;
mod provider_config;
mod request;
mod target;
mod types;

#[cfg(any(test, debug_assertions))]
pub mod testkit;

pub mod query;

pub use content_type::{CONTENT_TYPE, ContentType};
pub use multipart::{MULTIPART_FORM_DATA, MultipartForm};
pub use provider_config::RemoteProviderConfig;
pub use query::{build_path_with_query, build_request_url};
use request::BodyMethod;
pub use request::{GetRequest, PostRequest};
pub use target::Target;
use target::body_headers;
pub use types::{ClientError, Response, decode_json_byte_array, deserialize_response, encode_request_body, validate_response};

pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub trait Client: Send + Sync + Debug {
    fn get_with<R>(&self, path: &str, headers: HashMap<String, String>) -> impl Future<Output = Result<R, ClientError>> + Send
    where
        R: DeserializeOwned;

    fn get_url<R>(&self, url: &str) -> impl Future<Output = Result<R, ClientError>> + Send
    where
        R: DeserializeOwned;

    fn post_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> impl Future<Output = Result<R, ClientError>> + Send
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;

    fn patch_with<T, R>(&self, path: &str, body: &T, headers: HashMap<String, String>) -> impl Future<Output = Result<R, ClientError>> + Send
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned;
}

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

    fn get_or_error<R, E>(&self, target: impl Target + Send) -> impl Future<Output = Result<R, ClientError<Option<E>>>> + Send
    where
        R: DeserializeOwned + Send,
        E: DeserializeOwned + Send,
    {
        async move { self.get(target).await.map_err(ClientError::decode_body) }
    }

    fn post_or_error<T, R, E>(&self, target: impl Target + Send, body: &T) -> impl Future<Output = Result<R, ClientError<Option<E>>>> + Send
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned + Send,
        E: DeserializeOwned + Send,
    {
        async move { self.post(target, body).await.map_err(ClientError::decode_body) }
    }
}

impl<T: Client + ?Sized> ClientExt for T {}
