use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use gem_client::{ClientError, ReqwestClient, Response};
use gem_tracing::error_with_fields;
use primitives::SwapProvider;
use reqwest::{Method, RequestBuilder};
use serde::Serialize;

#[async_trait]
pub trait SwapDepositAddressStore: Send + Sync {
    async fn add_deposit_address(&self, provider: &SwapProvider, address: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl SwapDepositAddressStore for CacherClient {
    async fn add_deposit_address(&self, provider: &SwapProvider, address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.add_to_set_cached(CacheKey::SwapDepositAddresses(provider.as_ref()), &[address.to_string()]).await.map(|_| ())
    }
}

pub(super) struct SwapProxyClient {
    client: ReqwestClient,
    deposit_addresses: Arc<dyn SwapDepositAddressStore>,
    provider: SwapProvider,
    deposit_address_pointer: &'static str,
}

impl SwapProxyClient {
    pub(super) fn new(url: String, deposit_addresses: Arc<dyn SwapDepositAddressStore>, provider: SwapProvider, deposit_address_pointer: &'static str) -> Self {
        Self {
            client: ReqwestClient::new(url, gem_client::reqwest_client()),
            deposit_addresses,
            provider,
            deposit_address_pointer,
        }
    }

    pub(super) async fn get<Q: Serialize + ?Sized>(&self, path: &str, query: &Q) -> Result<Response, ClientError> {
        self.send(self.client.request(Method::GET, path).query(query)).await
    }

    pub(super) async fn post<T: Serialize + ?Sized>(&self, path: &str, body: &T) -> Result<Response, ClientError> {
        self.send(self.client.request(Method::POST, path).json(body)).await
    }

    async fn send(&self, request: RequestBuilder) -> Result<Response, ClientError> {
        let response = self.client.send(request).await?;
        self.cache_deposit_address(&response.data).await;
        Ok(response)
    }

    async fn cache_deposit_address(&self, data: &[u8]) {
        let Ok(response) = serde_json::from_slice::<serde_json::Value>(data) else {
            return;
        };
        let Some(address) = response.pointer(self.deposit_address_pointer).and_then(|value| value.as_str()).filter(|address| !address.is_empty()) else {
            return;
        };
        if let Err(error) = self.deposit_addresses.add_deposit_address(&self.provider, address).await {
            error_with_fields!("swap deposit address cache failed", &*error, provider = self.provider.as_ref());
        }
    }
}
