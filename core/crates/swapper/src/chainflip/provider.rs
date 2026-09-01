use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::contracts::IERC20;
use gem_evm::u256::biguint_to_u256;
use gem_evm::u256::u256_to_biguint;
use gem_solana::DEFAULT_SWAP_GAS_LIMIT;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::{fmt::Debug, sync::Arc, time::Duration};

use super::{
    ChainflipRouteData,
    broker::{
        AssetsResponse, BrokerClient, ChainflipAsset, DcaParameters, QuoteDetails, QuoteRequest as ChainflipQuoteRequest, QuoteResponse, QuoteType, RefundParameters,
        TronVaultSwapResponse, VaultSwapChainExtras, VaultSwapExtras, VaultSwapResponse, VaultSwapSolanaExtras,
    },
    capitalize::capitalize_first_letter,
    client::{ChainflipClient, SUPPORTED_ASSETS, map_swap_result},
    price::{apply_slippage, price_to_hex_price},
    seed::generate_random_seed,
    tx_builder,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapAmountMode, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    alien::RpcProvider,
    approval::{check_approval_erc20, get_swap_gas_limit_with_approval},
    cross_chain::VaultAddresses,
    fees::DEFAULT_CHAINFLIP_FEE_BPS as DEFAULT_FEE_BPS,
    route_cache::Cache,
};
use primitives::{
    Asset, AssetId, ChainType, MINUTE,
    chain::Chain,
    hex::{decode_hex, encode_with_0x},
    swap::QuoteAsset,
};

const DEFAULT_SWAP_ERC20_GAS_LIMIT: u64 = 100_000;
const REFUND_RETRY_BLOCKS: u32 = 150;
const ASSETS_CACHE_TTL: Duration = MINUTE.saturating_mul(5);

const VAULT_ETH: &str = "0xF5e10380213880111522dd0efD3dbb45b9f62Bcc";
const VAULT_ARB: &str = "0x79001a5e762f3bEFC8e5871b42F6734e00498920";
const VAULT_SOL: &str = "J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT";
const VAULT_TRON: &str = "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC";

#[derive(Debug)]
pub struct ChainflipProvider<CX, BR>
where
    CX: Client + Clone + Send + Sync + Debug + 'static,
    BR: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    chainflip_client: ChainflipClient<CX>,
    broker_client: BrokerClient<BR>,
    rpc_provider: Arc<dyn RpcProvider>,
    assets_cache: Cache<(), AssetsResponse>,
}

impl<CX, BR> ChainflipProvider<CX, BR>
where
    CX: Client + Clone + Send + Sync + Debug + 'static,
    BR: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn with_clients(chainflip_client: ChainflipClient<CX>, broker_client: BrokerClient<BR>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Chainflip),
            chainflip_client,
            broker_client,
            rpc_provider,
            assets_cache: Cache::new(ASSETS_CACHE_TTL),
        }
    }

    async fn get_assets(&self) -> Result<AssetsResponse, SwapperError> {
        if let Some(assets) = self.assets_cache.get(&()) {
            return Ok(assets);
        }
        let assets = self.broker_client.get_assets().await?;
        self.assets_cache.put((), assets.clone());
        Ok(assets)
    }
}

fn vault_deposit_addresses() -> Vec<String> {
    vec![VAULT_ETH.to_string(), VAULT_ARB.to_string(), VAULT_SOL.to_string(), VAULT_TRON.to_string()]
}

fn map_asset_id(asset: &QuoteAsset) -> ChainflipAsset {
    let asset_id = asset.asset_id();
    let chain_name = capitalize_first_letter(asset_id.chain.as_ref());
    let symbol = if asset.symbol.is_empty() && asset_id.is_native() {
        Asset::from_chain(asset_id.chain).symbol
    } else {
        asset.symbol.clone()
    };
    ChainflipAsset { chain: chain_name, asset: symbol }
}

fn build_quote_request(request: &QuoteRequest, assets: &AssetsResponse) -> Result<(ChainflipQuoteRequest, BigUint), SwapperError> {
    match request.from_asset.chain().chain_type() {
        ChainType::Ethereum | ChainType::Solana | ChainType::Tron => {}
        _ => return Err(SwapperError::NotSupportedChain),
    }
    let source_asset = map_asset_id(&request.from_asset);
    let destination_asset = map_asset_id(&request.to_asset);
    let source_broker_asset = assets.asset(&source_asset).filter(|asset| asset.supports_ingress()).ok_or(SwapperError::NoQuoteAvailable)?;
    let destination_broker_asset = assets
        .asset(&destination_asset)
        .filter(|asset| asset.supports_egress())
        .ok_or(SwapperError::NoQuoteAvailable)?;

    Ok((
        ChainflipQuoteRequest {
            amount: request.value.clone(),
            source_asset: source_broker_asset.id.clone(),
            destination_asset: destination_broker_asset.id.clone(),
            commission_bps: DEFAULT_FEE_BPS,
            is_vault_swap: true,
        },
        source_broker_asset.minimal_amount_native.clone(),
    ))
}

fn get_best_quote(quotes: Vec<QuoteResponse>, request: &ChainflipQuoteRequest) -> Result<(BigUint, u32, u32, ChainflipRouteData), SwapperError> {
    let ingress_amount = request.amount.clone();
    let matches_request = |details: &QuoteDetails| {
        details.ingress_asset == request.source_asset && details.ingress_amount_native == ingress_amount && details.egress_asset == request.destination_asset
    };
    let (details, quote_type, recommended_slippage_tolerance_percent, estimated_boost_fee_bps) = quotes
        .into_iter()
        .filter(|quote| matches_request(&quote.details) && quote.boost_quote.as_ref().is_none_or(|quote| matches_request(&quote.details)))
        .filter_map(|quote| {
            let QuoteResponse {
                details,
                quote_type,
                recommended_slippage_tolerance_percent,
                boost_quote,
            } = quote;
            match boost_quote {
                Some(boost_quote) if !boost_quote.details.low_liquidity_warning => Some((
                    boost_quote.details,
                    quote_type,
                    recommended_slippage_tolerance_percent,
                    Some(boost_quote.estimated_boost_fee_bps),
                )),
                _ if !details.low_liquidity_warning => Some((details, quote_type, recommended_slippage_tolerance_percent, None)),
                _ => None,
            }
        })
        .max_by(|(left, ..), (right, ..)| left.egress_amount_native.cmp(&right.egress_amount_native))
        .ok_or(SwapperError::NoQuoteAvailable)?;
    let boost_fee = estimated_boost_fee_bps
        .map(|fee| fee.ceil().to_u32().filter(|fee| *fee <= u8::MAX as u32).ok_or(SwapperError::InvalidRoute))
        .transpose()?;
    if details.egress_amount_native == BigUint::from(0u32)
        || !details.estimated_price.is_finite()
        || details.estimated_price <= 0.0
        || !recommended_slippage_tolerance_percent.is_finite()
        || !(0.0..100.0).contains(&recommended_slippage_tolerance_percent)
    {
        return Err(SwapperError::InvalidRoute);
    }
    let slippage_bps = (recommended_slippage_tolerance_percent * 100.0).to_u32().ok_or(SwapperError::InvalidRoute)?;
    let eta_in_seconds = details
        .estimated_duration_seconds
        .ceil()
        .to_u32()
        .filter(|eta| *eta > 0)
        .ok_or(SwapperError::InvalidRoute)?;
    let dca_parameters = match quote_type {
        QuoteType::Regular => None,
        QuoteType::Dca => {
            let number_of_chunks = details.number_of_chunks.filter(|value| *value > 0).ok_or(SwapperError::InvalidRoute)?;
            let chunk_interval = details.chunk_interval_blocks.filter(|value| *value > 0).ok_or(SwapperError::InvalidRoute)?;
            Some(DcaParameters { number_of_chunks, chunk_interval })
        }
    };

    Ok((
        details.egress_amount_native,
        slippage_bps,
        eta_in_seconds,
        ChainflipRouteData {
            boost_fee,
            estimated_price: details.estimated_price,
            dca_parameters,
        },
    ))
}

fn refund_parameters(retry_duration: u32, refund_address: &str, min_price: &str) -> RefundParameters {
    RefundParameters {
        retry_duration,
        refund_address: refund_address.to_string(),
        min_price: min_price.to_string(),
        max_oracle_price_slippage: None,
    }
}

fn validate_minimum_amount(from_value: &BigUint, minimum_amount: &BigUint) -> Result<(), SwapperError> {
    if from_value < minimum_amount {
        return Err(SwapperError::InputAmountError {
            min_amount: Some(minimum_amount.to_string()),
        });
    }
    Ok(())
}

fn tron_trc20_transfer_value(calldata: &str) -> Result<BigUint, SwapperError> {
    let data = decode_hex(calldata).map_err(|_| SwapperError::TransactionError("invalid Tron token transfer calldata".to_string()))?;
    IERC20::transferCall::abi_decode(&data)
        .map(|call| u256_to_biguint(&call.value))
        .map_err(|_| SwapperError::TransactionError("invalid Tron token transfer calldata".to_string()))
}

fn tron_quote_value(from_asset: &AssetId, input_amount: &BigUint, response: &TronVaultSwapResponse) -> Result<BigUint, SwapperError> {
    let is_native = from_asset.is_native();
    let broker_value = if is_native {
        response.value.clone()
    } else {
        if response.value != BigUint::from(0u32) {
            return Err(SwapperError::TransactionError(format!("Tron token swap value must be zero: broker={}", response.value)));
        }
        tron_trc20_transfer_value(&response.calldata)?
    };

    if broker_value != *input_amount {
        return Err(SwapperError::TransactionError(format!(
            "Tron swap amount mismatch: quote={input_amount}, broker={broker_value}"
        )));
    }

    Ok(if is_native { input_amount.clone() } else { BigUint::ZERO })
}

#[async_trait]
impl<CX, BR> Swapper for ChainflipProvider<CX, BR>
where
    CX: Client + Clone + Send + Sync + Debug + 'static,
    BR: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        SUPPORTED_ASSETS.clone()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn preload_routes(&self, _from_asset: &AssetId, _to_asset: &AssetId) {
        _ = self.get_assets().await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let assets = self.get_assets().await?;
        let (quote_request, minimum_amount) = build_quote_request(request, &assets)?;
        validate_minimum_amount(&quote_request.amount, &minimum_amount)?;

        let quotes = self.broker_client.get_quotes(&quote_request).await?;
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, &quote_request)?;

        Ok(Quote {
            min_from_value: Some(minimum_amount),
            from_value: quote_request.amount,
            to_value: egress_amount,
            data: ProviderData {
                provider: self.provider.clone(),
                slippage_bps,
                routes: vec![Route {
                    input: request.from_asset.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&route_data)?,
                }],
            },
            eta_in_seconds: Some(eta_in_seconds),
            request: request.clone(),
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        let source_asset = map_asset_id(&quote.request.from_asset);
        let destination_asset = map_asset_id(&quote.request.to_asset);

        let input_amount = quote.from_value.clone();

        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: ChainflipRouteData = serde_json::from_str(&route.route_data)?;
        let chain = source_asset.chain.clone();
        let price = route_data.estimated_price;
        let price_slippage = apply_slippage(price, quote.data.slippage_bps);
        let quote_asset_decimals = quote.request.to_asset.decimals;
        let base_asset_decimals = quote.request.from_asset.decimals;
        let min_price = price_to_hex_price(price_slippage, quote_asset_decimals, base_asset_decimals).map_err(SwapperError::TransactionError)?;
        let source_chain_type = from_asset.chain.chain_type();
        let extra_params = match source_chain_type {
            ChainType::Ethereum => VaultSwapExtras::Evm(VaultSwapChainExtras {
                chain,
                input_amount: input_amount.clone(),
                refund_parameters: refund_parameters(REFUND_RETRY_BLOCKS, &quote.request.wallet_address, &min_price),
            }),
            ChainType::Tron => VaultSwapExtras::Tron(VaultSwapChainExtras {
                chain,
                input_amount: input_amount.clone(),
                refund_parameters: refund_parameters(REFUND_RETRY_BLOCKS, &quote.request.wallet_address, &min_price),
            }),
            ChainType::Solana => VaultSwapExtras::Solana(VaultSwapSolanaExtras {
                from: quote.request.wallet_address.clone(),
                seed: encode_with_0x(&generate_random_seed(32)),
                chain,
                input_amount: input_amount.to_u64().ok_or_else(|| SwapperError::transaction_error("Solana input amount exceeds u64"))?,
                refund_parameters: refund_parameters(REFUND_RETRY_BLOCKS, &quote.request.wallet_address, &min_price),
            }),
            _ => return Err(SwapperError::NotSupportedChain),
        };

        let broker = self.broker_client.encode_vault_swap(
            source_asset,
            destination_asset,
            quote.request.destination_address.clone(),
            DEFAULT_FEE_BPS,
            route_data.boost_fee,
            extra_params,
            route_data.dca_parameters,
        );
        let (response, solana_blockhash) = if source_chain_type == ChainType::Solana {
            let (response, blockhash) = futures::try_join!(broker, tx_builder::get_solana_blockhash(self.rpc_provider.clone()))?;
            (response, Some(blockhash))
        } else {
            (broker.await?, None)
        };

        match (source_chain_type, response) {
            (ChainType::Ethereum, VaultSwapResponse::Evm(response)) => {
                let value = if from_asset.is_native() { quote.from_value.clone() } else { BigUint::from(0u64) };

                let approval = if !from_asset.is_native() {
                    let token_id = from_asset.token_id.ok_or(SwapperError::NotSupportedAsset)?;
                    let approval_amount = biguint_to_u256(&quote.from_value).ok_or_else(|| SwapperError::compute_quote_error("swap amount is too large"))?;
                    let approval = check_approval_erc20(
                        quote.request.wallet_address.clone(),
                        token_id,
                        response.to.clone(),
                        approval_amount,
                        self.rpc_provider.clone(),
                        &from_asset.chain,
                    )
                    .await?;
                    approval.approval_data()
                } else {
                    None
                };

                let gas_limit = get_swap_gas_limit_with_approval(&approval, None, DEFAULT_SWAP_ERC20_GAS_LIMIT);

                Ok(SwapperQuoteData::new_contract(response.to, value, response.calldata, approval, gas_limit))
            }
            (ChainType::Tron, VaultSwapResponse::Tron(response)) => {
                let value = tron_quote_value(&from_asset, &input_amount, &response)?;
                tx_builder::build_tron_quote_data(&response, value)
            }
            (ChainType::Solana, VaultSwapResponse::Solana(response)) => {
                let blockhash = solana_blockhash.ok_or(SwapperError::InvalidRoute)?;
                let data = tx_builder::build_solana_transaction(&quote.request.wallet_address, &response, blockhash)?;
                Ok(SwapperQuoteData::new_contract(
                    response.program_id,
                    BigUint::ZERO,
                    data,
                    None,
                    Some(DEFAULT_SWAP_GAS_LIMIT.to_string()),
                ))
            }
            _ => Err(SwapperError::InvalidRoute),
        }
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        let deposit = vault_deposit_addresses();
        Ok(VaultAddresses { deposit, send: vec![] })
    }

    async fn get_swap_result(&self, _chain: Chain, transaction_hash: &str) -> Result<SwapResult, SwapperError> {
        let response = self.chainflip_client.get_tx_status(transaction_hash).await?;
        Ok(map_swap_result(&response))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SwapperQuoteAsset, alien::mock::ProviderMock};
    use gem_client::testkit::MockClient;
    use primitives::AssetId;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[cfg(feature = "swap_integration_tests")]
    use crate::{NativeProvider, Options};
    #[cfg(feature = "swap_integration_tests")]
    use primitives::swap::{SwapQuoteDataType, SwapStatus};

    fn assets_response() -> AssetsResponse {
        serde_json::from_str(include_str!("./broker/test/assets.json")).unwrap()
    }

    fn quote_request(amount: &str, source_asset: &str, destination_asset: &str) -> ChainflipQuoteRequest {
        ChainflipQuoteRequest {
            amount: amount.parse().unwrap(),
            source_asset: source_asset.to_string(),
            destination_asset: destination_asset.to_string(),
            commission_bps: DEFAULT_FEE_BPS,
            is_vault_swap: true,
        }
    }

    #[test]
    fn test_validate_minimum_amount() {
        let minimum_amount = BigUint::from(68_000_000u32);

        assert_eq!(
            validate_minimum_amount(&BigUint::from(1_000_000u32), &minimum_amount),
            Err(SwapperError::InputAmountError {
                min_amount: Some("68000000".to_string())
            })
        );
        assert_eq!(validate_minimum_amount(&BigUint::from(68_000_000u32), &minimum_amount), Ok(()));
        assert_eq!(validate_minimum_amount(&BigUint::from(68_000_001u32), &minimum_amount), Ok(()));
    }

    #[tokio::test]
    async fn test_chainflip_preload_caches_minimum_amount() {
        let assets_requests = Arc::new(AtomicUsize::new(0));
        let quote_requests = Arc::new(AtomicUsize::new(0));
        let assets_counter = assets_requests.clone();
        let quote_counter = quote_requests.clone();
        let broker_client = MockClient::new().with_get(move |path| match path {
            "/assets" => {
                assets_counter.fetch_add(1, Ordering::Relaxed);
                Ok(include_str!("./broker/test/assets.json").as_bytes().to_vec())
            }
            "/quotes-native?amount=68000000&sourceAsset=sol.sol&destinationAsset=btc.btc&commissionBps=45&isVaultSwap=true" => {
                quote_counter.fetch_add(1, Ordering::Relaxed);
                Ok(br#"[{"type":"regular","ingressAsset":"sol.sol","ingressAmountNative":"68000000","egressAsset":"btc.btc","egressAmountNative":"1","lowLiquidityWarning":false,"recommendedSlippageTolerancePercent":1,"estimatedDurationSeconds":60,"estimatedPrice":1}]"#.to_vec())
            }
            _ => panic!("unexpected path: {path}"),
        });
        let provider = ChainflipProvider::with_clients(
            ChainflipClient::new(MockClient::new()),
            BrokerClient::new(broker_client),
            Arc::new(ProviderMock::new(String::new())),
        );
        let mut request = QuoteRequest::mock(Chain::Solana, None);
        request.to_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin));
        provider.preload_routes(&request.from_asset.asset_id(), &request.to_asset.asset_id()).await;

        let error = provider.get_quote(&request).await.unwrap_err();
        assert_eq!(
            error,
            SwapperError::InputAmountError {
                min_amount: Some("68000000".to_string())
            }
        );

        request.value = BigUint::from(68000000u64);
        let quote = provider.get_quote(&request).await.unwrap();
        assert_eq!(quote.min_from_value, Some(BigUint::from(68000000u64)));
        assert_eq!(assets_requests.load(Ordering::Relaxed), 1);
        assert_eq!(quote_requests.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_build_quote_request_rejects_bitcoin_source() {
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            value: BigUint::from(89100u64),
            ..QuoteRequest::mock(Chain::Bitcoin, None)
        };

        assert!(build_quote_request(&request, &assets_response()).is_err());
    }

    #[test]
    fn test_build_quote_request_uses_broker_asset_ids() {
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)),
            value: BigUint::parse_bytes(b"1000000000000000000", 10).unwrap(),
            ..QuoteRequest::mock(Chain::Ethereum, None)
        };

        let (quote_request, _) = build_quote_request(&request, &assets_response()).unwrap();
        assert_eq!(
            gem_client::build_path_with_query("/quotes-native", &quote_request).unwrap(),
            "/quotes-native?amount=1000000000000000000&sourceAsset=eth.eth&destinationAsset=btc.btc&commissionBps=45&isVaultSwap=true"
        );
    }

    #[test]
    fn test_tron_quote_value_pins_native_amount_to_quote() {
        let from_asset = AssetId::from_chain(Chain::Tron);
        let response = TronVaultSwapResponse {
            calldata: "0x".to_string(),
            value: BigUint::from(50_000_000u32),
            to: "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC".to_string(),
            note: "0x0300".to_string(),
            source_token_address: None,
        };

        assert_eq!(
            tron_quote_value(&from_asset, &BigUint::from(50_000_000u32), &response).unwrap(),
            BigUint::from(50_000_000u64)
        );

        let err = tron_quote_value(&from_asset, &BigUint::from(40_000_000u32), &response).unwrap_err();
        assert!(matches!(err, SwapperError::TransactionError(message) if message.contains("Tron swap amount mismatch")));
    }

    #[test]
    fn test_tron_quote_value_rejects_token_native_value() {
        let from_asset = AssetId::from_token(Chain::Tron, primitives::asset_constants::TRON_USDT_TOKEN_ID);
        let response = TronVaultSwapResponse {
            calldata: "0xa9059cbb".to_string(),
            value: BigUint::from(1u32),
            to: "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC".to_string(),
            note: "0x0300".to_string(),
            source_token_address: Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()),
        };

        let err = tron_quote_value(&from_asset, &BigUint::from(10_000_000u32), &response).unwrap_err();
        assert!(matches!(err, SwapperError::TransactionError(message) if message.contains("Tron token swap value must be zero")));
    }

    #[test]
    fn test_tron_quote_value_pins_token_amount_to_calldata() {
        let from_asset = AssetId::from_token(Chain::Tron, primitives::asset_constants::TRON_USDT_TOKEN_ID);
        let response = TronVaultSwapResponse {
            calldata: "0xa9059cbb0000000000000000000000002523ae929fecd9d665f472f59b99a8ce6b1795100000000000000000000000000000000000000000000000000000000000989680".to_string(),
            value: BigUint::from(0u32),
            to: "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC".to_string(),
            note: "0x0300".to_string(),
            source_token_address: Some("TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".to_string()),
        };

        assert_eq!(tron_quote_value(&from_asset, &BigUint::from(10_000_000u32), &response).unwrap(), BigUint::ZERO);

        let err = tron_quote_value(&from_asset, &BigUint::from(9_999_999u32), &response).unwrap_err();
        assert!(matches!(err, SwapperError::TransactionError(message) if message.contains("Tron swap amount mismatch")));
    }

    #[tokio::test]
    async fn test_quote_data_rejects_wrong_chain_response() {
        let broker = MockClient::new().with_post(|path, _| {
            assert_eq!(path, "/rpc");
            Ok(serde_json::to_vec(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "calldata": "0x",
                    "value": "0x0",
                    "to": "0x1111111111111111111111111111111111111111"
                }
            }))
            .unwrap())
        });
        let provider = ChainflipProvider::with_clients(
            ChainflipClient::new(MockClient::new()),
            BrokerClient::new(broker),
            Arc::new(ProviderMock::new(String::new())),
        );
        let mut quote = Quote::mock(Chain::Tron, None);
        quote.request.to_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum));
        quote.data.routes = vec![Route {
            input: quote.request.from_asset.asset_id(),
            output: quote.request.to_asset.asset_id(),
            route_data: serde_json::to_string(&ChainflipRouteData {
                boost_fee: None,
                estimated_price: 1.0,
                dca_parameters: None,
            })
            .unwrap(),
        }];

        assert_eq!(provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_best_quote() {
        let quotes: Vec<QuoteResponse> = serde_json::from_str(include_str!("./test/chainflip_quotes.json")).unwrap();
        let request = quote_request("10000000000", "sol.sol", "btc.btc");
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, &request).unwrap();

        assert_eq!(egress_amount.to_string(), "145118751424");
        assert_eq!(slippage_bps, 250);
        assert_eq!(eta_in_seconds, 193);
        assert_eq!(
            route_data,
            ChainflipRouteData {
                boost_fee: None,
                estimated_price: 14.5118765424,
                dca_parameters: None,
            }
        );
    }

    #[test]
    fn test_fractional_estimated_duration_rounds_up() {
        let quotes = serde_json::from_value::<Vec<QuoteResponse>>(serde_json::json!([{
            "ingressAsset": "eth.eth",
            "ingressAmountNative": "1",
            "egressAsset": "btc.btc",
            "egressAmountNative": "1",
            "recommendedSlippageTolerancePercent": 1,
            "estimatedDurationSeconds": 163.5,
            "type": "regular",
            "lowLiquidityWarning": false,
            "estimatedPrice": 1
        }]))
        .unwrap();

        assert_eq!(get_best_quote(quotes, &quote_request("1", "eth.eth", "btc.btc")).unwrap().2, 164);
    }

    #[test]
    fn test_empty_quotes_are_unavailable() {
        assert_eq!(get_best_quote(vec![], &quote_request("1", "eth.eth", "btc.btc")), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_low_liquidity_quotes_are_excluded() {
        let response = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        let request = quote_request("100000000", "btc.btc", "eth.eth");

        let mut warned_boost = serde_json::json!([response[0].clone()]);
        warned_boost[0]["boostQuote"]["lowLiquidityWarning"] = serde_json::json!(true);
        let (egress_amount, _, _, route_data) = get_best_quote(serde_json::from_value(warned_boost).unwrap(), &request).unwrap();
        assert_eq!(egress_amount.to_string(), "4082976513112383071");
        assert_eq!(route_data.boost_fee, None);

        let mut warned_regular = serde_json::json!([response[0].clone()]);
        warned_regular[0]["lowLiquidityWarning"] = serde_json::json!(true);
        let (egress_amount, _, _, route_data) = get_best_quote(serde_json::from_value(warned_regular).unwrap(), &request).unwrap();
        assert_eq!(egress_amount.to_string(), "4080934615929730944");
        assert_eq!(route_data.boost_fee, Some(5));

        let mut warned_best = response.clone();
        warned_best[1]["lowLiquidityWarning"] = serde_json::json!(true);
        warned_best[1]["boostQuote"]["lowLiquidityWarning"] = serde_json::json!(true);
        let (egress_amount, _, _, _) = get_best_quote(serde_json::from_value(warned_best.clone()).unwrap(), &request).unwrap();
        assert_eq!(egress_amount.to_string(), "4080934615929730944");

        warned_best[0]["lowLiquidityWarning"] = serde_json::json!(true);
        warned_best[0]["boostQuote"]["lowLiquidityWarning"] = serde_json::json!(true);
        let quotes = serde_json::from_value(warned_best).unwrap();
        assert_eq!(get_best_quote(quotes, &request), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_missing_low_liquidity_warning_is_allowed() {
        let response = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        let request = quote_request("100000000", "btc.btc", "eth.eth");

        let mut regular = response[0].clone();
        regular.as_object_mut().unwrap().remove("lowLiquidityWarning");
        regular.as_object_mut().unwrap().remove("boostQuote");
        let (egress_amount, _, _, route_data) = get_best_quote(serde_json::from_value(serde_json::json!([regular])).unwrap(), &request).unwrap();
        assert_eq!(egress_amount.to_string(), "4082976513112383071");
        assert_eq!(route_data.boost_fee, None);

        let mut boosted = response[0].clone();
        boosted["boostQuote"].as_object_mut().unwrap().remove("lowLiquidityWarning");
        let (egress_amount, _, _, route_data) = get_best_quote(serde_json::from_value(serde_json::json!([boosted])).unwrap(), &request).unwrap();
        assert_eq!(egress_amount.to_string(), "4080934615929730944");
        assert_eq!(route_data.boost_fee, Some(5));
    }

    #[test]
    fn test_quotes_rank_safe_execution_amount() {
        let mut response = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        let request = quote_request("100000000", "btc.btc", "eth.eth");
        response[0]["lowLiquidityWarning"] = serde_json::json!(true);
        response[0]["egressAmountNative"] = serde_json::json!("1000");
        response[0]["boostQuote"]["egressAmountNative"] = serde_json::json!("10");
        response[1]["egressAmountNative"] = serde_json::json!("20");
        response[1]["boostQuote"]["lowLiquidityWarning"] = serde_json::json!(true);

        let (egress_amount, _, _, route_data) = get_best_quote(serde_json::from_value(response).unwrap(), &request).unwrap();
        assert_eq!(egress_amount, BigUint::from(20u32));
        assert_eq!(route_data.boost_fee, None);
    }

    #[test]
    fn test_quote_must_match_request() {
        let quote = serde_json::json!([{
            "type": "regular",
            "ingressAsset": "eth.eth",
            "ingressAmountNative": "1",
            "egressAsset": "btc.btc",
            "egressAmountNative": "1",
            "lowLiquidityWarning": false,
            "recommendedSlippageTolerancePercent": 1,
            "estimatedDurationSeconds": 60,
            "estimatedPrice": 1
        }]);
        let request = quote_request("1", "eth.eth", "btc.btc");

        for (field, value) in [
            ("ingressAsset", serde_json::json!("sol.sol")),
            ("ingressAmountNative", serde_json::json!("2")),
            ("egressAsset", serde_json::json!("eth.eth")),
        ] {
            let mut mismatched_quote = quote.clone();
            mismatched_quote[0][field] = value;
            let quotes = serde_json::from_value(mismatched_quote).unwrap();

            assert_eq!(get_best_quote(quotes, &request), Err(SwapperError::NoQuoteAvailable));
        }
    }

    #[test]
    fn test_quote_requires_safe_numeric_values() {
        let quote = serde_json::json!([{
            "type": "regular",
            "ingressAsset": "eth.eth",
            "ingressAmountNative": "1",
            "egressAsset": "btc.btc",
            "egressAmountNative": "1",
            "lowLiquidityWarning": false,
            "recommendedSlippageTolerancePercent": 1,
            "estimatedDurationSeconds": 60,
            "estimatedPrice": 1
        }]);
        let request = quote_request("1", "eth.eth", "btc.btc");

        for (field, value) in [
            ("egressAmountNative", serde_json::json!("0")),
            ("recommendedSlippageTolerancePercent", serde_json::json!(100)),
            ("estimatedDurationSeconds", serde_json::json!(0)),
            ("estimatedPrice", serde_json::json!(0)),
        ] {
            let mut invalid_quote = quote.clone();
            invalid_quote[0][field] = value;
            let quotes = serde_json::from_value(invalid_quote).unwrap();

            assert_eq!(get_best_quote(quotes, &request), Err(SwapperError::InvalidRoute));
        }
    }

    #[test]
    fn test_dca_quote_requires_execution_parameters() {
        let quotes = serde_json::from_value::<Vec<QuoteResponse>>(serde_json::json!([{
            "ingressAsset": "eth.eth",
            "ingressAmountNative": "1",
            "egressAsset": "btc.btc",
            "egressAmountNative": "1",
            "recommendedSlippageTolerancePercent": 1,
            "estimatedDurationSeconds": 60,
            "type": "dca",
            "lowLiquidityWarning": false,
            "estimatedPrice": 1
        }]))
        .unwrap();

        assert_eq!(get_best_quote(quotes, &quote_request("1", "eth.eth", "btc.btc")), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_refund_parameters() {
        assert_eq!(
            refund_parameters(REFUND_RETRY_BLOCKS, "refund-address", "0x1234"),
            RefundParameters {
                retry_duration: REFUND_RETRY_BLOCKS,
                refund_address: "refund-address".to_string(),
                min_price: "0x1234".to_string(),
                max_oracle_price_slippage: None,
            }
        );
    }

    #[test]
    fn test_best_boost_quote() {
        let quotes: Vec<QuoteResponse> = serde_json::from_str(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        let request = quote_request("100000000", "btc.btc", "eth.eth");
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, &request).unwrap();

        assert_eq!(egress_amount.to_string(), "4080936927013539226");
        assert_eq!(slippage_bps, 150);
        assert_eq!(eta_in_seconds, 744);
        assert_eq!(
            route_data,
            ChainflipRouteData {
                boost_fee: Some(5),
                estimated_price: 40.83388759199202,
                dca_parameters: Some(DcaParameters {
                    number_of_chunks: 3,
                    chunk_interval: 2
                }),
            }
        );
    }

    #[test]
    fn test_boost_quote_must_match_request() {
        let mut quotes = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        quotes[0]["boostQuote"]["egressAsset"] = serde_json::json!("btc.btc");
        quotes[1]["boostQuote"]["egressAsset"] = serde_json::json!("btc.btc");
        let quotes = serde_json::from_value(quotes).unwrap();

        assert_eq!(
            get_best_quote(quotes, &quote_request("100000000", "btc.btc", "eth.eth")),
            Err(SwapperError::NoQuoteAvailable)
        );
    }

    #[test]
    fn test_boost_fee_rounds_up_within_protocol_limit() {
        let request = quote_request("100000000", "btc.btc", "eth.eth");
        let mut quotes = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        quotes[1]["boostQuote"]["estimatedBoostFeeBps"] = serde_json::json!(5.5);
        let quotes = serde_json::from_value(quotes).unwrap();

        assert_eq!(get_best_quote(quotes, &request).unwrap().3.boost_fee, Some(6));
    }

    #[test]
    fn test_boost_fee_rejects_value_above_protocol_limit() {
        let request = quote_request("100000000", "btc.btc", "eth.eth");
        let mut quotes = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        quotes[1]["boostQuote"]["estimatedBoostFeeBps"] = serde_json::json!(256);

        assert_eq!(get_best_quote(serde_json::from_value(quotes).unwrap(), &request), Err(SwapperError::InvalidRoute));
    }

    #[tokio::test]
    #[cfg(feature = "swap_integration_tests")]
    async fn test_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = ChainflipProvider::new(network_provider.clone());

        // Swap ID: 902663
        let tx_hash = "3sbA7vTDa8tmuokNeQxWJBPpxG3A1Vw5rhDxSm63w7hW31bo2nbci8CfLr27JsbhcebLwcJcwqbL8UP5aVCMFLGb";
        let chain = Chain::Solana;

        let result = swap_provider.get_swap_result(chain, tx_hash).await?;

        println!("Chainflip swap result: {:?}", result);
        assert_eq!(result.status, SwapStatus::Completed);

        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "swap_integration_tests")]
    async fn test_get_quote_data_tron_usdt_to_arbitrum_usdc() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = ChainflipProvider::new(network_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::mock_with_asset_id(primitives::known_assets::TRON_USDT.id.clone(), "USDT", 6),
            to_asset: SwapperQuoteAsset::mock_with_asset_id(primitives::known_assets::ARBITRUM_USDC.id.clone(), "USDC", 6),
            wallet_address: "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            // Route-specific minimums can exceed the global minimum returned by /assets.
            value: BigUint::from(100000000u64),
            options: Options::default(),
        };

        let quote = swap_provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(quote.data.slippage_bps, 50);

        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.data_type, SwapQuoteDataType::Contract);
        assert_eq!(quote_data.to, primitives::asset_constants::TRON_USDT_TOKEN_ID);
        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert!(quote_data.data.starts_with("a9059cbb"));
        assert!(quote_data.memo.as_deref().is_some_and(|memo| memo.starts_with("0x")));

        Ok(())
    }
}
