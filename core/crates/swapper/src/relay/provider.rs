use std::sync::Arc;

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use primitives::{AssetId, Chain, ChainType, EVMChain, swap::ApprovalData};

use super::{
    asset::{SUPPORTED_CHAINS, asset_to_currency},
    chain::RelayChain,
    client::RelayClient,
    mapper,
    model::{RelayAppFee, RelayQuoteRequest, RelayQuoteResponse},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperProvider, SwapperQuoteData, approval::check_approval_erc20, config::get_swap_proxy_url, cross_chain::VaultAddresses, fees::DEFAULT_REFERRER,
    fees::default_referral_fees,
};

#[derive(Debug)]
pub struct Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    client: RelayClient<C>,
}

impl Relay<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let url = get_swap_proxy_url("relay");
        let client = RelayClient::new(RpcClient::new(url, rpc_provider.clone()));
        Self {
            provider: ProviderType::new(SwapperProvider::Relay),
            rpc_provider,
            client,
        }
    }
}

fn resolve_app_fees() -> Vec<RelayAppFee> {
    let fee = default_referral_fees().evm;
    if fee.address.is_empty() {
        return vec![];
    }
    vec![RelayAppFee {
        recipient: fee.address,
        fee: fee.bps.to_string(),
    }]
}

#[async_trait]
impl<C> Swapper for Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        SUPPORTED_CHAINS.clone()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_chain = RelayChain::from_chain(&request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let to_chain = RelayChain::from_chain(&request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;

        let from_asset_id = request.from_asset.asset_id();
        let to_asset_id = request.to_asset.asset_id();

        let origin_currency = asset_to_currency(&from_asset_id)?;
        let destination_currency = asset_to_currency(&to_asset_id)?;
        let app_fees = resolve_app_fees();
        let from_value = request.value.clone();

        let relay_request = RelayQuoteRequest {
            user: request.wallet_address.clone(),
            origin_chain_id: from_chain.chain_id(),
            destination_chain_id: to_chain.chain_id(),
            origin_currency,
            destination_currency,
            amount: from_value.clone(),
            recipient: request.destination_address.clone(),
            trade_type: "EXACT_INPUT".to_string(),
            referrer: if app_fees.is_empty() { None } else { Some(DEFAULT_REFERRER.to_string()) },
            app_fees,
            refund_to: request.wallet_address.clone(),
            max_route_length: 6,
        };

        let response = self.client.get_quote(relay_request).await?;

        let to_value = response.details.currency_out.amount.clone();
        let eta_in_seconds = response.details.time_estimate_u32();

        let quote = Quote {
            from_value,
            min_from_value: None,
            to_value,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: vec![Route {
                    input: from_asset_id,
                    output: to_asset_id,
                    route_data: serde_json::to_string(&response).map_err(SwapperError::compute_quote_error)?,
                }],
                slippage_bps: response.details.slippage_bps().unwrap_or(request.options.slippage.bps),
            },
            request: request.clone(),
            eta_in_seconds,
        };

        Ok(quote)
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let response: RelayQuoteResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;

        let from_asset_id = quote.request.from_asset.asset_id();
        let approval = self.check_evm_approval(quote, &response, &from_asset_id).await?;
        mapper::map_quote_data(&response, approval)
    }

    async fn get_swap_result(&self, _chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let response = self.client.get_request(transaction_hash).await?;
        let request = response.requests.first().ok_or(SwapperError::InvalidRoute)?;
        Ok(mapper::map_swap_result(request))
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        let response = self.client.get_chains().await?;
        Ok(VaultAddresses {
            deposit: response.deposit_addresses(),
            send: response.send_addresses(),
        })
    }
}

impl<C> Relay<C>
where
    C: Client + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    async fn check_evm_approval(&self, quote: &Quote, quote_response: &RelayQuoteResponse, from_asset_id: &AssetId) -> Result<Option<ApprovalData>, SwapperError> {
        if from_asset_id.chain.chain_type() != ChainType::Ethereum {
            return Ok(None);
        }
        let token = if let Some(token_id) = from_asset_id.token_id.clone() {
            token_id
        } else {
            let evm_chain = EVMChain::from_chain(from_asset_id.chain).ok_or(SwapperError::NotSupportedChain)?;
            let Some(token_id) = evm_chain.native_asset_contract() else {
                return Ok(None);
            };
            token_id.to_string()
        };

        let spender = quote_response.router_address().ok_or(SwapperError::InvalidRoute)?;
        let amount: U256 = quote.from_value.parse().map_err(SwapperError::from)?;

        Ok(check_approval_erc20(
            quote.request.wallet_address.clone(),
            token,
            spender,
            amount,
            self.rpc_provider.clone(),
            &from_asset_id.chain,
        )
        .await?
        .approval_data())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{alien::mock::ProviderMock, relay::model::Step};
    use gem_client::testkit::MockClient;
    use primitives::asset_constants::CELO_WETH_TOKEN_ID;

    const ROUTER_ADDRESS: &str = "0xCcC88a9d1B4ED6b0EABA998850414b24f1c315bE";
    const QUOTE_VALUE: &str = "40000000000000000000";
    const ZERO_ALLOWANCE: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
    const SUFFICIENT_ALLOWANCE: &str = "0x0000000000000000000000000000000000000000000000022b1c8c1227a00000";

    fn mock_relay_with_allowance(allowance_result: &str) -> Relay<MockClient> {
        Relay {
            provider: ProviderType::new(SwapperProvider::Relay),
            rpc_provider: Arc::new(ProviderMock::new(format!(r#"{{"id":1,"jsonrpc":"2.0","result":"{allowance_result}"}}"#))),
            client: RelayClient::new(MockClient::new()),
        }
    }

    fn mock_quote(chain: Chain) -> Quote {
        let mut quote = Quote::mock(chain, None);
        quote.from_value = QUOTE_VALUE.to_string();
        quote.request.wallet_address = "0x1085c5f70F7F7591D97da281A64688385455c2bD".to_string();
        quote
    }

    fn mock_quote_response() -> RelayQuoteResponse {
        RelayQuoteResponse::mock_with_steps(vec![Step::mock_transaction("deposit", ROUTER_ADDRESS, "0", "0xf9e4bab4")])
    }

    #[tokio::test]
    async fn test_check_evm_approval_native_asset_contract() -> Result<(), SwapperError> {
        let relay = mock_relay_with_allowance(ZERO_ALLOWANCE);
        let approval = relay
            .check_evm_approval(&mock_quote(Chain::Celo), &mock_quote_response(), &AssetId::from_chain(Chain::Celo))
            .await?
            .unwrap();

        assert_eq!(approval.token, CELO_WETH_TOKEN_ID);
        assert_eq!(approval.spender, ROUTER_ADDRESS);
        assert_eq!(approval.value, QUOTE_VALUE);

        let relay = mock_relay_with_allowance(SUFFICIENT_ALLOWANCE);
        let approval = relay
            .check_evm_approval(&mock_quote(Chain::Celo), &mock_quote_response(), &AssetId::from_chain(Chain::Celo))
            .await?;
        assert!(approval.is_none());

        let relay = mock_relay_with_allowance(ZERO_ALLOWANCE);
        let approval = relay
            .check_evm_approval(&mock_quote(Chain::Ethereum), &mock_quote_response(), &AssetId::from_chain(Chain::Ethereum))
            .await?;
        assert!(approval.is_none());

        Ok(())
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::{
        AssetId,
        asset_constants::{BASE_USDC_ASSET_ID, CELO_WETH_TOKEN_ID, SMARTCHAIN_USDT_ASSET_ID, TEMPO_USDC_ASSET_ID},
    };
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_relay_eth_to_base() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use primitives::asset_constants::{ARBITRUM_USDC_ASSET_ID, BASE_USDC_ASSET_ID};

        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(ARBITRUM_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "500000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, data_len={}", quote_data.to, quote_data.value, quote_data.data.len());

        assert_eq!(quote.from_value, request.value);
        assert!(!quote.to_value.is_empty());
        assert!(!quote_data.data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_usdt_eth_to_base() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use primitives::asset_constants::ETHEREUM_USDT_ASSET_ID;

        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(ETHEREUM_USDT_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Base)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "5000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, data_len={}", quote_data.to, quote_data.value, quote_data.data.len());
        println!("approval: {:?}", quote_data.approval);

        assert_eq!(quote.from_value, request.value);
        assert!(!quote.to_value.is_empty());
        assert!(!quote_data.data.is_empty());
        assert!(!quote_data.to.is_empty());
        assert!(quote_data.approval.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_celo_to_bsc_usdt() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::new_with_endpoints(HashMap::from([(Chain::Celo, "https://forno.celo.org".to_string())])));
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Celo)),
            to_asset: SwapperQuoteAsset::from(SMARTCHAIN_USDT_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "40000000000000000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, gas_limit={:?}", quote_data.to, quote_data.value, quote_data.gas_limit);

        let approval = quote_data.approval.expect("native CELO swap requires an approval");
        assert_eq!(approval.token, CELO_WETH_TOKEN_ID);
        assert_eq!(quote_data.value, "0");
        assert!(quote_data.gas_limit.is_some());
        assert!(!quote_data.data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_arbitrum_eth_to_robinhood_eth() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Arbitrum)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Robinhood)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "5000000000000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, data_len={}", quote_data.to, quote_data.value, quote_data.data.len());

        assert_eq!(quote.from_value, request.value);
        assert!(!quote.to_value.is_empty());
        assert!(!quote_data.data.is_empty());
        assert!(!quote_data.to.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_tempo_usdc() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(TEMPO_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: "1000000".to_string(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!(quote_data.value, "0");
        assert!(!quote_data.to.is_empty());

        Ok(())
    }
}
