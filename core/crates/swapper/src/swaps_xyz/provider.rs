use std::{fmt::Debug, sync::Arc};

use async_trait::async_trait;
use gem_client::Client;
use gem_encoding::decode_base64;
use gem_ton::rpc::client::TonClient;
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, Chain, OptionStringExt, TransactionSwapMetadata, TransactionSwapReferralFee,
    contract_constants::EVM_ZERO_ADDRESS,
    swap::{HUNDRED_PERCENT_IN_BPS, SwapStatus},
};

use super::{
    chain::SwapsXyzChain,
    client::SwapsXyzClient,
    model::{ActionRequest, ActionResponse, AmountLimits, AppFee, StatusResponse},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, SwapResult, SwapResultRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    client_factory::create_ton_client, config::get_swap_provider_url, fees::default_referral_fees,
};

const APP_ID: &str = "0xcf511a8a";

pub struct SwapsXyz<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: SwapsXyzClient<C>,
    ton_client: TonClient<C>,
}

impl<C> Debug for SwapsXyz<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_struct("SwapsXyz").field("provider", &self.provider).field("client", &self.client).finish()
    }
}

impl SwapsXyz<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        let ton_client = create_ton_client(rpc_provider.clone()).ok()?;
        Some(Self::with_client(
            SwapsXyzClient::new(RpcClient::new(super::base_url(), rpc_provider.clone()), RpcClient::new(get_swap_provider_url(SwapperProvider::SwapsXyz), rpc_provider)),
            ton_client,
        ))
    }
}

impl<C> SwapsXyz<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn with_client(client: SwapsXyzClient<C>, ton_client: TonClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::SwapsXyz),
            client,
            ton_client,
        }
    }

    fn build_action_request(request: &QuoteRequest, source: SwapsXyzChain, destination: SwapsXyzChain) -> Result<ActionRequest, SwapperError> {
        if !request.from_asset.is_native() || !request.to_asset.is_native() {
            return Err(SwapperError::NotSupportedAsset);
        }
        let fee = default_referral_fees().evm;
        let app_fees = serde_json::to_string(&[AppFee { bps: fee.bps, receiver_address: fee.address }])?;
        Ok(ActionRequest {
            action_type: "swap-action".into(),
            sender: request.wallet_address.clone(),
            src_chain_id: source.id,
            src_token: EVM_ZERO_ADDRESS.into(),
            dst_chain_id: destination.id,
            dst_token: EVM_ZERO_ADDRESS.into(),
            slippage: request.options.slippage.bps,
            amount: request.value.to_string(),
            swap_direction: "exact-amount-in".into(),
            recipient: request.destination_address.clone(),
            refund_to: request.wallet_address.clone(),
            return_deposit_address: true,
            app_fees,
        })
    }

    fn validate_amount(limits: &AmountLimits, value: &str, decimals: u32) -> Result<(), SwapperError> {
        let value = value.parse::<BigUint>().map_err(SwapperError::compute_quote_error)?;
        let minimum = BigNumberFormatter::value_from_amount_biguint(&limits.min_amount, decimals).map_err(|_| SwapperError::ComputeQuoteError("Invalid minimum amount".into()))?;
        if value < minimum {
            return Err(SwapperError::InputAmountError { min_amount: Some(minimum.to_string()) });
        }
        if let Some(maximum) = limits.max_amount.as_deref() {
            let maximum = BigNumberFormatter::value_from_amount_biguint(maximum, decimals).map_err(|_| SwapperError::ComputeQuoteError("Invalid maximum amount".into()))?;
            if value > maximum {
                return Err(SwapperError::ComputeQuoteError("Input amount exceeds route maximum".into()));
            }
        }
        Ok(())
    }

    fn validate_response(response: &ActionResponse, request: &QuoteRequest, source: SwapsXyzChain, destination: SwapsXyzChain) -> Result<(), SwapperError> {
        let fee = default_referral_fees().evm;
        let input = request.value.clone();
        let expected_fee = &input * BigUint::from(fee.bps) / BigUint::from(HUNDRED_PERCENT_IN_BPS);
        let valid = response.vm_id == "alt-vm"
            && !response.tx.to.is_empty()
            && response.tx.value == input
            && response.tx.chain_id == source.id
            && response.tx.chain_key == source.key
            && response.amount_in.amount == input
            && response.amount_in.native_chain() == Some(source)
            && response.amount_out.native_chain() == Some(destination)
            && response.amount_out.amount > BigUint::from(0_u8)
            && response.application_fee.native_chain() == Some(source)
            && response.application_fee.amount == expected_fee;
        if valid { Ok(()) } else { Err(SwapperError::InvalidRoute) }
    }

    fn map_status(status: &str) -> SwapStatus {
        match status.to_ascii_lowercase().as_str() {
            "success" | "completed" => SwapStatus::Completed,
            "refunded" => SwapStatus::Refunded,
            "failed" | "expired" => SwapStatus::Failed,
            _ => SwapStatus::Pending,
        }
    }

    async fn get_ton_status(&self, transaction_hash: &str, chain_id: u64) -> Result<Option<StatusResponse>, SwapperError> {
        let traces = self.ton_client.get_traces_by_hash(transaction_hash.to_string()).await.map_err(SwapperError::transaction_error)?;
        let Some(transaction) = traces.root_transaction() else {
            return Ok(None);
        };
        let message_hash = transaction
            .in_msg
            .as_ref()
            .and_then(|message| message.hash.as_deref())
            .ok_or_else(|| SwapperError::TransactionError("TON external message hash is missing".into()))?;
        let message_hash = hex::encode(decode_base64(message_hash).map_err(SwapperError::transaction_error)?);
        self.client.get_status(&message_hash, chain_id).await
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
        Self::validate_amount(&path.amount_limits, &request.value.to_string(), source.decimals())?;
        let action_request = Self::build_action_request(request, source, destination)?;
        let response = self.client.get_action(&action_request).await?;
        Self::validate_response(&response, request, source, destination)?;
        let eta_in_seconds = if response.estimated_tx_time.is_finite() && response.estimated_tx_time > 0.0 && response.estimated_tx_time <= u32::MAX as f64 {
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
        Ok(SwapperQuoteData::new_transfer(response.tx.to, response.tx.value, response.tx.to_extra.non_empty()))
    }

    async fn get_swap_result(&self, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        let chain = SwapsXyzChain::from_chain(request.chain).ok_or(SwapperError::NotSupportedChain)?;
        let response = if request.chain == Chain::Ton {
            self.get_ton_status(&request.transaction_hash, chain.id).await?
        } else {
            self.client.get_status(&request.transaction_hash, chain.id).await?
        };
        let Some(response) = response else {
            return Ok(SwapResult::pending());
        };
        let metadata = response.action_response.zip(response.dst_tx.and_then(|transaction| transaction.payment_token)).and_then(|(status, amount_out)| {
            let referral_fee = status.application_fee.filter(|_| response.org.as_ref().and_then(|org| org.app_id.as_deref()) == Some(APP_ID)).and_then(|fee| {
                Some(TransactionSwapReferralFee {
                    asset_id: AssetId::from_chain(fee.native_chain()?.chain),
                    value: fee.amount,
                })
            });
            Some(
                TransactionSwapMetadata::new(
                    AssetId::from_chain(status.amount_in.native_chain()?.chain),
                    status.amount_in.amount,
                    AssetId::from_chain(amount_out.native_chain()?.chain),
                    amount_out.amount,
                    SwapperProvider::SwapsXyz,
                )
                .with_referral_fee(referral_fee),
            )
        });
        Ok(SwapResult {
            status: Self::map_status(&response.status),
            metadata,
            eta_in_seconds: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::{ClientError, testkit::MockClient};

    #[test]
    fn test_build_action_request_includes_referral_fee() {
        let request = QuoteRequest::mock_cosmos_to_stellar();
        let action = SwapsXyz::<MockClient>::build_action_request(&request, SwapsXyzChain::from_chain(request.from_asset.chain()).unwrap(), SwapsXyzChain::from_chain(request.to_asset.chain()).unwrap()).unwrap();
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
        let limits = AmountLimits { min_amount: "10.5".into(), max_amount: None };
        assert_eq!(SwapsXyz::<MockClient>::validate_amount(&limits, "100000000", 7), Err(SwapperError::InputAmountError { min_amount: Some("105000000".into()) }));
        assert_eq!(SwapsXyz::<MockClient>::validate_amount(&limits, "105000000", 7), Ok(()));

        let whole_token_limit = AmountLimits { min_amount: "1".into(), max_amount: None };
        assert_eq!(
            SwapsXyz::<MockClient>::validate_amount(&whole_token_limit, "9999999", 7),
            Err(SwapperError::InputAmountError { min_amount: Some("10000000".into()) })
        );

        let invalid_maximum = AmountLimits {
            min_amount: "1".into(),
            max_amount: Some("invalid".into()),
        };
        assert_eq!(SwapsXyz::<MockClient>::validate_amount(&invalid_maximum, "105000000", 7), Err(SwapperError::ComputeQuoteError("Invalid maximum amount".into())));
    }

    #[test]
    fn test_validate_response() {
        let request = QuoteRequest::mock_cosmos_to_stellar();
        let source = SwapsXyzChain::from_chain(request.from_asset.chain()).unwrap();
        let destination = SwapsXyzChain::from_chain(request.to_asset.chain()).unwrap();
        let response = ActionResponse::mock();
        assert_eq!(SwapsXyz::<MockClient>::validate_response(&response, &request, source, destination), Ok(()));

        let mut wrong_fee = response;
        wrong_fee.application_fee.amount = BigUint::from(0_u8);
        assert_eq!(SwapsXyz::<MockClient>::validate_response(&wrong_fee, &request, source, destination), Err(SwapperError::InvalidRoute));
    }

    #[tokio::test]
    async fn test_missing_unregistered_status_is_pending() {
        for error in [
            ClientError::Http { status: 404, body: vec![] },
            ClientError::Response {
                status: 404,
                message: "Failed to get tx status for txHash source-hash".to_string(),
                body: vec![],
            },
        ] {
            let upstream = MockClient::new().with_get(move |_| Err(error.clone()));
            let provider = SwapsXyz::with_client(SwapsXyzClient::new(upstream, MockClient::new()), TonClient::new(MockClient::new()));
            let result = provider.get_swap_result(&SwapResultRequest::new(Chain::Algorand, "source-hash")).await.unwrap();
            assert_eq!(result.status, SwapStatus::Pending);
        }
    }

    #[tokio::test]
    async fn test_quote_data_preserves_deposit_memo() {
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(MockClient::new(), MockClient::new()), TonClient::new(MockClient::new()));
        let quote = Quote::mock_swaps_xyz(QuoteRequest::mock_cosmos_to_stellar(), ActionResponse::mock());
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!(data, SwapperQuoteData::new_transfer("deposit".to_string(), BigUint::from(1_000_000_u64), Some("1234".to_string())));
    }

    #[tokio::test]
    async fn test_sui_quote_data_is_transfer() {
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(MockClient::new(), MockClient::new()), TonClient::new(MockClient::new()));
        let quote = Quote::mock_swaps_xyz(QuoteRequest::mock_sui_to_ton(), ActionResponse::mock_sui_to_ton());
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!(
            data,
            SwapperQuoteData::new_transfer("0x82d03c6b623f1e15164c9329b1c49e066ff7814be70ff86f8e19125292661040".to_string(), BigUint::from(3_379_105_015_u64), None)
        );
    }

    #[test]
    fn test_map_status() {
        assert_eq!(SwapsXyz::<MockClient>::map_status("completed"), SwapStatus::Completed);
        assert_eq!(SwapsXyz::<MockClient>::map_status("requires refund"), SwapStatus::Pending);
        assert_eq!(SwapsXyz::<MockClient>::map_status("expired"), SwapStatus::Failed);
        assert_eq!(SwapsXyz::<MockClient>::map_status("submitted"), SwapStatus::Pending);
    }

    #[tokio::test]
    async fn test_swap_result_uses_received_amount() {
        let upstream = MockClient::new().with_get(|_| Ok(include_bytes!("testdata/status_cosmos_to_ton.json").to_vec()));
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(upstream, MockClient::new()), TonClient::new(MockClient::new()));
        let result = provider.get_swap_result(&SwapResultRequest::new(Chain::Cosmos, "source-hash")).await.unwrap();
        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(
            result.metadata,
            Some(
                TransactionSwapMetadata::new(
                    Chain::Cosmos.as_asset_id(),
                    BigUint::from(29_087_679_u64),
                    Chain::Ton.as_asset_id(),
                    BigUint::from(36_580_790_000_u64),
                    SwapperProvider::SwapsXyz
                )
                .with_referral_fee(Some(TransactionSwapReferralFee {
                    asset_id: Chain::Cosmos.as_asset_id(),
                    value: BigUint::from(145_438_u64)
                }))
            )
        );
    }

    #[tokio::test]
    async fn test_swap_result_preserves_foreign_swap_without_referral_fee() {
        for app_id in [Some("0x41a2fb79"), Some("0x225f7707"), Some("0x5d0a204f"), None] {
            let mut response: serde_json::Value = serde_json::from_slice(include_bytes!("testdata/status_foreign_xrp_to_stellar.json")).unwrap();
            match app_id {
                Some(app_id) => response["org"]["appId"] = app_id.into(),
                None => {
                    response.as_object_mut().unwrap().remove("org");
                }
            }
            let upstream = MockClient::new().with_get(move |_| Ok(serde_json::to_vec(&response).unwrap()));
            let provider = SwapsXyz::with_client(SwapsXyzClient::new(upstream, MockClient::new()), TonClient::new(MockClient::new()));
            let result = provider.get_swap_result(&SwapResultRequest::new(Chain::Xrp, "source-hash")).await.unwrap();
            assert_eq!(
                result,
                SwapResult {
                    status: SwapStatus::Completed,
                    metadata: Some(TransactionSwapMetadata::new(
                        Chain::Xrp.as_asset_id(),
                        BigUint::from(1_000_000_u64),
                        Chain::Stellar.as_asset_id(),
                        BigUint::from(70_001_865_u64),
                        SwapperProvider::SwapsXyz
                    )),
                    eta_in_seconds: None,
                }
            );
        }
    }

    #[tokio::test]
    async fn test_ton_swap_result_uses_external_message_hash() {
        let ton_client = MockClient::new().with_get(|path| {
            if path.starts_with("/api/v3/traces?msg_hash=") {
                Ok(br#"{"traces":[]}"#.to_vec())
            } else {
                assert_eq!(path, "/api/v3/traces?tx_hash=8f7f20d5f727af5c6f632d94ff676c70049a4b5aeaf0be974f8b742b6670b371&include_actions=true");
                Ok(include_bytes!("testdata/ton_deposit_trace.json").to_vec())
            }
        });
        let upstream = MockClient::new().with_get(|path| {
            assert_eq!(path, "/getStatus?txHash=4d0644f73d746cb1e921ff2a1edb3e651e7b708df59d9c8b705d1aecedbddacc&chainId=999000337");
            Ok(include_bytes!("testdata/status_ton_to_algorand.json").to_vec())
        });
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(upstream, MockClient::new()), TonClient::new(ton_client));
        let result = provider.get_swap_result(&SwapResultRequest::new(Chain::Ton, "8f7f20d5f727af5c6f632d94ff676c70049a4b5aeaf0be974f8b742b6670b371")).await.unwrap();
        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(result.metadata.unwrap().to_value, BigUint::from(885_912_855_u64));
    }

    #[tokio::test]
    async fn test_swap_result_without_destination_preserves_status() {
        let mut response: serde_json::Value = serde_json::from_slice(include_bytes!("testdata/status_cosmos_to_ton.json")).unwrap();
        response["dstTx"] = serde_json::Value::Null;
        let upstream = MockClient::new().with_get(move |_| Ok(serde_json::to_vec(&response).unwrap()));
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(upstream, MockClient::new()), TonClient::new(MockClient::new()));
        let result = provider.get_swap_result(&SwapResultRequest::new(Chain::Cosmos, "source-hash")).await.unwrap();
        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Completed,
                metadata: None,
                eta_in_seconds: None
            }
        );
    }

    #[tokio::test]
    async fn test_ton_swap_result_waits_for_trace() {
        let ton_client = MockClient::new().with_get(|_| Ok(br#"{"traces":[]}"#.to_vec()));
        let provider = SwapsXyz::with_client(SwapsXyzClient::new(MockClient::new(), MockClient::new()), TonClient::new(ton_client));
        let result = provider.get_swap_result(&SwapResultRequest::new(Chain::Ton, "source-hash")).await.unwrap();
        assert_eq!(result, SwapResult::pending());
    }
}
