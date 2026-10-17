use super::{
    client::StonfiClient,
    constants::{DISCOVERY_ROUTERS, RouterInfo},
    model::{QuotePath, SwapSimulation},
    quote::{DiscoveredPool, PoolData, compute_amount_out, router_model, scaled_next_min_ask_amount, static_candidates, token_address},
    tx_builder::{self, NextSwapParams, ReferralParams, SwapTransactionParams, build_swap_transaction},
};
use crate::{
    FetchQuoteData, ProviderData, ProviderType, Quote, QuoteRequest, Route, RpcClient, RpcProvider, SwapAmountMode, Swapper, SwapperChainAsset, SwapperError, SwapperProvider, SwapperQuoteAsset, SwapperQuoteData,
    fees::{ReferralFee, default_referral_fees, subtract_bps},
    models::nonzero_swap_value,
    route_cache::DiscoveryCache,
    route_path::RoutePath,
};
use async_trait::async_trait;
use futures::future::join_all;
use gem_client::Client;
use gem_ton::{address::Address, models::TonConnectMessage, rpc::client::TonClient};
use num_bigint::BigUint;
use primitives::{AssetId, Chain, asset_constants::TON_USDT_ASSET_ID};
use std::{cmp::Reverse, fmt::Debug, str::FromStr, sync::Arc};

#[derive(Debug)]
pub struct Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    provider: ProviderType,
    client: StonfiClient<C>,
    route_cache: DiscoveryCache<DiscoveredPool, String>,
}

impl Stonfi<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        let endpoint = rpc_provider.get_endpoint(Chain::Ton).ok()?;
        Some(Self::new_with_client(TonClient::new(RpcClient::new(endpoint, rpc_provider))))
    }
}

impl<C> Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    pub fn new_with_client(ton_client: TonClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::StonfiV2),
            client: StonfiClient::new(ton_client),
            route_cache: DiscoveryCache::default(),
        }
    }

    fn route_paths(from_asset: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset) -> Vec<RoutePath<SwapperQuoteAsset>> {
        let intermediaries = if from_asset.is_native() || to_asset.is_native() {
            vec![]
        } else {
            vec![SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)), SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone())]
        };
        RoutePath::all(from_asset, to_asset, intermediaries, |left, right| left.asset_id() == right.asset_id())
    }

    async fn quote_path_via_intermediary(&self, intermediary: &SwapperQuoteAsset, from_value: &str, request: &QuoteRequest, allow_discovery: bool) -> Result<QuotePath, SwapperError> {
        if !self.should_quote_intermediary_path(&request.from_asset, intermediary, &request.to_asset, allow_discovery) {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let to_intermediary = self.quote_swap(&request.from_asset, from_value, intermediary, request.options.slippage.bps, allow_discovery, true).await?;
        if !to_intermediary.router.is_supported_v2() {
            return Err(SwapperError::InvalidRoute);
        }

        let from_intermediary = self.quote_swap(intermediary, &to_intermediary.ask_units, &request.to_asset, request.options.slippage.bps, allow_discovery, true).await?;
        if !from_intermediary.router.is_supported_v2() {
            return Err(SwapperError::InvalidRoute);
        }

        Ok(QuotePath {
            to_value: from_intermediary.ask_units.clone(),
            routes: vec![
                Route {
                    input: request.from_asset.asset_id(),
                    output: intermediary.asset_id(),
                    route_data: serde_json::to_string(&to_intermediary)?,
                },
                Route {
                    input: intermediary.asset_id(),
                    output: request.to_asset.asset_id(),
                    route_data: serde_json::to_string(&from_intermediary)?,
                },
            ],
        })
    }

    async fn quote_direct(&self, request: &QuoteRequest, from_value: &str, allow_discovery: bool) -> Result<QuotePath, SwapperError> {
        let simulation = self.quote_swap(&request.from_asset, from_value, &request.to_asset, request.options.slippage.bps, allow_discovery, false).await?;
        Ok(QuotePath {
            to_value: simulation.ask_units.clone(),
            routes: vec![Route {
                input: request.from_asset.asset_id(),
                output: request.to_asset.asset_id(),
                route_data: serde_json::to_string(&simulation)?,
            }],
        })
    }

    async fn quote_paths(&self, request: &QuoteRequest, from_value: &str, allow_discovery: bool) -> Vec<Result<QuotePath, SwapperError>> {
        join_all(Self::route_paths(&request.from_asset, &request.to_asset).into_iter().map(|path| async move {
            match path {
                RoutePath::Direct => self.quote_direct(request, from_value, allow_discovery).await,
                RoutePath::Intermediary(intermediary) => self.quote_path_via_intermediary(&intermediary, from_value, request, allow_discovery).await,
            }
        }))
        .await
    }

    fn has_known_candidates(&self, from_asset: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset) -> bool {
        let from_token = token_address(from_asset);
        let to_token = token_address(to_asset);
        !self.known_candidates(&from_token, &to_token, true).is_empty()
    }

    fn should_quote_intermediary_path(&self, from_asset: &SwapperQuoteAsset, intermediary: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset, allow_discovery: bool) -> bool {
        allow_discovery || (self.has_known_candidates(from_asset, intermediary) && self.has_known_candidates(intermediary, to_asset))
    }

    fn referral_fee() -> ReferralFee {
        default_referral_fees().ton
    }

    async fn quote_swap(&self, from_asset: &SwapperQuoteAsset, from_value: &str, to_asset: &SwapperQuoteAsset, slippage_bps: u32, allow_discovery: bool, require_v2: bool) -> Result<SwapSimulation, SwapperError> {
        let from_token = token_address(from_asset);
        let to_token = token_address(to_asset);
        let amount = BigUint::from_str(from_value)?;
        nonzero_swap_value(&amount)?;

        match self.quote_best_candidate(self.known_candidates(&from_token, &to_token, require_v2), &[], &from_token, &to_token, &amount, slippage_bps).await {
            Ok((_, simulation)) => return Ok(simulation),
            Err(error) if is_retryable_get_method_error(&error) => return Err(error),
            Err(_) => {}
        }

        if !allow_discovery {
            return Err(SwapperError::NoQuoteAvailable);
        }

        let pool_data = self.discover_candidates(&from_token, &to_token, require_v2).await?;
        self.quote_best_candidate(self.known_candidates(&from_token, &to_token, require_v2), &pool_data, &from_token, &to_token, &amount, slippage_bps)
            .await
            .map(|(_, simulation)| simulation)
    }

    fn missing_probes(&self, from_token: &str, to_token: &str, require_v2: bool) -> Vec<String> {
        let static_probes = filter_candidates(static_candidates(from_token, to_token), require_v2).into_iter().map(|candidate| candidate.router.address).collect::<Vec<_>>();
        self.route_cache
            .missing_probes(from_token, to_token, &eligible_probes(require_v2))
            .into_iter()
            .filter(|probe| !static_probes.contains(probe))
            .collect()
    }

    fn route_discovery_complete(&self, from_asset: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset) -> bool {
        Self::discovery_pairs(from_asset, to_asset).iter().all(|(from, to, require_v2)| self.missing_probes(from, to, *require_v2).is_empty())
    }

    fn discovery_pairs(from_asset: &SwapperQuoteAsset, to_asset: &SwapperQuoteAsset) -> Vec<(String, String, bool)> {
        Self::route_paths(from_asset, to_asset)
            .into_iter()
            .flat_map(|path| {
                let require_v2 = match path {
                    RoutePath::Direct => false,
                    RoutePath::Intermediary(_) => true,
                };
                path.pairs(from_asset, to_asset).into_iter().map(move |(from, to)| (token_address(&from), token_address(&to), require_v2))
            })
            .collect()
    }

    async fn discover_candidates(&self, from_token: &str, to_token: &str, require_v2: bool) -> Result<Vec<(String, PoolData)>, SwapperError> {
        let probes = eligible_probes(require_v2);
        let missing = self.missing_probes(from_token, to_token, require_v2);
        let discoveries = join_all(
            DISCOVERY_ROUTERS
                .iter()
                .filter(|router| missing.iter().any(|address| address == router.address))
                .map(|router| async move { (router.address.to_string(), self.discover_candidate(from_token, to_token, router).await) }),
        )
        .await;
        let error = discoveries
            .iter()
            .filter_map(|(_, discovery)| discovery.as_ref().err())
            .find(|error| is_retryable_get_method_error(error))
            .or_else(|| discoveries.iter().filter_map(|(_, discovery)| discovery.as_ref().err()).next_back())
            .cloned()
            .unwrap_or(SwapperError::NoQuoteAvailable);
        let pool_data = discoveries
            .iter()
            .filter_map(|(_, result)| result.as_ref().ok())
            .map(|(candidate, pool_data)| (candidate.pool_address.clone(), pool_data.clone()))
            .collect::<Vec<_>>();
        self.route_cache.record_discovery(
            from_token,
            to_token,
            discoveries.into_iter().filter_map(|(probe, result)| match result {
                Ok((candidate, _)) => Some((probe, Some(candidate))),
                Err(SwapperError::NoQuoteAvailable) => Some((probe, None)),
                Err(_) => None,
            }),
        );
        let candidates = self.route_cache.candidates_for_probes(from_token, to_token, &probes);
        if candidates.is_empty() { Err(error) } else { Ok(pool_data) }
    }

    async fn preload_pair(&self, from_token: &str, to_token: &str, require_v2: bool) {
        _ = self.discover_candidates(from_token, to_token, require_v2).await;
    }

    fn known_candidates(&self, from_token: &str, to_token: &str, require_v2: bool) -> Vec<DiscoveredPool> {
        let static_pools = filter_candidates(static_candidates(from_token, to_token), require_v2);
        self.route_cache
            .candidates_with(from_token, to_token, &eligible_probes(require_v2), static_pools, |known, cached| known.pool_address == cached.pool_address)
    }

    async fn discover_candidate(&self, from_token: &str, to_token: &str, router: &RouterInfo) -> Result<(DiscoveredPool, PoolData), SwapperError> {
        let (wallet0, wallet1) = futures::try_join!(self.client.router_jetton_wallet(router, from_token), self.client.router_jetton_wallet(router, to_token))?;
        let pool_address = self.client.get_pool_address(router, &wallet0, &wallet1).await?;
        let pool_data = self.client.get_pool_data(&pool_address).await?;
        Ok((
            DiscoveredPool {
                pool_address,
                router: router_model(router),
                asset0: from_token.to_string(),
                asset1: to_token.to_string(),
                wallet0,
                wallet1,
                lp_fee_bps: None,
            },
            pool_data,
        ))
    }

    async fn quote_best_candidate(&self, candidates: Vec<DiscoveredPool>, pool_data: &[(String, PoolData)], from_token: &str, to_token: &str, amount: &BigUint, slippage_bps: u32) -> Result<(DiscoveredPool, SwapSimulation), SwapperError> {
        let quotes = join_all(candidates.into_iter().map(|candidate| {
            let pool_data = pool_data.iter().find(|(address, _)| address == &candidate.pool_address).map(|(_, data)| data);
            self.quote_candidate(candidate, pool_data, from_token, to_token, amount, slippage_bps)
        }))
        .await;
        if let Some(error) = quotes.iter().filter_map(|quote| quote.as_ref().err()).find(|error| is_retryable_get_method_error(error)) {
            return Err(error.clone());
        }
        let ranked = quotes.into_iter().flatten().map(|quote| Ok((BigUint::from_str(&quote.1.ask_units)?, quote))).collect::<Result<Vec<_>, SwapperError>>()?;
        ranked.into_iter().min_by_key(|(amount, _)| Reverse(amount.clone())).map(|(_, quote)| quote).ok_or(SwapperError::NoQuoteAvailable)
    }

    async fn quote_candidate(&self, candidate: DiscoveredPool, pool_data: Option<&PoolData>, from_token: &str, to_token: &str, amount: &BigUint, slippage_bps: u32) -> Result<(DiscoveredPool, SwapSimulation), SwapperError> {
        let fetched_pool_data;
        let pool_data = match pool_data {
            Some(pool_data) => pool_data,
            None => {
                fetched_pool_data = self.client.get_pool_data(&candidate.pool_address).await?;
                &fetched_pool_data
            }
        };
        if pool_data.is_locked {
            self.route_cache.record_unusable(from_token, to_token, [candidate.router.address.clone()]);
            return Err(SwapperError::NoQuoteAvailable);
        }
        if let Some(lp_fee_bps) = candidate.lp_fee_bps
            && pool_data.lp_fee != lp_fee_bps
        {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let offer_wallet = candidate.wallet_for(from_token).ok_or(SwapperError::InvalidRoute)?;
        let ask_wallet = candidate.wallet_for(to_token).ok_or(SwapperError::InvalidRoute)?;
        let ask_units = compute_amount_out(pool_data, offer_wallet, amount)?;
        if ask_units == BigUint::from(0u8) {
            return Err(SwapperError::NoQuoteAvailable);
        }
        let min_ask_units = subtract_bps(&ask_units, slippage_bps);
        let simulation = SwapSimulation {
            offer_jetton_wallet: offer_wallet.to_string(),
            ask_jetton_wallet: ask_wallet.to_string(),
            router: candidate.router.clone(),
            ask_units: ask_units.to_string(),
            min_ask_units: min_ask_units.to_string(),
        };
        Ok((candidate, simulation))
    }

    async fn get_quotes(&self, request: &QuoteRequest, from_value: &str) -> Result<QuotePath, SwapperError> {
        let paths = self.quote_paths(request, from_value, false).await;
        if let Some(err) = retryable_path_error(paths.iter()) {
            return Err(err);
        }
        let best = Self::select_best_quote_path(paths);
        if best.is_ok() || self.route_discovery_complete(&request.from_asset, &request.to_asset) {
            return best;
        }

        let paths = self.quote_paths(request, from_value, true).await;
        if let Some(err) = retryable_path_error(paths.iter()) {
            return Err(err);
        }
        Self::select_best_quote_path(paths)
    }

    fn select_best_quote_path(paths: impl IntoIterator<Item = Result<QuotePath, SwapperError>>) -> Result<QuotePath, SwapperError> {
        let (quoted, failed): (Vec<_>, Vec<_>) = paths.into_iter().partition(Result::is_ok);
        let ranked = quoted.into_iter().flatten().map(|path| Ok((BigUint::from_str(&path.to_value)?, path))).collect::<Result<Vec<_>, SwapperError>>()?;
        match ranked.into_iter().min_by_key(|(amount, _)| Reverse(amount.clone())) {
            Some((_, path)) => Ok(path),
            None => Err(failed.into_iter().find_map(Result::err).unwrap_or(SwapperError::NoQuoteAvailable)),
        }
    }
}

#[async_trait]
impl<C> Swapper for Stonfi<C>
where
    C: Client + Clone + Send + Sync + Debug + 'static,
{
    fn provider(&self) -> &ProviderType {
        &self.provider
    }

    fn supported_assets(&self) -> Vec<SwapperChainAsset> {
        vec![SwapperChainAsset::All(Chain::Ton)]
    }

    fn amount_mode(&self, _request: &QuoteRequest) -> SwapAmountMode {
        SwapAmountMode::Fixed
    }

    async fn preload_routes(&self, from_asset: &AssetId, to_asset: &AssetId) {
        if from_asset.chain != Chain::Ton || to_asset.chain != Chain::Ton {
            return;
        }
        let pairs = Self::discovery_pairs(&SwapperQuoteAsset::from(from_asset.clone()), &SwapperQuoteAsset::from(to_asset.clone()));
        join_all(pairs.iter().map(|(from, to, require_v2)| self.preload_pair(from, to, *require_v2))).await;
    }

    async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote, SwapperError> {
        let request = request.reserve_native_attachment(&tx_builder::native_attachment())?;
        let from_value = request.value.clone();
        let path = self.get_quotes(&request, &from_value.to_string()).await?;

        Ok(Quote {
            from_value,
            min_from_value: None,
            to_value: BigUint::from_str(&path.to_value).map_err(SwapperError::compute_quote_error)?,
            data: ProviderData {
                provider: self.provider().clone(),
                routes: path.routes,
                slippage_bps: request.options.slippage.bps,
            },
            request,
            eta_in_seconds: None,
        })
    }

    async fn get_quote_data(&self, quote: &Quote, _data: FetchQuoteData) -> Result<SwapperQuoteData, SwapperError> {
        let simulations = quote
            .data
            .routes
            .iter()
            .map(|route| serde_json::from_str::<SwapSimulation>(&route.route_data).map_err(|_| SwapperError::InvalidRoute))
            .collect::<Result<Vec<_>, _>>()?;
        if simulations.len() > 2 {
            return Err(SwapperError::InvalidRoute);
        }
        let simulation = simulations.first().ok_or(SwapperError::InvalidRoute)?;
        let next_swap = simulations
            .get(1)
            .map(|next_simulation| {
                Ok::<_, SwapperError>(NextSwapParams {
                    simulation: next_simulation,
                    min_ask_amount: scaled_next_min_ask_amount(simulation, next_simulation)?,
                })
            })
            .transpose()?;
        let referral_fee = Self::referral_fee();
        let receiver_address = if quote.request.destination_address.is_empty() {
            &quote.request.wallet_address
        } else {
            &quote.request.destination_address
        };
        let sender_jetton_wallet = self.client.sender_jetton_wallet(quote).await?;

        let tx = build_swap_transaction(SwapTransactionParams {
            simulation,
            next_swap,
            from_native: quote.request.from_asset.is_native(),
            to_native: quote.request.to_asset.is_native(),
            sender_jetton_wallet: sender_jetton_wallet.as_deref(),
            from_value: &quote.from_value.to_string(),
            wallet_address: Address::parse(&quote.request.wallet_address)?,
            receiver_address: Address::parse(receiver_address)?,
            referral: ReferralParams {
                address: Address::parse(&referral_fee.address)?,
                bps: referral_fee.bps,
            },
            deadline: None,
        })?;

        let message = TonConnectMessage::new(Address::parse(&tx.to)?.encode_bounceable(), &BigUint::from_str(&tx.value)?, Some(tx.data));
        TonConnectMessage::swap_quote_data(&[message]).map_err(SwapperError::compute_quote_error)
    }
}

fn is_retryable_get_method_error(err: &SwapperError) -> bool {
    match err {
        SwapperError::ComputeQuoteError(message) => {
            let message = message.to_ascii_lowercase();
            message.contains("ratelimit") || message.contains("rate limit") || message.contains("429") || message.contains("too many requests")
        }
        _ => false,
    }
}

fn retryable_path_error<'a>(paths: impl IntoIterator<Item = &'a Result<QuotePath, SwapperError>>) -> Option<SwapperError> {
    paths.into_iter().find_map(|path| match path {
        Err(error) if is_retryable_get_method_error(error) => Some(error.clone()),
        Ok(_) | Err(_) => None,
    })
}

fn filter_candidates(candidates: Vec<DiscoveredPool>, require_v2: bool) -> Vec<DiscoveredPool> {
    if require_v2 {
        candidates.into_iter().filter(|candidate| candidate.router.is_supported_v2()).collect()
    } else {
        candidates
    }
}

fn eligible_probes(require_v2: bool) -> Vec<String> {
    DISCOVERY_ROUTERS.iter().filter(|router| !require_v2 || router.is_supported_v2()).map(|router| router.address.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::super::{
        constants::STATIC_POOLS,
        testkit::{TEST_PTON_WALLET, TEST_USDT_WALLET, mock_cell_response, mock_pool_data_response, mock_v1_pool_data_response},
    };
    use super::*;
    use crate::Options;
    use gem_ton::constants::TON_PROXY_JETTON_ADDRESS;
    use primitives::{asset_constants::TON_USDT_TOKEN_ID, testkit::signer_mock::TEST_TON_SENDER};
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    const GRAM_TOKEN_ID: &str = "EQC47093oX5Xhb0xuk2lCr2RhS8rj-vul61u4W2UH5ORmG_O";
    const DISCOVERED_POOL: &str = "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4";
    const V1_POOL: &str = "EQD8TJ8xEWB1SpnRE4d89YO3jl0W0EiBnNS4IBaHaUmdfizE";

    #[tokio::test]
    async fn test_preload_discovers_once() {
        let calls = Arc::new(Mutex::new(Vec::<String>::new()));
        let calls_ref = calls.clone();
        let provider = Stonfi::mock(move |method, _| {
            calls_ref.lock().unwrap().push(method.to_string());
            match method {
                "get_wallet_address" => mock_cell_response(TEST_USDT_WALLET),
                "get_pool_address" => mock_cell_response(DISCOVERED_POOL),
                "get_pool_data" => mock_pool_data_response(false, 1, 1, TEST_USDT_WALLET, TEST_PTON_WALLET, 7),
                _ => unreachable!("{method}"),
            }
        });

        provider.preload_pair("unknown-a", "unknown-b", false).await;
        assert_eq!(calls.lock().unwrap().iter().filter(|method| method.as_str() == "get_pool_data").count(), DISCOVERY_ROUTERS.len());

        calls.lock().unwrap().clear();
        provider.preload_pair("unknown-a", "unknown-b", false).await;
        assert!(calls.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_failed_pool_validation_is_retried() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_ref = calls.clone();
        let provider = Stonfi::mock(move |method, _| {
            calls_ref.fetch_add(1, Ordering::Relaxed);
            match method {
                "get_wallet_address" => mock_cell_response(TEST_USDT_WALLET),
                "get_pool_address" => mock_cell_response(DISCOVERED_POOL),
                "get_pool_data" => br#"{"error":"node unavailable"}"#.to_vec(),
                _ => unreachable!("{method}"),
            }
        });

        provider.preload_pair("unknown-a", "unknown-b", false).await;
        let first_attempt_calls = calls.load(Ordering::Relaxed);
        assert_eq!(first_attempt_calls, DISCOVERY_ROUTERS.len() * 4);
        let probes = eligible_probes(false);
        assert_eq!(provider.route_cache.missing_probes("unknown-a", "unknown-b", &probes), probes);

        provider.preload_pair("unknown-a", "unknown-b", false).await;
        assert_eq!(calls.load(Ordering::Relaxed), first_attempt_calls + DISCOVERY_ROUTERS.len() * 2);
    }

    #[tokio::test]
    async fn test_native_preload_discovers_direct_pair_only() {
        let calls = Arc::new(Mutex::new(Vec::<String>::new()));
        let calls_ref = calls.clone();
        let provider = Stonfi::mock(move |method, address| {
            calls_ref.lock().unwrap().push(method.to_string());
            match method {
                "get_wallet_address" => mock_cell_response(TEST_USDT_WALLET),
                "get_pool_address" => mock_cell_response(DISCOVERED_POOL),
                "get_pool_data" => mock_pool_data_response(false, 1, 1, TEST_USDT_WALLET, TEST_PTON_WALLET, 7),
                _ => unreachable!("{method} {address}"),
            }
        });
        let ton = AssetId::from_chain(Chain::Ton);
        let token = AssetId::from_token(Chain::Ton, "unknown-token");

        provider.preload_routes(&ton, &token).await;

        let calls = calls.lock().unwrap();
        assert_eq!(calls.iter().filter(|method| method.as_str() == "get_wallet_address").count(), DISCOVERY_ROUTERS.len());
        assert_eq!(calls.iter().filter(|method| method.as_str() == "get_pool_address").count(), DISCOVERY_ROUTERS.len());
        assert_eq!(calls.iter().filter(|method| method.as_str() == "get_pool_data").count(), DISCOVERY_ROUTERS.len());
        assert_eq!(calls.len(), DISCOVERY_ROUTERS.len() * 3);
    }

    #[tokio::test]
    async fn test_failed_quote_skips_complete_discovery() {
        let pool_data_calls = Arc::new(AtomicUsize::new(0));
        let pool_data_calls_ref = pool_data_calls.clone();
        let provider = Stonfi::mock(move |method, address| match method {
            "get_wallet_address" => mock_cell_response(TEST_USDT_WALLET),
            "get_pool_address" => mock_cell_response(DISCOVERED_POOL),
            "get_pool_data" => {
                pool_data_calls_ref.fetch_add(1, Ordering::Relaxed);
                mock_pool_data_response(true, 1, 1, TEST_USDT_WALLET, TEST_PTON_WALLET, 7)
            }
            _ => unreachable!("{method} {address}"),
        });
        let ton = AssetId::from_chain(Chain::Ton);
        let token = AssetId::from_token(Chain::Ton, "unknown-token");
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(ton.clone()),
            to_asset: SwapperQuoteAsset::from(token.clone()),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: BigUint::from(1_000_000_000u64),
            options: Options::new_with_slippage(100.into()),
        };

        provider.preload_routes(&ton, &token).await;
        pool_data_calls.store(0, Ordering::Relaxed);

        assert_eq!(provider.get_quote(&request).await.unwrap_err(), SwapperError::NoQuoteAvailable);
        assert_eq!(pool_data_calls.load(Ordering::Relaxed), DISCOVERY_ROUTERS.len());
    }

    #[tokio::test]
    async fn test_preload_retries_missing_router_after_partial_success() {
        let calls = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let calls_ref = calls.clone();
        let v1_attempts = Arc::new(AtomicUsize::new(0));
        let v1_attempts_ref = v1_attempts.clone();
        let provider = Stonfi::mock(move |method, address| {
            calls_ref.lock().unwrap().push((method.to_string(), address.to_string()));
            match method {
                "get_wallet_address" => mock_cell_response(TEST_USDT_WALLET),
                "get_pool_address" if address == DISCOVERY_ROUTERS[0].address => mock_cell_response(DISCOVERED_POOL),
                "get_pool_address" if address == DISCOVERY_ROUTERS[1].address && v1_attempts_ref.fetch_add(1, Ordering::Relaxed) == 0 => br#"{"exit_code":1,"stack":[]}"#.to_vec(),
                "get_pool_address" if address == DISCOVERY_ROUTERS[1].address => mock_cell_response(V1_POOL),
                "get_pool_address" => mock_cell_response(DISCOVERED_POOL),
                "get_pool_data" => mock_pool_data_response(false, 1, 1, TEST_USDT_WALLET, TEST_PTON_WALLET, 7),
                _ => unreachable!("{method} {address}"),
            }
        });

        provider.preload_pair("unknown-a", "unknown-b", false).await;
        let probes = eligible_probes(false);
        assert_eq!(provider.route_cache.candidates_for_probes("unknown-a", "unknown-b", std::slice::from_ref(&probes[0])).len(), 1);
        assert_eq!(provider.route_cache.candidates_for_probes("unknown-a", "unknown-b", std::slice::from_ref(&probes[1])).len(), 0);

        calls.lock().unwrap().clear();
        provider.preload_pair("unknown-a", "unknown-b", false).await;
        assert_eq!(
            calls.lock().unwrap().iter().filter(|(method, _)| method == "get_pool_address").cloned().collect::<Vec<_>>(),
            vec![("get_pool_address".to_string(), DISCOVERY_ROUTERS[1].address.to_string())]
        );
        assert_eq!(provider.route_cache.candidates_for_probes("unknown-a", "unknown-b", &probes).len(), DISCOVERY_ROUTERS.len());
    }

    #[tokio::test]
    async fn test_preload_caches_definitive_missing_pool() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_ref = calls.clone();
        let provider = Stonfi::mock(move |method, _| {
            calls_ref.fetch_add(1, Ordering::Relaxed);
            match method {
                "get_wallet_address" => mock_cell_response(TEST_USDT_WALLET),
                "get_pool_address" => mock_cell_response(DISCOVERED_POOL),
                "get_pool_data" => br#"{"exit_code":1,"stack":[]}"#.to_vec(),
                _ => unreachable!("{method}"),
            }
        });

        provider.preload_pair("unknown-a", "unknown-b", false).await;
        let first_attempt_calls = calls.load(Ordering::Relaxed);
        let probes = eligible_probes(false);
        assert_eq!(provider.route_cache.missing_probes("unknown-a", "unknown-b", &probes).len(), 0);

        provider.preload_pair("unknown-a", "unknown-b", false).await;
        assert_eq!(calls.load(Ordering::Relaxed), first_attempt_calls);
    }

    #[tokio::test]
    async fn test_quote_candidate_rejects_locked_pool() {
        let provider = Stonfi::mock(|_, _| mock_pool_data_response(true, 3_809_436_784_065, 1_784_561_670_122_756, TEST_USDT_WALLET, TEST_PTON_WALLET, 7));
        let amount = BigUint::from(1_000_000_000u64);

        let pool = DiscoveredPool::mock("pool-a");
        let probes = [pool.router.address.clone()];

        assert_eq!(
            provider.quote_candidate(pool, None, TON_PROXY_JETTON_ADDRESS, TON_USDT_TOKEN_ID, &amount, 100).await.unwrap_err(),
            SwapperError::NoQuoteAvailable
        );
        assert!(provider.route_cache.missing_probes(TON_PROXY_JETTON_ADDRESS, TON_USDT_TOKEN_ID, &probes).is_empty());
        assert!(provider.route_cache.candidates_for_probes(TON_PROXY_JETTON_ADDRESS, TON_USDT_TOKEN_ID, &probes).is_empty());
    }

    #[tokio::test]
    async fn test_direct_quote_can_select_v1_static_pool() {
        let v1_pool = STATIC_POOLS.iter().find(|pool| pool.pool_address == V1_POOL).unwrap();
        let provider = Stonfi::mock(move |_, address| match address {
            DISCOVERED_POOL => mock_pool_data_response(false, 1_000_000_000_000, 1, TEST_PTON_WALLET, TEST_USDT_WALLET, 7),
            V1_POOL => mock_v1_pool_data_response(1_000_000_000, 10_000_000_000, v1_pool.token0_wallet, v1_pool.token1_wallet, 20),
            _ => unreachable!("{address}"),
        });
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: BigUint::from(100000000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let path = provider.quote_direct(&request, &request.value.to_string(), false).await.unwrap();
        let simulation: SwapSimulation = serde_json::from_str(&path.routes[0].route_data).unwrap();

        assert_eq!(path.routes.len(), 1);
        assert_eq!(simulation.router.major_version, 1);
        assert_eq!(simulation.offer_jetton_wallet, v1_pool.token0_wallet);
        assert_eq!(simulation.ask_jetton_wallet, v1_pool.token1_wallet);
    }

    #[tokio::test]
    async fn test_discovered_direct_quote_selects_best_router_pool() {
        let calls = Arc::new(Mutex::new(Vec::<String>::new()));
        let calls_ref = calls.clone();
        let provider = Stonfi::mock(move |method, address| {
            calls_ref.lock().unwrap().push(format!("{method} {address}"));
            match method {
                "get_wallet_address" if address == TON_USDT_TOKEN_ID => mock_cell_response(TEST_USDT_WALLET),
                "get_wallet_address" if address == GRAM_TOKEN_ID => mock_cell_response(TEST_PTON_WALLET),
                "get_pool_address" if address == DISCOVERY_ROUTERS[0].address => mock_cell_response(DISCOVERED_POOL),
                "get_pool_address" if address == DISCOVERY_ROUTERS[1].address => mock_cell_response(V1_POOL),
                "get_pool_address" => mock_cell_response(DISCOVERED_POOL),
                "get_pool_data" if address == DISCOVERED_POOL => mock_pool_data_response(false, 1, 19_811_277, TEST_USDT_WALLET, TEST_PTON_WALLET, 20),
                "get_pool_data" if address == V1_POOL => mock_v1_pool_data_response(226_348_366, 194_933_327_038_860, TEST_USDT_WALLET, TEST_PTON_WALLET, 20),
                _ => unreachable!("{method} {address}"),
            }
        });
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID)),
            to_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, GRAM_TOKEN_ID)),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: BigUint::from(233000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let path = provider.quote_direct(&request, &request.value.to_string(), true).await.unwrap();
        let simulation: SwapSimulation = serde_json::from_str(&path.routes[0].route_data).unwrap();
        let cold_calls = calls.lock().unwrap().clone();
        let pool_call_count = cold_calls.iter().filter(|call| call.starts_with("get_pool_data ")).count();
        let discovered_pool_call = format!("get_pool_data {DISCOVERED_POOL}");
        let v1_pool_call = format!("get_pool_data {V1_POOL}");

        assert_eq!(pool_call_count, DISCOVERY_ROUTERS.len());
        assert_eq!(cold_calls.iter().filter(|call| *call == &discovered_pool_call).count(), DISCOVERY_ROUTERS.len() - 1);
        assert_eq!(cold_calls.iter().filter(|call| *call == &v1_pool_call).count(), 1);
        assert_eq!(simulation.router.major_version, 1);
        assert_eq!(simulation.ask_units, "199854680472");

        calls.lock().unwrap().clear();
        provider.quote_direct(&request, &request.value.to_string(), true).await.unwrap();
        let warm_calls = calls.lock().unwrap().clone();
        let warm_pool_call_count = warm_calls.iter().filter(|call| call.starts_with("get_pool_data ")).count();
        assert_eq!(warm_pool_call_count, DISCOVERY_ROUTERS.len());
        assert_eq!(warm_calls.iter().filter(|call| *call == &discovered_pool_call).count(), DISCOVERY_ROUTERS.len() - 1);
        assert_eq!(warm_calls.iter().filter(|call| *call == &v1_pool_call).count(), 1);
    }

    #[tokio::test]
    async fn test_quote_best_candidate_selects_largest_output() {
        let provider = Stonfi::mock(|_, address| match address {
            "pool-a" => mock_pool_data_response(false, 3_000_000_000_000, 1_800_000_000_000_000, TEST_USDT_WALLET, TEST_PTON_WALLET, 7),
            "pool-b" => mock_pool_data_response(false, 4_000_000_000_000, 1_800_000_000_000_000, TEST_USDT_WALLET, TEST_PTON_WALLET, 7),
            _ => unreachable!(),
        });
        let amount = BigUint::from(1_000_000_000u64);
        let (pool, simulation) = provider
            .quote_best_candidate(vec![DiscoveredPool::mock("pool-a"), DiscoveredPool::mock("pool-b")], &[], TON_PROXY_JETTON_ADDRESS, TON_USDT_TOKEN_ID, &amount, 100)
            .await
            .unwrap();

        assert_eq!(pool.pool_address, "pool-b");
        assert_eq!(simulation.ask_units, "2219998");
    }

    #[test]
    fn test_intermediary_discovery_policy() {
        let provider = Stonfi::mock(|_, _| unreachable!());
        let unknown_a = SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, "unknown-a"));
        let unknown_b = SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, "unknown-b"));
        let ton = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton));
        let usdt = SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone());

        assert!(provider.should_quote_intermediary_path(&unknown_a, &ton, &unknown_b, true));
        assert!(provider.should_quote_intermediary_path(&unknown_a, &ton, &usdt, true));
        assert!(!provider.should_quote_intermediary_path(&unknown_a, &ton, &usdt, false));
    }
}

#[cfg(all(test, feature = "swap_integration_tests"))]
mod swap_integration_tests {
    use super::*;
    use crate::{Options, SwapperQuoteAsset, alien::reqwest_provider::NativeProvider, stonfi::testkit::NOT_TOKEN_ID, testkit::mock_ton};
    use primitives::{AssetId, asset_constants::TON_USDT_ASSET_ID, testkit::signer_mock::TEST_TON_SENDER};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_stonfi_quote_and_quote_data_ton_to_usdt() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Stonfi::new(rpc_provider).unwrap();
        let request = mock_ton(TEST_TON_SENDER.to_string());

        let quote = provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 1);
        println!("STON.fi TON -> USDT quote: {quote:?}");

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());
        assert!(quote_data.value > BigUint::ZERO);
        assert_eq!(serde_json::from_str::<Vec<TonConnectMessage>>(&quote_data.data)?.len(), 1);
        println!("STON.fi TON -> USDT quote_data: {quote_data:?}");

        Ok(())
    }

    #[tokio::test]
    async fn test_stonfi_quote_and_quote_data_hmstr_native() -> Result<(), SwapperError> {
        let rpc_provider = NativeProvider::new_with_endpoints(HashMap::from([(Chain::Ton, "https://gemnodes.com/ton".to_string())]));
        let provider = Stonfi::new(Arc::new(rpc_provider)).unwrap();
        let hmstr = SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, "EQAJ8uWd7EBqsmpSWaRdf_I-8R8-XHwh3gsNKhy-UrdrPcUo"));
        let ton = SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton));
        for (from_asset, to_asset, value) in [(hmstr.clone(), ton.clone(), 100_000_000_000u64), (ton, hmstr, 10_000_000u64)] {
            let request = QuoteRequest {
                from_asset,
                to_asset,
                wallet_address: TEST_TON_SENDER.to_string(),
                destination_address: TEST_TON_SENDER.to_string(),
                value: BigUint::from(value),
                options: Options::new_with_slippage(100.into()),
            };
            let quote = provider.get_quote(&request).await?;
            assert_eq!(quote.from_value, request.value);
            assert!(quote.to_value > BigUint::ZERO);
            assert_eq!(quote.data.routes.len(), 1);
            assert_eq!(quote.data.routes[0].input, request.from_asset.asset_id());
            assert_eq!(quote.data.routes[0].output, request.to_asset.asset_id());
            let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
            assert!(!quote_data.to.is_empty());
            assert!(quote_data.value > BigUint::ZERO);
            assert_eq!(serde_json::from_str::<Vec<TonConnectMessage>>(&quote_data.data)?.len(), 1);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_stonfi_quote_and_quote_data_not_to_usdt() -> Result<(), SwapperError> {
        let rpc_provider = Arc::new(NativeProvider::default());
        let provider = Stonfi::new(rpc_provider).unwrap();
        let request = QuoteRequest {
            from_asset: SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, NOT_TOKEN_ID)),
            to_asset: SwapperQuoteAsset::from(TON_USDT_ASSET_ID.clone()),
            wallet_address: TEST_TON_SENDER.to_string(),
            destination_address: TEST_TON_SENDER.to_string(),
            value: BigUint::from(1000000000000u64),
            options: Options::new_with_slippage(100.into()),
        };

        let quote = provider.get_quote(&request).await?;
        assert_eq!(quote.from_value, request.value);
        assert!(quote.to_value > BigUint::ZERO);
        assert_eq!(quote.data.provider, provider.provider().clone());
        assert_eq!(quote.data.routes.len(), 2);
        assert_eq!(quote.data.routes[0].input, AssetId::from_token(Chain::Ton, NOT_TOKEN_ID));
        assert_eq!(quote.data.routes[0].output, AssetId::from_chain(Chain::Ton));
        assert_eq!(quote.data.routes[1].input, AssetId::from_chain(Chain::Ton));
        assert_eq!(quote.data.routes[1].output, TON_USDT_ASSET_ID.clone());
        println!("STON.fi NOT -> USDT quote: {quote:?}");

        let quote_data = provider.get_quote_data(&quote, FetchQuoteData::None).await?;
        assert!(!quote_data.to.is_empty());
        assert!(quote_data.value > BigUint::ZERO);
        assert_eq!(serde_json::from_str::<Vec<TonConnectMessage>>(&quote_data.data)?.len(), 1);
        println!("STON.fi NOT -> USDT quote_data: {quote_data:?}");

        Ok(())
    }
}
