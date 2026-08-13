use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use gem_client::Client;
use gem_sui::{build_transfer_message_bytes, rpc::SuiClient};
use num_bigint::BigUint;
use primitives::{
    AssetId, Chain, TransactionSwapMetadata,
    contract_constants::EVM_ZERO_ADDRESS,
    swap::{HUNDRED_PERCENT_IN_BPS, SwapStatus},
};

use super::{
    chain::SwapsXyzChain,
    client::SwapsXyzClient,
    model::{ActionRequest, ActionResponse, AmountLimits, AppFee},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, SwapResult, Swapper, SwapperChainAsset, SwapperError,
    SwapperProvider, SwapperQuoteData, amount_to_value, client_factory::create_sui_client, config::API_BASE_URL, fees::default_referral_fees,
};

pub struct SwapsXyz<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: SwapsXyzClient<C>,
    sui_client: SuiClient,
}

impl<C> Debug for SwapsXyz<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SwapsXyz")
            .field("provider", &self.provider)
            .field("client", &self.client)
            .field("sui_client", &"SuiClient")
            .finish()
    }
}

impl SwapsXyz<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let sui_client = create_sui_client(rpc_provider.clone()).expect("failed to create Sui gRPC client");
        Self::with_client(
            SwapsXyzClient::new(
                RpcClient::new(super::base_url(), rpc_provider.clone()),
                RpcClient::new(format!("{API_BASE_URL}/v1/swaps/{}", SwapperProvider::SwapsXyz.as_ref()), rpc_provider),
            ),
            sui_client,
        )
    }
}

impl<C> SwapsXyz<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn with_client(client: SwapsXyzClient<C>, sui_client: SuiClient) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::SwapsXyz),
            client,
            sui_client,
        }
    }

    fn build_action_request(request: &QuoteRequest, source: SwapsXyzChain, destination: SwapsXyzChain) -> Result<ActionRequest, SwapperError> {
        if !request.from_asset.is_native() || !request.to_asset.is_native() {
            return Err(SwapperError::NotSupportedAsset);
        }
        let fee = default_referral_fees().evm;
        let app_fees = serde_json::to_string(&[AppFee {
            bps: fee.bps,
            receiver_address: fee.address,
        }])?;
        Ok(ActionRequest {
            action_type: "swap-action".into(),
            sender: request.wallet_address.clone(),
            src_chain_id: source.id,
            src_token: EVM_ZERO_ADDRESS.into(),
            dst_chain_id: destination.id,
            dst_token: EVM_ZERO_ADDRESS.into(),
            slippage: request.options.slippage.bps,
            amount: request.value.clone(),
            swap_direction: "exact-amount-in".into(),
            recipient: request.destination_address.clone(),
            refund_to: request.wallet_address.clone(),
            return_deposit_address: true,
            app_fees,
        })
    }

    fn validate_amount(limits: &AmountLimits, value: &str, decimals: u32) -> Result<(), SwapperError> {
        let value = value.parse::<BigUint>().map_err(SwapperError::compute_quote_error)?;
        let minimum = amount_to_value(&limits.min_amount, decimals).ok_or_else(|| SwapperError::ComputeQuoteError("Invalid minimum amount".into()))?;
        let minimum = minimum.parse::<BigUint>().map_err(SwapperError::compute_quote_error)?;
        if value < minimum {
            return Err(SwapperError::InputAmountError {
                min_amount: Some(minimum.to_string()),
            });
        }
        if let Some(maximum) = limits.max_amount.as_deref() {
            let maximum = amount_to_value(maximum, decimals).ok_or_else(|| SwapperError::ComputeQuoteError("Invalid maximum amount".into()))?;
            let maximum = maximum.parse::<BigUint>().map_err(SwapperError::compute_quote_error)?;
            if value > maximum {
                return Err(SwapperError::ComputeQuoteError("Input amount exceeds route maximum".into()));
            }
        }
        Ok(())
    }

    fn validate_response(response: &ActionResponse, request: &QuoteRequest, source: SwapsXyzChain, destination: SwapsXyzChain) -> Result<(), SwapperError> {
        let fee = default_referral_fees().evm;
        let input = request.value.parse::<BigUint>().map_err(SwapperError::compute_quote_error)?;
        let expected_fee = input * BigUint::from(fee.bps) / BigUint::from(HUNDRED_PERCENT_IN_BPS);
        let actual_fee = response.application_fee.amount.parse::<BigUint>().map_err(SwapperError::compute_quote_error)?;
        let valid = response.vm_id == "alt-vm"
            && !response.tx.to.is_empty()
            && response.tx.value == request.value
            && response.tx.chain_id == source.id
            && response.tx.chain_key == source.key
            && response.amount_in.amount == request.value
            && response.amount_in.native_chain() == Some(source)
            && response.amount_out.native_chain() == Some(destination)
            && response.amount_out.amount.parse::<BigUint>().is_ok_and(|amount| amount > BigUint::from(0_u8))
            && response.application_fee.native_chain() == Some(source)
            && actual_fee == expected_fee;
        if valid { Ok(()) } else { Err(SwapperError::InvalidRoute) }
    }

    fn map_status(status: &str) -> SwapStatus {
        match status.to_ascii_lowercase().as_str() {
            "success" | "completed" => SwapStatus::Completed,
            "failed" | "refunded" | "requires refund" | "expired" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
    }
}

#[async_trait]
impl<C> Swapper for SwapsXyz<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        SwapsXyzChain::ALL.iter().map(|chain| SwapperChainAsset::assets(chain.chain, [])).collect()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let source = SwapsXyzChain::from_chain(request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let destination = SwapsXyzChain::from_chain(request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let paths = self.client.get_paths(source.id, destination.id).await?;
        let path = paths
            .paths
            .iter()
            .find(|path| path.chain_id == destination.id && path.supports_exact_amount_in && path.tokens.iter().any(|token| token.is_native && token.address == EVM_ZERO_ADDRESS))
            .ok_or(SwapperError::NoQuoteAvailable)?;
        Self::validate_amount(&path.amount_limits, &request.value, source.decimals())?;
        let action_request = Self::build_action_request(request, source, destination)?;
        let response = self.client.get_action(&action_request).await?;
        Self::validate_response(&response, request, source, destination)?;
        let eta_in_seconds = if response.estimated_tx_time.is_finite() && response.estimated_tx_time >= 0.0 && response.estimated_tx_time <= u32::MAX as f64 {
            Some(response.estimated_tx_time.ceil() as u32)
        } else {
            None
        };
        Ok(Quote {
            from_value: response.amount_in.amount.clone(),
            min_from_value: None,
            to_value: response.amount_out.amount.clone(),
            data: ProviderData {
                provider: self.provider.clone(),
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&response)?,
                }],
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds,
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let response: ActionResponse = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let source = SwapsXyzChain::from_chain(quote.request.from_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        let destination = SwapsXyzChain::from_chain(quote.request.to_asset.chain()).ok_or(SwapperError::NotSupportedChain)?;
        Self::validate_response(&response, &quote.request, source, destination)?;
        let data = SwapperQuoteData::new_transfer(response.tx.to, response.tx.value, response.tx.to_extra.filter(|memo| !memo.is_empty()));
        if source.chain != Chain::Sui {
            return Ok(data);
        }
        let amount = data
            .value
            .parse::<u64>()
            .map_err(|_| SwapperError::ComputeQuoteError("Invalid Sui amount provided for deposit".into()))?;
        let payload = build_transfer_message_bytes(&self.sui_client, &quote.request.wallet_address, &data.to, amount, None)
            .await
            .map_err(|error| SwapperError::TransactionError(format!("Failed to build Sui deposit data: {error}")))?;
        Ok(SwapperQuoteData { data: payload, ..data })
    }

    async fn get_swap_result(&self, chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let chain = SwapsXyzChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let Some(response) = self.client.get_status(transaction_hash, chain.id).await? else {
            return Ok(SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
            });
        };
        let metadata = response.action_response.and_then(|status| {
            Some(TransactionSwapMetadata {
                from_asset: AssetId::from_chain(status.amount_in.native_chain()?.chain),
                from_value: status.amount_in.amount,
                to_asset: AssetId::from_chain(status.amount_out.native_chain()?.chain),
                to_value: status.amount_out.amount,
                provider: Some(SwapperProvider::SwapsXyz.as_ref().to_string()),
            })
        });
        Ok(SwapResult {
            status: Self::map_status(&response.status),
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, SwapperQuoteAsset};
    use gem_client::{ClientError, testkit::MockClient};

    fn request() -> QuoteRequest {
        QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Cosmos)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Stellar)),
            wallet_address: "COSMOS_ADDRESS".into(),
            destination_address: "STELLAR_ADDRESS".into(),
            value: "1000000".into(),
            options: Options::default(),
        }
    }

    fn response() -> ActionResponse {
        serde_json::from_str(include_str!("testdata/action_response.json")).unwrap()
    }

    #[test]
    fn test_build_action_request_includes_referral_fee() {
        let request = request();
        let action = SwapsXyz::<MockClient>::build_action_request(
            &request,
            SwapsXyzChain::from_chain(request.from_asset.chain()).unwrap(),
            SwapsXyzChain::from_chain(request.to_asset.chain()).unwrap(),
        )
        .unwrap();
        let fees: Vec<AppFee> = serde_json::from_str(&action.app_fees).unwrap();
        let referral_fee = default_referral_fees().evm;
        assert_eq!(
            fees,
            vec![AppFee {
                bps: referral_fee.bps,
                receiver_address: referral_fee.address,
            }]
        );
        assert!(action.return_deposit_address);
    }

    #[test]
    fn test_validate_amount() {
        let limits = AmountLimits {
            min_amount: "10.5".into(),
            max_amount: None,
        };
        assert_eq!(
            SwapsXyz::<MockClient>::validate_amount(&limits, "100000000", 7),
            Err(SwapperError::InputAmountError {
                min_amount: Some("105000000".into())
            })
        );
        assert_eq!(SwapsXyz::<MockClient>::validate_amount(&limits, "105000000", 7), Ok(()));

        let invalid_maximum = AmountLimits {
            min_amount: "1".into(),
            max_amount: Some("invalid".into()),
        };
        assert_eq!(
            SwapsXyz::<MockClient>::validate_amount(&invalid_maximum, "105000000", 7),
            Err(SwapperError::ComputeQuoteError("Invalid maximum amount".into()))
        );
    }

    #[test]
    fn test_validate_response() {
        let request = request();
        let source = SwapsXyzChain::from_chain(request.from_asset.chain()).unwrap();
        let destination = SwapsXyzChain::from_chain(request.to_asset.chain()).unwrap();
        let response = response();
        assert_eq!(SwapsXyz::<MockClient>::validate_response(&response, &request, source, destination), Ok(()));

        let mut wrong_fee = response;
        wrong_fee.application_fee.amount = "0".into();
        assert_eq!(
            SwapsXyz::<MockClient>::validate_response(&wrong_fee, &request, source, destination),
            Err(SwapperError::InvalidRoute)
        );
    }

    #[tokio::test]
    async fn test_missing_unregistered_status_is_pending() {
        let upstream = MockClient::new().with_get(|_| Err(ClientError::Http { status: 404, body: vec![] }));
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(upstream, MockClient::new()), SuiClient::new("https://example.com"));
        let result = provider.get_swap_result(Chain::Algorand, "source-hash").await.unwrap();
        assert_eq!(result.status, SwapStatus::Pending);
    }

    #[tokio::test]
    async fn test_quote_data_preserves_deposit_memo() {
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(MockClient::new(), MockClient::new()), SuiClient::new("https://example.com"));
        let request = request();
        let response = response();
        let quote = Quote {
            from_value: response.amount_in.amount.clone(),
            min_from_value: None,
            to_value: response.amount_out.amount.clone(),
            data: ProviderData {
                provider: provider.provider.clone(),
                slippage_bps: request.options.slippage.bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&response).unwrap(),
                }],
            },
            request,
            eta_in_seconds: None,
        };

        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!(data.memo.as_deref(), Some("1234"));
    }

    #[test]
    fn test_map_status() {
        assert_eq!(SwapsXyz::<MockClient>::map_status("completed"), SwapStatus::Completed);
        assert_eq!(SwapsXyz::<MockClient>::map_status("requires refund"), SwapStatus::Failed);
        assert_eq!(SwapsXyz::<MockClient>::map_status("submitted"), SwapStatus::Pending);
    }
}
