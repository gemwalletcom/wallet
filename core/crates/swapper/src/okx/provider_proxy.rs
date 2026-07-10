use super::{
    client::OkxDexClient,
    constants::BASE_URL,
    model::{OkxApiResponse, OkxClientConfig, QuoteData, QuoteParams, SwapDataResult, SwapParams},
    params::{build_quote_params, build_swap_params, limit_slippage_bps},
    quote_data::{build_swap_quote_data, output_min_value},
};
use crate::{
    Options, QuoteRequest, SwapperError, SwapperSlippage,
    alien::{RpcClient, RpcProvider},
};
use gem_client::Client;
use primitives::swap::{ProxyQuote, ProxyQuoteRequest, SwapQuoteData};
use std::{fmt::Debug, sync::Arc};

pub fn error_response(error: SwapperError) -> serde_json::Value {
    serde_json::json!({ "code": "gem_proxy_error", "msg": error.to_string(), "data": [] })
}

fn quote_request(request: &ProxyQuoteRequest) -> QuoteRequest {
    QuoteRequest {
        from_asset: request.from_asset.clone(),
        to_asset: request.to_asset.clone(),
        wallet_address: request.from_address.clone(),
        destination_address: request.to_address.clone(),
        value: request.from_value.clone(),
        options: Options {
            slippage: SwapperSlippage {
                bps: request.slippage_bps,
                mode: request.slippage_mode,
            },
            use_max_amount: request.use_max_amount,
        },
    }
}

#[derive(Debug)]
pub struct OkxProviderProxy<C> {
    client: OkxDexClient<C>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl OkxProviderProxy<RpcClient> {
    pub fn new(config: OkxClientConfig, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_client(RpcClient::new(BASE_URL.to_string(), rpc_provider.clone()), config, rpc_provider)
    }
}

impl<C> OkxProviderProxy<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(client: C, config: OkxClientConfig, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            client: OkxDexClient::new(client, config),
            rpc_provider,
        }
    }

    pub async fn get_quote(&self, params: QuoteParams) -> Result<serde_json::Value, SwapperError> {
        self.client.quote(&params).await
    }

    pub async fn get_swap(&self, params: SwapParams) -> Result<serde_json::Value, SwapperError> {
        self.client.swap(&params).await
    }

    pub async fn get_quote_legacy(&self, request: ProxyQuoteRequest) -> Result<ProxyQuote, SwapperError> {
        let params = build_quote_params(&quote_request(&request))?;
        let response: OkxApiResponse<QuoteData> = self.client.quote(&params).await?;
        let route = response.first_data("Failed to fetch OKX quote")?;

        let chain = request.from_asset.chain();
        let output_min_value = output_min_value(&route.to_token_amount, limit_slippage_bps(request.slippage_bps, chain))?;
        let route_data = serde_json::to_value(&route)?;

        Ok(ProxyQuote {
            output_value: route.to_token_amount,
            output_min_value,
            route_data,
            eta_in_seconds: 0,
            quote: request,
        })
    }

    pub async fn get_quote_data_legacy(&self, quote: ProxyQuote) -> Result<SwapQuoteData, SwapperError> {
        let route: QuoteData = serde_json::from_value(quote.route_data.clone()).map_err(|_| SwapperError::InvalidRoute)?;
        let request = &quote.quote;
        let params = build_swap_params(&quote_request(request), &route)?;

        let response: OkxApiResponse<SwapDataResult> = self.client.swap(&params).await?;
        let transaction_data = response.swap_transaction()?;
        let chain = request.from_asset.chain();
        build_swap_quote_data(
            &transaction_data,
            &request.from_asset,
            &request.from_value,
            chain,
            &request.from_address,
            self.rpc_provider.clone(),
        )
        .await
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{alien::reqwest_provider::NativeProvider, testkit::mock_proxy_quote_request_from_assets};
    use primitives::{
        AssetId, Chain,
        asset_constants::{HYPEREVM_USDT_ASSET_ID, PLASMA_USDT_ASSET_ID, ROBINHOOD_USDG_ASSET_ID, SOLANA_USDC_ASSET_ID, TRON_USDT_ASSET_ID},
        swap::{QuoteAsset, SlippageMode},
        testkit::signer_mock::TEST_SOLANA_SENDER,
    };

    const EVM_WALLET: &str = "0x1085c5f70F7F7591D97da281A64688385455c2bD";
    const TRON_WALLET: &str = "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z";

    fn okx_provider() -> OkxProviderProxy<RpcClient> {
        let settings = settings::testkit::get_test_settings();
        let config = OkxClientConfig {
            api_key: settings.swap.okx.key.public,
            secret_key: settings.swap.okx.key.secret,
            passphrase: settings.swap.okx.passphrase,
            project: settings.swap.okx.project,
        };
        OkxProviderProxy::new(config, Arc::new(NativeProvider::default()))
    }

    #[tokio::test]
    async fn test_okx_fetch_quote_and_quote_data_hyperevm_hype_to_usdt() -> Result<(), SwapperError> {
        let provider = okx_provider();
        let request = mock_proxy_quote_request_from_assets(
            AssetId::from_chain(Chain::Hyperliquid),
            HYPEREVM_USDT_ASSET_ID.clone(),
            EVM_WALLET,
            "100000000000000000",
            100,
        );

        let quote = provider.get_quote_legacy(request).await?;
        assert!(quote.output_value.parse::<u64>().unwrap() > 0);

        let quote_data = provider.get_quote_data_legacy(quote).await?;
        assert!(!quote_data.to.is_empty());
        assert_eq!(quote_data.value, "100000000000000000");
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.approval.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_okx_fetch_quote_and_quote_data_tron_trx_to_usdt() -> Result<(), SwapperError> {
        let provider = okx_provider();
        let request = mock_proxy_quote_request_from_assets(AssetId::from_chain(Chain::Tron), TRON_USDT_ASSET_ID.clone(), TRON_WALLET, "100000000", 100);

        let quote = provider.get_quote_legacy(request).await?;
        assert!(quote.output_value.parse::<u64>().unwrap() > 0);

        let quote_data = provider.get_quote_data_legacy(quote).await?;
        assert!(!quote_data.to.is_empty());
        assert_eq!(quote_data.value, "100000000");
        assert!(!quote_data.data.is_empty());
        assert!(!quote_data.data.starts_with("0x"));
        assert!(quote_data.approval.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_okx_fetch_quote_and_quote_data_sol_to_usdc() -> Result<(), SwapperError> {
        let provider = okx_provider();
        let request = mock_proxy_quote_request_from_assets(AssetId::from_chain(Chain::Solana), SOLANA_USDC_ASSET_ID.clone(), TEST_SOLANA_SENDER, "100000000", 300);

        let quote = provider.get_quote_legacy(request).await?;
        assert!(quote.output_value.parse::<u64>().unwrap() > 0);
        assert!(quote.output_min_value.parse::<u64>().unwrap() > 0);

        let quote_data = provider.get_quote_data_legacy(quote).await?;
        assert!(!quote_data.to.is_empty());
        assert_eq!(quote_data.value, "0");
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_okx_fetch_quote_and_quote_data_robinhood_eth_to_usdg() -> Result<(), SwapperError> {
        let provider = okx_provider();
        let request = mock_proxy_quote_request_from_assets(AssetId::from_chain(Chain::Robinhood), ROBINHOOD_USDG_ASSET_ID.clone(), EVM_WALLET, "100000000000000", 100);

        let quote = provider.get_quote_legacy(request).await?;
        assert!(quote.output_value.parse::<u64>().unwrap() > 0);

        let quote_data = provider.get_quote_data_legacy(quote).await?;
        assert!(!quote_data.to.is_empty());
        assert_eq!(quote_data.value, "100000000000000");
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.approval.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_okx_fetch_quote_and_quote_data_plasma_xpl_to_usdt() -> Result<(), SwapperError> {
        let provider = okx_provider();
        let request = mock_proxy_quote_request_from_assets(AssetId::from_chain(Chain::Plasma), PLASMA_USDT_ASSET_ID.clone(), EVM_WALLET, "100000000000000000", 100);

        let quote = provider.get_quote_legacy(request).await?;
        assert!(quote.output_value.parse::<u64>().unwrap() > 0);

        let quote_data = provider.get_quote_data_legacy(quote).await?;
        assert!(!quote_data.to.is_empty());
        assert_eq!(quote_data.value, "100000000000000000");
        assert!(!quote_data.data.is_empty());
        assert!(quote_data.approval.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_okx_fetch_quote_and_quote_data_exact_slippage() -> Result<(), SwapperError> {
        let provider = okx_provider();
        let request = ProxyQuoteRequest {
            from_address: TEST_SOLANA_SENDER.to_string(),
            to_address: TEST_SOLANA_SENDER.to_string(),
            from_asset: QuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: QuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
            from_value: "100000000".to_string(),
            referral_bps: 50,
            slippage_bps: 300,
            slippage_mode: SlippageMode::Exact,
            use_max_amount: false,
        };

        let quote = provider.get_quote_legacy(request).await?;
        assert!(quote.output_value.parse::<u64>().unwrap() > 0);

        let quote_data = provider.get_quote_data_legacy(quote).await?;
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.data.is_empty());
        Ok(())
    }
}
