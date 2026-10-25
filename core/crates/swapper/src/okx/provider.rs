use super::{
    constants,
    constants::{PROXY_QUOTE_PATH, PROXY_SWAP_PATH},
    model::{OkxApiResponse, QuoteData, SwapDataResult},
    params::{build_quote_params, build_swap_params, max_slippage_bps},
    quote_data::build_swap_quote_data,
};
use crate::{
    Contracts, FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    config::get_swap_provider_url,
    models::SwapperChainAsset,
};
use async_trait::async_trait;
use gem_client::{Client, ClientExt};
use num_bigint::BigUint;
use primitives::AssetId;
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
        SwapAmountMode::Fixed
    }

    fn contracts(&self, from_asset: &AssetId) -> Contracts {
        constants::contracts(from_asset.chain)
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let params = build_quote_params(request)?;
        let response: OkxApiResponse<QuoteData> = self.client.post(PROXY_QUOTE_PATH, &params).await.map_err(SwapperError::from)?;
        let route = response.first_data("Failed to fetch OKX quote")?;
        let to_value = BigUint::from_str(&route.to_token_amount).map_err(SwapperError::compute_quote_error)?;
        if to_value == BigUint::ZERO {
            return Err(SwapperError::NoQuoteAvailable);
        }
        Quote::from_route(request, &self.provider, to_value, max_slippage_bps(request), &route, None)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let route_data: QuoteData = quote.route_data()?;
        let params = build_swap_params(request, &route_data)?;

        let response: OkxApiResponse<SwapDataResult> = self.client.post(PROXY_SWAP_PATH, &params).await.map_err(SwapperError::from)?;
        let transaction_data = response.swap_transaction()?;
        let chain = request.from_asset.chain();
        build_swap_quote_data(&transaction_data, &request.from_asset, &request.value.to_string(), chain, &request.wallet_address, self.rpc_provider.clone()).await
    }
}

fn support_assets() -> Vec<SwapperChainAsset> {
    constants::supported_chains().map(SwapperChainAsset::All).collect()
}

#[cfg(test)]
mod tests {
    use super::super::testkit::{mock_client, mock_solana_request};
    use super::*;
    use crate::{SwapperProviderMode, SwapperQuoteAsset, testkit::mock_quote};
    use gem_client::testkit::MockClient;
    use gem_solana::{SignatureBytes, VersionedTransaction, decode_transaction};
    use primitives::{
        AssetId, Chain,
        asset_constants::{ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDC_TOKEN_ID},
        contract_constants::SOLANA_OKX_DEX_V2_PROGRAM_ID,
        testkit::signer_mock::{TEST_EVM_SENDER, TEST_SOLANA_SENDER},
    };

    const EVM_ROUTER: &str = "0x8feab81d36e7576107d5de0758c1b839be31b4f6";
    const EVM_TOKEN_APPROVE: &str = "0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f";

    const EVM_ZERO_ALLOWANCE: &str = r#"{"id":1,"jsonrpc":"2.0","result":"0x0000000000000000000000000000000000000000000000000000000000000000"}"#;

    #[test]
    fn test_support_assets_includes_new_okx_evm_chains() {
        assert!(support_assets().contains(&SwapperChainAsset::All(Chain::Plasma)));
        assert!(support_assets().contains(&SwapperChainAsset::All(Chain::Robinhood)));
    }

    #[test]
    fn test_contracts() {
        let provider = OkxProvider::mock(MockClient::new(), "{}");

        assert_eq!(
            provider.contracts(&AssetId::from_chain(Chain::Ethereum)),
            Contracts {
                targets: vec!["0x8feAB81D36E7576107D5dE0758c1b839Be31B4F6"],
                spenders: vec![EVM_TOKEN_APPROVE],
            }
        );
        assert_eq!(provider.contracts(&ETHEREUM_USDC_ASSET_ID), provider.contracts(&AssetId::from_chain(Chain::Ethereum)));
        assert_eq!(provider.contracts(&AssetId::from_chain(Chain::Solana)), Contracts::new(vec![SOLANA_OKX_DEX_V2_PROGRAM_ID]));
        assert_eq!(provider.contracts(&AssetId::from_chain(Chain::Bitcoin)), Contracts::default());
        for asset in support_assets() {
            assert!(!provider.contracts(&asset.get_chain().as_asset_id()).targets.is_empty(), "{:?} ships no contracts", asset.get_chain());
        }
    }

    #[test]
    fn test_provider_metadata() {
        let provider = OkxProvider::mock(MockClient::new(), "{}");
        assert_eq!(provider.provider().id, SwapperProvider::Okx);
        assert_eq!(provider.provider().mode, SwapperProviderMode::OnChain);
        assert!(!provider.supported_assets().is_empty());
        assert_eq!(provider.amount_mode(&mock_solana_request()), SwapAmountMode::Fixed, "a contract call spends exactly the quoted amount");
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
        assert_eq!(result.unwrap_err(), SwapperError::ComputeQuoteError("Request frequency too high".to_string()));
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

        let response: serde_json::Value = serde_json::from_str(include_str!("testdata/swap_sol_to_usdc.json")).unwrap();
        let okx_bytes = bs58::decode(response["data"][0]["tx"]["data"].as_str().unwrap()).into_vec().unwrap();
        let okx_transaction = VersionedTransaction::deserialize_with_version(&okx_bytes).unwrap();
        let mut unsigned = okx_transaction.clone();
        *unsigned.signatures_mut() = vec![SignatureBytes::default()];

        assert_ne!(okx_transaction, unsigned, "OKX fills the wallet signature slot with a placeholder");
        assert_eq!(decode_transaction(&quote_data.data).unwrap(), unsigned);
        assert_eq!(quote_data.to, "proVF4pMXVaYqmy4NjniPh4pqKNfMmsihgd4wdkCX3u");
        let swapper: &dyn Swapper = &provider;
        swapper.verify_contracts(&quote, &quote_data).unwrap();
        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert!(quote_data.approval.is_none());
        assert_eq!(quote_data.gas_limit, unsigned.get_compute_unit_limit().map(|limit| limit.to_string()));
        assert!(quote_data.gas_limit.is_some());
    }

    #[tokio::test]
    async fn test_get_quote_data_wires_evm_approval() {
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
        let mut request = mock_quote(SwapperQuoteAsset::from(ETHEREUM_USDC_ASSET_ID.clone()), SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)));
        request.wallet_address = TEST_EVM_SENDER.to_string();

        let quote = provider.get_quote(&request).await.unwrap();
        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();

        assert_eq!(quote_data.to, EVM_ROUTER);
        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert_eq!(quote_data.data, "0xabc123");
        let swapper: &dyn Swapper = &provider;
        swapper.verify_contracts(&quote, &quote_data).unwrap();
        let approval = quote_data.approval.unwrap();
        assert_eq!(approval.token, ETHEREUM_USDC_TOKEN_ID);
        assert_eq!(approval.spender, EVM_TOKEN_APPROVE);
        assert_eq!(approval.value, BigUint::from(1000000u64));
        assert_eq!(quote_data.gas_limit.as_deref(), Some("300000"));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::super::testkit::TEST_EVM_WALLET;
    use super::*;
    use crate::{SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode, testkit::mock_quote};
    use primitives::{
        AssetId, Chain,
        asset_constants::{HYPEREVM_USDT_ASSET_ID, PLASMA_USDT_ASSET_ID, ROBINHOOD_USDG_ASSET_ID, SOLANA_USDC_ASSET_ID},
        testkit::signer_mock::TEST_SOLANA_SENDER,
    };

    #[tokio::test]
    async fn test_okx_swap_through_proxy() -> Result<(), SwapperError> {
        let provider = OkxProvider::mock_through_proxy();
        let cases = [
            (AssetId::from_chain(Chain::Solana), SOLANA_USDC_ASSET_ID.clone(), TEST_SOLANA_SENDER, "100000000", "0"),
            (AssetId::from_chain(Chain::Hyperliquid), HYPEREVM_USDT_ASSET_ID.clone(), TEST_EVM_WALLET, "100000000000000000", "100000000000000000"),
            (AssetId::from_chain(Chain::Robinhood), ROBINHOOD_USDG_ASSET_ID.clone(), TEST_EVM_WALLET, "100000000000000", "100000000000000"),
            (AssetId::from_chain(Chain::Plasma), PLASMA_USDT_ASSET_ID.clone(), TEST_EVM_WALLET, "100000000000000000", "100000000000000000"),
        ];

        for (from_asset, to_asset, wallet_address, value, expected_value) in cases {
            let mut request = mock_quote(SwapperQuoteAsset::from(from_asset), SwapperQuoteAsset::from(to_asset));
            request.wallet_address = wallet_address.to_string();
            request.value = value.parse().unwrap();

            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let quote = provider.get_quote(&request).await?;
            assert!(quote.to_value > BigUint::ZERO);

            let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;

            (&provider as &dyn Swapper).verify_contracts(&quote, &quote_data)?;
            assert!(!quote_data.to.is_empty());
            assert_eq!(quote_data.value.to_string(), expected_value);
            assert!(!quote_data.data.is_empty());
            assert!(quote_data.approval.is_none());
        }

        let mut request = mock_quote(SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)), SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()));
        request.wallet_address = TEST_SOLANA_SENDER.to_string();
        request.value = BigUint::from(100000000u64);
        request.options.slippage = SwapperSlippage { bps: 300, mode: SwapperSlippageMode::Exact };

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let quote = provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;

        (&provider as &dyn Swapper).verify_contracts(&quote, &quote_data)?;
        assert!(!quote_data.to.is_empty());
        assert!(!quote_data.data.is_empty());
        Ok(())
    }
}
