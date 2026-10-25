use alloy_primitives::{Address, B256, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolValue;
use async_trait::async_trait;
use std::{fmt, str::FromStr, sync::Arc, vec};

use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteData,
    alien::RpcProvider,
    approval::check_permit2_approvals,
    client_factory::create_client_with_chain,
    fees::{on_chain_referral_fees, subtract_bps},
    uniswap::{
        deadline::get_sig_deadline,
        discovery::{PoolDiscovery, candidate_pairs, discover_v4_pools},
        fee_token::is_quote_input_fee_token,
        quote_result::{OutpricedPaths, get_best_quote},
        routed_asset::{Funding, Protocol, RoutedAsset, base_pair, routed_pair},
        swap_route::{PoolPath, RouteData, get_intermediaries},
    },
};
use gem_evm::uniswap::{
    FeeTier,
    command::{Permit2Permit, encode_commands},
    deployment::v4::get_uniswap_deployment_by_chain,
    path::BasePair,
};
use gem_hash::keccak::keccak256;
use primitives::{AssetId, Chain, EVMChain};

use super::{
    DEFAULT_SWAP_GAS_LIMIT, TEMPO_SWAP_GAS_LIMIT,
    commands::build_commands,
    path::{build_pool_key, build_pool_keys, build_quote_exact_params},
    quoter::{build_quote_exact_request, build_quote_exact_single_request},
};

const PROTOCOL: Protocol = Protocol::V4;

pub struct UniswapV4 {
    pub provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    pool_discovery: PoolDiscovery,
    outpriced_paths: OutpricedPaths,
}

impl UniswapV4 {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::UniswapV4),
            rpc_provider,
            pool_discovery: PoolDiscovery::default(),
            outpriced_paths: OutpricedPaths::default(),
        }
    }

    fn support_chain(&self, chain: &Chain) -> bool {
        get_uniswap_deployment_by_chain(chain).is_some()
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn routed_request(request: &QuoteRequest) -> Result<(EVMChain, RoutedAsset, RoutedAsset, u128), SwapperError> {
        let (evm_chain, input, output) = routed_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id(), PROTOCOL)?;
        let amount_in = U256::from_str(&request.value.to_string()).map_err(SwapperError::from)? / input.scale;
        let amount_in = u128::try_from(amount_in).map_err(|_| SwapperError::ComputeQuoteError("amount is too large".into()))?;
        Ok((evm_chain, input, output, amount_in))
    }

    async fn preload_pool_candidates(&self, chain: Chain, token_in: Address, token_out: Address) -> Result<(), SwapperError> {
        let deployment = get_uniswap_deployment_by_chain(&chain).ok_or(SwapperError::NotSupportedChain)?;
        let evm_chain = EVMChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or_else(|| SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let client = create_client_with_chain(self.rpc_provider.clone(), chain)?;
        let fee_tiers = self.get_tiers();
        let pairs = candidate_pairs(token_in, token_out, route_intermediaries(&token_in, &token_out, &base_pair));
        let pools = self
            .pool_discovery
            .missing_pools(chain, &pairs, &fee_tiers)
            .into_iter()
            .filter_map(|pool| {
                let pool_key = build_pool_key(&pool.token_in, &pool.token_out, &pool.fee_tier)?;
                Some((pool, B256::from(keccak256(&pool_key.abi_encode()))))
            })
            .collect::<Vec<_>>();
        if pools.is_empty() {
            return Ok(());
        }
        let discovered = discover_v4_pools(&client, deployment.state_view, &pools).await?;
        self.pool_discovery.record_pools(chain, &discovered);
        Ok(())
    }
}

fn route_intermediaries(token_in: &Address, token_out: &Address, base_pair: &BasePair) -> Vec<Address> {
    let base_tokens = base_pair.path_building_array();
    if base_tokens.contains(token_in) && base_tokens.contains(token_out) {
        return vec![];
    }
    get_intermediaries(token_in, token_out, base_pair)
}

impl fmt::Debug for UniswapV4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UniswapV4").finish()
    }
}

#[async_trait]
impl Swapper for UniswapV4 {
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        Chain::all().iter().filter(|x| self.support_chain(x)).map(|x| SwapperChainAsset::All(*x)).collect()
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        let Ok((_, input, output)) = routed_pair(from_asset, to_asset, PROTOCOL) else {
            return;
        };
        _ = self.preload_pool_candidates(from_asset.chain, input.address, output.address).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_chain = request.from_asset.chain();
        let deployment = get_uniswap_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, input, output, from_value) = Self::routed_request(request)?;
        let (token_in, token_out) = (input.address, output.address);
        let fee_tiers = self.get_tiers();
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or_else(|| SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let fee_token_is_input = is_quote_input_fee_token(Some(&base_pair), request, token_in, token_out);
        let fee_bps = on_chain_referral_fees(request).evm.bps;
        let quote_amount_in = if fee_token_is_input && fee_bps > 0 { subtract_bps(&from_value, fee_bps) } else { from_value };

        _ = self.preload_pool_candidates(from_chain, token_in, token_out).await;
        let direct_paths = build_pool_keys(&token_in, &token_out, &fee_tiers)
            .into_iter()
            .map(|(pairs, pool_key)| (PoolPath::new(PROTOCOL, pairs), build_quote_exact_single_request(&token_in, deployment.quoter, quote_amount_in, &pool_key)));
        let intermediaries = route_intermediaries(&token_in, &token_out, &base_pair);
        let multi_hop_paths = build_quote_exact_params(quote_amount_in, &token_in, &token_out, &fee_tiers, &intermediaries)
            .into_iter()
            .map(|(pairs, params)| (PoolPath::new(PROTOCOL, pairs), build_quote_exact_request(deployment.quoter, &params)));
        let paths = direct_paths.chain(multi_hop_paths).filter(|(path, _)| self.pool_discovery.path_may_exist(from_chain, &path.pairs)).collect();
        let client = create_client_with_chain(self.rpc_provider.clone(), from_chain)?;
        let quote_result = get_best_quote(&client, &self.outpriced_paths, (from_chain, token_in), (from_chain, token_out), quote_amount_in, paths, super::quoter::decode_quoter_response).await?;

        Ok(quote_result.into_quote(request, self.provider().clone(), fee_token_is_input, output.scale))
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_asset = request.from_asset.asset_id();
        let (evm_chain, input, output, amount_in) = Self::routed_request(request)?;
        let deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let route_data: RouteData = quote.route_data()?;
        let to_amount = u128::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let client = create_client_with_chain(self.rpc_provider.clone(), from_asset.chain)?;
        let permit = data.permit2_data().map(Permit2Permit::try_from).transpose()?;

        let (approval, permit2) = match input.funding {
            Funding::Token => {
                let spenders = (deployment.permit2, deployment.universal_router);
                check_permit2_approvals(&client, &request.wallet_address, &input.address.to_checksum(None), U256::from(amount_in), spenders, permit.is_some()).await?
            }
            Funding::Value | Funding::RouterBalance => (None, None),
        };
        let swap_gas_limit = match from_asset.chain {
            Chain::Tempo => TEMPO_SWAP_GAS_LIMIT,
            _ => DEFAULT_SWAP_GAS_LIMIT,
        };
        let gas_limit = (approval.is_some() || permit2.is_some()).then(|| swap_gas_limit.to_string());

        let sig_deadline = get_sig_deadline();
        let base_pair = base_pair(evm_chain, PROTOCOL);
        let fee_token_is_input = is_quote_input_fee_token(base_pair.as_ref(), request, input.address, output.address);

        let commands = build_commands(request, &input, &output, amount_in, to_amount, &quote.data.routes, permit, fee_token_is_input, deployment.universal_router_abi)?;
        let encoded = encode_commands(&commands, U256::from(sig_deadline));

        let value = input.value(U256::from(amount_in));

        Ok(SwapperQuoteData {
            permit2,
            ..SwapperQuoteData::new_contract(deployment.universal_router.into(), value, HexEncode(encoded), approval, gas_limit)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::mock::ProviderMock;
    use primitives::asset_constants::TEMPO_BRIDGED_USDC_ASSET_ID;
    use std::sync::Arc;

    #[test]
    fn test_route_intermediaries() {
        let base_pair = base_pair(EVMChain::Ethereum, PROTOCOL).unwrap();
        let usdc = base_pair.stables[0];
        let cfg = Address::repeat_byte(7);

        assert!(route_intermediaries(&base_pair.primary, &usdc, &base_pair).is_empty());
        assert!(route_intermediaries(&usdc, &cfg, &base_pair).contains(&base_pair.primary));
    }

    #[test]
    fn test_robinhood_supported() {
        let provider = Arc::new(ProviderMock::new("{}".to_string()));
        let swapper = UniswapV4::new(provider);

        assert!(swapper.support_chain(&Chain::Robinhood));
        assert!(swapper.supported_assets().contains(&SwapperChainAsset::All(Chain::Robinhood)));
    }

    #[test]
    fn rejects_tempo_network_asset() {
        let native = AssetId::from_chain(Chain::Tempo);
        assert!(routed_pair(&native, &TEMPO_BRIDGED_USDC_ASSET_ID, PROTOCOL).is_err());
        assert!(routed_pair(&TEMPO_BRIDGED_USDC_ASSET_ID, &native, PROTOCOL).is_err());
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use crate::testkit::mock_quote;
    use crate::{FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError, client_factory::create_eth_client, uniswap};
    use num_bigint::BigUint;
    use num_traits::ToPrimitive;
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARC_EURC_ASSET_ID, ETHEREUM_USDC_ASSET_ID, ROBINHOOD_USDG_TOKEN_ID, TEMPO_BRIDGED_USDC_ASSET_ID, TEMPO_PATHUSD_ASSET_ID},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_ethereum_usdc_to_cfg_routes_through_eth() -> Result<(), SwapperError> {
        let swap_provider = uniswap::default::boxed_uniswap_v4(Arc::new(NativeProvider::default()));
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
        let request = QuoteRequest {
            from_asset: ETHEREUM_USDC_ASSET_ID.clone().into(),
            to_asset: AssetId::from_token(Chain::Ethereum, "0xcccCCCcCCC33D538DBC2EE4fEab0a7A1FF4e8A94").into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::from(100_000_000u64),
            options: Options { slippage: 100.into(), use_max_amount: false },
        };

        let quote = swap_provider.get_quote(&request).await?;
        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;

        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(quote.data.routes.len(), 2);
        assert!(!quote_data.data.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_arc_native_usdc_eurc_quotes() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v4(network_provider);
        let options = Options { slippage: 100.into(), use_max_amount: false };
        let native = AssetId::from_chain(Chain::Arc);
        let wallet = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

        let request = QuoteRequest {
            from_asset: native.clone().into(),
            to_asset: ARC_EURC_ASSET_ID.clone().into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::from(1_000_000_000_000_000_000u64),
            options: options.clone(),
        };
        let quote = swap_provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::from(500_000u64) && quote.to_value < BigUint::from(1_000_000u64));
        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.value, BigUint::from(1_000_000_000_000_000_000u64));
        assert!(quote_data.approval.is_none());

        let request = QuoteRequest {
            from_asset: ARC_EURC_ASSET_ID.clone().into(),
            to_asset: native.into(),
            wallet_address: wallet.into(),
            destination_address: wallet.into(),
            value: BigUint::from(1_000_000u64),
            options,
        };
        let quote = swap_provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::from(1_000_000_000_000_000_000u64) && quote.to_value < BigUint::from(2_000_000_000_000_000_000u64));

        Ok(())
    }

    #[tokio::test]
    async fn test_v4_quoter() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v4(network_provider.clone());
        let options = Options { slippage: 100.into(), use_max_amount: false };

        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Unichain).into(),
            to_asset: AssetId::from(Chain::Unichain, Some("0x078D782b760474a361dDA0AF3839290b0EF57AD6".to_string())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(10000000000000000u64),
            options,
        };

        let quote = swap_provider.get_quote(&request).await?;

        assert!(quote.to_value > BigUint::ZERO);

        swap_provider.get_quote_data(&quote, FetchQuoteData::EstimateGas).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_robinhood_eth_to_usdg_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v4(network_provider.clone());
        let options = Options { slippage: 100.into(), use_max_amount: false };

        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Robinhood).into(),
            to_asset: AssetId::from(Chain::Robinhood, Some(ROBINHOOD_USDG_TOKEN_ID.to_string())).into(),
            wallet_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            destination_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            value: BigUint::from(100000000000000u64),
            options,
        };

        let quote = swap_provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::EstimateGas).await?;

        let estimate_value = format!("0x{:x}", quote_data.value.to_u128().ok_or_else(|| SwapperError::ComputeQuoteError("quote value is too large".to_string()))?);
        let gas = create_eth_client(network_provider.clone(), Chain::Robinhood)?
            .estimate_gas(Some(&request.wallet_address), &quote_data.to, Some(&estimate_value), Some(&quote_data.data))
            .await
            .map_err(|error| SwapperError::ComputeQuoteError(error.to_string()))?;
        assert!(!gas.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_tempo_quotes() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v4(network_provider);
        let options = Options { slippage: 100.into(), use_max_amount: false };
        let pathusd = TEMPO_PATHUSD_ASSET_ID.clone();

        for (from_asset, to_asset) in [(TEMPO_BRIDGED_USDC_ASSET_ID.clone(), pathusd.clone()), (pathusd, TEMPO_BRIDGED_USDC_ASSET_ID.clone())] {
            let request = QuoteRequest {
                options: options.clone(),
                ..mock_quote(from_asset.into(), to_asset.into())
            };
            let quote = swap_provider.get_quote(&request).await?;
            assert!(quote.to_value > BigUint::ZERO);
        }

        Ok(())
    }
}
