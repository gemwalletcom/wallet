use alloy_primitives::{Address, B256, U256, hex::encode_prefixed as HexEncode};
use alloy_sol_types::SolValue;
use async_trait::async_trait;
use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use std::{collections::HashSet, fmt, iter, str::FromStr, sync::Arc, vec};

use crate::{
    FetchQuoteData, Permit2ApprovalData, ProviderData, ProviderType, Quote, QuoteRequest, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider,
    SwapperQuoteData,
    alien::{RpcClient, RpcProvider},
    approval::evm::{check_approval_erc20_with_client, check_approval_permit2_with_client},
    approval::get_swap_gas_limit_with_approval,
    fees::{apply_slippage_in_bp, default_referral_fees},
    uniswap::{
        deadline::get_sig_deadline,
        discovery::{PoolDiscovery, candidate_pairs, discover_v4_pools},
        fee_token::is_quote_input_fee_token,
        quote_result::{QuotePosition, get_best_quote},
        routed_asset::{Funding, Protocol, RoutedAsset, base_pair},
        swap_route::{RouteData, build_swap_route, get_intermediaries},
    },
};
use gem_evm::{
    jsonrpc::EthereumRpc,
    uniswap::{
        FeeTier,
        command::{Permit2Permit, encode_commands},
        deployment::v4::get_uniswap_deployment_by_chain,
    },
};
use gem_hash::keccak::keccak256;
use gem_jsonrpc::client::JsonRpcClient;
use primitives::{AssetId, Chain, EVMChain, swap::ApprovalData};

use super::{
    DEFAULT_SWAP_GAS_LIMIT, TEMPO_SWAP_GAS_LIMIT,
    commands::build_commands,
    path::{build_pool_key, build_pool_keys, build_quote_exact_params, get_intermediary_token},
    quoter::{build_quote_exact_requests, build_quote_exact_single_request},
};

const PROTOCOL: Protocol = Protocol::V4;

pub struct UniswapV4 {
    pub provider: ProviderType,
    rpc_provider: Arc<dyn RpcProvider>,
    pool_discovery: PoolDiscovery,
}

impl UniswapV4 {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::UniswapV4),
            rpc_provider,
            pool_discovery: PoolDiscovery::default(),
        }
    }

    fn support_chain(&self, chain: &Chain) -> bool {
        get_uniswap_deployment_by_chain(chain).is_some()
    }

    fn get_tiers(&self) -> Vec<FeeTier> {
        vec![FeeTier::Hundred, FeeTier::FiveHundred, FeeTier::ThreeThousand, FeeTier::TenThousand]
    }

    fn client_for(&self, chain: Chain) -> Result<JsonRpcClient<RpcClient>, SwapperError> {
        let endpoint = self.rpc_provider.get_endpoint(chain).map_err(SwapperError::from)?;
        let client = RpcClient::new(endpoint, self.rpc_provider.clone());
        Ok(JsonRpcClient::new(client))
    }

    fn is_base_pair(token_in: &Address, token_out: &Address, evm_chain: &EVMChain) -> bool {
        let Some(base_pair) = base_pair(*evm_chain, PROTOCOL) else {
            return false;
        };
        let base_set: HashSet<Address> = HashSet::from_iter(base_pair.path_building_array());
        base_set.contains(token_in) || base_set.contains(token_out)
    }

    fn routed_pair(from_asset: &AssetId, to_asset: &AssetId) -> Result<(EVMChain, RoutedAsset, RoutedAsset), SwapperError> {
        if from_asset.chain != to_asset.chain {
            return Err(SwapperError::NotSupportedChain);
        }
        let evm_chain = EVMChain::from_chain(from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        Ok((
            evm_chain,
            RoutedAsset::from_asset(from_asset, evm_chain, PROTOCOL)?,
            RoutedAsset::from_asset(to_asset, evm_chain, PROTOCOL)?,
        ))
    }

    fn routed_request(request: &QuoteRequest) -> Result<(EVMChain, RoutedAsset, RoutedAsset, u128), SwapperError> {
        let (evm_chain, input, output) = Self::routed_pair(&request.from_asset.asset_id(), &request.to_asset.asset_id())?;
        let amount_in = U256::from_str(&request.value.to_string()).map_err(SwapperError::from)? / input.scale;
        let amount_in = u128::try_from(amount_in).map_err(|_| SwapperError::ComputeQuoteError("amount is too large".into()))?;
        Ok((evm_chain, input, output, amount_in))
    }

    async fn preload_pool_candidates(&self, chain: Chain, token_in: Address, token_out: Address) -> Result<(), SwapperError> {
        let deployment = get_uniswap_deployment_by_chain(&chain).ok_or(SwapperError::NotSupportedChain)?;
        let evm_chain = EVMChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or_else(|| SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let client = self.client_for(chain)?;
        let fee_tiers = self.get_tiers();
        let pairs = candidate_pairs(token_in, token_out, get_intermediaries(&token_in, &token_out, &base_pair));
        let pools = self
            .pool_discovery
            .missing_pools(chain, &pairs, &fee_tiers)
            .into_iter()
            .map(|pool| {
                let (pool_key, _) = build_pool_key(&pool.token_in, &pool.token_out, &pool.fee_tier);
                (pool, B256::from(keccak256(&pool_key.abi_encode())))
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
        let Ok((_, input, output)) = Self::routed_pair(from_asset, to_asset) else {
            return;
        };
        _ = self.preload_pool_candidates(from_asset.chain, input.address, output.address).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let from_chain = request.from_asset.chain();
        let to_chain = request.to_asset.chain();
        let deployment = get_uniswap_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, input, output, from_value) = Self::routed_request(request)?;
        let (token_in, token_out) = (input.address, output.address);
        let fee_tiers = self.get_tiers();
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or(SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let fee_token_is_input = is_quote_input_fee_token(Some(&base_pair), request, token_in, token_out);
        let fee_bps = default_referral_fees().evm.bps;
        let quote_amount_in = if fee_token_is_input && fee_bps > 0 {
            apply_slippage_in_bp(&from_value, fee_bps)
        } else {
            from_value
        };

        _ = self.preload_pool_candidates(from_chain, token_in, token_out).await;
        let pool_keys = build_pool_keys(&token_in, &token_out, &fee_tiers);
        let pool_keys = pool_keys
            .into_iter()
            .filter(|(pairs, _)| self.pool_discovery.path_may_exist(from_chain, pairs))
            .collect::<Vec<_>>();
        let quote_exact_params = if !Self::is_base_pair(&token_in, &token_out, &evm_chain) {
            let intermediaries = get_intermediaries(&token_in, &token_out, &base_pair);
            build_quote_exact_params(quote_amount_in, &token_in, &token_out, &fee_tiers, &intermediaries)
                .into_iter()
                .map(|paths| {
                    paths
                        .into_iter()
                        .filter(|(pairs, _)| self.pool_discovery.path_may_exist(from_chain, pairs))
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            Vec::new()
        };
        let direct_calls = pool_keys
            .iter()
            .map(|pool_key| build_quote_exact_single_request(&token_in, deployment.quoter, quote_amount_in, &pool_key.1))
            .collect();
        let quote_calls = iter::once(direct_calls)
            .chain(build_quote_exact_requests(deployment.quoter, &quote_exact_params))
            .enumerate()
            .flat_map(|(route_idx, calls)| {
                calls
                    .into_iter()
                    .enumerate()
                    .map(move |(fee_tier_idx, call)| (QuotePosition { route_idx, fee_tier_idx }, call))
            })
            .collect::<Vec<_>>();
        let (positions, calls): (Vec<_>, Vec<EthereumRpc>) = quote_calls.into_iter().unzip();
        let results = self.client_for(from_chain)?.batch_request(calls).await?;
        let quote_result = get_best_quote(&results, &positions, super::quoter::decode_quoter_response)?;

        let fee_tier_idx = quote_result.fee_tier_idx;
        let route_idx = quote_result.route_idx;

        let to_value = if fee_token_is_input {
            quote_result.amount_out
        } else {
            apply_slippage_in_bp(&quote_result.amount_out, fee_bps)
        };
        let to_min_value = apply_slippage_in_bp(&to_value, request.options.slippage.bps);

        let fee_tier = if route_idx == 0 {
            pool_keys.get(fee_tier_idx).and_then(|(pairs, _)| pairs.first()).map(|pair| pair.fee_tier as u32)
        } else {
            quote_exact_params
                .get(route_idx - 1)
                .and_then(|params| params.get(fee_tier_idx))
                .and_then(|(pairs, _)| pairs.first())
                .map(|pair| pair.fee_tier as u32)
        }
        .ok_or(SwapperError::InvalidRoute)?;
        let asset_id_in = AssetId::from(from_chain, Some(token_in.to_checksum(None)));
        let asset_id_out = AssetId::from(to_chain, Some(token_out.to_checksum(None)));
        let asset_id_intermediary: Option<AssetId> = get_intermediary_token(&quote_exact_params, route_idx).map(|token| AssetId::from(to_chain, Some(token.to_checksum(None))));
        let route_data = RouteData {
            fee_tier: fee_tier.to_string(),
            min_amount_out: to_min_value.to_string(),
        };
        let routes = build_swap_route(&asset_id_in, asset_id_intermediary.as_ref(), &asset_id_out, &route_data);

        Ok(Quote {
            from_value: request.value.clone(),
            min_from_value: None,
            to_value: u256_to_biguint(&(to_value * output.scale)),
            data: ProviderData {
                provider: self.provider().clone(),
                routes,
                slippage_bps: request.options.slippage.bps,
            },
            request: request.clone(),
            eta_in_seconds: None,
        })
    }

    async fn get_permit2_for_quote(&self, quote: &Quote) -> Result<Option<Permit2ApprovalData>, SwapperError> {
        let from_asset = quote.request.from_asset.asset_id();
        let (_, input, _, amount_in) = Self::routed_request(&quote.request)?;
        if input.funding != Funding::Permit2 {
            return Ok(None);
        }
        let deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;

        let client = self.client_for(from_asset.chain)?;
        let permit2_data = check_approval_permit2_with_client(
            deployment.permit2,
            quote.request.wallet_address.clone(),
            input.address.to_string(),
            deployment.universal_router.to_string(),
            U256::from(amount_in),
            &client,
        )
        .await?
        .permit2_data();

        Ok(permit2_data)
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_asset = request.from_asset.asset_id();
        let (evm_chain, input, output, amount_in) = Self::routed_request(request)?;
        let deployment = get_uniswap_deployment_by_chain(&from_asset.chain).ok_or(SwapperError::NotSupportedChain)?;
        let route = quote.data.routes.first().ok_or(SwapperError::InvalidRoute)?;
        let route_data: RouteData = serde_json::from_str(&route.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let to_amount = u128::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let client = self.client_for(from_asset.chain)?;
        let permit = data.permit2_data().map(Permit2Permit::try_from).transpose()?;

        let approval: Option<ApprovalData> = if input.funding == Funding::Permit2 {
            check_approval_erc20_with_client(
                request.wallet_address.clone(),
                input.address.to_string(),
                deployment.permit2.to_string(),
                U256::from(amount_in),
                &client,
            )
            .await?
            .approval_data()
        } else {
            None
        };
        let swap_gas_limit = match from_asset.chain {
            Chain::Tempo => TEMPO_SWAP_GAS_LIMIT,
            _ => DEFAULT_SWAP_GAS_LIMIT,
        };
        let gas_limit = get_swap_gas_limit_with_approval(&approval, None, swap_gas_limit);

        let sig_deadline = get_sig_deadline();
        let base_pair = base_pair(evm_chain, PROTOCOL);
        let fee_token_is_input = is_quote_input_fee_token(base_pair.as_ref(), request, input.address, output.address);

        let commands = build_commands(
            request,
            &input,
            &output,
            amount_in,
            to_amount,
            &quote.data.routes,
            permit,
            fee_token_is_input,
            deployment.universal_router_abi,
        )?;
        let encoded = encode_commands(&commands, U256::from(sig_deadline));

        let value = match input.funding {
            Funding::Value | Funding::RouterBalance => u256_to_biguint(&(U256::from(amount_in) * input.scale)),
            Funding::Permit2 => BigUint::ZERO,
        };

        Ok(SwapperQuoteData::new_contract(
            deployment.universal_router.into(),
            value,
            HexEncode(encoded),
            approval,
            gas_limit,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Options, alien::mock::ProviderMock};
    use num_bigint::BigUint;
    use primitives::asset_constants::TEMPO_BRIDGED_USDC_ASSET_ID;
    use std::sync::Arc;

    #[test]
    fn test_is_base_pair() {
        let provider = Arc::new(ProviderMock::new("{}".to_string()));
        let swapper = UniswapV4::new(provider);
        let request = QuoteRequest {
            from_asset: AssetId::from(Chain::SmartChain, Some("0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82".to_string())).into(),
            to_asset: AssetId::from_chain(Chain::SmartChain).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(40000000000000000u64), // 0.04 Cake
            options: Options::default(),
        };

        let (evm_chain, input, output, _) = UniswapV4::routed_request(&request).unwrap();

        assert!(UniswapV4::is_base_pair(&input.address, &output.address, &evm_chain));
        // Ensure provider field is used to avoid warnings
        assert_eq!(swapper.provider.id, SwapperProvider::UniswapV4);
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
        assert!(UniswapV4::routed_pair(&native, &TEMPO_BRIDGED_USDC_ASSET_ID).is_err());
        assert!(UniswapV4::routed_pair(&TEMPO_BRIDGED_USDC_ASSET_ID, &native).is_err());
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use crate::{FetchQuoteData, NativeProvider, Options, QuoteRequest, SwapperError, client_factory::create_eth_client, uniswap};
    use num_bigint::BigUint;
    use num_traits::ToPrimitive;
    use primitives::{
        AssetId, Chain,
        asset_constants::{ARC_EURC_ASSET_ID, ROBINHOOD_USDG_TOKEN_ID, TEMPO_BRIDGED_USDC_ASSET_ID, TEMPO_PATHUSD_ASSET_ID},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_arc_native_usdc_eurc_quotes() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v4(network_provider);
        let options = Options {
            slippage: 100.into(),
            use_max_amount: false,
        };
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
        let options = Options {
            slippage: 100.into(),
            use_max_amount: false,
        };

        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Unichain).into(),
            to_asset: AssetId::from(Chain::Unichain, Some("0x078D782b760474a361dDA0AF3839290b0EF57AD6".to_string())).into(),
            wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
            value: BigUint::from(10000000000000000u64), // 0.01 ETH
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
        let options = Options {
            slippage: 100.into(),
            use_max_amount: false,
        };

        let request = QuoteRequest {
            from_asset: AssetId::from_chain(Chain::Robinhood).into(),
            to_asset: AssetId::from(Chain::Robinhood, Some(ROBINHOOD_USDG_TOKEN_ID.to_string())).into(),
            wallet_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            destination_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            value: BigUint::from(100000000000000u64), // 0.0001 ETH
            options,
        };

        let quote = swap_provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::EstimateGas).await?;

        let estimate_value = format!(
            "0x{:x}",
            quote_data
                .value
                .to_u128()
                .ok_or_else(|| SwapperError::ComputeQuoteError("quote value is too large".to_string()))?
        );
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
        let options = Options {
            slippage: 100.into(),
            use_max_amount: false,
        };
        let pathusd = TEMPO_PATHUSD_ASSET_ID.clone();

        for (from_asset, to_asset) in [(TEMPO_BRIDGED_USDC_ASSET_ID.clone(), pathusd.clone()), (pathusd, TEMPO_BRIDGED_USDC_ASSET_ID.clone())] {
            let request = QuoteRequest {
                from_asset: from_asset.into(),
                to_asset: to_asset.into(),
                wallet_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                destination_address: "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7".into(),
                value: BigUint::from(1000000u64),
                options: options.clone(),
            };
            let quote = swap_provider.get_quote(&request).await?;
            assert!(quote.to_value > BigUint::ZERO);
        }

        Ok(())
    }
}
