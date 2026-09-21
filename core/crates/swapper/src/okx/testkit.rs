use super::{
    constants::{PROXY_QUOTE_PATH, PROXY_SWAP_PATH},
    model::{QuoteData, TokenInfo},
    provider::OkxProvider,
};
#[cfg(feature = "swap_integration_tests")]
use super::{
    model::OkxClientConfig,
    provider_proxy::{OkxProviderProxy, error_response},
};
#[cfg(feature = "swap_integration_tests")]
use crate::{NativeProvider, RpcClient};
use crate::{QuoteRequest, SwapperQuoteAsset, alien::mock::ProviderMock, testkit::mock_quote};
#[cfg(feature = "swap_integration_tests")]
use gem_client::ClientError;
use gem_client::testkit::MockClient;
use num_bigint::BigUint;
use primitives::{AssetId, Chain, asset_constants::SOLANA_USDC_ASSET_ID, testkit::signer_mock::TEST_SOLANA_SENDER};
#[cfg(feature = "swap_integration_tests")]
use serde::{Serialize, de::DeserializeOwned};
#[cfg(feature = "swap_integration_tests")]
use std::collections::HashMap;
use std::sync::Arc;

pub(super) const TEST_TRON_WALLET: &str = "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z";
#[cfg(feature = "swap_integration_tests")]
pub(super) const TEST_EVM_WALLET: &str = "0x1085c5f70F7F7591D97da281A64688385455c2bD";

impl OkxProvider<MockClient> {
    pub fn mock(client: MockClient, rpc_result: &str) -> Self {
        Self::new_with_client(client, Arc::new(ProviderMock::new(rpc_result.to_string())))
    }
}

pub(super) fn mock_solana_request() -> QuoteRequest {
    let mut request = mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)), SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()));
    request.wallet_address = TEST_SOLANA_SENDER.to_string();
    request.value = BigUint::from(100000000u64);
    request
}

pub(super) fn mock_client(quote_response: &'static str, swap_response: &'static str) -> MockClient {
    MockClient::new().with_post(move |path, _| match path {
        PROXY_QUOTE_PATH => Ok(quote_response.as_bytes().to_vec()),
        PROXY_SWAP_PATH => Ok(swap_response.as_bytes().to_vec()),
        other => panic!("unexpected path: {other}"),
    })
}

impl QuoteData {
    pub fn mock(from_token: &str, to_token: &str) -> Self {
        Self {
            from_token: TokenInfo {
                token_contract_address: from_token.to_string(),
            },
            to_token: TokenInfo {
                token_contract_address: to_token.to_string(),
            },
            to_token_amount: "200".to_string(),
        }
    }
}

#[cfg(feature = "swap_integration_tests")]
#[derive(Clone, Debug)]
pub(super) struct ProxyPassthroughClient {
    proxy: Arc<OkxProviderProxy<RpcClient>>,
}

#[cfg(feature = "swap_integration_tests")]
#[async_trait::async_trait]
impl gem_client::Client for ProxyPassthroughClient {
    async fn get_with<R: DeserializeOwned>(&self, _path: &str, _headers: HashMap<String, String>) -> Result<R, ClientError> {
        Err(ClientError::Network("not supported".into()))
    }

    async fn get_url<R: DeserializeOwned>(&self, _url: &str) -> Result<R, ClientError> {
        Err(ClientError::Network("not supported".into()))
    }

    async fn patch_with<T, R>(&self, _path: &str, _body: &T, _headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        Err(ClientError::Network("not supported".into()))
    }

    async fn post_with<T, R>(&self, path: &str, body: &T, _headers: HashMap<String, String>) -> Result<R, ClientError>
    where
        T: Serialize + Send + Sync,
        R: DeserializeOwned,
    {
        let body = serde_json::to_vec(body).map_err(|error| ClientError::Serialization(error.to_string()))?;
        let result = match path {
            PROXY_QUOTE_PATH => {
                let params = serde_json::from_slice(&body).map_err(|error| ClientError::Serialization(error.to_string()))?;
                self.proxy.get_quote(params).await
            }
            PROXY_SWAP_PATH => {
                let params = serde_json::from_slice(&body).map_err(|error| ClientError::Serialization(error.to_string()))?;
                self.proxy.get_swap(params).await
            }
            other => return Err(ClientError::Network(format!("unexpected path: {other}"))),
        };
        let response = result.unwrap_or_else(error_response);
        serde_json::from_value(response).map_err(|error| ClientError::Serialization(error.to_string()))
    }
}

#[cfg(feature = "swap_integration_tests")]
impl OkxProvider<ProxyPassthroughClient> {
    pub fn mock_through_proxy() -> Self {
        let settings = settings::testkit::get_test_settings();
        let config = OkxClientConfig {
            api_key: settings.swap.okx.key.public,
            secret_key: settings.swap.okx.key.secret,
            passphrase: settings.swap.okx.passphrase,
            project: settings.swap.okx.project,
        };
        let rpc_provider = Arc::new(NativeProvider::default());
        let proxy = Arc::new(OkxProviderProxy::new(settings.swap.okx.url, config, rpc_provider.clone()));
        Self::new_with_client(ProxyPassthroughClient { proxy }, rpc_provider)
    }
}
