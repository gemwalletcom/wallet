use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::contracts::IERC20;
use gem_solana::DEFAULT_SWAP_GAS_LIMIT;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::{fmt::Debug, sync::Arc, time::Duration};

use super::{
    ChainflipRouteData,
    broker::{
        AssetsResponse, BrokerClient, ChainflipAsset, DcaParameters, RefundParameters, TronVaultSwapResponse, VaultSwapChainExtras, VaultSwapExtras, VaultSwapResponse,
        VaultSwapSolanaExtras,
    },
    capitalize::capitalize_first_letter,
    client::{ChainflipClient, QuoteRequest as ChainflipQuoteRequest, QuoteResponse, SUPPORTED_ASSETS, map_swap_result},
    price::{apply_slippage, price_to_hex_price},
    seed::generate_random_seed,
    tx_builder,
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, SwapAmountMode, SwapResult, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    alien::RpcProvider,
    amount_to_value,
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
const EVM_REFUND_RETRY_BLOCKS: u32 = 150;
const DEFAULT_REFUND_RETRY_BLOCKS: u32 = 10;
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

struct ChainflipQuoteRequestData {
    from_value: String,
    quote_request: ChainflipQuoteRequest,
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

fn build_quote_request(request: &QuoteRequest) -> Result<ChainflipQuoteRequestData, SwapperError> {
    match request.from_asset.chain().chain_type() {
        ChainType::Ethereum | ChainType::Solana | ChainType::Tron => {}
        _ => return Err(SwapperError::NotSupportedChain),
    }
    let from_value = request.value.clone();
    let src_asset = map_asset_id(&request.from_asset);
    let dest_asset = map_asset_id(&request.to_asset);
    let fee_bps = DEFAULT_FEE_BPS;

    Ok(ChainflipQuoteRequestData {
        from_value: from_value.clone(),
        quote_request: ChainflipQuoteRequest {
            amount: from_value,
            src_chain: src_asset.chain,
            src_asset: src_asset.asset,
            dest_chain: dest_asset.chain,
            dest_asset: dest_asset.asset,
            is_vault_swap: true,
            dca_enabled: true,
            broker_commission_bps: Some(fee_bps),
        },
    })
}

fn get_best_quote(mut quotes: Vec<QuoteResponse>, fee_bps: u32) -> (BigUint, u32, u32, ChainflipRouteData) {
    quotes.sort_by(|a, b| b.egress_amount.cmp(&a.egress_amount));
    let quote = &quotes[0];
    let quote_live_price_slippage_bps = quote.live_price_slippage_bps();
    let quote_retry_duration_blocks = quote.retry_duration_blocks();

    let (egress_amount, slippage_bps, eta_in_seconds, boost_fee, estimated_price, dca_parameters, live_price_slippage_bps, retry_duration_blocks) =
        if let Some(boost_quote) = &quote.boost_quote {
            (
                boost_quote.egress_amount.clone(),
                boost_quote.slippage_bps(),
                boost_quote.estimated_duration_seconds.ceil() as u32,
                Some(boost_quote.estimated_boost_fee_bps),
                boost_quote.estimated_price.clone(),
                boost_quote.dca_params.as_ref().map(|dca| DcaParameters {
                    number_of_chunks: dca.number_of_chunks,
                    chunk_interval: dca.chunk_interval_blocks,
                }),
                boost_quote.live_price_slippage_bps().or(quote_live_price_slippage_bps),
                boost_quote.retry_duration_blocks().or(quote_retry_duration_blocks),
            )
        } else {
            (
                quote.egress_amount.clone(),
                quote.slippage_bps(),
                quote.estimated_duration_seconds.ceil() as u32,
                None,
                quote.estimated_price.clone(),
                quote.dca_params.as_ref().map(|dca| DcaParameters {
                    number_of_chunks: dca.number_of_chunks,
                    chunk_interval: dca.chunk_interval_blocks,
                }),
                quote_live_price_slippage_bps,
                quote_retry_duration_blocks,
            )
        };

    (
        egress_amount,
        slippage_bps,
        eta_in_seconds,
        ChainflipRouteData {
            boost_fee,
            fee_bps,
            estimated_price,
            dca_parameters,
            live_price_slippage_bps,
            retry_duration_blocks,
        },
    )
}

fn refund_parameters(route_data: &ChainflipRouteData, default_retry_duration: u32, refund_address: &str, min_price: &str) -> RefundParameters {
    RefundParameters {
        retry_duration: route_data
            .retry_duration_blocks
            .map(|blocks| blocks.max(default_retry_duration))
            .unwrap_or(default_retry_duration),
        refund_address: refund_address.to_string(),
        min_price: min_price.to_string(),
        max_oracle_price_slippage: route_data.live_price_slippage_bps,
    }
}

fn map_chainflip_quote_error(error: SwapperError, from_decimals: u32) -> SwapperError {
    match error {
        SwapperError::ComputeQuoteError(message) => {
            let lower = message.to_ascii_lowercase();
            if lower.contains("expected amount is below minimum swap amount") {
                SwapperError::InputAmountError {
                    min_amount: parse_min_amount(&message, from_decimals),
                }
            } else {
                SwapperError::ComputeQuoteError(message)
            }
        }
        other => other,
    }
}

fn parse_min_amount(message: &str, decimals: u32) -> Option<String> {
    let open = message.rfind('(')?;
    let close = message[open + 1..].find(')')? + open + 1;
    let token = message[open + 1..close].trim();
    amount_to_value(token, decimals)
}

fn validate_minimum_amount(from_value: &str, minimum_amount: Option<&BigUint>) -> Result<(), SwapperError> {
    let from_value = from_value.parse::<BigUint>()?;
    if let Some(minimum_amount) = minimum_amount
        && from_value < *minimum_amount
    {
        return Err(SwapperError::InputAmountError {
            min_amount: Some(minimum_amount.to_string()),
        });
    }
    Ok(())
}

fn tron_trc20_transfer_value(calldata: &str) -> Result<String, SwapperError> {
    let data = decode_hex(calldata).map_err(|_| SwapperError::TransactionError("invalid Tron token transfer calldata".to_string()))?;
    IERC20::transferCall::abi_decode(&data)
        .map(|call| call.value.to_string())
        .map_err(|_| SwapperError::TransactionError("invalid Tron token transfer calldata".to_string()))
}

fn tron_quote_value(from_asset: &AssetId, input_amount: &BigUint, response: &TronVaultSwapResponse) -> Result<String, SwapperError> {
    let is_native = from_asset.is_native();
    let broker_value = if is_native {
        response.value.to_string()
    } else {
        if response.value != BigUint::from(0u32) {
            return Err(SwapperError::TransactionError(format!("Tron token swap value must be zero: broker={}", response.value)));
        }
        tron_trc20_transfer_value(&response.calldata)?
    };

    let expected = input_amount.to_string();
    if broker_value != expected {
        return Err(SwapperError::TransactionError(format!(
            "Tron swap amount mismatch: quote={expected}, broker={broker_value}"
        )));
    }

    Ok(if is_native { expected } else { "0".to_string() })
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
        let fee_bps = DEFAULT_FEE_BPS;
        let quote_request_data = build_quote_request(request)?;
        let source_asset = ChainflipAsset {
            chain: quote_request_data.quote_request.src_chain.clone(),
            asset: quote_request_data.quote_request.src_asset.clone(),
        };
        let destination_asset = ChainflipAsset {
            chain: quote_request_data.quote_request.dest_chain.clone(),
            asset: quote_request_data.quote_request.dest_asset.clone(),
        };
        let minimum_amount = self
            .get_assets()
            .await?
            .minimum_amount(&source_asset, &destination_asset)
            .ok_or(SwapperError::NoQuoteAvailable)?;
        validate_minimum_amount(&quote_request_data.from_value, Some(&minimum_amount))?;

        let quotes = match self.chainflip_client.get_quote(&quote_request_data.quote_request).await {
            Ok(quotes) => quotes,
            Err(err) => return Err(map_chainflip_quote_error(err, request.from_asset.decimals)),
        };
        if quotes.is_empty() {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, fee_bps);

        Ok(Quote {
            min_from_value: Some(minimum_amount.to_string()),
            from_value: quote_request_data.from_value,
            to_value: egress_amount.to_string(),
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

        let input_amount: BigUint = quote.from_value.parse()?;

        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: ChainflipRouteData = serde_json::from_str(&route.route_data)?;
        let chain = source_asset.chain.clone();
        let price = route_data.estimated_price.parse::<f64>().map_err(|_| SwapperError::transaction_error("Invalid price"))?;
        let price_slippage = apply_slippage(price, quote.data.slippage_bps);
        let quote_asset_decimals = quote.request.to_asset.decimals;
        let base_asset_decimals = quote.request.from_asset.decimals;
        let min_price = price_to_hex_price(price_slippage, quote_asset_decimals, base_asset_decimals).map_err(SwapperError::TransactionError)?;
        let source_chain_type = from_asset.chain.chain_type();
        let extra_params = match source_chain_type {
            ChainType::Ethereum => VaultSwapExtras::Evm(VaultSwapChainExtras {
                chain,
                input_amount: input_amount.clone(),
                refund_parameters: refund_parameters(&route_data, EVM_REFUND_RETRY_BLOCKS, &quote.request.wallet_address, &min_price),
            }),
            ChainType::Tron => VaultSwapExtras::Tron(VaultSwapChainExtras {
                chain,
                input_amount: input_amount.clone(),
                refund_parameters: refund_parameters(&route_data, DEFAULT_REFUND_RETRY_BLOCKS, &quote.request.wallet_address, &min_price),
            }),
            ChainType::Solana => VaultSwapExtras::Solana(VaultSwapSolanaExtras {
                from: quote.request.wallet_address.clone(),
                seed: encode_with_0x(&generate_random_seed(32)),
                chain,
                input_amount: input_amount.to_u64().ok_or_else(|| SwapperError::transaction_error("Solana input amount exceeds u64"))?,
                refund_parameters: refund_parameters(&route_data, DEFAULT_REFUND_RETRY_BLOCKS, &quote.request.wallet_address, &min_price),
            }),
            _ => return Err(SwapperError::NotSupportedChain),
        };

        let broker = self.broker_client.encode_vault_swap(
            source_asset,
            destination_asset,
            quote.request.destination_address.clone(),
            route_data.fee_bps,
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
                let value = if from_asset.is_native() { quote.from_value.clone() } else { "0".to_string() };

                let approval = if !from_asset.is_native() {
                    let token_id = from_asset.token_id.ok_or(SwapperError::NotSupportedAsset)?;
                    let approval_amount = quote.from_value.parse().map_err(SwapperError::from)?;
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
                    "".into(),
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

    #[test]
    fn test_chainflip_min_amount_error() {
        let message = "expected amount is below minimum swap amount (68000000)".to_string();
        let err = map_chainflip_quote_error(SwapperError::ComputeQuoteError(message), 6);
        assert_eq!(
            err,
            SwapperError::InputAmountError {
                min_amount: Some("68000000".into())
            }
        );

        let message = "expected amount is below minimum swap amount (1.23)".to_string();
        let err = map_chainflip_quote_error(SwapperError::ComputeQuoteError(message), 6);
        assert_eq!(
            err,
            SwapperError::InputAmountError {
                min_amount: Some("1230000".into())
            }
        );
    }

    #[test]
    fn test_validate_minimum_amount() {
        let minimum_amount = BigUint::from(68_000_000u32);

        assert_eq!(
            validate_minimum_amount("1000000", Some(&minimum_amount)),
            Err(SwapperError::InputAmountError {
                min_amount: Some("68000000".to_string())
            })
        );
        assert_eq!(validate_minimum_amount("68000000", Some(&minimum_amount)), Ok(()));
        assert_eq!(validate_minimum_amount("68000001", Some(&minimum_amount)), Ok(()));
        assert_eq!(validate_minimum_amount("1", None), Ok(()));
    }

    #[tokio::test]
    async fn test_chainflip_preload_caches_minimum_amount() {
        let assets_requests = Arc::new(AtomicUsize::new(0));
        let quote_requests = Arc::new(AtomicUsize::new(0));
        let assets_counter = assets_requests.clone();
        let quote_counter = quote_requests.clone();
        let assets_client = MockClient::new().with_get(move |path| {
            assert_eq!(path, "/assets");
            assets_counter.fetch_add(1, Ordering::Relaxed);
            Ok(include_str!("./broker/test/assets.json").as_bytes().to_vec())
        });
        let quote_client = MockClient::new().with_get(move |_| {
            quote_counter.fetch_add(1, Ordering::Relaxed);
            Ok(
                br#"[{"egressAmount":"1","recommendedSlippageTolerancePercent":1,"estimatedDurationSeconds":60,"type":"REGULAR","depositAmount":"68000000","isVaultSwap":true,"estimatedPrice":"1"}]"#
                    .to_vec(),
            )
        });
        let provider = ChainflipProvider::with_clients(
            ChainflipClient::new(quote_client),
            BrokerClient::new(assets_client),
            Arc::new(ProviderMock::new(String::new())),
        );
        let mut request = QuoteRequest::mock(Chain::Solana, None);
        provider.preload_routes(&request.from_asset.asset_id(), &request.to_asset.asset_id()).await;

        let error = provider.get_quote(&request).await.unwrap_err();
        assert_eq!(
            error,
            SwapperError::InputAmountError {
                min_amount: Some("68000000".to_string())
            }
        );

        request.value = "68000000".to_string();
        let quote = provider.get_quote(&request).await.unwrap();
        assert_eq!(quote.min_from_value, Some("68000000".to_string()));
        assert_eq!(assets_requests.load(Ordering::Relaxed), 1);
        assert_eq!(quote_requests.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_build_quote_request_rejects_bitcoin_source() {
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            value: "89100".to_string(),
            ..QuoteRequest::mock(Chain::Bitcoin, None)
        };

        assert!(build_quote_request(&request).is_err());
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

        assert_eq!(tron_quote_value(&from_asset, &BigUint::from(50_000_000u32), &response).unwrap(), "50000000");

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

        assert_eq!(tron_quote_value(&from_asset, &BigUint::from(10_000_000u32), &response).unwrap(), "0");

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
                fee_bps: DEFAULT_FEE_BPS,
                estimated_price: "1".to_string(),
                dca_parameters: None,
                live_price_slippage_bps: None,
                retry_duration_blocks: None,
            })
            .unwrap(),
        }];

        assert_eq!(provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_best_quote() {
        let quotes: Vec<QuoteResponse> = serde_json::from_str(include_str!("./test/chainflip_quotes.json")).unwrap();
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, DEFAULT_FEE_BPS);

        assert_eq!(egress_amount.to_string(), "145118751424");
        assert_eq!(slippage_bps, 250);
        assert_eq!(eta_in_seconds, 193);
        assert_eq!(
            route_data,
            ChainflipRouteData {
                boost_fee: None,
                fee_bps: DEFAULT_FEE_BPS,
                estimated_price: "14.5118765424".to_string(),
                dca_parameters: None,
                live_price_slippage_bps: Some(100),
                retry_duration_blocks: Some(50),
            }
        );
    }

    #[test]
    fn test_fractional_estimated_duration_rounds_up() {
        let quotes = serde_json::from_value::<Vec<QuoteResponse>>(serde_json::json!([{
            "egressAmount": "1",
            "recommendedSlippageTolerancePercent": 1,
            "estimatedDurationSeconds": 163.5,
            "type": "REGULAR",
            "depositAmount": "1",
            "isVaultSwap": true,
            "estimatedPrice": "1"
        }]))
        .unwrap();

        assert_eq!(get_best_quote(quotes, DEFAULT_FEE_BPS).2, 164);
    }

    #[test]
    fn test_refund_parameters_use_quote_recommendations() {
        let route_data = ChainflipRouteData {
            boost_fee: None,
            fee_bps: DEFAULT_FEE_BPS,
            estimated_price: "1".to_string(),
            dca_parameters: None,
            live_price_slippage_bps: Some(100),
            retry_duration_blocks: Some(50),
        };

        assert_eq!(
            refund_parameters(&route_data, DEFAULT_REFUND_RETRY_BLOCKS, "refund-address", "0x1234"),
            RefundParameters {
                retry_duration: 50,
                refund_address: "refund-address".to_string(),
                min_price: "0x1234".to_string(),
                max_oracle_price_slippage: Some(100),
            }
        );
        assert_eq!(
            refund_parameters(&route_data, EVM_REFUND_RETRY_BLOCKS, "refund-address", "0x1234").retry_duration,
            EVM_REFUND_RETRY_BLOCKS
        );
    }

    #[test]
    fn test_best_boost_quote() {
        let quotes: Vec<QuoteResponse> = serde_json::from_str(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, DEFAULT_FEE_BPS);

        assert_eq!(egress_amount.to_string(), "4080936927013539226");
        assert_eq!(slippage_bps, 100);
        assert_eq!(eta_in_seconds, 744);
        assert_eq!(
            route_data,
            ChainflipRouteData {
                boost_fee: Some(5),
                fee_bps: DEFAULT_FEE_BPS,
                estimated_price: "40.83388759199201533512".to_string(),
                dca_parameters: Some(DcaParameters {
                    number_of_chunks: 3,
                    chunk_interval: 2
                }),
                live_price_slippage_bps: Some(75),
                retry_duration_blocks: Some(30),
            }
        );
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
            value: "10000000".to_string(),
            options: Options::default(),
        };

        let quote = swap_provider.get_quote(&request).await?;
        let route_data: ChainflipRouteData = serde_json::from_str(&quote.data.routes[0].route_data)?;
        assert_eq!(quote.from_value, request.value);
        assert!(!quote.to_value.is_empty());
        assert_eq!(quote.data.slippage_bps, 50);
        assert_eq!(route_data.live_price_slippage_bps, Some(100));
        assert!(route_data.retry_duration_blocks.is_some_and(|blocks| blocks >= 50));

        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.data_type, SwapQuoteDataType::Contract);
        assert_eq!(quote_data.to, primitives::asset_constants::TRON_USDT_TOKEN_ID);
        assert_eq!(quote_data.value, "0");
        assert!(quote_data.data.starts_with("a9059cbb"));
        assert!(quote_data.memo.as_deref().is_some_and(|memo| memo.starts_with("0x")));

        Ok(())
    }
}
