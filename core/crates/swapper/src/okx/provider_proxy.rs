use super::{
    auth::build_headers,
    model::{OkxClientConfig, QuoteParams, SwapParams},
    target::OkxTarget,
};
use crate::{
    SwapperError,
    alien::{RpcClient, RpcProvider},
};
use gem_client::{Client, ClientExt, Target};
use primitives::{DateTime, SecondsFormat};
use std::{fmt::Debug, sync::Arc};

pub fn error_response(error: SwapperError) -> serde_json::Value {
    serde_json::json!({ "code": "gem_proxy_error", "msg": error.to_string(), "data": [] })
}

#[derive(Debug)]
pub struct OkxProviderProxy<C> {
    client: C,
    config: OkxClientConfig,
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
        Self { client, config }
    }

    pub async fn get_quote(&self, params: QuoteParams) -> Result<serde_json::Value, SwapperError> {
        self.signed_get(OkxTarget::Quote { params }).await
    }

    pub async fn get_swap(&self, params: SwapParams) -> Result<serde_json::Value, SwapperError> {
        self.signed_get(OkxTarget::Swap { params }).await
    }

    async fn signed_get(&self, target: OkxTarget) -> Result<serde_json::Value, SwapperError> {
        let path = target.path();
        let timestamp = DateTime::now().to_rfc3339(SecondsFormat::Millis);
        let headers = build_headers(&self.config, &timestamp, &path);
        self.client.get(&path).headers(headers).await.map_err(SwapperError::from)
    }
}
