use super::{
    client::OkxDexClient,
    model::{OkxClientConfig, QuoteParams, SwapParams},
};
use crate::{
    SwapperError,
    alien::{RpcClient, RpcProvider},
};
use gem_client::Client;
use std::{fmt::Debug, sync::Arc};

pub fn error_response(error: SwapperError) -> serde_json::Value {
    serde_json::json!({ "code": "gem_proxy_error", "msg": error.to_string(), "data": [] })
}

#[derive(Debug)]
pub struct OkxProviderProxy<C> {
    client: OkxDexClient<C>,
}

impl OkxProviderProxy<RpcClient> {
    pub fn new(url: String, config: OkxClientConfig, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_client(RpcClient::new(url, rpc_provider), config)
    }
}

impl<C> OkxProviderProxy<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(client: C, config: OkxClientConfig) -> Self {
        Self {
            client: OkxDexClient::new(client, config),
        }
    }

    pub async fn get_quote(&self, params: QuoteParams) -> Result<serde_json::Value, SwapperError> {
        self.client.quote(&params).await
    }

    pub async fn get_swap(&self, params: SwapParams) -> Result<serde_json::Value, SwapperError> {
        self.client.swap(&params).await
    }
}
