use crate::{
    FetchQuoteData, ProviderType, Quote, QuoteRequest, RpcClient, SwapAmountMode, Swapper, SwapperError, SwapperQuoteData,
    alien::RpcProvider,
    approval::check_permit2_approvals,
    client_factory::create_client_with_chain,
    fees::{on_chain_referral_fees, subtract_bps},
    models::*,
    route_cache::Cache,
    uniswap::{
        deadline::get_sig_deadline,
        discovery::{PoolDiscovery, candidate_pairs, discover_pools},
        fee_token::is_quote_input_fee_token,
        quote_result::{OutpricedPaths, QuoteResult, best_path_quote, path_outcomes},
        routed_asset::{Funding, Protocol, base_pair, routed_pair, routed_request},
        solidly,
        swap_route::{PoolPath, RouteData, RouterPath, candidate_paths, get_intermediaries},
        v2::{
            pair::{PairReserves, path_amount_out, reserves_request},
            tax_check::{RoundTrip, is_taxed, suspects_trade_against, tax_suspects},
        },
    },
};
use alloy_primitives::{Address, U256, hex::encode_prefixed as HexEncode};
use async_trait::async_trait;
use gem_evm::{
    constants::DEFAULT_SWAP_GAS_LIMIT,
    jsonrpc::EthereumRpc,
    uniswap::{
        command::{Permit2Permit, encode_commands},
        deployment::{v2::V2Deployment, v3::V3Deployment},
        path::{BasePair, TokenPair, build_pairs},
    },
};
use gem_jsonrpc::{client::JsonRpcClient, types::JsonRpcResult};
use primitives::{AssetId, Chain, EVMChain};
use std::{collections::HashMap, fmt, str::FromStr, sync::Arc};

use super::{
    UniversalRouterProvider,
    commands::build_commands,
    quoter_v2::{build_quoter_request, decode_quoter_response},
};

const PROTOCOL: Protocol = Protocol::V3;
const TAX_CHECKED_PROTOCOLS: [Protocol; 2] = [Protocol::V2, Protocol::Solidly];

type PathOutcome = (PoolPath, Option<U256>);

pub struct UniswapV3 {
    provider: Box<dyn UniversalRouterProvider>,
    rpc_provider: Arc<dyn RpcProvider>,
    pool_discovery: PoolDiscovery,
    v2_pair_discovery: PoolDiscovery,
    solidly_pool_discovery: PoolDiscovery,
    taxed_tokens: Cache<(Chain, Address), bool>,
    outpriced_paths: OutpricedPaths,
}

impl UniswapV3 {
    pub fn new(provider: Box<dyn UniversalRouterProvider>, rpc_provider: Arc<dyn RpcProvider>) -> Self {
        Self {
            provider,
            rpc_provider,
            pool_discovery: PoolDiscovery::default(),
            v2_pair_discovery: PoolDiscovery::default(),
            solidly_pool_discovery: PoolDiscovery::default(),
            taxed_tokens: Cache::default(),
            outpriced_paths: OutpricedPaths::default(),
        }
    }

    pub fn support_chain(&self, chain: &Chain) -> bool {
        self.provider.get_deployment_by_chain(chain).is_some()
    }

    async fn preload_pool_candidates(&self, chain: Chain, token_in: Address, token_out: Address) -> Result<(), SwapperError> {
        let deployment = self.provider.get_deployment_by_chain(&chain).ok_or(SwapperError::NotSupportedChain)?;
        let evm_chain = EVMChain::from_chain(chain).ok_or(SwapperError::NotSupportedChain)?;
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or_else(|| SwapperError::ComputeQuoteError("base pair not found".into()))?;
        let client = create_client_with_chain(self.rpc_provider.clone(), chain)?;
        let fee_tiers = self.provider.get_tiers();
        let pairs = candidate_pairs(token_in, token_out, get_intermediaries(&token_in, &token_out, &base_pair));
        let v3_pools = self
            .pool_discovery
            .missing_pool_requests(chain, &pairs, &fee_tiers, (deployment.factory, deployment.pool_lookup))
            .into_iter()
            .map(|(pool, request)| ((Protocol::V3, pool), request));
        let solidly_pools = deployment.solidly_deployment.iter().flat_map(|solidly_deployment| {
            self.solidly_pool_discovery.missing_pools(chain, &pairs, &solidly::POOL_TYPES).into_iter().map(move |pool| {
                let request = solidly::pool_request(solidly_deployment, &pool);
                ((Protocol::Solidly, pool), request)
            })
        });
        let pools = v3_pools.chain(solidly_pools).collect::<Vec<_>>();
        if pools.is_empty() {
            return Ok(());
        }
        let (solidly_discovered, v3_discovered): (Vec<_>, Vec<_>) = discover_pools(&client, pools).await?.into_iter().partition(|((protocol, _), _)| *protocol == Protocol::Solidly);
        let pool_facts = |discovered: Vec<((Protocol, TokenPair), bool)>| discovered.into_iter().map(|((_, pool), exists)| (pool, exists)).collect::<Vec<_>>();
        self.pool_discovery.record_pools(chain, &pool_facts(v3_discovered));
        self.solidly_pool_discovery.record_pools(chain, &pool_facts(solidly_discovered));
        Ok(())
    }

    fn tax_verdict(&self, chain: Chain, token: Address) -> Option<bool> {
        self.taxed_tokens.get(&(chain, token))
    }

    fn v2_paths(&self, chain: Chain, v2: &V2Deployment, (token_in, token_out): (&Address, &Address), base_pair: &BasePair, suspects: &[Address]) -> Vec<PoolPath> {
        candidate_paths(token_in, token_out, &[v2.fee_tier], base_pair)
            .into_iter()
            .filter(|pairs| suspects_trade_against(pairs, suspects, base_pair.primary) && self.v2_pair_discovery.path_may_exist(chain, pairs))
            .map(|pairs| PoolPath::new(Protocol::V2, pairs))
            .collect()
    }

    fn solidly_paths(&self, chain: Chain, (token_in, token_out): (&Address, &Address), base_pair: &BasePair) -> Vec<PoolPath> {
        candidate_paths(token_in, token_out, &solidly::POOL_TYPES, base_pair)
            .into_iter()
            .filter(|pairs| self.solidly_pool_discovery.path_may_exist(chain, pairs))
            .map(|pairs| PoolPath::new(Protocol::Solidly, pairs))
            .collect()
    }

    async fn quote_paths(
        &self,
        client: &JsonRpcClient<RpcClient>,
        deployment: &V3Deployment,
        request: &QuoteRequest,
        (wrapped, suspects): (Address, &[Address]),
        (from, to): (&(Chain, Address), &(Chain, Address)),
        amount_in: U256,
        paths: Vec<PoolPath>,
    ) -> Result<QuoteResult, SwapperError> {
        let chain = from.0;
        let (v2_paths, paths): (Vec<_>, Vec<_>) = paths.into_iter().partition(|path| path.protocol == Protocol::V2);
        let (solidly_paths, v3_paths): (Vec<_>, Vec<_>) = paths.into_iter().partition(|path| path.protocol == Protocol::Solidly);
        let unchecked = suspects.iter().filter(|token| self.tax_verdict(chain, **token).is_none()).copied().collect::<Vec<_>>();
        let reserve_pairs = v2_paths.iter().flat_map(|path| path.pairs.clone()).collect::<Vec<_>>();

        let quoter_calls = v3_paths
            .iter()
            .map(|path| build_quoter_request(&request.wallet_address, deployment.quoter_v2, amount_in, &build_pairs(&path.pairs)))
            .collect::<Vec<_>>();
        let (solidly_calls, solidly_pool_calls): (Vec<_>, Vec<_>) = match deployment.solidly_deployment.filter(|_| !solidly_paths.is_empty()) {
            Some(solidly_deployment) => (
                solidly_paths.iter().map(|path| solidly::quote_request(&solidly_deployment, &path.pairs, amount_in)).collect(),
                unchecked.iter().flat_map(|token| solidly::volatile_pool_requests(&solidly_deployment, &wrapped, token)).collect(),
            ),
            None => (vec![], vec![]),
        };
        let reserve_calls = deployment.v2_deployment.iter().flat_map(|v2| reserve_pairs.iter().map(|pair| reserves_request(v2, pair))).collect::<Vec<_>>();
        let (quoter_count, solidly_count, solidly_pool_count) = (quoter_calls.len(), solidly_calls.len(), solidly_pool_calls.len());
        let results = client.requests::<String, _>([quoter_calls, solidly_calls, solidly_pool_calls, reserve_calls].concat()).await?.0;
        let (v3_results, results) = results.split_at(quoter_count);
        let (solidly_results, results) = results.split_at(solidly_count);
        let (solidly_pool_results, reserve_results) = results.split_at(solidly_pool_count);

        let quoted = path_outcomes(v3_paths, v3_results, &decode_quoter_response)
            .into_iter()
            .chain(path_outcomes(solidly_paths, solidly_results, &solidly::decode_quote_response));
        let solidly_round_trips = unchecked.iter().zip(solidly_pool_results.chunks(2)).filter_map(|(token, results)| {
            let (reserves, fee) = solidly::decode_volatile_pool(results)?;
            Some((*token, Protocol::Solidly, RoundTrip::solidly(fee, wrapped, *token, &reserves)?))
        });
        let (outcomes, v2_round_trips) = match &deployment.v2_deployment {
            Some(v2) => {
                let reserves = self.record_v2_reserves(chain, v2, &reserve_pairs, reserve_results);
                let v2_outcomes = v2_paths.into_iter().filter_map(|path| path_amount_out(v2, &path.pairs, &reserves, amount_in).map(|amount_out| (path, amount_out)));
                let round_trips = unchecked
                    .iter()
                    .filter_map(|token| Some((*token, Protocol::V2, RoundTrip::v2(v2, wrapped, *token, reserves.get(&v2.pair_address(&wrapped, token))?)?)))
                    .collect::<Vec<_>>();
                (quoted.chain(v2_outcomes).collect::<Vec<_>>(), round_trips)
            }
            None => (quoted.collect(), vec![]),
        };
        let round_trips = v2_round_trips.into_iter().chain(solidly_round_trips).collect::<Vec<_>>();
        let outcomes = self.without_taxed_paths(client, deployment, (chain, &request.wallet_address), suspects, &round_trips, outcomes).await;
        best_path_quote(&self.outpriced_paths, (from, to), amount_in.saturating_to(), outcomes)
    }

    fn record_v2_reserves(&self, chain: Chain, v2: &V2Deployment, pairs: &[TokenPair], results: &[JsonRpcResult<String>]) -> HashMap<Address, PairReserves> {
        let discovered = pairs
            .iter()
            .zip(results)
            .filter_map(|(pair, result)| match result {
                JsonRpcResult::Value(response) => Some((pair.clone(), PairReserves::decode(&response.result).ok()?)),
                JsonRpcResult::Error(_) => None,
            })
            .collect::<Vec<_>>();
        self.v2_pair_discovery.record_pools(chain, &discovered.iter().map(|(pair, reserves)| (pair.clone(), reserves.is_some())).collect::<Vec<_>>());
        discovered.into_iter().filter_map(|(pair, reserves)| Some((v2.pair_address(&pair.token_in, &pair.token_out), reserves?))).collect()
    }

    async fn without_taxed_paths(
        &self,
        client: &JsonRpcClient<RpcClient>,
        deployment: &V3Deployment,
        (chain, wallet): (Chain, &str),
        suspects: &[Address],
        round_trips: &[(Address, Protocol, RoundTrip)],
        outcomes: Vec<PathOutcome>,
    ) -> Vec<PathOutcome> {
        let winner = outcomes
            .iter()
            .filter_map(|(path, amount_out)| Some((path.protocol, (*amount_out)?)))
            .max_by_key(|(_, amount_out)| *amount_out)
            .map(|(protocol, _)| protocol)
            .filter(|protocol| TAX_CHECKED_PROTOCOLS.contains(protocol));
        let Some(winner) = winner else {
            return outcomes;
        };
        let checks = suspects
            .iter()
            .filter(|token| self.tax_verdict(chain, **token).is_none())
            .filter_map(|token| {
                let (_, _, round_trip) = round_trips.iter().filter(|(checked, _, _)| checked == token).max_by_key(|(_, protocol, _)| *protocol == winner)?;
                Some((*token, round_trip.request(deployment.universal_router, deployment.universal_router_abi, wallet)))
            })
            .collect::<Vec<(Address, EthereumRpc)>>();
        if !checks.is_empty() {
            let (tokens, calls): (Vec<_>, Vec<_>) = checks.into_iter().unzip();
            if let Ok(results) = client.requests::<String, _>(calls).await {
                tokens
                    .into_iter()
                    .zip(results.0.iter())
                    .filter_map(|(token, result)| Some((token, is_taxed(result)?)))
                    .for_each(|(token, taxed)| self.taxed_tokens.put((chain, token), taxed));
            }
        }
        match suspects.iter().all(|token| self.tax_verdict(chain, *token) == Some(false)) {
            true => outcomes,
            false => outcomes.into_iter().filter(|(path, _)| !TAX_CHECKED_PROTOCOLS.contains(&path.protocol)).collect(),
        }
    }
}

impl fmt::Debug for UniswapV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UniswapV3").finish()
    }
}

#[async_trait]
impl Swapper for UniswapV3 {
    fn provider(&self) -> &ProviderType {
        self.provider.provider()
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
        let deployment = self.provider.get_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;
        let (evm_chain, input, output, from_value) = routed_request(request, PROTOCOL)?;
        let (token_in, token_out) = (input.address, output.address);

        let client = create_client_with_chain(self.rpc_provider.clone(), from_chain)?;

        let fee_tiers = self.provider.get_tiers();
        let base_pair = base_pair(evm_chain, PROTOCOL).ok_or_else(|| SwapperError::ComputeQuoteError("base pair not found".into()))?;

        let fee_token_is_input = is_quote_input_fee_token(Some(&base_pair), request, token_in, token_out);
        let fee_bps = on_chain_referral_fees(request).evm.bps;

        let quote_amount_in = if fee_token_is_input && fee_bps > 0 { subtract_bps(&from_value, fee_bps) } else { from_value };

        _ = self.preload_pool_candidates(from_chain, token_in, token_out).await;
        let suspects = tax_suspects(&token_in, &token_out, &base_pair);
        let tax_free = suspects.iter().all(|token| self.tax_verdict(from_chain, *token) != Some(true));
        let v3_paths = candidate_paths(&token_in, &token_out, &fee_tiers, &base_pair)
            .into_iter()
            .filter(|pairs| self.pool_discovery.path_may_exist(from_chain, pairs))
            .map(|pairs| PoolPath::new(Protocol::V3, pairs));
        let solidly_paths = deployment.solidly_deployment.iter().filter(|_| tax_free).flat_map(|_| self.solidly_paths(from_chain, (&token_in, &token_out), &base_pair));
        let v2_paths = deployment
            .v2_deployment
            .iter()
            .filter(|_| tax_free)
            .flat_map(|v2| self.v2_paths(from_chain, v2, (&token_in, &token_out), &base_pair, &suspects));
        let paths = v3_paths.chain(solidly_paths).chain(v2_paths).collect();
        let (from, to) = ((from_chain, token_in), (from_chain, token_out));
        let quote_result = self
            .outpriced_paths
            .quote_live_first((&from, &to), quote_amount_in.saturating_to(), paths, PoolPath::clone, |paths| {
                self.quote_paths(&client, &deployment, request, (base_pair.primary, &suspects), (&from, &to), quote_amount_in, paths)
            })
            .await?;

        Ok(quote_result.into_quote(request, self.provider().clone(), fee_token_is_input, output.scale))
    }

    async fn get_quote_data(&self, quote: &Quote, data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let request = &quote.request;
        let from_chain = request.from_asset.chain();
        let (evm_chain, input, output, amount_in) = routed_request(request, PROTOCOL)?;
        let deployment = self.provider.get_deployment_by_chain(&from_chain).ok_or(SwapperError::NotSupportedChain)?;

        let client = create_client_with_chain(self.rpc_provider.clone(), from_chain)?;

        let route_data: RouteData = quote.route_data()?;
        let to_amount = U256::from_str(&route_data.min_amount_out).map_err(SwapperError::from)?;

        let permit = data.permit2_data().map(Permit2Permit::try_from).transpose()?;

        let (approval, permit2) = match input.funding {
            Funding::Token => {
                let spenders = (deployment.permit2, deployment.universal_router);
                check_permit2_approvals(&client, &request.wallet_address, &input.address.to_checksum(None), amount_in, spenders, permit.is_some()).await?
            }
            Funding::Value | Funding::RouterBalance => (None, None),
        };
        let gas_limit = (approval.is_some() || permit2.is_some()).then(|| DEFAULT_SWAP_GAS_LIMIT.to_string());

        let sig_deadline = get_sig_deadline();

        let base_pair = base_pair(evm_chain, PROTOCOL);
        let fee_token_is_input = is_quote_input_fee_token(base_pair.as_ref(), request, input.address, output.address);

        let path = RouterPath::from_routes(route_data.protocol, &quote.data.routes)?;
        let commands = build_commands(request, &input, &output, amount_in, to_amount, &path, permit, fee_token_is_input, deployment.universal_router_abi)?;
        let encoded = encode_commands(&commands, U256::from(sig_deadline));

        let value = input.value(amount_in);

        Ok(SwapperQuoteData {
            permit2,
            ..SwapperQuoteData::new_contract(deployment.universal_router.into(), value, HexEncode(encoded), approval, gas_limit)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SwapperProvider;
    use crate::{
        alien::mock::ProviderMock,
        testkit::mock_quote,
        uniswap::{
            self,
            testkit::{Posts, mock_reserves_result, mock_taxed_token_provider},
        },
    };
    use alloy_primitives::{Bytes, hex::encode as hex_encode};
    use alloy_sol_types::SolCall;
    use chrono::DateTime;
    use gem_evm::rpc::{
        model::{Transaction, TransactionReceipt},
        parsers::ProtocolParsers,
    };
    use gem_evm::uniswap::{
        FeeTier,
        command::{SolidlyRoute, TRANSFER_COMMAND, V2_SWAP_EXACT_IN_COMMAND, V2SwapExactIn, V2SwapExactInSolidly, WRAP_ETH_COMMAND},
        contracts::{
            IUniversalRouter,
            solidly::{ISolidlyPoolFactory, ISolidlyRouter},
        },
        deployment::v3::{get_aerodrome_router_deployment_by_chain, get_pancakeswap_router_deployment_by_chain},
    };
    use num_bigint::BigUint;
    use primitives::TransactionSwapMetadata;
    use primitives::{
        asset_constants::{BASE_USDC_TOKEN_ID, BASE_WETH_TOKEN_ID, SMARTCHAIN_USDC_TOKEN_ID, SMARTCHAIN_USDT_TOKEN_ID},
        decode_hex,
        hex::encode_with_0x,
    };
    use serde_json::json;
    use std::mem::take;

    #[tokio::test]
    async fn test_get_quote_v2_paths() {
        let wrapped: Address = EVMChain::SmartChain.weth_contract().unwrap().parse().unwrap();
        let (clean, taxed) = (Address::repeat_byte(0xee), Address::repeat_byte(0xdd));
        let v2 = get_pancakeswap_router_deployment_by_chain(&Chain::SmartChain).unwrap().v2_deployment.unwrap();
        let pairs = [v2.pair_address(&wrapped, &clean), v2.pair_address(&wrapped, &taxed)].map(|pair| pair.to_checksum(None));
        let reserves = mock_reserves_result(100_000_000_000_000_000_000, 1_000_000_000_000_000_000_000_000);
        let (provider, posts) = mock_taxed_token_provider(taxed, move |to, data| match data.get(..10) {
            Some("0x0902f1ac") if pairs.iter().any(|pair| pair == to) => Some(reserves.clone()),
            Some("0x0902f1ac") => Some("0x".to_string()),
            _ => None,
        });
        let swapper = uniswap::universal_router::new_pancakeswap(Arc::new(provider));

        let quote = assert_tax_checked_quotes(&swapper, &posts, Chain::SmartChain, (clean, taxed), [vec![20, 1, 1], vec![12, 1, 1]]).await;
        let route_data: RouteData = quote.route_data().unwrap();

        assert_eq!(quote.to_value, BigUint::from(9_827_584_990_520_955_699_562u128));
        assert_eq!((route_data.protocol, quote.data.routes.len()), (Protocol::V2, 1));
    }

    #[tokio::test]
    async fn test_get_quote_data_v2_path() {
        let swapper = uniswap::universal_router::new_pancakeswap(Arc::new(ProviderMock::new("{}".to_string())));
        let wrapped: Address = EVMChain::SmartChain.weth_contract().unwrap().parse().unwrap();
        let (usdc, usdt): (Address, Address) = (SMARTCHAIN_USDC_TOKEN_ID.parse().unwrap(), SMARTCHAIN_USDT_TOKEN_ID.parse().unwrap());
        let request = QuoteRequest {
            value: BigUint::from(1_000_000_000_000_000_000u64),
            ..mock_quote(AssetId::from_chain(Chain::SmartChain).into(), AssetId::from_token(Chain::SmartChain, SMARTCHAIN_USDT_TOKEN_ID).into())
        };
        let path = PoolPath::new(Protocol::V2, TokenPair::new_two_hop(&wrapped, &usdc, &usdt, [FeeTier::TwoThousandFiveHundred; 2]));

        let input = assert_wrapped_swap(&swapper, &request, path, SwapperProvider::PancakeswapV3).await;

        assert_eq!(
            V2SwapExactIn::abi_decode(&input).unwrap(),
            V2SwapExactIn {
                recipient: request.wallet_address.parse().unwrap(),
                amount_in: U256::from(995_000_000_000_000_000u64),
                amount_out_min: U256::from(1_990_000u64),
                path: vec![wrapped, usdc, usdt],
                payer_is_user: false,
            }
        );
    }

    #[tokio::test]
    async fn test_get_quote_solidly_paths() {
        let wrapped: Address = BASE_WETH_TOKEN_ID.parse().unwrap();
        let (clean, taxed) = (Address::repeat_byte(0xee), Address::repeat_byte(0xdd));
        let solidly_deployment = get_aerodrome_router_deployment_by_chain(&Chain::Base).unwrap().solidly_deployment.unwrap();
        let pools = [clean, taxed].map(|token| solidly_deployment.pool_address(&wrapped, &token, false).to_checksum(None));
        let volatile_pool = encode_with_0x(&ISolidlyPoolFactory::getPoolCall::abi_encode_returns(&Address::repeat_byte(0x55)));
        let amounts_out = encode_with_0x(&ISolidlyRouter::getAmountsOutCall::abi_encode_returns(&vec![U256::from(995_000_000_000_000_000u64), U256::from(2_380_000_000u64)]));
        let reserves = mock_reserves_result(100_000_000_000_000_000_000, 1_000_000_000_000_000_000_000_000);
        let fee = encode_with_0x(&ISolidlyPoolFactory::getFeeCall::abi_encode_returns(&U256::from(30)));
        let (provider, posts) = mock_taxed_token_provider(taxed, move |to, data| match data.get(..10) {
            Some("0x79bc57d5") if data.contains(&hex_encode(wrapped)) && data.ends_with(&"0".repeat(64)) => Some(volatile_pool.clone()),
            Some("0x5509a1ac") => Some(amounts_out.clone()),
            Some("0x0902f1ac") if pools.iter().any(|pool| pool == to) => Some(reserves.clone()),
            Some("0xcc56b2c5") => Some(fee.clone()),
            _ => None,
        });
        let swapper = uniswap::universal_router::new_aerodrome(Arc::new(provider));

        let quote = assert_tax_checked_quotes(&swapper, &posts, Chain::Base, (clean, taxed), [vec![40, 3, 1], vec![24, 3, 1]]).await;
        let route_data: RouteData = quote.route_data().unwrap();

        assert_eq!(quote.to_value, BigUint::from(2_380_000_000u64));
        assert_eq!((route_data.protocol, route_data.fee_tier.as_str(), quote.data.routes.len()), (Protocol::Solidly, "3000", 1));
    }

    #[tokio::test]
    async fn test_get_quote_data_solidly_path() {
        let swapper = uniswap::universal_router::new_aerodrome(Arc::new(ProviderMock::new("{}".to_string())));
        let (wrapped, usdc, token): (Address, Address, Address) = (BASE_WETH_TOKEN_ID.parse().unwrap(), BASE_USDC_TOKEN_ID.parse().unwrap(), Address::repeat_byte(0xee));
        let request = QuoteRequest {
            value: BigUint::from(1_000_000_000_000_000_000u64),
            ..mock_quote(AssetId::from_chain(Chain::Base).into(), AssetId::from_token(Chain::Base, &token.to_checksum(None)).into())
        };
        let path = PoolPath::new(Protocol::Solidly, TokenPair::new_two_hop(&wrapped, &usdc, &token, [SolidlyRoute::VOLATILE, SolidlyRoute::STABLE]));

        let input = assert_wrapped_swap(&swapper, &request, path, SwapperProvider::Aerodrome).await;

        assert_eq!(
            V2SwapExactInSolidly::abi_decode(&input).unwrap(),
            V2SwapExactInSolidly {
                recipient: request.wallet_address.parse().unwrap(),
                amount_in: U256::from(995_000_000_000_000_000u64),
                amount_out_min: U256::from(1_990_000u64),
                routes: vec![SolidlyRoute { from: wrapped, to: usdc, stable: false }, SolidlyRoute { from: usdc, to: token, stable: true },],
                payer_is_user: false,
            }
        );
    }

    async fn assert_tax_checked_quotes(swapper: &UniswapV3, posts: &Posts, chain: Chain, (clean, taxed): (Address, Address), [clean_posts, taxed_posts]: [Vec<usize>; 2]) -> Quote {
        let clean_request = QuoteRequest {
            value: BigUint::from(1_000_000_000_000_000_000u64),
            ..mock_quote(AssetId::from_chain(chain).into(), AssetId::from_token(chain, &clean.to_checksum(None)).into())
        };
        let taxed_request = QuoteRequest {
            to_asset: AssetId::from_token(chain, &taxed.to_checksum(None)).into(),
            ..clean_request.clone()
        };

        let quote = swapper.get_quote(&clean_request).await.unwrap();
        assert_eq!(take(&mut *posts.lock().unwrap()), clean_posts);

        assert_eq!(swapper.get_quote(&clean_request).await.unwrap().to_value, quote.to_value);
        assert_eq!(take(&mut *posts.lock().unwrap()), vec![1]);

        assert_eq!(swapper.get_quote(&taxed_request).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(take(&mut *posts.lock().unwrap()), taxed_posts);

        assert_eq!(swapper.get_quote(&taxed_request).await, Err(SwapperError::NoQuoteAvailable));
        assert_eq!(take(&mut *posts.lock().unwrap()), Vec::<usize>::new());
        quote
    }

    async fn assert_wrapped_swap(swapper: &UniswapV3, request: &QuoteRequest, path: PoolPath, provider: SwapperProvider) -> Bytes {
        let quote = QuoteResult { amount_out: U256::from(2_000_000u64), path }.into_quote(request, swapper.provider().clone(), true, U256::from(1));
        let quote_data = swapper.get_quote_data(&quote, FetchQuoteData::None).await.unwrap();
        let execute = IUniversalRouter::executeCall::abi_decode(&decode_hex(&quote_data.data).unwrap()).unwrap();
        let transaction: Transaction = serde_json::from_value(json!({
            "from": request.wallet_address,
            "gas": "0x0",
            "hash": "0x01",
            "input": quote_data.data,
            "to": quote_data.to,
            "value": format!("0x{:x}", quote_data.value),
        }))
        .unwrap();
        let parsed = ProtocolParsers::map_transaction(&request.from_asset.chain(), &transaction, &TransactionReceipt::mock(), DateTime::default()).unwrap();

        assert_eq!(execute.commands.to_vec(), vec![WRAP_ETH_COMMAND, TRANSFER_COMMAND, V2_SWAP_EXACT_IN_COMMAND]);
        assert_eq!(
            parsed.swap_metadata().unwrap(),
            TransactionSwapMetadata::new(request.from_asset.asset_id(), request.value.clone(), request.to_asset.asset_id(), BigUint::from(1_990_000u64), provider,)
        );
        execute.inputs[2].clone()
    }

    #[test]
    fn test_robinhood_supported() {
        let provider = Arc::new(ProviderMock::new("{}".to_string()));
        let swapper = uniswap::universal_router::new_uniswap_v3(provider);

        assert!(swapper.support_chain(&Chain::Robinhood));
        assert!(swapper.supported_assets().contains(&SwapperChainAsset::All(Chain::Robinhood)));
    }

    #[test]
    fn test_robinhood_pancakeswap_supported() {
        let provider = Arc::new(ProviderMock::new("{}".to_string()));
        let swapper = uniswap::universal_router::new_pancakeswap(provider);

        assert!(swapper.support_chain(&Chain::Robinhood));
        assert!(swapper.supported_assets().contains(&SwapperChainAsset::All(Chain::Robinhood)));
    }
}

#[cfg(all(test, feature = "swap_integration_tests", feature = "reqwest_provider"))]
mod swap_integration_tests {
    use crate::testkit::{dynode_provider, mock_quote, simulate_swap};
    use crate::{
        FetchQuoteData, NativeProvider, Options, QuoteRequest, Swapper, SwapperError,
        client_factory::{create_client_with_chain, create_eth_client},
        eth_address,
        fees::{bps_of, default_referral_fees},
        uniswap::{
            self,
            routed_asset::{Protocol, base_pair},
            swap_route::RouteData,
            v2::{
                pair::{PairReserves, reserves_request},
                tax_check::RoundTrip,
            },
        },
    };
    use alloy_primitives::{Address, U256};
    use alloy_sol_types::SolCall;
    use gem_evm::uniswap::{
        command::{TRANSFER_COMMAND, Transfer},
        contracts::IUniversalRouter,
        deployment::v3::get_uniswap_router_deployment_by_chain,
        path::TokenPair,
    };
    use num_bigint::BigUint;
    use num_traits::ToPrimitive;
    use primitives::{
        AssetId, Chain, EVMChain,
        asset_constants::{ARC_EURC_ASSET_ID, ARC_USDC_TOKEN_ID, BASE_USDC_TOKEN_ID, ETHEREUM_USDC_ASSET_ID, ETHEREUM_USDC_TOKEN_ID, POLYGON_USDT_TOKEN_ID, ROBINHOOD_USDG_TOKEN_ID, STABLE_TOKEN_ID},
        decode_hex,
        swap::EVM_REFERRAL_ADDRESS,
    };
    use std::sync::Arc;

    const TEST_V2_WALLET: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";
    const TEST_SMARTCHAIN_FOUR_MEME_TOKEN: &str = "0xeCCBb861c0dda7eFd964010085488B69317e4444";
    const TEST_SMARTCHAIN_BABYDOGE_TOKEN: &str = "0xc748673057861a797275CD8A068AbB95A902e8de";
    const TEST_SMARTCHAIN_PAND_TOKEN: &str = "0x9fFde5558d64c253B9344B10C0E772efFDBe2A1a";
    const TEST_SMARTCHAIN_SAFEMOON_TOKEN: &str = "0x42981d0bfbAf196529376EE702F2a9Eb9092fcB5";
    const TEST_SMARTCHAIN_FLOKI_TOKEN: &str = "0xfb5B838b6cfEEdC2873aB27866079AC55363D37E";
    const TEST_ETHEREUM_ECAT_TOKEN: &str = "0x22A866848D506D2B5B7d067dBCc9Be17bf4f11d1";
    const TEST_ETHEREUM_PEPE_TOKEN: &str = "0x6982508145454Ce325dDbE47a25d4ec3d2311933";
    const TEST_ROBINHOOD_ASTRO_TOKEN: &str = "0x5b81efE5E0ba2b31c0Ed843B908a2c45Fe6E14c5";
    const TEST_ROBINHOOD_KEK_TOKEN: &str = "0x86E5D8818eFf06f0dEA9E5eCa87f056Bc44c7777";
    const TEST_BASE_IDRISS_TOKEN: &str = "0x000096630066820566162C94874A776532705231";
    const TEST_BASE_MORPHO_TOKEN: &str = "0xBAa5CC21fd487B8Fcc2F632f3F4E8D37262a0842";

    fn v2_request(from_asset: AssetId, to_asset: AssetId, value: u64) -> QuoteRequest {
        QuoteRequest {
            from_asset: from_asset.into(),
            to_asset: to_asset.into(),
            wallet_address: TEST_V2_WALLET.into(),
            destination_address: TEST_V2_WALLET.into(),
            value: BigUint::from(value),
            options: Options { slippage: 100.into(), use_max_amount: false },
        }
    }

    async fn simulate_native_to_token(chain: Chain, swap_provider: &dyn Swapper, token: &str, value: u64) -> Result<Protocol, SwapperError> {
        let quote = swap_provider.get_quote(&v2_request(AssetId::from_chain(chain), AssetId::from_token(chain, token), value)).await?;
        let protocol = quote.route_data::<RouteData>()?.protocol;
        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        let calldata = decode_hex(&quote_data.data).unwrap();
        let execute = IUniversalRouter::executeCall::abi_decode(&calldata).unwrap();
        let referral = eth_address::parse_str(EVM_REFERRAL_ADDRESS)?;
        let fee = execute.commands.iter().zip(&execute.inputs).find_map(|(command, input)| match *command {
            TRANSFER_COMMAND => Transfer::abi_decode(input).ok().filter(|transfer| transfer.recipient == referral).map(|transfer| transfer.value),
            _ => None,
        });
        assert_eq!(fee, Some(bps_of(&U256::from(value), default_referral_fees().evm.bps)));

        simulate_swap(chain, TEST_V2_WALLET, &quote_data).await?;
        Ok(protocol)
    }

    #[tokio::test]
    async fn test_smartchain_v2_swaps_simulate() -> Result<(), SwapperError> {
        let swap_provider = uniswap::default::boxed_pancakeswap(dynode_provider([Chain::SmartChain]));

        assert_eq!(simulate_native_to_token(Chain::SmartChain, swap_provider.as_ref(), TEST_SMARTCHAIN_PAND_TOKEN, 10_000_000_000_000_000).await?, Protocol::V2);
        simulate_native_to_token(Chain::SmartChain, swap_provider.as_ref(), TEST_SMARTCHAIN_FOUR_MEME_TOKEN, 10_000_000_000_000_000).await?;
        simulate_native_to_token(Chain::SmartChain, swap_provider.as_ref(), TEST_SMARTCHAIN_BABYDOGE_TOKEN, 10_000_000_000_000_000).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_v2_swaps_simulate() -> Result<(), SwapperError> {
        let swap_provider = uniswap::default::boxed_uniswap_v3(dynode_provider([Chain::Ethereum]));

        assert_eq!(simulate_native_to_token(Chain::Ethereum, swap_provider.as_ref(), TEST_ETHEREUM_ECAT_TOKEN, 10_000_000_000_000_000).await?, Protocol::V2);
        simulate_native_to_token(Chain::Ethereum, swap_provider.as_ref(), TEST_ETHEREUM_PEPE_TOKEN, 10_000_000_000_000_000).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_robinhood_v2_swap_simulates() -> Result<(), SwapperError> {
        let swap_provider = uniswap::default::boxed_uniswap_v3(dynode_provider([Chain::Robinhood]));

        assert_eq!(simulate_native_to_token(Chain::Robinhood, swap_provider.as_ref(), TEST_ROBINHOOD_ASTRO_TOKEN, 1_000_000_000_000_000).await?, Protocol::V2);
        Ok(())
    }

    #[tokio::test]
    async fn test_v2_deployments_round_trip_through_the_router() -> Result<(), SwapperError> {
        let cases = [
            (Chain::Ethereum, ETHEREUM_USDC_TOKEN_ID),
            (Chain::Robinhood, ROBINHOOD_USDG_TOKEN_ID),
            (Chain::Polygon, POLYGON_USDT_TOKEN_ID),
            (Chain::Base, BASE_USDC_TOKEN_ID),
        ];
        for (chain, token) in cases {
            let deployment = get_uniswap_router_deployment_by_chain(&chain).unwrap();
            let v2 = deployment.v2_deployment.unwrap();
            let wrapped = base_pair(EVMChain::from_chain(chain).unwrap(), Protocol::V2).unwrap().primary;
            let token: Address = token.parse().unwrap();
            let client = create_client_with_chain(dynode_provider([chain]), chain)?;
            let pair = TokenPair {
                token_in: wrapped,
                token_out: token,
                fee_tier: v2.fee_tier,
            };
            let result = client.request::<String, _>(reserves_request(&v2, &pair)).await.map_err(SwapperError::compute_quote_error)?;
            let reserves = PairReserves::decode(&result)?.unwrap();
            println!("{chain} pair {} reserves {:?}", v2.pair_address(&wrapped, &token), reserves.oriented(&wrapped, &token));

            let round_trip = RoundTrip::v2(&v2, wrapped, token, &reserves).unwrap();
            let request = round_trip.request(deployment.universal_router, deployment.universal_router_abi, TEST_V2_WALLET);
            client.request::<String, _>(request).await.map_err(SwapperError::compute_quote_error)?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_base_solidly_swap_simulates() -> Result<(), SwapperError> {
        let swap_provider = uniswap::default::boxed_aerodrome(dynode_provider([Chain::Base]));

        assert_eq!(simulate_native_to_token(Chain::Base, swap_provider.as_ref(), TEST_BASE_IDRISS_TOKEN, 10_000_000_000_000_000).await?, Protocol::Solidly);
        Ok(())
    }

    #[tokio::test]
    async fn test_base_slipstream_swap_simulates() -> Result<(), SwapperError> {
        let swap_provider = uniswap::default::boxed_aerodrome(dynode_provider([Chain::Base]));

        assert_eq!(simulate_native_to_token(Chain::Base, swap_provider.as_ref(), TEST_BASE_MORPHO_TOKEN, 10_000_000_000_000_000).await?, Protocol::V3);
        Ok(())
    }

    #[tokio::test]
    async fn test_tax_tokens_get_no_v2_quote() -> Result<(), SwapperError> {
        let cases = [
            (Chain::SmartChain, uniswap::default::boxed_pancakeswap(dynode_provider([Chain::SmartChain])), TEST_SMARTCHAIN_SAFEMOON_TOKEN),
            (Chain::SmartChain, uniswap::default::boxed_pancakeswap(dynode_provider([Chain::SmartChain])), TEST_SMARTCHAIN_FLOKI_TOKEN),
            (Chain::Robinhood, uniswap::default::boxed_uniswap_v3(dynode_provider([Chain::Robinhood])), TEST_ROBINHOOD_KEK_TOKEN),
        ];
        for (chain, swap_provider, token) in cases {
            let request = v2_request(AssetId::from_chain(chain), AssetId::from_token(chain, token), 10_000_000_000_000_000);
            for _ in 0..2 {
                let protocol = swap_provider.get_quote(&request).await.and_then(|quote| quote.route_data::<RouteData>()).map(|route_data| route_data.protocol);
                assert_ne!(protocol, Ok(Protocol::V2));
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_ethereum_pancakeswap_eth_to_usdc_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_pancakeswap(network_provider.clone());
        assert_native_to_token_quote(Chain::Ethereum, ETHEREUM_USDC_ASSET_ID.clone(), network_provider, swap_provider).await
    }

    #[tokio::test]
    async fn test_robinhood_eth_to_usdg_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v3(network_provider.clone());
        assert_native_to_token_quote(Chain::Robinhood, AssetId::from(Chain::Robinhood, Some(ROBINHOOD_USDG_TOKEN_ID.to_string())), network_provider, swap_provider).await
    }

    #[tokio::test]
    async fn test_robinhood_pancakeswap_eth_to_usdg_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_pancakeswap(network_provider.clone());
        assert_native_to_token_quote(Chain::Robinhood, AssetId::from(Chain::Robinhood, Some(ROBINHOOD_USDG_TOKEN_ID.to_string())), network_provider, swap_provider).await
    }

    #[tokio::test]
    async fn test_arc_native_usdc_to_eurc_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v3(network_provider);
        let request = QuoteRequest {
            value: BigUint::from(1_000_000_000_000_000_000u64),
            options: Options { slippage: 100.into(), use_max_amount: false },
            ..mock_quote(AssetId::from_chain(Chain::Arc).into(), ARC_EURC_ASSET_ID.clone().into())
        };
        let quote = swap_provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::from(500_000u64) && quote.to_value < BigUint::from(1_000_000u64));
        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert_eq!(quote_data.value, BigUint::from(1_000_000_000_000_000_000u64));
        assert!(quote_data.approval.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_stable_native_usdt0_to_stable_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v3(network_provider);
        let request = QuoteRequest {
            value: BigUint::from(1_000_000_000_000_000_000u64),
            options: Options { slippage: 100.into(), use_max_amount: false },
            ..mock_quote(AssetId::from_chain(Chain::Stable).into(), AssetId::from(Chain::Stable, Some(STABLE_TOKEN_ID.to_string())).into())
        };
        let quote = swap_provider.get_quote(&request).await?;
        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::None).await?;

        assert!(quote.to_value > BigUint::from(10u64.pow(18)));
        assert_eq!(quote_data.value, BigUint::from(1_000_000_000_000_000_000u64));
        assert!(quote_data.approval.is_none());

        let reverse = QuoteRequest {
            from_asset: request.to_asset.clone(),
            to_asset: request.from_asset.clone(),
            value: BigUint::from(100_000_000_000_000_000_000u128),
            ..request
        };
        let quote = swap_provider.get_quote(&reverse).await?;

        assert!(quote.to_value > BigUint::from(10u64.pow(18)) && quote.to_value < BigUint::from(10u64.pow(19)));
        Ok(())
    }

    #[tokio::test]
    async fn test_arc_usdc_to_eurc_quote() -> Result<(), SwapperError> {
        let network_provider = Arc::new(NativeProvider::default());
        let swap_provider = uniswap::default::boxed_uniswap_v3(network_provider);
        let request = QuoteRequest {
            options: Options { slippage: 100.into(), use_max_amount: false },
            ..mock_quote(AssetId::from(Chain::Arc, Some(ARC_USDC_TOKEN_ID.to_string())).into(), ARC_EURC_ASSET_ID.clone().into())
        };
        let quote = swap_provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::from(500_000u64) && quote.to_value < BigUint::from(1_000_000u64));
        Ok(())
    }

    async fn assert_native_to_token_quote(chain: Chain, to_asset: AssetId, network_provider: Arc<NativeProvider>, swap_provider: Box<dyn Swapper>) -> Result<(), SwapperError> {
        let options = Options { slippage: 100.into(), use_max_amount: false };

        let request = QuoteRequest {
            from_asset: AssetId::from_chain(chain).into(),
            to_asset: to_asset.into(),
            wallet_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            destination_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".into(),
            value: BigUint::from(100000000000000u64),
            options,
        };

        let quote = swap_provider.get_quote(&request).await?;
        assert!(quote.to_value > BigUint::ZERO);

        let quote_data = swap_provider.get_quote_data(&quote, FetchQuoteData::EstimateGas).await?;

        let estimate_value = format!("0x{:x}", quote_data.value.to_u128().ok_or_else(|| SwapperError::ComputeQuoteError("quote value is too large".to_string()))?);
        let gas = create_eth_client(network_provider.clone(), chain)?
            .estimate_gas(Some(&request.wallet_address), &quote_data.to, Some(&estimate_value), Some(&quote_data.data))
            .await
            .map_err(|error| SwapperError::ComputeQuoteError(error.to_string()))?;
        assert!(!gas.is_empty());

        Ok(())
    }
}
