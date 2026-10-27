use std::sync::Arc;

use cacher::{SwapVaultAddressCacher, SwapVaultAddressKind};
use gem_client::{ClientError, ReqwestClient, Response};
use gem_tracing::error_with_fields;
use primitives::SwapProvider;
use reqwest::{Method, RequestBuilder};
use serde::Serialize;
use swapper::cross_chain::deposit_id_from_response;

pub(super) struct SwapProxyClient {
    client: ReqwestClient,
    deposit_addresses: Arc<dyn SwapVaultAddressCacher>,
    provider: SwapProvider,
}

impl SwapProxyClient {
    pub(super) fn new(url: String, deposit_addresses: Arc<dyn SwapVaultAddressCacher>, provider: SwapProvider) -> Self {
        Self {
            client: ReqwestClient::new(url, gem_client::reqwest_client()),
            deposit_addresses,
            provider,
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
        self.cache_deposit_id(&response.data).await;
        Ok(response)
    }

    async fn cache_deposit_id(&self, data: &[u8]) {
        let Some(deposit_id) = deposit_id_from_response(self.provider, data) else {
            return;
        };
        if let Err(error) = self.deposit_addresses.add_vault_addresses(self.provider, SwapVaultAddressKind::Deposit, &[deposit_id]).await {
            error_with_fields!("swap deposit address cache failed", &*error, provider = self.provider.as_ref());
        }
    }
}
