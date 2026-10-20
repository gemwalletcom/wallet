use std::sync::Arc;

use async_trait::async_trait;
use chain_traits::ChainBalances;
use gem_evm::{
    constants::TOKEN_TRANSFER_GAS_LIMIT,
    u256::{biguint_to_u256, u256_to_biguint},
};
use gem_ton::{address::Address as TonAddress, models::TonConnectMessage};
use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{
    AssetId, Chain, ChainType,
    swap::{ApprovalData, SwapResult, SwapResultRequest, SwapStatus},
};

use super::{
    asset::{Network, get_token_code, supported_assets, vault_addresses},
    client::BridgersClient,
    model::{DepositMemo, QuoteRequest as BridgersQuoteRequest, QuoteTxData, Record, RecordsRequest, RouteData, SwapRequest, SwapTransaction},
    ton::{JETTON_TRANSFER_TON_AMOUNT, get_jetton_payload, get_native_payload, get_payload_channel, get_transaction_payload},
    transaction::get_transaction_value,
    xrp::get_transaction_memo,
};
use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    approval::{check_approval_erc20, get_swap_gas_limit_with_approval},
    client_factory::{create_ton_client, create_xrp_client},
    config::get_swap_proxy_url,
    cross_chain::VaultAddresses,
};

#[derive(Debug)]
pub struct Bridgers {
    provider: ProviderType,
    client: BridgersClient<RpcClient>,
    rpc_provider: Arc<dyn RpcProvider>,
}

impl Bridgers {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Bridgers),
            client: BridgersClient::new(RpcClient::new(get_swap_proxy_url(SwapperProvider::Bridgers.as_ref()), rpc_provider.clone())),
            rpc_provider,
        }
    }

    async fn get_evm_quote_data(&self, network: Network, from_asset: &AssetId, to_code: &str, swap: &SwapRequest) -> Result<SwapperQuoteData, SwapperError> {
        let router = network.router()?;
        let (swap_data, approval) = futures::try_join!(self.client.get_swap(swap), self.get_approval(network, router, from_asset, swap))?;
        let SwapTransaction::Evm(transaction) = swap_data.tx_data else {
            return Err(SwapperError::InvalidRoute);
        };
        let value = get_transaction_value(&transaction, router, from_asset, to_code, swap)?;
        let gas_limit = get_swap_gas_limit_with_approval(&approval, None, TOKEN_TRANSFER_GAS_LIMIT * 2);
        Ok(SwapperQuoteData::new_contract(router.to_string(), u256_to_biguint(&value), transaction.data, approval, gas_limit))
    }

    async fn get_ton_quote_data(&self, network: Network, from_asset: &AssetId, to_code: &str, swap: &SwapRequest) -> Result<SwapperQuoteData, SwapperError> {
        let SwapTransaction::Ton(transaction) = self.client.get_swap(swap).await?.tx_data else {
            return Err(SwapperError::InvalidRoute);
        };
        let sender = TonAddress::parse(&swap.from_address)?;
        let router = TonAddress::parse(network.router()?)?;
        let from_amount = swap.quote.from_token_amount.clone();
        let memo = DepositMemo::new(swap, to_code, &get_payload_channel(&transaction, to_code, &swap.slippage)?);
        let (to, value, payload) = match &from_asset.token_id {
            None => (router, from_amount, get_native_payload(&sender, &memo)?),
            Some(token_id) => (
                create_ton_client(self.rpc_provider.clone())?
                    .get_jetton_wallet_address(&swap.from_address, token_id)
                    .await
                    .map_err(SwapperError::compute_quote_error)?,
                BigUint::from(JETTON_TRANSFER_TON_AMOUNT),
                get_jetton_payload(&from_amount, &router, &sender, &memo)?,
            ),
        };
        let payload = get_transaction_payload(&transaction, &to, &value, payload)?;
        TonConnectMessage::swap_quote_data(&[TonConnectMessage::new(to.encode_bounceable(), &value, Some(payload))]).map_err(SwapperError::compute_quote_error)
    }

    async fn get_xrp_quote_data(&self, network: Network, from_asset: &AssetId, to_code: &str, swap: &SwapRequest, value: BigUint) -> Result<SwapperQuoteData, SwapperError> {
        let SwapTransaction::Xrp(transaction) = self.client.get_swap(swap).await?.tx_data else {
            return Err(SwapperError::InvalidRoute);
        };
        let router = network.router()?;
        let memo = get_transaction_memo(&transaction, router, from_asset, &value, to_code, swap)?;
        Ok(SwapperQuoteData::new_transfer(router.to_string(), value, Some(memo)))
    }

    async fn check_destination_trust_line(&self, request: &QuoteRequest) -> Result<(), SwapperError> {
        let asset_id = request.to_asset.asset_id();
        let Some(token_id) = asset_id.token_id.filter(|_| asset_id.chain == Chain::Xrp) else {
            return Ok(());
        };
        let balances = create_xrp_client(self.rpc_provider.clone())?
            .get_balance_tokens(request.destination_address.clone(), vec![token_id])
            .await
            .map_err(SwapperError::compute_quote_error)?;
        match balances.first() {
            Some(balance) if balance.is_active => Ok(()),
            _ => Err(SwapperError::NotSupportedAsset),
        }
    }

    async fn get_approval(&self, network: Network, router: &str, from_asset: &AssetId, swap: &SwapRequest) -> Result<Option<ApprovalData>, SwapperError> {
        let Some(token_id) = &from_asset.token_id else {
            return Ok(None);
        };
        let amount = biguint_to_u256(&swap.quote.from_token_amount).ok_or(SwapperError::InvalidRoute)?;
        check_approval_erc20(swap.from_address.clone(), token_id.clone(), router.to_string(), amount, self.rpc_provider.clone(), &network.chain).await
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

#[async_trait]
impl Swapper for Bridgers {
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
        let bridgers_request = BridgersQuoteRequest::new(request, request.value.clone())?;
        let (quote, _) = futures::try_join!(self.client.get_quote(&bridgers_request), self.check_destination_trust_line(request))?;
        let quote = quote.tx_data;

        let min_value = get_min_from_value(&quote, request)?;
        let to_value = quote.get_to_value(request.to_asset.decimals)?;
        let eta_in_seconds = quote.eta_in_seconds();
        let route_data = RouteData { amount_out_min: quote.amount_out_min };

        Ok(Quote {
            min_from_value: Some(min_value),
            ..Quote::from_route(request, &self.provider, to_value, request.options.slippage.bps, &route_data, eta_in_seconds)?
        })
    }

    async fn get_swap_result(&self, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        let records = self.client.get_records(&RecordsRequest::new(request.from_address.clone().ok_or(SwapperError::InvalidRoute)?)).await?;
        let record = records.list.iter().find(|record| record.hash.eq_ignore_ascii_case(&request.transaction_hash));
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
        let route: RouteData = quote.route_data()?;
        let swap = SwapRequest {
            quote: BridgersQuoteRequest::new(request, quote.from_value.clone())?,
            from_address: request.wallet_address.clone(),
            to_address: request.destination_address.clone(),
            amount_out_min: route.amount_out_min,
            slippage: BigNumberFormatter::value(quote.data.slippage_bps, 4)?,
        };
        match network.chain.chain_type() {
            ChainType::Ethereum => self.get_evm_quote_data(network, &request.from_asset.asset_id(), to_code, &swap).await,
            ChainType::Ton => self.get_ton_quote_data(network, &request.from_asset.asset_id(), to_code, &swap).await,
            ChainType::Xrp => self.get_xrp_quote_data(network, &request.from_asset.asset_id(), to_code, &swap, quote.from_value.clone()).await,
            _ => Err(SwapperError::NotSupportedChain),
        }
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{
        alien::reqwest_provider::NativeProvider,
        bridgers::testkit::{
            TEST_EVM_WALLET, TEST_XRP_WALLET, mock_base_usdc_to_bsc_usdt_request, mock_base_usdc_to_ton_usdt_request, mock_bsc_usdt_to_rlusd_request, mock_opbnb_to_bsc_usdt_request, mock_rlusd_to_bsc_usdt_request,
            mock_stable_to_bsc_usdt_request, mock_ton_to_bsc_usdt_request, mock_ton_usdt_to_bsc_usdt_request, mock_xrp_to_bsc_usdt_request,
        },
    };
    use primitives::{
        Chain, TransactionSwapMetadata, TransactionSwapReferralFee,
        asset_constants::{ARBITRUM_USDC_ASSET_ID, TON_USDT_ASSET_ID, XRP_RLUSD_ASSET_ID, XRP_RLUSD_TOKEN_ID},
    };

    #[tokio::test]
    async fn test_bridgers_quote_and_quote_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Bridgers::new(rpc_provider);

        for request in [
            mock_opbnb_to_bsc_usdt_request(),
            mock_base_usdc_to_bsc_usdt_request(),
            mock_stable_to_bsc_usdt_request(),
            mock_base_usdc_to_ton_usdt_request(),
            mock_ton_to_bsc_usdt_request(),
            mock_xrp_to_bsc_usdt_request(),
        ] {
            let quote = provider.get_quote(&request).await?;
            let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            let network = Network::from_chain(request.from_asset.chain())?;

            assert!(quote.to_value > BigUint::ZERO);
            assert!(quote.eta_in_seconds > Some(0));
            assert_eq!(data.to, network.router()?);
        }

        let quote = provider.get_quote(&mock_rlusd_to_bsc_usdt_request()).await?;
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        let memo: serde_json::Value = serde_json::from_str(data.memo.as_deref().unwrap()).unwrap();

        assert_eq!((data.to.as_str(), data.value.clone()), ("rM27yzkCw6WA3T4g1sPaeC1kpxHUhuxRWn", BigUint::from(10_000_000_000_000_000u64)));
        assert_eq!((memo["fromToken"].as_str(), memo["fromAmount"].as_str()), (Some(XRP_RLUSD_TOKEN_ID), Some("10000000000000000000")));

        let to_rlusd = mock_bsc_usdt_to_rlusd_request();
        provider.get_quote(&to_rlusd).await?;
        assert_eq!(
            provider
                .get_quote(&QuoteRequest {
                    destination_address: "rHb9CJAWyB4rj91VRWn96DkukG4bwdtyTh".to_string(),
                    ..to_rlusd
                })
                .await
                .unwrap_err(),
            SwapperError::NotSupportedAsset
        );

        let quote = provider.get_quote(&mock_ton_usdt_to_bsc_usdt_request()).await?;
        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;

        assert_eq!((data.to.as_str(), data.value), ("EQCcPR6bD3c161LJe1hVCFni-z9y7fCkFtpSYwrKncccG4nG", BigUint::from(60_000_000u64)));
        assert_eq!(serde_json::from_str::<Vec<TonConnectMessage>>(&data.data)?.len(), 1);

        let request = SwapResultRequest {
            from_address: Some("0x1085c5f70f7f7591d97da281a64688385455c2bd".to_string()),
            ..SwapResultRequest::new(Chain::Arbitrum, "0x8cb98cd501b2e0b5da96a97492c1f1819fe2a72de6db8a8c40c6d5b862a7cd28")
        };
        let result = provider.get_swap_result(&request).await?;

        assert_eq!(
            result,
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(
                    TransactionSwapMetadata::new(
                        ARBITRUM_USDC_ASSET_ID.clone(),
                        BigUint::from(20_000_000u64),
                        AssetId::from_chain(Chain::OpBNB),
                        BigUint::from(25_126_000_000_000_000u64),
                        SwapperProvider::Bridgers,
                    )
                    .with_referral_fee(Some(TransactionSwapReferralFee {
                        asset_id: ARBITRUM_USDC_ASSET_ID.clone(),
                        value: BigUint::from(100_000u64),
                    }))
                ),
                eta_in_seconds: None,
            }
        );

        let request = SwapResultRequest {
            from_address: Some("UQAC-9BRZ2wObbLVcQjoXtv_eOkm6R4ADSPOHpSdl19zI-t7".to_string()),
            ..SwapResultRequest::new(Chain::Ton, "28a0a65db05a18b4c576117260dfebbe622729ca852f8f71ed72610966fa3ea6")
        };
        let result = provider.get_swap_result(&request).await?;

        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(
            result.metadata,
            Some(
                TransactionSwapMetadata::new(
                    TON_USDT_ASSET_ID.clone(),
                    BigUint::from(16_000_000u64),
                    AssetId::from_chain(Chain::SmartChain),
                    BigUint::from(21_898_000_000_000_000u64),
                    SwapperProvider::Bridgers
                )
                .with_referral_fee(Some(TransactionSwapReferralFee {
                    asset_id: TON_USDT_ASSET_ID.clone(),
                    value: BigUint::from(80_000u64),
                }))
            )
        );

        let request = SwapResultRequest {
            from_address: Some(TEST_XRP_WALLET.to_string()),
            ..SwapResultRequest::new(Chain::Xrp, "DA76328746E3CBB41AC4E98348893CD8988CE1C731548A9DDF5404794C9A066F")
        };
        let result = provider.get_swap_result(&request).await?;

        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(
            result.metadata,
            Some(
                TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Xrp),
                    BigUint::from(13_000_000u64),
                    AssetId::from_chain(Chain::SmartChain),
                    BigUint::from(22_876_000_000_000_000u64),
                    SwapperProvider::Bridgers
                )
                .with_referral_fee(Some(TransactionSwapReferralFee {
                    asset_id: AssetId::from_chain(Chain::Xrp),
                    value: BigUint::from(65_000u64),
                }))
            )
        );

        let request = SwapResultRequest {
            from_address: Some(TEST_EVM_WALLET.to_string()),
            ..SwapResultRequest::new(Chain::SmartChain, "0x13ffcaf02ca9c3cff66039839d41ae70df56b0be0a8d879eae25051c7a1ceba9")
        };
        let metadata = provider.get_swap_result(&request).await?.metadata.unwrap();

        assert_eq!((metadata.to_asset, metadata.to_value), (XRP_RLUSD_ASSET_ID.clone(), BigUint::from(14_137_307_000_000_000u64)));

        let request = SwapResultRequest {
            from_address: Some(TEST_XRP_WALLET.to_string()),
            ..SwapResultRequest::new(Chain::Xrp, "F11F4E9A9D3028B8858F4F212E3561CD235E9433CC17CB66224934E67D2BB287")
        };
        let result = provider.get_swap_result(&request).await?;

        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(
            result.metadata,
            Some(
                TransactionSwapMetadata::new(
                    XRP_RLUSD_ASSET_ID.clone(),
                    BigUint::from(18_000_000_000_000_000u64),
                    AssetId::from_chain(Chain::SmartChain),
                    BigUint::from(24_391_000_000_000_000u64),
                    SwapperProvider::Bridgers
                )
                .with_referral_fee(Some(TransactionSwapReferralFee {
                    asset_id: XRP_RLUSD_ASSET_ID.clone(),
                    value: BigUint::from(90_000_000_000_000u64),
                }))
            )
        );
        Ok(())
    }
}
