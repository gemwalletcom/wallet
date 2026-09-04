use num_bigint::BigUint;
use std::str::FromStr;
use std::sync::Arc;

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use gem_tron::address::TronAddress;
use primitives::{
    AssetId, Chain,
    swap::{ApprovalData, SlippageMode},
};

use super::{
    asset::{SUPPORTED_CHAINS, asset_to_currency},
    chain::{RelayChain, TON_CHAIN_ID},
    client::RelayClient,
    mapper,
    model::{RelayAppFee, RelayQuoteRequest, RelayQuoteResponse},
    solana, ton,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperProvider, SwapperQuoteData,
    approval::{check_approval_erc20, check_approval_trc20},
    client_factory::create_ton_client,
    config::get_swap_proxy_url,
    cross_chain::VaultAddresses,
    fees::{DEFAULT_REFERRER, default_referral_fees},
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
        let slippage_tolerance = match request.options.slippage.mode {
            SlippageMode::Auto => None,
            SlippageMode::Exact => Some(request.options.slippage.bps.to_string()),
        };

        let relay_request = RelayQuoteRequest {
            user: request.wallet_address.clone(),
            origin_chain_id: from_chain.chain_id().ok_or(SwapperError::NotSupportedChain)?,
            destination_chain_id: to_chain.chain_id().ok_or(SwapperError::NotSupportedChain)?,
            origin_currency,
            destination_currency,
            amount: from_value.to_string(),
            recipient: request.destination_address.clone(),
            trade_type: "EXACT_INPUT".to_string(),
            include_compute_unit_limit: true,
            slippage_tolerance,
            referrer: if app_fees.is_empty() { None } else { Some(DEFAULT_REFERRER.to_string()) },
            app_fees,
            refund_to: request.wallet_address.clone(),
            max_route_length: 6,
        };

        let response = self.client.get_quote(relay_request).await?;

        let to_value = BigUint::from_str(&response.details.currency_out.amount).map_err(SwapperError::compute_quote_error)?;
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
        let from_chain = RelayChain::from_chain(&from_asset_id.chain).ok_or(SwapperError::NotSupportedChain)?;
        let approval = self.check_approval(quote, &response, &from_asset_id).await?;
        match from_chain {
            RelayChain::Evm(_) => mapper::map_evm_quote_data(&response, approval),
            RelayChain::Tron => mapper::map_tron_quote_data(&response, approval),
            RelayChain::Solana => {
                let step = response.get_solana_step().ok_or(SwapperError::InvalidRoute)?;
                solana::build_quote_data(&quote.request.wallet_address, step, self.rpc_provider.clone()).await
            }
            RelayChain::Ton => mapper::map_ton_quote_data(&response),
        }
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        if chain == Chain::Ton {
            return self.get_ton_swap_result(transaction_hash).await;
        }
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
    async fn get_ton_swap_result(&self, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let client = create_ton_client(self.rpc_provider.clone())?;
        let Some(deposit) = ton::find_deposit(&client, transaction_hash).await? else {
            return Ok(SwapResult::pending());
        };
        let response = self.client.get_requests(&deposit.sender, TON_CHAIN_ID).await?;
        let request = response.requests.iter().find(|request| request.has_input_transaction(&deposit.transaction_hashes));
        Ok(request.map(mapper::map_swap_result).unwrap_or_else(SwapResult::pending))
    }

    async fn check_approval(&self, quote: &Quote, quote_response: &RelayQuoteResponse, from_asset_id: &AssetId) -> Result<Option<ApprovalData>, SwapperError> {
        let chain = RelayChain::from_chain(&from_asset_id.chain).ok_or(SwapperError::NotSupportedChain)?;
        let token = match (chain, from_asset_id.token_id.clone()) {
            (RelayChain::Solana | RelayChain::Ton, _) | (RelayChain::Tron, None) => return Ok(None),
            (_, Some(token)) => token,
            (RelayChain::Evm(chain), None) => match chain.native_asset_contract() {
                Some(token) => token.to_string(),
                None => return Ok(None),
            },
        };

        let spender = quote_response.router_address().ok_or(SwapperError::InvalidRoute)?;
        let amount = U256::from_str(&quote.from_value.to_string()).map_err(SwapperError::from)?;
        let approval = match chain {
            RelayChain::Evm(_) => {
                check_approval_erc20(
                    quote.request.wallet_address.clone(),
                    token,
                    spender,
                    amount,
                    self.rpc_provider.clone(),
                    &from_asset_id.chain,
                )
                .await?
            }
            RelayChain::Tron => {
                let spender = TronAddress::parse_hex_or_base58(&spender).map_err(|_| SwapperError::InvalidRoute)?.to_string();
                check_approval_trc20(quote.request.wallet_address.clone(), token, spender, amount, self.rpc_provider.clone()).await?
            }
            RelayChain::Solana | RelayChain::Ton => return Ok(None),
        };

        Ok(approval.approval_data())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperQuoteAsset, alien::mock::ProviderMock, approval::DEFAULT_TRON_SWAP_ENERGY_LIMIT, relay::model::Step};
    use gem_client::testkit::MockClient;
    use primitives::asset_constants::{BASE_USDC_ASSET_ID, CELO_WETH_TOKEN_ID, TRON_USDT_TOKEN_ID};

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

    fn mock_relay_with_tron_allowance(allowance: &str) -> Relay<MockClient> {
        Relay {
            provider: ProviderType::new(SwapperProvider::Relay),
            rpc_provider: Arc::new(ProviderMock::new(format!(r#"{{"constant_result":["{allowance}"]}}"#))),
            client: RelayClient::new(MockClient::new()),
        }
    }

    fn mock_quote(chain: Chain) -> Quote {
        let mut quote = Quote::mock(chain, None);
        quote.from_value = QUOTE_VALUE.parse().unwrap();
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
            .check_approval(&mock_quote(Chain::Celo), &mock_quote_response(), &AssetId::from_chain(Chain::Celo))
            .await?
            .unwrap();

        assert_eq!(approval.token, CELO_WETH_TOKEN_ID);
        assert_eq!(approval.spender, ROUTER_ADDRESS);
        assert_eq!(approval.value.to_string(), QUOTE_VALUE);

        let relay = mock_relay_with_allowance(SUFFICIENT_ALLOWANCE);
        let approval = relay
            .check_approval(&mock_quote(Chain::Celo), &mock_quote_response(), &AssetId::from_chain(Chain::Celo))
            .await?;
        assert!(approval.is_none());

        let relay = mock_relay_with_allowance(ZERO_ALLOWANCE);
        let approval = relay
            .check_approval(&mock_quote(Chain::Ethereum), &mock_quote_response(), &AssetId::from_chain(Chain::Ethereum))
            .await?;
        assert!(approval.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_tron_assets_mock() -> Result<(), SwapperError> {
        let route_data = include_str!("testdata/quote_tron_usdt_to_base_usdc.json").to_string();
        let mut quote = Quote::mock(Chain::Tron, Some(TRON_USDT_TOKEN_ID));
        quote.data.provider = ProviderType::new(SwapperProvider::Relay);
        quote.request.to_asset = SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone());
        quote.data.routes = vec![Route {
            input: AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID),
            output: BASE_USDC_ASSET_ID.clone(),
            route_data,
        }];
        quote.request.wallet_address = "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z".to_string();

        let relay = mock_relay_with_tron_allowance("0");
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;
        let approval = quote_data.approval.unwrap();
        assert_eq!(quote_data.to, "TXtEs6t2oUWQsNos7m68gbHdE9Q5n6x2oN");
        assert_eq!(approval.token, TRON_USDT_TOKEN_ID);
        assert_eq!(approval.spender, quote_data.to);
        assert_eq!(approval.value, quote.from_value);
        assert!(approval.is_unlimited);
        assert_eq!(quote_data.gas_limit, Some(DEFAULT_TRON_SWAP_ENERGY_LIMIT.to_string()));

        let relay = mock_relay_with_tron_allowance("f4240");
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(quote_data.approval.is_none());
        assert!(quote_data.gas_limit.is_none());

        let mut native_quote = Quote::mock(Chain::Tron, None);
        native_quote.data.provider = ProviderType::new(SwapperProvider::Relay);
        native_quote.request.to_asset = SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone());
        native_quote.request.value = BigUint::from(10000000u64);
        native_quote.data.routes = vec![Route {
            input: AssetId::from_chain(Chain::Tron),
            output: BASE_USDC_ASSET_ID.clone(),
            route_data: include_str!("testdata/quote_tron_to_base_usdc.json").to_string(),
        }];
        native_quote.request.wallet_address = quote.request.wallet_address.clone();
        native_quote.from_value = native_quote.request.value.clone();
        let quote_data = relay.get_quote_data(&native_quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.value, native_quote.from_value);
        assert!(quote_data.approval.is_none());

        Ok(())
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, models::Options};
    use primitives::{
        AssetId,
        asset_constants::{
            BASE_USDC_ASSET_ID, CELO_WETH_TOKEN_ID, SMARTCHAIN_USDT_ASSET_ID, SOLANA_USDC_ASSET_ID, SOLANA_USDT_ASSET_ID, TEMPO_BRIDGED_USDC_ASSET_ID, TRON_USDT_ASSET_ID,
        },
        swap::SwapStatus,
        testkit::signer_mock::TEST_TON_SENDER,
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
            value: BigUint::from(500000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, data_len={}", quote_data.to, quote_data.value, quote_data.data.len());

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert!(!quote_data.data.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_tron_assets_live() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let native_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Tron)),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "TW1dU4L3eNm7Lw8WvieLKEHpXWAussRG9Z".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(10000000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&native_request).await?;

        assert_eq!(quote.from_value, native_request.value);
        assert_eq!(quote.data.slippage_bps, native_request.options.slippage.bps);
        assert!(quote.to_value > BigUint::ZERO);

        let mut auto_slippage_request = native_request.clone();
        auto_slippage_request.options.slippage.mode = SlippageMode::Auto;
        let quote = relay.get_quote(&auto_slippage_request).await?;
        assert!(quote.data.slippage_bps > 0);

        let mut too_small_request = native_request.clone();
        too_small_request.value = BigUint::from(20000u64);
        assert_eq!(relay.get_quote(&too_small_request).await.unwrap_err(), SwapperError::InputAmountError { min_amount: None });

        let usdt_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(TRON_USDT_ASSET_ID.clone()),
            value: BigUint::from(1000000u64),
            ..native_request
        };
        let quote = relay.get_quote(&usdt_request).await?;

        assert_eq!(quote.from_value, usdt_request.value);
        assert!(quote.to_value > BigUint::ZERO);

        let reverse_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(TRON_USDT_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: usdt_request.wallet_address.clone(),
            value: BigUint::from(10000000u64),
            options: Options::new_with_slippage(100.into()),
        };
        let quote = relay.get_quote(&reverse_request).await?;

        assert_eq!(quote.from_value, reverse_request.value);
        assert!(quote.to_value > BigUint::ZERO);

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_solana_live() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Solana)),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(100000000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;
        println!("solana transaction: {}", quote_data.data);
        let transaction = gem_solana::decode_transaction(&quote_data.data)?;
        assert_eq!(transaction.num_required_signatures(), 1);
        assert!(quote_data.gas_limit.is_some());
        assert!(quote_data.approval.is_none());

        let usdt_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(SOLANA_USDT_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(20000000u64),
            options: Options::new_with_slippage(100.into()),
        };
        let quote = relay.get_quote(&usdt_request).await?;
        assert_eq!(quote.from_value, usdt_request.value);
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(gem_solana::decode_transaction(&quote_data.data).is_ok());
        assert!(quote_data.approval.is_none());

        let reverse_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(SOLANA_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: request.wallet_address.clone(),
            value: BigUint::from(10000000u64),
            options: Options::new_with_slippage(100.into()),
        };
        let quote = relay.get_quote(&reverse_request).await?;
        assert_eq!(quote.from_value, reverse_request.value);
        assert!(quote.to_value > BigUint::ZERO);

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_ton_live() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(5_000_000_000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;
        println!("ton quote_data: to={}, value={}, data={}", quote_data.to, quote_data.value, quote_data.data);
        assert!(quote_data.to.starts_with("EQ"));
        assert_eq!(quote_data.value, request.value);
        assert!(quote_data.data.starts_with("te6cc"));
        assert!(quote_data.approval.is_none());

        let reverse_request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: BigUint::from(10_000_000u64),
            options: Options::new_with_slippage(100.into()),
        };
        let quote = relay.get_quote(&reverse_request).await?;
        assert_eq!(quote.from_value, reverse_request.value);
        assert!(quote.to_value > BigUint::ZERO);

        let result = relay
            .get_swap_result(Chain::Ton, "e86159ff0662a587649bc1d2ff0cd146e6628c3cc37396f7b680bd28260f44b5")
            .await?;
        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(result.metadata.unwrap().from_asset, AssetId::from_chain(Chain::Ton));

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
            value: BigUint::from(5000000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, data_len={}", quote_data.to, quote_data.value, quote_data.data.len());
        println!("approval: {:?}", quote_data.approval);

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
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
            value: BigUint::parse_bytes(b"40000000000000000000", 10).unwrap(),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, gas_limit={:?}", quote_data.to, quote_data.value, quote_data.gas_limit);

        let approval = quote_data.approval.expect("native CELO swap requires an approval");
        assert_eq!(approval.token, CELO_WETH_TOKEN_ID);
        assert_eq!(quote_data.value, BigUint::from(0u64));
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
            value: BigUint::from(5000000000000000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        println!("quote: from_value={}, to_value={}", quote.from_value, quote.to_value);
        println!("quote_data: to={}, value={}, data_len={}", quote_data.to, quote_data.value, quote_data.data.len());

        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert!(!quote_data.data.is_empty());
        assert!(!quote_data.to.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_relay_tempo_usdc() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let provider = Arc::new(NativeProvider::default());
        let relay = Relay::new(provider);

        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(TEMPO_BRIDGED_USDC_ASSET_ID.clone()),
            to_asset: SwapperQuoteAsset::from(BASE_USDC_ASSET_ID.clone()),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(1000000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = relay.get_quote(&request).await?;
        let quote_data = relay.get_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert!(!quote_data.to.is_empty());

        Ok(())
    }
}
