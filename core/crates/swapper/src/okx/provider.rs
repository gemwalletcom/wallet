use super::{
    constants::{PROXY_QUOTE_PATH, PROXY_SWAP_PATH},
    model::{OkxApiResponse, QuoteData, SwapDataResult},
    params::{build_quote_params, build_swap_params},
    quote_data::build_swap_quote_data,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapAmountMode, Swapper, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    config::get_swap_provider_url,
    models::SwapperChainAsset,
};
use async_trait::async_trait;
use gem_client::{Client, ClientExt};
use num_bigint::BigUint;
use primitives::Chain;
use std::str::FromStr;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub struct OkxProvider<C> {
    provider: ProviderType,
    client: C,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl OkxProvider<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let client = RpcClient::new(get_swap_provider_url(SwapperProvider::Okx), rpc_provider.clone());
        Self::new_with_client(client, rpc_provider)
    }
}

impl<C> OkxProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(client: C, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Okx),
            client,
            rpc_provider,
        }
    }
}

#[async_trait]
impl<C> Swapper for OkxProvider<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        support_assets()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Flexible
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let params = build_quote_params(request)?;
        let response: OkxApiResponse<QuoteData> = self.client.post(PROXY_QUOTE_PATH, &params).await.map_err(SwapperError::from)?;
        let route = response.first_data("Failed to fetch OKX quote")?;
        let route_data = serde_json::to_string(&route)?;

        Ok(Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: BigUint::from_str(&route.to_token_amount).map_err(SwapperError::compute_quote_error)?,
            data: ProviderData {
                provider: self.provider.clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: Some(0),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: QuoteData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let params = build_swap_params(request, &route_data)?;

        let response: OkxApiResponse<SwapDataResult> = self.client.post(PROXY_SWAP_PATH, &params).await.map_err(SwapperError::from)?;
        let transaction_data = response.swap_transaction()?;
        let chain = request.from_asset.chain();
        build_swap_quote_data(
            &transaction_data,
            &request.from_asset,
            &request.value.to_string(),
            chain,
            &request.wallet_address,
            self.rpc_provider.clone(),
        )
        .await
    }
}

fn support_assets() -> Vec<SwapperChainAsset> {
    vec![
        SwapperChainAsset::All(Chain::Solana),
        SwapperChainAsset::All(Chain::Tron),
        SwapperChainAsset::All(Chain::Ethereum),
        SwapperChainAsset::All(Chain::SmartChain),
        SwapperChainAsset::All(Chain::Polygon),
        SwapperChainAsset::All(Chain::Arbitrum),
        SwapperChainAsset::All(Chain::Optimism),
        SwapperChainAsset::All(Chain::Base),
        SwapperChainAsset::All(Chain::AvalancheC),
        SwapperChainAsset::All(Chain::Fantom),
        SwapperChainAsset::All(Chain::Manta),
        SwapperChainAsset::All(Chain::Blast),
        SwapperChainAsset::All(Chain::ZkSync),
        SwapperChainAsset::All(Chain::Linea),
        SwapperChainAsset::All(Chain::Mantle),
        SwapperChainAsset::All(Chain::Plasma),
        SwapperChainAsset::All(Chain::Hyperliquid),
        SwapperChainAsset::All(Chain::Sonic),
        SwapperChainAsset::All(Chain::Unichain),
        SwapperChainAsset::All(Chain::Monad),
        SwapperChainAsset::All(Chain::XLayer),
        SwapperChainAsset::All(Chain::Robinhood),
    ]
}

#[cfg(test)]
mod tests {
    use super::super::constants::TRON_DEX_TOKEN_APPROVE_ADDRESS;
    use super::super::testkit::{TEST_TRON_WALLET, mock_client, mock_solana_request};
    use super::*;
    use crate::{SwapperProviderMode, SwapperQuoteAsset, testkit::mock_quote};
    use gem_client::testkit::MockClient;
    use primitives::{
        AssetId,
        asset_constants::{ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDC_TOKEN_ID, TRON_USDT_TOKEN_ID},
        testkit::signer_mock::{TEST_EVM_SENDER, TEST_SOLANA_SENDER},
    };

    const EVM_ROUTER: &str = "0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f";
    const TRON_ROUTER: &str = "TAGVH5t42MuofaAfUauPPRe4Qw3i8Z3QHM";

    const EVM_ZERO_ALLOWANCE: &str = r#"{"id":1,"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#;
    const TRON_ZERO_ALLOWANCE: &str = r#"{"result":{"result":true},"constant_result":["0000000000000000000000000000000000000000000000000000000000000000"],"energy_used":1000}"#;

    #[test]
    fn test_support_assets_includes_new_okx_evm_chains() {
        assert!(support_assets().contains(&SwapperChainAsset::All(Chain::Plasma)));
        assert!(support_assets().contains(&SwapperChainAsset::All(Chain::Robinhood)));
    }

    #[test]
    fn test_provider_metadata() {
        let provider = OkxProvider::mock(MockClient::new(), "{}");
        assert_eq!(provider.provider().id, SwapperProvider::Okx);
        assert_eq!(provider.provider().mode, SwapperProviderMode::OnChain);
        assert!(!provider.supported_assets().is_empty());
        assert_eq!(provider.amount_mode(&mock_solana_request()), SwapAmountMode::Flexible);
    }

    #[tokio::test]
    async fn test_get_quote() {
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, PROXY_QUOTE_PATH);
            let params: serde_json::Value = serde_json::from_slice(body).unwrap();
            assert_eq!(params["chainIndex"], "501");
            assert_eq!(params["amount"], "100000000");
            assert_eq!(params["feePercent"], "0.7");
            assert_eq!(params["fromTokenAddress"], "11111111111111111111111111111111");
            Ok(include_str!("testdata/quote_sol_to_usdc.json").as_bytes().to_vec())
        });
        let provider = OkxProvider::mock(client, "{}");

        let quote = provider.get_quote(&mock_solana_request()).await.unwrap();

        assert_eq!(quote.to_value, BigUint::from(14930750u64));
        assert_eq!(quote.from_value, BigUint::from(100000000u64));
        assert_eq!(quote.data.provider.id, SwapperProvider::Okx);
        let route_data: QuoteData = serde_json::from_str(&quote.data.routes[0].route_data).unwrap();
        assert_eq!(route_data.to_token_amount, "14930750");
    }

    #[tokio::test]
    async fn test_get_quote_propagates_okx_error() {
        let error = include_str!("testdata/quote_error_rate_limit.json");
        let provider = OkxProvider::mock(mock_client(error, error), "{}");

        let result = provider.get_quote(&mock_solana_request()).await;
        assert!(matches!(result, Err(SwapperError::ComputeQuoteError(msg)) if msg == "Request frequency too high"));
    }

    #[tokio::test]
    async fn test_get_quote_data_solana() {
        let client = MockClient::new().with_post(move |path, body| match path {
            PROXY_QUOTE_PATH => Ok(include_str!("testdata/quote_sol_to_usdc.json").as_bytes().to_vec()),
            PROXY_SWAP_PATH => {
                let params: serde_json::Value = serde_json::from_slice(body).unwrap();
                assert_eq!(params["chainIndex"], "501");
                assert_eq!(params["userWalletAddress"], TEST_SOLANA_SENDER);
                assert_eq!(params["fromTokenAddress"], "11111111111111111111111111111111");
                assert!(params.get("approveTransaction").is_none());
                Ok(include_str!("testdata/swap_sol_to_usdc.json").as_bytes().to_vec())
            }
            other => panic!("unexpected path: {other}"),
        });
        let provider = OkxProvider::mock(client, "{}");

        let quote = provider.get_quote(&mock_solana_request()).await.unwrap();
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.to, "RouterAddr");
        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert_eq!(quote_data.data, "aGVsbG8=");
        assert!(quote_data.approval.is_none());
        assert!(quote_data.gas_limit.is_none());
    }

    #[tokio::test]
    async fn test_get_quote_data_wires_approval_for_evm_and_tron() {
        // EVM: spender falls back to the tx target, zero allowance triggers approval, gas gets the 50% buffer.
        let client = MockClient::new().with_post(move |path, body| match path {
            PROXY_QUOTE_PATH => Ok(include_str!("testdata/quote_eth_usdc_to_eth.json").as_bytes().to_vec()),
            PROXY_SWAP_PATH => {
                let params: serde_json::Value = serde_json::from_slice(body).unwrap();
                assert_eq!(params["amount"], "1000000");
                assert_eq!(params["userWalletAddress"], TEST_EVM_SENDER);
                assert_eq!(params["approveTransaction"], true);
                Ok(include_str!("testdata/swap_eth_usdc_to_eth.json").as_bytes().to_vec())
            }
            other => panic!("unexpected path: {other}"),
        });
        let provider = OkxProvider::mock(client, EVM_ZERO_ALLOWANCE);
        let mut request = mock_quote(
            SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()),
            SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
        );
        request.wallet_address = TEST_EVM_SENDER.to_string();

        let quote = provider.get_quote(&request).await.unwrap();
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.to, EVM_ROUTER);
        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert_eq!(quote_data.data, "0xabc123");
        let approval = quote_data.approval.unwrap();
        assert_eq!(approval.token, ETHEREUM_USDC_TOKEN_ID);
        assert_eq!(approval.spender, EVM_ROUTER);
        assert_eq!(approval.value, BigUint::from(1000000u64));
        assert_eq!(quote_data.gas_limit.as_deref(), Some("300000"));

        // Tron: 0x is stripped, approval targets the fixed approve contract, energy kept unbuffered.
        let client = mock_client(include_str!("testdata/quote_tron_usdt_to_trx.json"), include_str!("testdata/swap_tron_usdt_to_trx.json"));
        let provider = OkxProvider::mock(client, TRON_ZERO_ALLOWANCE);
        let mut request = mock_quote(
            SwapperQuoteAsset::from(AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID)),
            SwapperQuoteAsset::from(AssetId::from_chain(Chain::Tron)),
        );
        request.wallet_address = TEST_TRON_WALLET.to_string();
        request.value = BigUint::from(50000000u64);

        let quote = provider.get_quote(&request).await.unwrap();
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.to, TRON_ROUTER);
        assert_eq!(quote_data.data, "f2c42696abcd");
        let approval = quote_data.approval.unwrap();
        assert_eq!(approval.token, TRON_USDT_TOKEN_ID);
        assert_eq!(approval.spender, TRON_DEX_TOKEN_APPROVE_ADDRESS);
        assert_eq!(approval.value, BigUint::from(50000000u64));
        assert_eq!(quote_data.gas_limit.as_deref(), Some("230400"));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::super::{
        model::OkxClientConfig,
        provider_proxy::{OkxProviderProxy, error_response},
        testkit::{TEST_EVM_WALLET, TEST_TRON_WALLET},
    };
    use super::*;
    use crate::{SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode, alien::reqwest_provider::NativeProvider, testkit::mock_quote};
    use gem_client::ClientError;
    use primitives::{
        AssetId,
        asset_constants::{HYPEREVM_USDT_ASSET_ID, PLASMA_USDT_ASSET_ID, ROBINHOOD_USDG_ASSET_ID, SOLANA_USDC_ASSET_ID, TRON_USDT_ASSET_ID},
        testkit::signer_mock::TEST_SOLANA_SENDER,
    };
    use serde::{Serialize, de::DeserializeOwned};
    use std::collections::HashMap;

    // Stands in for the deployed api: routes the client's POSTs into the in-process proxy.
    #[derive(Clone, Debug)]
    struct ProxyPassthroughClient {
        proxy: Arc<OkxProviderProxy<RpcClient>>,
    }

    #[async_trait]
    impl Client for ProxyPassthroughClient {
        async fn get_with<R: DeserializeOwned>(&self, _path: &str, _headers: HashMap<String, String>) -> Result<R, ClientError> {
            Err(ClientError::Network("not supported".into()))
        }

        async fn get_url<R: DeserializeOwned>(&self, _url: &str) -> Result<R, ClientError> {
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

    fn okx_provider_through_proxy() -> OkxProvider<ProxyPassthroughClient> {
        let settings = settings::testkit::get_test_settings();
        let config = OkxClientConfig {
            api_key: settings.swap.okx.key.public,
            secret_key: settings.swap.okx.key.secret,
            passphrase: settings.swap.okx.passphrase,
            project: settings.swap.okx.project,
        };
        let rpc_provider = Arc::new(NativeProvider::default());
        let proxy = Arc::new(OkxProviderProxy::new(settings.swap.okx.url, config, rpc_provider.clone()));
        OkxProvider::new_with_client(ProxyPassthroughClient { proxy }, rpc_provider)
    }

    fn mock_swap_request(from_asset: AssetId, to_asset: AssetId, wallet_address: &str, value: &str) -> QuoteRequest {
        let mut request = mock_quote(SwapperQuoteAsset::from(from_asset), SwapperQuoteAsset::from(to_asset));
        request.wallet_address = wallet_address.to_string();
        request.value = value.parse().unwrap();
        request
    }

    #[tokio::test]
    async fn test_okx_swap_through_proxy() -> Result<(), SwapperError> {
        let provider = okx_provider_through_proxy();
        let cases = [
            (AssetId::from_chain(Chain::Solana), SOLANA_USDC_ASSET_ID.clone(), TEST_SOLANA_SENDER, "100000000", "0"),
            (
                AssetId::from_chain(Chain::Hyperliquid),
                HYPEREVM_USDT_ASSET_ID.clone(),
                TEST_EVM_WALLET,
                "100000000000000000",
                "100000000000000000",
            ),
            (AssetId::from_chain(Chain::Tron), TRON_USDT_ASSET_ID.clone(), TEST_TRON_WALLET, "100000000", "100000000"),
            (
                AssetId::from_chain(Chain::Robinhood),
                ROBINHOOD_USDG_ASSET_ID.clone(),
                TEST_EVM_WALLET,
                "100000000000000",
                "100000000000000",
            ),
            (
                AssetId::from_chain(Chain::Plasma),
                PLASMA_USDT_ASSET_ID.clone(),
                TEST_EVM_WALLET,
                "100000000000000000",
                "100000000000000000",
            ),
        ];

        for (from_asset, to_asset, wallet_address, value, expected_value) in cases {
            let request = mock_swap_request(from_asset, to_asset, wallet_address, value);

            // OKX rate limits to ~1 request per second.
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let quote = provider.get_quote(&request).await?;
            assert!(quote.to_value > BigUint::ZERO);

            let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            assert!(!quote_data.to.is_empty());
            assert_eq!(quote_data.value.to_string(), expected_value);
            assert!(!quote_data.data.is_empty());
            assert!(quote_data.approval.is_none());
        }

        let mut request = mock_swap_request(AssetId::from_chain(Chain::Solana), SOLANA_USDC_ASSET_ID.clone(), TEST_SOLANA_SENDER, "100000000");
        request.options.slippage = SwapperSlippage {
            bps: 300,
            mode: SwapperSlippageMode::Exact,
        };

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.data.is_empty());
        Ok(())
    }
}
