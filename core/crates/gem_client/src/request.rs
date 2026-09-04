use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use serde::{Serialize, de::DeserializeOwned};

use crate::{Client, ClientError, build_path_with_query};

type ResponseFuture<'a, R> = Pin<Box<dyn Future<Output = Result<R, ClientError>> + Send + 'a>>;

pub struct GetRequest<'a, C: ?Sized, R> {
    client: &'a C,
    path: String,
    headers: HashMap<String, String>,
    response: Option<ResponseFuture<'a, R>>,
}

impl<'a, C: Client + ?Sized, R> GetRequest<'a, C, R> {
    pub(crate) fn new(client: &'a C, path: String, headers: HashMap<String, String>) -> Self {
        Self {
            client,
            path,
            headers,
            response: None,
        }
    }

    pub fn query<Q: Serialize + ?Sized>(mut self, query: &Q) -> Self {
        self.path = build_path_with_query(&self.path, query);
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }
}

impl<'a, C: Client + ?Sized, R: DeserializeOwned + Send + 'a> Future for GetRequest<'a, C, R> {
    type Output = Result<R, ClientError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let request = self.get_mut();
        if request.response.is_none() {
            let client = request.client;
            let path = std::mem::take(&mut request.path);
            let headers = std::mem::take(&mut request.headers);
            request.response = Some(Box::pin(async move { client.get_with(&path, headers).await }));
        }
        request.response.as_mut().map_or(Poll::Pending, |response| response.as_mut().poll(context))
    }
}

#[derive(Clone, Copy)]
pub(crate) enum BodyMethod {
    Post,
    Patch,
}

pub struct PostRequest<'a, C: ?Sized, T, R> {
    client: &'a C,
    method: BodyMethod,
    path: String,
    body: &'a T,
    headers: HashMap<String, String>,
    response: Option<ResponseFuture<'a, R>>,
}

impl<'a, C: Client + ?Sized, T: Serialize + Send + Sync, R> PostRequest<'a, C, T, R> {
    pub(crate) fn new(client: &'a C, method: BodyMethod, path: String, headers: HashMap<String, String>, body: &'a T) -> Self {
        Self {
            client,
            method,
            path,
            body,
            headers,
            response: None,
        }
    }

    pub fn query<Q: Serialize + ?Sized>(mut self, query: &Q) -> Self {
        self.path = build_path_with_query(&self.path, query);
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers.extend(headers);
        self
    }
}

impl<'a, C: Client + ?Sized, T: Serialize + Send + Sync, R: DeserializeOwned + Send + 'a> Future for PostRequest<'a, C, T, R> {
    type Output = Result<R, ClientError>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let request = self.get_mut();
        if request.response.is_none() {
            let client = request.client;
            let body = request.body;
            let method = request.method;
            let path = std::mem::take(&mut request.path);
            let headers = std::mem::take(&mut request.headers);
            request.response = Some(Box::pin(async move {
                match method {
                    BodyMethod::Post => client.post_with(&path, body, headers).await,
                    BodyMethod::Patch => client.patch_with(&path, body, headers).await,
                }
            }));
        }
        request.response.as_mut().map_or(Poll::Pending, |response| response.as_mut().poll(context))
    }
}
