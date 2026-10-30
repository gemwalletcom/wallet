use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use async_trait::async_trait;
use gem_client::Client;
use gem_evm::contracts::IERC20;
use gem_evm::u256::biguint_to_u256;
use gem_evm::u256::u256_to_biguint;
use gem_solana::DEFAULT_SWAP_GAS_LIMIT;
use gem_tron::address::TronAddress;
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use std::{fmt::Debug, str::FromStr, sync::Arc, time::Duration};

use super::{
    ChainflipRouteData,
    broker::{
        AssetsResponse, BrokerClient, ChainflipAsset, DcaParameters, DepositChannelRequest, EvmVaultSwapResponse, QuoteDetails, QuoteRequest as ChainflipQuoteRequest, QuoteResponse, QuoteType, RefundParameters, SolanaVaultSwapResponse,
        TronVaultSwapResponse, VaultSwapChainExtras, VaultSwapExtras, VaultSwapResponse, VaultSwapSolanaExtras,
    },
    client::{ChainflipClient, SUPPORTED_ASSETS, map_swap_result},
    contracts::IVault,
    destination::VaultSwapDestination,
    price::{price_after_slippage, price_to_hex_price},
    refund::VaultSwapRefund,
    seed::generate_random_seed,
    tx_builder,
};
use crate::{
    Contracts, Deployment, FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, SwapResult, SwapResultRequest, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData, SwapperSlippage,
    alien::RpcProvider,
    approval::{check_approval_erc20, get_swap_gas_limit_with_approval},
    cross_chain::VaultAddresses,
    fees::DEFAULT_CHAINFLIP_FEE_BPS as DEFAULT_FEE_BPS,
    route_cache::Cache,
};
use primitives::{
    AssetId, Chain, ChainType, MINUTE,
    hex::{decode_hex, encode_with_0x},
    swap::{ApprovalData, SlippageMode},
};

const DEFAULT_SWAP_ERC20_GAS_LIMIT: u64 = 100_000;
const REFUND_RETRY_BLOCKS: u32 = 150;
const ASSETS_CACHE_TTL: Duration = MINUTE.saturating_mul(5);
const ETHEREUM_VAULT: &str = "0xF5e10380213880111522dd0efD3dbb45b9f62Bcc";
const ARBITRUM_VAULT: &str = "0x79001a5e762f3bEFC8e5871b42F6734e00498920";
const SOLANA_VAULT: &str = "J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT";
const TRON_VAULT: &str = "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC";

pub const VAULT_ADDRESSES: &[Deployment] = &[
    Deployment::new(Chain::Ethereum, ETHEREUM_VAULT),
    Deployment::new(Chain::Arbitrum, ARBITRUM_VAULT),
    Deployment::new(Chain::Solana, SOLANA_VAULT),
    Deployment::new(Chain::Tron, TRON_VAULT),
];

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

    async fn vault_approval(&self, from_asset: &AssetId, wallet_address: &str, input_amount: &BigUint) -> Result<Option<ApprovalData>, SwapperError> {
        let (ChainType::Ethereum, Some(token_id)) = (from_asset.chain.chain_type(), from_asset.token_id.clone()) else {
            return Ok(None);
        };
        let amount = biguint_to_u256(input_amount).ok_or_else(|| SwapperError::compute_quote_error("swap amount is too large"))?;
        check_approval_erc20(wallet_address.to_string(), token_id, vault_address(from_asset.chain)?.to_string(), amount, self.rpc_provider.clone(), &from_asset.chain).await
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

fn is_vault_swap(chain: Chain) -> Result<bool, SwapperError> {
    match chain.chain_type() {
        ChainType::Ethereum | ChainType::Solana | ChainType::Tron => Ok(true),
        ChainType::Polkadot => Ok(false),
        _ => Err(SwapperError::NotSupportedChain),
    }
}

fn build_quote_request(request: &QuoteRequest, assets: &AssetsResponse) -> Result<(ChainflipQuoteRequest, BigUint), SwapperError> {
    let is_vault_swap = is_vault_swap(request.from_asset.chain())?;
    let source_asset = ChainflipAsset::from_asset_id(&request.from_asset.asset_id())?;
    let destination_asset = ChainflipAsset::from_asset_id(&request.to_asset.asset_id())?;
    let source_broker_asset = assets.asset(&source_asset).filter(|asset| asset.supports_ingress()).ok_or(SwapperError::NoQuoteAvailable)?;
    let destination_broker_asset = assets.asset(&destination_asset).filter(|asset| asset.supports_egress()).ok_or(SwapperError::NoQuoteAvailable)?;

    Ok((
        ChainflipQuoteRequest {
            amount: request.value.clone(),
            source_asset: source_broker_asset.id.clone(),
            destination_asset: destination_broker_asset.id.clone(),
            commission_bps: DEFAULT_FEE_BPS,
            is_vault_swap,
        },
        source_broker_asset.minimal_amount_native.clone(),
    ))
}

fn quote_slippage_bps(slippage: &SwapperSlippage, recommended_bps: u32) -> u32 {
    match slippage.mode {
        SlippageMode::Exact => slippage.bps,
        SlippageMode::Auto => recommended_bps,
    }
}

fn get_best_quote(quotes: Vec<QuoteResponse>, request: &ChainflipQuoteRequest) -> Result<(BigUint, u32, u32, ChainflipRouteData), SwapperError> {
    let ingress_amount = request.amount.clone();
    let matches_request = |details: &QuoteDetails| details.ingress_asset == request.source_asset && details.ingress_amount_native == ingress_amount && details.egress_asset == request.destination_asset;
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
                Some(boost_quote) if !boost_quote.details.low_liquidity_warning => Some((boost_quote.details, quote_type, recommended_slippage_tolerance_percent, Some(boost_quote.estimated_boost_fee_bps))),
                _ if !details.low_liquidity_warning => Some((details, quote_type, recommended_slippage_tolerance_percent, None)),
                _ => None,
            }
        })
        .max_by(|(left, ..), (right, ..)| left.egress_amount_native.cmp(&right.egress_amount_native))
        .ok_or(SwapperError::NoQuoteAvailable)?;
    let boost_fee = estimated_boost_fee_bps.map(|fee| fee.ceil().to_u32().filter(|fee| *fee <= u8::MAX as u32).ok_or(SwapperError::InvalidRoute)).transpose()?;
    if details.egress_amount_native == BigUint::from(0u32)
        || !details.estimated_price.is_finite()
        || details.estimated_price <= 0.0
        || !recommended_slippage_tolerance_percent.is_finite()
        || !(0.0..100.0).contains(&recommended_slippage_tolerance_percent)
    {
        return Err(SwapperError::InvalidRoute);
    }
    let slippage_bps = (recommended_slippage_tolerance_percent * 100.0).to_u32().ok_or(SwapperError::InvalidRoute)?;
    let eta_in_seconds = details.estimated_duration_seconds.ceil().to_u32().filter(|eta| *eta > 0).ok_or(SwapperError::InvalidRoute)?;
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

pub(super) fn refund_parameters(refund_address: &str, min_price: &str) -> RefundParameters {
    RefundParameters {
        retry_duration: REFUND_RETRY_BLOCKS,
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

fn vault_address(chain: Chain) -> Result<&'static str, SwapperError> {
    VAULT_ADDRESSES.iter().find(|vault| vault.chain == chain).map(|vault| vault.contract).ok_or(SwapperError::InvalidRoute)
}

fn vault_mismatch(chain: Chain, broker: &str) -> SwapperError {
    SwapperError::TransactionError(format!("Chainflip {chain} vault mismatch: broker={broker}"))
}

fn amount_mismatch(input_amount: &BigUint, broker: impl std::fmt::Display) -> SwapperError {
    SwapperError::TransactionError(format!("Chainflip swap amount mismatch: quote={input_amount}, broker={broker}"))
}

fn invalid_calldata(chain: Chain) -> SwapperError {
    SwapperError::TransactionError(format!("invalid Chainflip {chain} vault swap calldata"))
}

fn source_token_mismatch(broker: impl Debug) -> SwapperError {
    SwapperError::TransactionError(format!("Chainflip source token mismatch: broker={broker:?}"))
}

fn validate_amount(input_amount: &BigUint, broker_amount: &BigUint) -> Result<(), SwapperError> {
    if broker_amount != input_amount {
        return Err(amount_mismatch(input_amount, broker_amount));
    }
    Ok(())
}

fn vault_swap_value(token_id: Option<&str>, input_amount: &BigUint, broker_value: &BigUint) -> Result<BigUint, SwapperError> {
    match token_id {
        None => validate_amount(input_amount, broker_value).map(|()| input_amount.clone()),
        Some(_) if *broker_value == BigUint::ZERO => Ok(BigUint::ZERO),
        Some(_) => Err(SwapperError::TransactionError(format!("Chainflip token swap value must be zero: broker={broker_value}"))),
    }
}

fn validate_evm_vault_swap(chain: Chain, token_id: Option<&str>, input_amount: &BigUint, destination: &VaultSwapDestination, refund: &VaultSwapRefund, response: &EvmVaultSwapResponse) -> Result<BigUint, SwapperError> {
    let vault = Address::from_str(vault_address(chain)?).unwrap_or_default();
    if Address::from_str(&response.to).ok() != Some(vault) {
        return Err(vault_mismatch(chain, &response.to));
    }
    let value = vault_swap_value(token_id, input_amount, &response.value)?;
    let calldata = decode_hex(&response.calldata).map_err(|_| invalid_calldata(chain))?;
    match token_id {
        None => {
            let call = IVault::xSwapNativeCall::abi_decode(&calldata).map_err(|_| invalid_calldata(chain))?;
            destination.validate_call(call.dstChain, &call.dstAddress, call.dstToken)?;
            refund.validate(&call.cfParameters).map(|()| value)
        }
        Some(token_id) => {
            let call = IVault::xSwapTokenCall::abi_decode(&calldata).map_err(|_| invalid_calldata(chain))?;
            destination.validate_call(call.dstChain, &call.dstAddress, call.dstToken)?;
            refund.validate(&call.cfParameters)?;
            if Address::from_str(token_id).ok() != Some(call.srcToken) {
                return Err(source_token_mismatch(call.srcToken));
            }
            validate_amount(input_amount, &u256_to_biguint(&call.amount)).map(|()| value)
        }
    }
}

fn validate_solana_vault_swap(input_amount: &BigUint, destination: &VaultSwapDestination, refund: &VaultSwapRefund, response: &SolanaVaultSwapResponse) -> Result<(), SwapperError> {
    if response.program_id != SOLANA_VAULT {
        return Err(vault_mismatch(Chain::Solana, &response.program_id));
    }
    let data = decode_hex(&response.data).map_err(|_| invalid_calldata(Chain::Solana))?;
    let amount = data.get(8..16).and_then(|bytes| bytes.try_into().ok()).map(u64::from_le_bytes).ok_or_else(|| invalid_calldata(Chain::Solana))?;
    let cf_parameters = destination.validate_solana(&data)?;
    refund.validate(cf_parameters)?;
    validate_amount(input_amount, &BigUint::from(amount))
}

fn tron_trc20_transfer(calldata: &str) -> Result<IERC20::transferCall, SwapperError> {
    decode_hex(calldata).ok().and_then(|data| IERC20::transferCall::abi_decode(&data).ok()).ok_or_else(|| invalid_calldata(Chain::Tron))
}

fn validate_tron_vault_swap(token_id: Option<&str>, input_amount: &BigUint, destination: &VaultSwapDestination, refund: &VaultSwapRefund, response: &TronVaultSwapResponse) -> Result<(TronAddress, BigUint), SwapperError> {
    let note = decode_hex(&response.note).map_err(|_| invalid_calldata(Chain::Tron))?;
    let cf_parameters = destination.validate_tron(&note)?;
    refund.validate(cf_parameters)?;
    let is_vault = |address: &TronAddress| address.to_string() == TRON_VAULT;
    let value = vault_swap_value(token_id, input_amount, &response.value)?;
    match token_id {
        None => TronAddress::from_hex_or_base58(&response.to)
            .filter(is_vault)
            .map(|vault| (vault, value))
            .ok_or_else(|| vault_mismatch(Chain::Tron, &response.to)),
        Some(token_id) => {
            let source_token = response
                .source_token_address
                .as_deref()
                .and_then(TronAddress::from_hex_or_base58)
                .filter(|source_token| TronAddress::from_hex_or_base58(token_id) == Some(*source_token))
                .ok_or_else(|| source_token_mismatch(&response.source_token_address))?;
            let transfer = tron_trc20_transfer(&response.calldata)?;
            if !is_vault(&TronAddress::from(transfer.to)) {
                return Err(vault_mismatch(Chain::Tron, &transfer.to.to_string()));
            }
            validate_amount(input_amount, &u256_to_biguint(&transfer.value)).map(|()| (source_token, value))
        }
    }
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

    fn amount_mode(&self, request: &QuoteRequest) -> SwapAmountMode {
        match is_vault_swap(request.from_asset.chain()) {
            Ok(false) => SwapAmountMode::Flexible,
            Ok(true) | Err(_) => SwapAmountMode::Fixed,
        }
    }

    fn contracts(&self, from_asset: &AssetId) -> Contracts {
        Deployment::contracts(VAULT_ADDRESSES, from_asset.chain)
    }

    async fn preload_routes(&self, _from_asset: &AssetId, _to_asset: &AssetId) {
        _ = self.get_assets().await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let assets = self.get_assets().await?;
        let (quote_request, minimum_amount) = build_quote_request(request, &assets)?;
        validate_minimum_amount(&quote_request.amount, &minimum_amount)?;

        let quotes = self.broker_client.get_quotes(&quote_request).await?;
        let (egress_amount, recommended_slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, &quote_request)?;
        let slippage_bps = quote_slippage_bps(&request.options.slippage, recommended_slippage_bps);

        Ok(Quote {
            min_from_value: Some(minimum_amount),
            from_value: quote_request.amount,
            ..Quote::from_route(request, &self.provider, egress_amount, slippage_bps, &route_data, Some(eta_in_seconds))?
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        let source_asset = ChainflipAsset::from_asset_id(&from_asset)?;
        let destination_asset = ChainflipAsset::from_asset_id(&quote.request.to_asset.asset_id())?;

        let input_amount = quote.from_value.clone();
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: ChainflipRouteData = serde_json::from_str(&route.route_data)?;
        let chain = source_asset.chain.clone();
        let price_slippage = price_after_slippage(route_data.estimated_price, quote.data.slippage_bps);
        let min_price = price_to_hex_price(price_slippage, quote.request.to_asset.decimals, quote.request.from_asset.decimals).map_err(SwapperError::TransactionError)?;
        let refund_parameters = refund_parameters(&quote.request.wallet_address, &min_price);

        if !is_vault_swap(from_asset.chain)? {
            let request = DepositChannelRequest {
                source_asset,
                destination_asset,
                destination_address: quote.request.destination_address.clone(),
                boost_fee: route_data.boost_fee,
                refund_parameters,
                dca_parameters: route_data.dca_parameters,
            };
            let response = self.broker_client.request_deposit_channel(&request).await?;
            return Ok(SwapperQuoteData::new_transfer(response.deposit_address()?, input_amount, None));
        }
        let source_chain_type = from_asset.chain.chain_type();
        let destination = VaultSwapDestination::new(&quote.request.to_asset.asset_id(), &quote.request.destination_address)?;
        let refund = VaultSwapRefund::new(from_asset.chain, &refund_parameters)?;
        let extra_params = match source_chain_type {
            ChainType::Ethereum => VaultSwapExtras::Evm(VaultSwapChainExtras {
                chain,
                input_amount: input_amount.clone(),
                refund_parameters,
            }),
            ChainType::Tron => VaultSwapExtras::Tron(VaultSwapChainExtras {
                chain,
                input_amount: input_amount.clone(),
                refund_parameters,
            }),
            ChainType::Solana => VaultSwapExtras::Solana(VaultSwapSolanaExtras {
                from: quote.request.wallet_address.clone(),
                seed: encode_with_0x(&generate_random_seed(32)),
                chain,
                input_amount: input_amount.to_u64().ok_or_else(|| SwapperError::transaction_error("Solana input amount exceeds u64"))?,
                refund_parameters,
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
        let (response, approval) = futures::try_join!(broker, self.vault_approval(&from_asset, &quote.request.wallet_address, &input_amount))?;
        match (source_chain_type, response) {
            (ChainType::Ethereum, VaultSwapResponse::Evm(response)) => {
                let value = validate_evm_vault_swap(from_asset.chain, from_asset.token_id.as_deref(), &input_amount, &destination, &refund, &response)?;
                let gas_limit = get_swap_gas_limit_with_approval(&approval, None, DEFAULT_SWAP_ERC20_GAS_LIMIT);

                Ok(SwapperQuoteData::new_contract(vault_address(from_asset.chain)?.to_string(), value, response.calldata, approval, gas_limit))
            }
            (ChainType::Tron, VaultSwapResponse::Tron(response)) => {
                let (to, value) = validate_tron_vault_swap(from_asset.token_id.as_deref(), &input_amount, &destination, &refund, &response)?;
                tx_builder::build_tron_quote_data(&response, to, value)
            }
            (ChainType::Solana, VaultSwapResponse::Solana(response)) => {
                validate_solana_vault_swap(&input_amount, &destination, &refund, &response)?;
                let data = tx_builder::build_solana_transaction(&quote.request.wallet_address, &response)?;
                Ok(SwapperQuoteData::new_contract(response.program_id, BigUint::ZERO, data, None, Some(DEFAULT_SWAP_GAS_LIMIT.to_string())))
            }
            _ => Err(SwapperError::InvalidRoute),
        }
    }

    async fn get_vault_addresses(&self, _from_timestamp: Option<u64>) -> Result<VaultAddresses, SwapperError> {
        Ok(VaultAddresses {
            deposit: VAULT_ADDRESSES.iter().map(|vault| vault.contract.to_string()).collect(),
            send: vec![],
        })
    }

    async fn get_swap_result(&self, request: &SwapResultRequest) -> Result<SwapResult, SwapperError> {
        if !is_vault_swap(request.chain)? {
            let (assets, response) = futures::try_join!(self.get_assets(), self.broker_client.get_status_by_transaction_hash(&request.transaction_hash))?;
            return Ok(response.map_or_else(SwapResult::pending, |response| response.status.swap_result(&assets)));
        }
        let response = self.chainflip_client.get_tx_status(&request.transaction_hash).await?;
        Ok(response.as_ref().map_or_else(SwapResult::pending, map_swap_result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProviderData, SwapperQuoteAsset,
        alien::mock::ProviderMock,
        chainflip::testkit::{
            ARBITRUM_DESTINATION, ETHEREUM_NATIVE_MIN_PRICE, SOLANA_MIN_PRICE, TRON_NOTE, TRON_REFUND_ADDRESS, arbitrum_usdc_destination, bitcoin_destination, ethereum_native_refund, ethereum_usdc_refund, mock_refund, mock_routes,
            solana_refund, tron_refund,
        },
    };
    use gem_client::testkit::MockClient;
    use primitives::AssetId;
    use primitives::Chain;
    use primitives::asset_constants::TRON_USDT_TOKEN_ID;
    use primitives::known_assets::ARBITRUM_USDC;
    use primitives::swap::SwapStatus;
    use primitives::testkit::signer_mock::{TEST_EVM_RECIPIENT, TEST_POLKADOT_SENDER};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_contracts() {
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(MockClient::new()), BrokerClient::new(MockClient::new(), MockClient::new()), Arc::new(ProviderMock::new(String::new())));

        assert_eq!(provider.contracts(&AssetId::from_chain(Chain::Ethereum)), Contracts::new(vec![ETHEREUM_VAULT]));
        assert_eq!(provider.contracts(&AssetId::from_token(Chain::Tron, TRON_USDT_TOKEN_ID)), Contracts::new(vec![TRON_VAULT]));
        assert_eq!(provider.contracts(&AssetId::from_chain(Chain::Bitcoin)), Contracts::default(), "a deposit channel address is made per swap");
    }

    #[test]
    fn test_validate_minimum_amount() {
        let minimum_amount = BigUint::from(68_000_000u32);

        assert_eq!(
            validate_minimum_amount(&BigUint::from(1_000_000u32), &minimum_amount),
            Err(SwapperError::InputAmountError { min_amount: Some("68000000".to_string()) })
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
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(MockClient::new()), BrokerClient::new(broker_client, MockClient::new()), Arc::new(ProviderMock::new(String::new())));
        let mut request = QuoteRequest::mock(Chain::Solana, None);
        request.to_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Bitcoin));
        provider.preload_routes(&request.from_asset.asset_id(), &request.to_asset.asset_id()).await;

        let error = provider.get_quote(&request).await.unwrap_err();
        assert_eq!(error, SwapperError::InputAmountError { min_amount: Some("68000000".to_string()) });

        request.value = BigUint::from(68000000u64);
        let quote = provider.get_quote(&request).await.unwrap();
        assert_eq!(quote.min_from_value, Some(BigUint::from(68000000u64)));
        assert_eq!(assets_requests.load(Ordering::Relaxed), 1);
        assert_eq!(quote_requests.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_is_vault_swap() {
        assert_eq!(is_vault_swap(Chain::Ethereum), Ok(true));
        assert_eq!(is_vault_swap(Chain::Polkadot), Ok(false));
        assert_eq!(is_vault_swap(Chain::Bitcoin), Err(SwapperError::NotSupportedChain));
    }

    #[test]
    fn test_build_quote_request_uses_deposit_channel_for_polkadot() {
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Polkadot)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
            value: BigUint::from(40_000_000_000u64),
            ..QuoteRequest::mock(Chain::Polkadot, None)
        };

        let (quote_request, minimum_amount) = build_quote_request(&request, &AssetsResponse::mock()).unwrap();
        assert_eq!(
            gem_client::build_path_with_query("/quotes-native", &quote_request),
            "/quotes-native?amount=40000000000&sourceAsset=dot.hub&destinationAsset=eth.eth&commissionBps=45&isVaultSwap=false"
        );
        assert_eq!(minimum_amount, BigUint::from(40_000_000_000u64));
    }

    #[test]
    fn test_amount_mode() {
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(MockClient::new()), BrokerClient::new(MockClient::new(), MockClient::new()), Arc::new(ProviderMock::new(String::new())));

        assert_eq!(provider.amount_mode(&QuoteRequest::mock(Chain::Polkadot, None)), SwapAmountMode::Flexible);
        assert_eq!(provider.amount_mode(&QuoteRequest::mock(Chain::Ethereum, None)), SwapAmountMode::Fixed);
    }

    #[tokio::test]
    async fn test_quote_data_sends_polkadot_to_deposit_channel() {
        let deposit_channel = MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/deposit_channel");
            let request: DepositChannelRequest = serde_json::from_slice(body).unwrap();
            assert_eq!(
                request.source_asset,
                ChainflipAsset {
                    chain: "Assethub".into(),
                    asset: "DOT".into()
                }
            );
            assert_eq!(request.destination_address, TEST_EVM_RECIPIENT);
            assert_eq!(request.refund_parameters.refund_address, TEST_POLKADOT_SENDER);
            Ok(br#"{"jsonrpc":"2.0","id":1,"result":{"address":"0xb72845b75b44e7e2ee29a390e9cf2e71291f2a1823299eedf5d016662fdb96cf","issued_block":15237919,"channel_id":23}}"#.to_vec())
        });
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(MockClient::new()), BrokerClient::new(MockClient::new(), deposit_channel), Arc::new(ProviderMock::new(String::new())));
        let mut quote = Quote::mock(Chain::Polkadot, None);
        quote.request.wallet_address = TEST_POLKADOT_SENDER.to_string();
        quote.request.destination_address = TEST_EVM_RECIPIENT.to_string();
        quote.request.to_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum));
        quote.from_value = BigUint::from(40_000_000_000u64);
        quote.data.routes = mock_routes(&quote, 0.0004);

        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!(data, SwapperQuoteData::new_transfer("1599j8DFFZutdkvmxe81QNCpjNWAmmLD5pHSYph6gPVbp6av".to_string(), BigUint::from(40_000_000_000u64), None));
    }

    #[tokio::test]
    async fn test_polkadot_swap_result_uses_broker_status() {
        let broker = MockClient::new().with_get(|path| match path {
            "/assets" => Ok(include_str!("./broker/test/assets.json").as_bytes().to_vec()),
            "/status-by-transaction-hash?transactionHash=0xe3e6e6d9177796e473696d4f3beec9475f2e645f02a8bd524a1d194cd93d0026" => Ok(include_str!("./broker/test/status_dot_to_sol.json").as_bytes().to_vec()),
            "/status-by-transaction-hash?transactionHash=0xunknown" => Err(gem_client::ClientError::Http { status: 404, body: vec![] }),
            _ => panic!("unexpected path: {path}"),
        });
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(MockClient::new()), BrokerClient::new(broker, MockClient::new()), Arc::new(ProviderMock::new(String::new())));

        let result = provider
            .get_swap_result(&SwapResultRequest::new(Chain::Polkadot, "0xe3e6e6d9177796e473696d4f3beec9475f2e645f02a8bd524a1d194cd93d0026"))
            .await
            .unwrap();
        assert_eq!(result.status, SwapStatus::Completed);
        assert_eq!(result.metadata.unwrap().from_asset, AssetId::from_chain(Chain::Polkadot));

        assert_eq!(provider.get_swap_result(&SwapResultRequest::new(Chain::Polkadot, "0xunknown")).await.unwrap(), SwapResult::pending());
    }

    #[tokio::test]
    async fn test_vault_swap_result_pending_until_witnessed() {
        let client = MockClient::new().with_get(|path| match path {
            "/v2/swaps/0xunknown" => Err(gem_client::ClientError::Http { status: 404, body: vec![] }),
            _ => panic!("unexpected path: {path}"),
        });
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(client), BrokerClient::new(MockClient::new(), MockClient::new()), Arc::new(ProviderMock::new(String::new())));

        assert_eq!(provider.get_swap_result(&SwapResultRequest::new(Chain::Tron, "0xunknown")).await.unwrap(), SwapResult::pending());
    }

    #[test]
    fn test_build_quote_request_uses_broker_asset_ids() {
        for (chain, asset_id) in [(Chain::Bitcoin, "btc.btc"), (Chain::Polkadot, "dot.hub")] {
            let request = QuoteRequest {
                from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum)),
                to_asset: SwapperQuoteAsset::from(AssetId::from_chain(chain)),
                value: BigUint::parse_bytes(b"1000000000000000000", 10).unwrap(),
                ..QuoteRequest::mock(Chain::Ethereum, None)
            };

            let (quote_request, _) = build_quote_request(&request, &AssetsResponse::mock()).unwrap();
            assert_eq!(
                gem_client::build_path_with_query("/quotes-native", &quote_request),
                format!("/quotes-native?amount=1000000000000000000&sourceAsset=eth.eth&destinationAsset={asset_id}&commissionBps=45&isVaultSwap=true")
            );
        }
    }

    #[test]
    fn test_validate_tron_vault_swap_pins_native_amount_to_quote() {
        let response = TronVaultSwapResponse {
            calldata: "0x".to_string(),
            value: BigUint::from(50_000_000u32),
            to: TRON_VAULT.to_string(),
            note: TRON_NOTE.to_string(),
            source_token_address: None,
        };

        assert_eq!(
            validate_tron_vault_swap(None, &BigUint::from(50_000_000u32), &arbitrum_usdc_destination(), &tron_refund(), &response),
            Ok((TronAddress::parse(TRON_VAULT).unwrap(), BigUint::from(50_000_000u32)))
        );
        assert_eq!(
            validate_tron_vault_swap(None, &BigUint::from(40_000_000u32), &arbitrum_usdc_destination(), &tron_refund(), &response),
            Err(amount_mismatch(&BigUint::from(40_000_000u32), &response.value))
        );
        assert_eq!(
            validate_tron_vault_swap(None, &BigUint::from(50_000_000u32), &bitcoin_destination(), &solana_refund(), &response),
            Err(SwapperError::TransactionError("Chainflip Bitcoin destination mismatch".to_string()))
        );
    }

    #[test]
    fn test_validate_tron_vault_swap_rejects_token_native_value() {
        let response = TronVaultSwapResponse {
            calldata: "0xa9059cbb".to_string(),
            value: BigUint::from(1u32),
            to: TRON_VAULT.to_string(),
            note: TRON_NOTE.to_string(),
            source_token_address: Some(TRON_USDT_TOKEN_ID.to_string()),
        };

        assert_eq!(
            validate_tron_vault_swap(Some(TRON_USDT_TOKEN_ID), &BigUint::from(10_000_000u32), &arbitrum_usdc_destination(), &tron_refund(), &response),
            Err(SwapperError::TransactionError("Chainflip token swap value must be zero: broker=1".to_string()))
        );
    }

    #[test]
    fn test_validate_tron_vault_swap_pins_token_amount_to_calldata() {
        let response = TronVaultSwapResponse {
            calldata: "0xa9059cbb00000000000000000000000032e07e5dfcb75c2977adddc593eb8d6b6e71f96e0000000000000000000000000000000000000000000000000000000000989680".to_string(),
            value: BigUint::ZERO,
            to: TRON_VAULT.to_string(),
            note: TRON_NOTE.to_string(),
            source_token_address: Some("0xa614f803b6fd780986a42c78ec9c7f77e6ded13c".to_string()),
        };

        assert_eq!(
            validate_tron_vault_swap(Some(TRON_USDT_TOKEN_ID), &BigUint::from(10_000_000u32), &arbitrum_usdc_destination(), &tron_refund(), &response),
            Ok((TronAddress::parse(TRON_USDT_TOKEN_ID).unwrap(), BigUint::ZERO))
        );
        assert_eq!(
            validate_tron_vault_swap(Some(TRON_USDT_TOKEN_ID), &BigUint::from(9_999_999u32), &arbitrum_usdc_destination(), &tron_refund(), &response),
            Err(amount_mismatch(&BigUint::from(9_999_999u32), BigUint::from(10_000_000u32)))
        );
    }

    #[test]
    fn test_validate_tron_vault_swap_requires_the_vault() {
        let native = TronVaultSwapResponse {
            calldata: "0x".to_string(),
            value: BigUint::from(50_000_000u32),
            to: "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi".to_string(),
            note: TRON_NOTE.to_string(),
            source_token_address: None,
        };
        let token = TronVaultSwapResponse {
            calldata: "0xa9059cbb0000000000000000000000002523ae929fecd9d665f472f59b99a8ce6b1795100000000000000000000000000000000000000000000000000000000000989680".to_string(),
            value: BigUint::ZERO,
            to: TRON_VAULT.to_string(),
            note: TRON_NOTE.to_string(),
            source_token_address: Some(TRON_USDT_TOKEN_ID.to_string()),
        };
        let other_token = TronVaultSwapResponse {
            source_token_address: Some("TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf".to_string()),
            ..token.clone()
        };

        assert_eq!(
            validate_tron_vault_swap(None, &BigUint::from(50_000_000u32), &arbitrum_usdc_destination(), &tron_refund(), &native),
            Err(vault_mismatch(Chain::Tron, "TDMakP1fbWc7XXoSWZpujpjRAuePPEn4oi"))
        );
        assert_eq!(
            validate_tron_vault_swap(Some(TRON_USDT_TOKEN_ID), &BigUint::from(10_000_000u32), &arbitrum_usdc_destination(), &tron_refund(), &token),
            Err(vault_mismatch(Chain::Tron, "0x2523AE929feCd9D665f472F59B99a8CE6b179510"))
        );
        assert_eq!(
            validate_tron_vault_swap(Some(TRON_USDT_TOKEN_ID), &BigUint::from(10_000_000u32), &arbitrum_usdc_destination(), &tron_refund(), &other_token),
            Err(source_token_mismatch(&other_token.source_token_address))
        );
    }

    #[tokio::test]
    async fn test_quote_data_sends_native_trx_to_the_vault() {
        let broker = MockClient::new().with_post(|path, _| {
            assert_eq!(path, "/rpc");
            Ok(serde_json::to_vec(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "calldata": "0x",
                    "value": "0xf4240",
                    "to": TRON_VAULT,
                    "note": TRON_NOTE,
                    "source_token_address": "TXYZopYRdj2D9XRtbG411XZZ3kM5VkAeBf"
                }
            }))
            .unwrap())
        });
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(MockClient::new()), BrokerClient::new(broker, MockClient::new()), Arc::new(ProviderMock::new(String::new())));
        let quote = Quote {
            request: QuoteRequest {
                to_asset: SwapperQuoteAsset::from(ARBITRUM_USDC.id.clone()),
                destination_address: ARBITRUM_DESTINATION.to_string(),
                wallet_address: TRON_REFUND_ADDRESS.to_string(),
                ..QuoteRequest::mock(Chain::Tron, None)
            },
            ..Quote::mock(Chain::Tron, None)
        };
        let quote = Quote {
            data: ProviderData {
                routes: mock_routes(&quote, 0.0),
                ..quote.data.clone()
            },
            ..quote
        };

        let data = provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        assert_eq!(data.to, TRON_VAULT);
        assert_eq!(data.value, quote.from_value);
    }

    #[test]
    fn test_validate_evm_vault_swap() {
        let native: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_native.json")).unwrap();
        let usdc: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_usdc.json")).unwrap();
        let usdc_id = Some(primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID);
        let usdt_id = Some(primitives::asset_constants::ETHEREUM_USDT_TOKEN_ID);
        let eth_amount = BigUint::from(100_000_000_000_000_000u64);
        let usdc_amount = BigUint::from(500_000_000u64);
        let other_target = EvmVaultSwapResponse {
            to: TEST_EVM_RECIPIENT.to_string(),
            ..native.clone()
        };

        assert_eq!(validate_evm_vault_swap(Chain::Ethereum, None, &eth_amount, &bitcoin_destination(), &ethereum_native_refund(), &native), Ok(eth_amount.clone()));
        assert_eq!(validate_evm_vault_swap(Chain::Ethereum, usdc_id, &usdc_amount, &bitcoin_destination(), &ethereum_usdc_refund(), &usdc), Ok(BigUint::ZERO));
        assert_eq!(
            validate_evm_vault_swap(Chain::Ethereum, None, &eth_amount, &bitcoin_destination(), &ethereum_native_refund(), &other_target),
            Err(vault_mismatch(Chain::Ethereum, TEST_EVM_RECIPIENT))
        );
        assert_eq!(
            validate_evm_vault_swap(Chain::Arbitrum, None, &eth_amount, &bitcoin_destination(), &ethereum_native_refund(), &native),
            Err(vault_mismatch(Chain::Arbitrum, &native.to))
        );
        assert_eq!(
            validate_evm_vault_swap(Chain::Ethereum, None, &BigUint::from(1u32), &bitcoin_destination(), &ethereum_native_refund(), &native),
            Err(amount_mismatch(&BigUint::from(1u32), &eth_amount))
        );
        assert_eq!(
            validate_evm_vault_swap(Chain::Ethereum, usdc_id, &BigUint::from(1u32), &bitcoin_destination(), &ethereum_usdc_refund(), &usdc),
            Err(amount_mismatch(&BigUint::from(1u32), &usdc_amount))
        );
        assert_eq!(
            validate_evm_vault_swap(Chain::Ethereum, usdt_id, &usdc_amount, &bitcoin_destination(), &ethereum_usdc_refund(), &usdc),
            Err(source_token_mismatch(Address::from_str(primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID).unwrap()))
        );
        assert_eq!(
            validate_evm_vault_swap(
                Chain::Ethereum,
                usdc_id,
                &usdc_amount,
                &bitcoin_destination(),
                &ethereum_native_refund(),
                &EvmVaultSwapResponse { value: BigUint::ZERO, ..native }
            ),
            Err(invalid_calldata(Chain::Ethereum))
        );
    }

    #[test]
    fn test_validate_evm_vault_swap_binds_destination() {
        let native: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_native.json")).unwrap();
        let usdc: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_usdc.json")).unwrap();
        let usdc_id = Some(primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID);
        let other_address = VaultSwapDestination::new(&AssetId::from_chain(Chain::Bitcoin), "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh").unwrap();
        let mismatch = Err(SwapperError::TransactionError("Chainflip Bitcoin destination mismatch".to_string()));

        assert_eq!(
            validate_evm_vault_swap(Chain::Ethereum, None, &BigUint::from(100_000_000_000_000_000u64), &other_address, &ethereum_native_refund(), &native),
            mismatch
        );
        assert_eq!(validate_evm_vault_swap(Chain::Ethereum, usdc_id, &BigUint::from(500_000_000u64), &other_address, &ethereum_usdc_refund(), &usdc), mismatch);
        assert!(validate_evm_vault_swap(Chain::Ethereum, None, &BigUint::from(100_000_000_000_000_000u64), &arbitrum_usdc_destination(), &tron_refund(), &native).is_err());
    }

    #[test]
    fn test_validate_vault_swap_binds_refund() {
        let mismatch = SwapperError::TransactionError("Chainflip refund mismatch".to_string());
        let other_address = "0x1111111111111111111111111111111111111111";

        let native: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_native.json")).unwrap();
        let usdc: EvmVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_eth_usdc.json")).unwrap();
        let usdc_id = Some(primitives::asset_constants::ETHEREUM_USDC_TOKEN_ID);
        let eth_amount = BigUint::from(100_000_000_000_000_000u64);
        let other_refund = mock_refund(Chain::Ethereum, other_address, ETHEREUM_NATIVE_MIN_PRICE);
        let other_price = mock_refund(Chain::Ethereum, ARBITRUM_DESTINATION, "0x3292bb84399604d6d7d2345");
        assert_eq!(validate_evm_vault_swap(Chain::Ethereum, None, &eth_amount, &bitcoin_destination(), &other_refund, &native), Err(mismatch.clone()));
        assert_eq!(validate_evm_vault_swap(Chain::Ethereum, None, &eth_amount, &bitcoin_destination(), &other_price, &native), Err(mismatch.clone()));
        assert_eq!(
            validate_evm_vault_swap(Chain::Ethereum, usdc_id, &BigUint::from(500_000_000u64), &bitcoin_destination(), &other_refund, &usdc),
            Err(mismatch.clone())
        );

        let solana: SolanaVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_sol_native.json")).unwrap();
        let other_solana = mock_refund(Chain::Solana, "7g2rVN8fAAQdPh1mkajpvELqYa3gWvFXJsBLnKfEQfqy", SOLANA_MIN_PRICE);
        assert_eq!(validate_solana_vault_swap(&BigUint::from(500_000_000u64), &bitcoin_destination(), &other_solana, &solana), Err(mismatch.clone()));

        let tron = TronVaultSwapResponse {
            calldata: "0x".to_string(),
            value: BigUint::from(50_000_000u32),
            to: TRON_VAULT.to_string(),
            note: TRON_NOTE.to_string(),
            source_token_address: None,
        };
        let other_tron = mock_refund(Chain::Tron, TRON_VAULT, "0x0");
        assert_eq!(validate_tron_vault_swap(None, &BigUint::from(50_000_000u32), &arbitrum_usdc_destination(), &other_tron, &tron), Err(mismatch));
    }

    #[test]
    fn test_validate_solana_vault_swap() {
        let response: SolanaVaultSwapResponse = serde_json::from_str(include_str!("broker/test/vault_swap_sol_native.json")).unwrap();
        let amount = BigUint::from(500_000_000u64);
        let other_program = SolanaVaultSwapResponse {
            program_id: "11111111111111111111111111111111".to_string(),
            ..response.clone()
        };

        assert_eq!(validate_solana_vault_swap(&amount, &bitcoin_destination(), &solana_refund(), &response), Ok(()));
        assert_eq!(
            validate_solana_vault_swap(&BigUint::from(1u32), &bitcoin_destination(), &solana_refund(), &response),
            Err(amount_mismatch(&BigUint::from(1u32), &amount))
        );
        assert_eq!(
            validate_solana_vault_swap(&amount, &bitcoin_destination(), &solana_refund(), &other_program),
            Err(vault_mismatch(Chain::Solana, "11111111111111111111111111111111"))
        );
        assert!(validate_solana_vault_swap(&amount, &arbitrum_usdc_destination(), &solana_refund(), &response).is_err());
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
        let provider = ChainflipProvider::with_clients(ChainflipClient::new(MockClient::new()), BrokerClient::new(broker, MockClient::new()), Arc::new(ProviderMock::new(String::new())));
        let mut quote = Quote::mock(Chain::Tron, None);
        quote.request.to_asset = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ethereum));
        quote.request.destination_address = ARBITRUM_DESTINATION.to_string();
        quote.request.wallet_address = TRON_REFUND_ADDRESS.to_string();
        quote.data.routes = mock_routes(&quote, 1.0);

        assert_eq!(provider.get_quote_data(&quote, FetchQuoteData::None).await.unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_quote_slippage_bps() {
        assert_eq!(quote_slippage_bps(&SwapperSlippage::mock_exact(50), 250), 50);
        assert_eq!(quote_slippage_bps(&SwapperSlippage { mode: SlippageMode::Auto, bps: 100 }, 250), 250);
    }

    #[test]
    fn test_best_quote() {
        let quotes: Vec<QuoteResponse> = serde_json::from_str(include_str!("./test/chainflip_quotes.json")).unwrap();
        let request = ChainflipQuoteRequest::mock("10000000000", "sol.sol", "btc.btc");
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

        assert_eq!(get_best_quote(quotes, &ChainflipQuoteRequest::mock("1", "eth.eth", "btc.btc")).unwrap().2, 164);
    }

    #[test]
    fn test_empty_quotes_are_unavailable() {
        assert_eq!(get_best_quote(vec![], &ChainflipQuoteRequest::mock("1", "eth.eth", "btc.btc")), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_low_liquidity_quotes_are_excluded() {
        let response = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        let request = ChainflipQuoteRequest::mock("100000000", "btc.btc", "eth.eth");

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

        let mut warned_best = response;
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
        let request = ChainflipQuoteRequest::mock("100000000", "btc.btc", "eth.eth");

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
        let request = ChainflipQuoteRequest::mock("100000000", "btc.btc", "eth.eth");
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
        let request = ChainflipQuoteRequest::mock("1", "eth.eth", "btc.btc");

        for (field, value) in [("ingressAsset", serde_json::json!("sol.sol")), ("ingressAmountNative", serde_json::json!("2")), ("egressAsset", serde_json::json!("eth.eth"))] {
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
        let request = ChainflipQuoteRequest::mock("1", "eth.eth", "btc.btc");

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

        assert_eq!(get_best_quote(quotes, &ChainflipQuoteRequest::mock("1", "eth.eth", "btc.btc")), Err(SwapperError::InvalidRoute));
    }

    #[test]
    fn test_best_boost_quote() {
        let quotes: Vec<QuoteResponse> = serde_json::from_str(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        let request = ChainflipQuoteRequest::mock("100000000", "btc.btc", "eth.eth");
        let (egress_amount, slippage_bps, eta_in_seconds, route_data) = get_best_quote(quotes, &request).unwrap();

        assert_eq!(egress_amount.to_string(), "4080936927013539226");
        assert_eq!(slippage_bps, 150);
        assert_eq!(eta_in_seconds, 744);
        assert_eq!(
            route_data,
            ChainflipRouteData {
                boost_fee: Some(5),
                estimated_price: 40.83388759199202,
                dca_parameters: Some(DcaParameters { number_of_chunks: 3, chunk_interval: 2 }),
            }
        );
    }

    #[test]
    fn test_boost_quote_must_match_request() {
        let mut quotes = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        quotes[0]["boostQuote"]["egressAsset"] = serde_json::json!("btc.btc");
        quotes[1]["boostQuote"]["egressAsset"] = serde_json::json!("btc.btc");
        let quotes = serde_json::from_value(quotes).unwrap();

        assert_eq!(get_best_quote(quotes, &ChainflipQuoteRequest::mock("100000000", "btc.btc", "eth.eth")), Err(SwapperError::NoQuoteAvailable));
    }

    #[test]
    fn test_boost_fee_rounds_up_within_protocol_limit() {
        let request = ChainflipQuoteRequest::mock("100000000", "btc.btc", "eth.eth");
        let mut quotes = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        quotes[1]["boostQuote"]["estimatedBoostFeeBps"] = serde_json::json!(5.5);
        let quotes = serde_json::from_value(quotes).unwrap();

        assert_eq!(get_best_quote(quotes, &request).unwrap().3.boost_fee, Some(6));
    }

    #[test]
    fn test_boost_fee_rejects_value_above_protocol_limit() {
        let request = ChainflipQuoteRequest::mock("100000000", "btc.btc", "eth.eth");
        let mut quotes = serde_json::from_str::<serde_json::Value>(include_str!("./test/chainflip_boost_quotes.json")).unwrap();
        quotes[1]["boostQuote"]["estimatedBoostFeeBps"] = serde_json::json!(256);

        assert_eq!(get_best_quote(serde_json::from_value(quotes).unwrap(), &request), Err(SwapperError::InvalidRoute));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{NativeProvider, Options, SwapperQuoteAsset};
    use primitives::Chain;
    use primitives::{
        AssetId,
        asset_constants::TRON_USDT_TOKEN_ID,
        known_assets::TRON_USDT,
        swap::{SwapQuoteDataType, SwapStatus},
    };

    #[tokio::test]
    async fn test_get_swap_result() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = ChainflipProvider::new(network_provider.clone());

        let tx_hash = "3sbA7vTDa8tmuokNeQxWJBPpxG3A1Vw5rhDxSm63w7hW31bo2nbci8CfLr27JsbhcebLwcJcwqbL8UP5aVCMFLGb";
        let chain = Chain::Solana;

        let result = swap_provider.get_swap_result(&SwapResultRequest::new(chain, tx_hash)).await?;

        println!("Chainflip swap result: {:?}", result);
        assert_eq!(result.status, SwapStatus::Completed);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_quote_data_tron_usdt_to_arbitrum_usdc() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = ChainflipProvider::new(network_provider);
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::mock_with_asset_id(primitives::known_assets::TRON_USDT.id.clone(), "USDT", 6),
            to_asset: SwapperQuoteAsset::mock_with_asset_id(primitives::known_assets::ARBITRUM_USDC.id.clone(), "USDC", 6),
            wallet_address: "TEcDijvKSXcfWT7S6rd44H5vNgufm7Y4XC".to_string(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".to_string(),
            value: BigUint::from(100000000u64),
            options: Options::default(),
        };

        let quote = swap_provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(quote.data.slippage_bps, request.options.slippage.bps);

        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.data_type, SwapQuoteDataType::Contract);
        assert_eq!(quote_data.to, primitives::asset_constants::TRON_USDT_TOKEN_ID);
        assert_eq!(quote_data.value, BigUint::from(0u64));
        assert!(quote_data.data.starts_with("a9059cbb"));
        assert!(quote_data.memo.as_deref().is_some_and(|memo| memo.starts_with("0x")));

        Ok(())
    }

    #[tokio::test]
    async fn test_get_quote_data_tron_usdt_to_trx() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let swap_provider = ChainflipProvider::new(Arc::new(NativeProvider::default()));
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::mock_with_asset_id(TRON_USDT.id.clone(), "USDT", 6),
            to_asset: SwapperQuoteAsset::mock_with_asset_id(AssetId::from_chain(Chain::Tron), "TRX", 6),
            wallet_address: TRON_VAULT.to_string(),
            destination_address: TRON_VAULT.to_string(),
            value: BigUint::from(25_000_000u64),
            options: Options::default(),
        };

        let quote = swap_provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.data_type, SwapQuoteDataType::Contract);
        assert_eq!(quote_data.to, TRON_USDT_TOKEN_ID);
        assert_eq!(quote_data.value, BigUint::ZERO);
        assert_eq!(quote_data.approval, None);
        assert!(quote_data.data.starts_with("a9059cbb"));
        assert!(quote_data.memo.as_deref().is_some_and(|memo| memo.starts_with("0x")));
        Ok(())
    }
}
