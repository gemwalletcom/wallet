use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use gem_client::Client;
use gem_evm::{
    constants::TOKEN_TRANSFER_GAS_LIMIT,
    u256::{biguint_to_u256, u256_to_biguint},
};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, ChainType,
    swap::{ApprovalData, SwapResult, SwapResultRequest, SwapStatus},
};

use super::{
    asset::{Network, get_token_address, get_token_code, supported_assets, vault_addresses},
    client::BridgersClient,
    model::{QuoteRequest as BridgersQuoteRequest, QuoteTxData, Record, RecordsRequest, RouteData, SwapRequest},
    transaction::get_transaction_value,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    approval::{check_approval_erc20, get_swap_gas_limit_with_approval},
    config::get_swap_proxy_url,
    cross_chain::VaultAddresses,
    fees::DEFAULT_REFERRER,
    models::ApprovalType,
};

#[derive(Debug)]
pub struct Bridgers<C: Client + Clone + Send + Sync + Debug + 'static> {
    provider: ProviderType,
    client: BridgersClient<C>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl Bridgers<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Bridgers),
            client: BridgersClient::new(RpcClient::new(get_swap_proxy_url(SwapperProvider::Bridgers.as_ref()), rpc_provider.clone())),
            rpc_provider,
        }
    }
}

impl<C: Client + Clone + Send + Sync + Debug + 'static> Bridgers<C> {
    async fn get_evm_quote_data(&self, network: Network, from_asset: &AssetId, to_code: &str, swap: &SwapRequest) -> Result<SwapperQuoteData, SwapperError> {
        let transaction = self.client.get_swap(swap).await?.tx_data;
        let router = network.router()?;
        let value = get_transaction_value(&transaction, router, from_asset, to_code, swap)?;
        let approval = self.get_approval(network, router, from_asset, swap).await?;
        let gas_limit = get_swap_gas_limit_with_approval(&approval, None, TOKEN_TRANSFER_GAS_LIMIT * 2);
        Ok(SwapperQuoteData::new_contract(router.to_string(), u256_to_biguint(&value), transaction.data, approval, gas_limit))
    }

    async fn get_approval(&self, network: Network, router: &str, from_asset: &AssetId, swap: &SwapRequest) -> Result<Option<ApprovalData>, SwapperError> {
        let Some(token_id) = &from_asset.token_id else {
            return Ok(None);
        };
        let amount = biguint_to_u256(&swap.quote.from_token_amount).ok_or(SwapperError::InvalidRoute)?;
        match check_approval_erc20(swap.from_address.clone(), token_id.clone(), router.to_string(), amount, self.rpc_provider.clone(), &network.chain).await? {
            ApprovalType::Approve(data) => Ok(Some(data)),
            ApprovalType::Permit2(_) | ApprovalType::None => Ok(None),
        }
    }
}

fn get_min_from_value(quote: &QuoteTxData, request: &QuoteRequest) -> Result<BigUint, SwapperError> {
    let (min_value, max_value) = quote.get_deposit_range(request.from_asset.decimals)?;
    if request.value < min_value {
        return Err(SwapperError::InputAmountError { min_amount: Some(min_value.to_string()) });
    }
    if request.value > max_value {
        return Err(SwapperError::NoQuoteAvailable);
    }
    Ok(min_value)
}

fn get_quote_request(request: &QuoteRequest, value: &BigUint) -> Result<BridgersQuoteRequest, SwapperError> {
    let from_network = Network::from_chain(request.from_asset.chain())?;
    from_network.router()?;
    let from_asset = request.from_asset.asset_id();
    let to_asset = request.to_asset.asset_id();
    get_token_code(&from_asset)?;
    get_token_code(&to_asset)?;
    Ok(BridgersQuoteRequest {
        source_flag: DEFAULT_REFERRER.to_string(),
        from_token_address: get_token_address(&from_asset),
        to_token_address: get_token_address(&to_asset),
        from_token_amount: value.clone(),
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

        let min_value = get_min_from_value(&quote, request)?;
        let to_value = quote.get_to_value(request.to_asset.decimals)?;
        let eta_in_seconds = quote.eta_in_seconds();
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
                    route_data: serde_json::to_string(&route_data)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds,
        })
    }

    async fn get_swap_result(&self, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        let records = self.client.get_records(&RecordsRequest::new(request.from_address.clone().ok_or(SwapperError::InvalidRoute)?)).await?;
        let record = records.list.iter().find(|record| record.hash == request.transaction_hash);
        Ok(SwapResult {
            status: record.map_or(SwapStatus::Pending, |record| record.status.swap_status()),
            metadata: record.and_then(Record::swap_metadata),
            eta_in_seconds: None,
        })
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(vault_addresses())
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let network = Network::from_chain(request.from_asset.chain())?;
        let to_code = get_token_code(&request.to_asset.asset_id())?;
        let route: RouteData = serde_json::from_str(&quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let swap = SwapRequest {
            quote: get_quote_request(request, &quote.from_value)?,
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            amount_out_min: route.amount_out_min,
            slippage: BigNumberFormatter::value(&quote.data.slippage_bps.to_string(), 4)?,
        };
        match network.chain.chain_type() {
            ChainType::Ethereum => self.get_evm_quote_data(network, &request.from_asset.asset_id(), to_code, &swap).await,
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
    use primitives::{Chain, TransactionSwapMetadata, asset_constants::ARBITRUM_USDC_ASSET_ID};

    #[tokio::test]
    async fn test_bridgers_quote_and_quote_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Bridgers::new(rpc_provider);

        for request in [mock_opbnb_to_bsc_usdt_request(), mock_base_usdc_to_bsc_usdt_request()] {
            let quote = provider.get_quote(&request).await?;
            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            let network = Network::from_chain(request.from_asset.chain())?;

            assert!(quote.to_value > BigUint::ZERO);
            assert!(quote.eta_in_seconds > Some(0));
            assert_eq!(data.to, network.router()?);
        }

        let request = SwapResultRequest {
            from_address: Some("0x1085c5f70f7f7591d97da281a64688385455c2bd".to_string()),
            ..SwapResultRequest::new(Chain::Arbitrum, "0x8cb98cd501b2e0b5da96a97492c1f1819fe2a72de6db8a8c40c6d5b862a7cd28")
        };
        let result = provider.get_swap_result(&request).await?;

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(TransactionSwapMetadata::new(
                    ARBITRUM_USDC_ASSET_ID.clone(),
                    BigUint::from(20_000_000u64),
                    AssetId::from_chain(Chain::OpBNB),
                    BigUint::from(25_126_000_000_000_000u64),
                    SwapperProvider::Bridgers,
                )),
                eta_in_seconds: None,
            }
        );
        Ok(())
    }
}
