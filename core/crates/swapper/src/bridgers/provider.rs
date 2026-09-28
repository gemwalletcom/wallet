use std::{fmt::Debug, str::FromStr, sync::Arc};

use alloy_primitives::U256;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    Chain, ChainType,
    swap::{SwapResult, SwapStatus},
};

use super::{
    asset::{Network, Token, supported_assets, vault_addresses},
    client::BridgersClient,
    model::{QuoteRequest as BridgersQuoteRequest, RecordsRequest, RouteData, SwapRequest},
    transaction::get_transaction_value,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    approval::{check_approval_erc20, get_swap_gas_limit_with_approval},
    client_factory::create_eth_client,
    config::get_swap_proxy_url,
    cross_chain::VaultAddresses,
    fees::DEFAULT_REFERRER,
    hyperliquid::provider::spot::math::scale_units,
    models::ApprovalType,
};

const SLIPPAGE_DECIMALS: i32 = 4;
const SWAP_GAS_LIMIT: u64 = 150_000;

#[derive(Debug)]
pub struct Bridgers<C: Client + Clone + Send + Sync + Debug + 'static> {
    provider: ProviderType,
    client: BridgersClient<C>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl Bridgers<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self::new_with_client(RpcClient::new(get_swap_proxy_url(SwapperProvider::Bridgers.as_ref()), rpc_provider.clone()), rpc_provider)
    }
}

impl<C: Client + Clone + Send + Sync + Debug + 'static> Bridgers<C> {
    pub fn new_with_client(client: C, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Bridgers),
            client: BridgersClient::new(client),
            rpc_provider,
        }
    }

    async fn get_evm_quote_data(&self, network: Network, from_token: &Token, to_token: &Token, swap: &SwapRequest) -> Result<SwapperQuoteData, SwapperError> {
        let transaction = self.client.get_swap(swap).await?.tx_data;
        let router = network.router()?;
        let value = get_transaction_value(&transaction, router, from_token, to_token, swap)?;
        let approval = match &from_token.asset_id.token_id {
            None => None,
            Some(token_id) => match check_approval_erc20(
                swap.from_address.clone(),
                token_id.clone(),
                router.to_string(),
                U256::from_str(&swap.quote.from_token_amount)?,
                self.rpc_provider.clone(),
                &network.chain,
            )
            .await?
            {
                ApprovalType::Approve(data) => Some(data),
                ApprovalType::Permit2(_) | ApprovalType::None => None,
            },
        };
        let gas_limit = get_swap_gas_limit_with_approval(&approval, None, SWAP_GAS_LIMIT);
        Ok(SwapperQuoteData::new_contract(router.to_string(), u256_to_biguint(&value), transaction.data, approval, gas_limit))
    }
}

fn get_quote_request(request: &QuoteRequest, value: &BigUint) -> Result<BridgersQuoteRequest, SwapperError> {
    let from_network = Network::from_chain(request.from_asset.chain())?;
    from_network.router()?;
    let from_token = Token::from_asset_id(&request.from_asset.asset_id())?;
    let to_token = Token::from_asset_id(&request.to_asset.asset_id())?;
    Ok(BridgersQuoteRequest {
        source_flag: DEFAULT_REFERRER.to_string(),
        from_token_address: from_token.address(),
        to_token_address: to_token.address(),
        from_token_amount: scale_units(value.clone(), request.from_asset.decimals, from_token.decimals)?.to_string(),
        from_token_chain: from_network.code.to_string(),
        to_token_chain: Network::from_chain(request.to_asset.chain())?.code.to_string(),
    })
}

#[async_trait]
impl<C: Client + Clone + Send + Sync + Debug + 'static> Swapper for Bridgers<C> {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        supported_assets()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let quote = self.client.get_quote(&get_quote_request(request, &request.value)?).await?.tx_data;

        let min_value = BigNumberFormatter::value_from_amount_biguint(&quote.deposit_min, request.from_asset.decimals).map_err(SwapperError::compute_quote_error)?;
        if request.value < min_value {
            return Err(SwapperError::InputAmountError { min_amount: Some(min_value.to_string()) });
        }
        if request.value > BigNumberFormatter::value_from_amount_biguint(&quote.deposit_max, request.from_asset.decimals).map_err(SwapperError::compute_quote_error)? {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let to_amount = BigNumberFormatter::value_from_amount_biguint(&quote.to_token_amount, request.to_asset.decimals).map_err(SwapperError::compute_quote_error)?;
        let chain_fee = BigNumberFormatter::value_from_amount_biguint(&quote.chain_fee, request.to_asset.decimals).map_err(SwapperError::compute_quote_error)?;
        if to_amount <= chain_fee {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let to_value = to_amount - chain_fee;
        let route_data = RouteData { amount_out_min: quote.amount_out_min };

        Ok(Quote {
            from_value: request.value.clone(),
            min_from_value: Some(min_value),
            to_value,
            data: ProviderData {
                provider: self.provider.clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data).map_err(SwapperError::compute_quote_error)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let sender = match chain.chain_type() {
            ChainType::Ethereum => {
                create_eth_client(self.rpc_provider.clone(), chain)?
                    .get_transaction_by_hash(transaction_hash)
                    .await
                    .map_err(SwapperError::compute_quote_error)?
                    .ok_or(SwapperError::InvalidRoute)?
                    .from
            }
            _ => return Err(SwapperError::NotSupportedChain),
        };
        let records = self.client.get_records(&RecordsRequest { from_address: sender }).await?;
        let status = records
            .list
            .iter()
            .find(|record| record.hash.eq_ignore_ascii_case(transaction_hash))
            .map_or(SwapStatus::Pending, |record| record.status.swap_status());
        Ok(SwapResult {
            status,
            metadata: None,
            eta_in_seconds: None,
        })
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(vault_addresses())
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let network = Network::from_chain(request.from_asset.chain())?;
        let from_token = Token::from_asset_id(&request.from_asset.asset_id())?;
        let to_token = Token::from_asset_id(&request.to_asset.asset_id())?;
        let route: RouteData = serde_json::from_str(&quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let swap = SwapRequest {
            quote: get_quote_request(request, &quote.from_value)?,
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            amount_out_min: route.amount_out_min,
            slippage: BigNumberFormatter::value(&quote.data.slippage_bps.to_string(), SLIPPAGE_DECIMALS).map_err(SwapperError::compute_quote_error)?,
        };
        match network.chain.chain_type() {
            ChainType::Ethereum => self.get_evm_quote_data(network, &from_token, &to_token, &swap).await,
            _ => Err(SwapperError::NotSupportedChain),
        }
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        alien::reqwest_provider::NativeProvider,
        bridgers::testkit::{mock_base_usdc_to_bsc_usdt_request, mock_opbnb_to_bsc_usdt_request},
    };

    const BRIDGERS_API_URL: &str = "https://api.bridgers.xyz";

    #[tokio::test]
    async fn test_bridgers_quote_and_quote_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Bridgers::new_with_client(RpcClient::new(BRIDGERS_API_URL.to_string(), rpc_provider.clone()), rpc_provider);

        for request in [mock_opbnb_to_bsc_usdt_request(), mock_base_usdc_to_bsc_usdt_request()] {
            let quote = provider.get_quote(&request).await?;
            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            let network = Network::from_chain(request.from_asset.chain())?;

            assert!(quote.to_value > BigUint::ZERO);
            assert_eq!(data.to, network.router()?);
        }

        let result = provider.get_swap_result(Chain::OpBNB, "0xb55fa0488d55291f1036b1b707309b1e6101f57dddb3b43fc5d3d336fd960db5").await?;

        assert_eq!(result.status, SwapStatus::Completed);
        Ok(())
    }
}
