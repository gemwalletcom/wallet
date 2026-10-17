use super::{
    constants::{FUNCTION_GET_AMOUNT_OUT, FUNCTION_POOL_RESERVE_AMOUNT, HYPERION_FEE_TIERS, MODULE_POOL},
    model::{INTERMEDIATE_TOKENS, Pool, PoolQuote, PoolRoute},
};
use crate::{
    ProviderType, RpcClient, RpcProvider, SwapperError, SwapperProvider,
    client_factory::create_aptos_client,
    fees::{ReferralFee, default_referral_fees},
    route_cache::{DiscoveryCache, OutpricedCache},
    route_path::RoutePath,
};
use futures::future::{join, join_all};
use gem_aptos::{
    AccountAddress, AptosClient, HYPERION_CONTRACT, VM_STATUS_ABORTED,
    models::{TransactionResponse, ViewRequest},
};
use gem_client::{Client, ClientError};
use serde_json::json;
use std::{cmp::Reverse, collections::BTreeMap, error::Error, sync::Arc};

type ViewError = Box<dyn Error + Send + Sync>;

#[derive(Debug)]
pub struct Hyperion<C: Client> {
    pub(super) provider: ProviderType,
    pub(super) client: AptosClient<C>,
    discovery_cache: DiscoveryCache<Pool, u8>,
    outpriced_pools: OutpricedCache<String>,
    outpriced_paths: OutpricedCache<RoutePath<String>>,
}

impl Hyperion<RpcClient> {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        Some(Self::with_client(create_aptos_client(rpc_provider).ok()?))
    }
}

impl<C: Client> Hyperion<C> {
    pub fn with_client(client: AptosClient<C>) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::Hyperion),
            client,
            discovery_cache: DiscoveryCache::default(),
            outpriced_pools: OutpricedCache::default(),
            outpriced_paths: OutpricedCache::default(),
        }
    }

    pub(super) fn referral_fee() -> ReferralFee {
        default_referral_fees().aptos
    }

    pub(super) async fn find_route(&self, from: &str, to: &str, amount_in: u64) -> Result<(PoolRoute, u64), SwapperError> {
        let keys = (&from.to_string(), &to.to_string());
        self.outpriced_paths
            .quote_live_first(keys, u128::from(amount_in), route_paths(from, to), Clone::clone, |paths| self.quote_paths(paths, keys, amount_in))
            .await
    }

    async fn quote_paths(&self, paths: Vec<RoutePath<String>>, (from, to): (&String, &String), amount_in: u64) -> Result<(PoolRoute, u64), SwapperError> {
        let quotes = join_all(paths.into_iter().map(|path| async move { Some((path.clone(), self.quote_path(path, from, to, amount_in).await?)) }))
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        self.outpriced_paths
            .record(from, to, u128::from(amount_in), quotes.iter().map(|(path, (_, amount_out))| (path.clone(), u128::from(*amount_out))));
        quotes.into_iter().map(|(_, quote)| quote).max_by_key(|(_, amount_out)| *amount_out).ok_or(SwapperError::NoQuoteAvailable)
    }

    pub(super) async fn preload_pair(&self, from: &str, to: &str) {
        join_all(discovery_pairs(from, to).iter().map(|(pair_from, pair_to)| self.discover_pools(pair_from, pair_to))).await;
    }

    async fn quote_path(&self, path: RoutePath<String>, from: &str, to: &str, amount_in: u64) -> Option<(PoolRoute, u64)> {
        match path {
            RoutePath::Direct => {
                let hop = self.best_pool(&self.discover_pools(from, to).await, from, to, amount_in).await?;
                Some((PoolRoute { pools: vec![hop.pool] }, hop.amount_out))
            }
            RoutePath::Intermediary(intermediate) => {
                let (first_pools, second_pools) = join(self.discover_pools(from, &intermediate), self.discover_pools(&intermediate, to)).await;
                if second_pools.is_empty() {
                    return None;
                }
                let first = self.best_pool(&first_pools, from, &intermediate, amount_in).await?;
                let second = self.best_pool(&second_pools, &intermediate, to, first.amount_out).await?;
                Some((PoolRoute { pools: vec![first.pool, second.pool] }, second.amount_out))
            }
        }
    }

    async fn best_pool(&self, pools: &[Pool], from: &str, to: &str, amount_in: u64) -> Option<PoolQuote> {
        let quote = |pools: Vec<Pool>| async move { self.quote_pools(&pools, from, to, amount_in).await.ok_or(SwapperError::NoQuoteAvailable) };
        self.outpriced_pools
            .quote_live_first((&from.to_string(), &to.to_string()), u128::from(amount_in), pools.to_vec(), |pool| pool.address.clone(), quote)
            .await
            .ok()
    }

    async fn quote_pools(&self, pools: &[Pool], from: &str, to: &str, amount_in: u64) -> Option<PoolQuote> {
        let amounts = join_all(pools.iter().map(|pool| self.get_amount_out(&pool.address, from, amount_in))).await;
        let quoted = pools
            .iter()
            .zip(amounts)
            .filter_map(|(pool, amount_out)| match amount_out {
                Ok(amount_out) => Some((pool, Some(amount_out))),
                Err(error) => vm_status(&error).map(|_| (pool, None)),
            })
            .collect::<Vec<_>>();
        let ranked = quoted
            .iter()
            .enumerate()
            .filter_map(|(index, (pool, amount_out))| Some(((Reverse((*amount_out)?), index), *pool)))
            .collect::<BTreeMap<_, _>>()
            .into_iter()
            .map(|((Reverse(amount_out), _), pool)| (amount_out, pool))
            .collect::<Vec<_>>();
        let (quote, unpayable) = self.first_payable(&ranked, from).await;
        let outcomes = quoted.iter().map(|(pool, amount_out)| {
            let amount_out = if unpayable.contains(&pool.address) { 0 } else { amount_out.map_or(0, u128::from) };
            (pool.address.clone(), amount_out)
        });
        self.outpriced_pools.record(&from.to_string(), &to.to_string(), u128::from(amount_in), outcomes);
        quote
    }

    async fn first_payable(&self, ranked: &[(u64, &Pool)], from: &str) -> (Option<PoolQuote>, Vec<String>) {
        let Some(((amount_out, pool), rest)) = ranked.split_first() else {
            return (None, vec![]);
        };
        let reserves = self.get_pool_reserves(&pool.address).await;
        if let Some(quote) = reserves.as_ref().ok().and_then(|reserves| pool.quote(from, *amount_out, *reserves)) {
            return (Some(quote), vec![]);
        }
        let (quote, unpayable) = Box::pin(self.first_payable(rest, from)).await;
        match reserves {
            Ok(_) => (quote, [vec![pool.address.clone()], unpayable].concat()),
            Err(_) => (quote, unpayable),
        }
    }

    async fn discover_pools(&self, token_a: &str, token_b: &str) -> Vec<Pool> {
        let missing = self.discovery_cache.missing_probes(token_a, token_b, &HYPERION_FEE_TIERS);
        let pools = missing.iter().filter_map(|tier| Some((*tier, Pool::new(token_a, token_b, *tier).ok()?))).collect::<Vec<_>>();
        let probes = join_all(pools.into_iter().map(|(tier, pool)| async move { (tier, pool.clone(), self.get_pool_reserves(&pool.address).await) })).await;
        let tradable = |(reserve_a, reserve_b): &(u64, u64)| *reserve_a > 0 && *reserve_b > 0;
        let discoveries = probes.iter().filter_map(|(tier, pool, reserves)| match reserves {
            Ok(reserves) => tradable(reserves).then(|| (*tier, Some(pool.clone()))),
            Err(error) => (vm_status(error) == Some(VM_STATUS_ABORTED)).then_some((*tier, None)),
        });
        self.discovery_cache.record_discovery(token_a, token_b, discoveries);
        self.discovery_cache
            .record_unusable(token_a, token_b, probes.iter().filter(|(_, _, reserves)| reserves.as_ref().is_ok_and(|reserves| !tradable(reserves))).map(|(tier, _, _)| *tier));
        self.discovery_cache.candidates_for_probes(token_a, token_b, &HYPERION_FEE_TIERS)
    }

    async fn get_amount_out(&self, pool: &str, from: &str, amount_in: u64) -> Result<u64, ViewError> {
        let request = ViewRequest::new(pool_function(FUNCTION_GET_AMOUNT_OUT), vec![json!(pool), json!(from), json!(amount_in.to_string())]);
        let (amount_out, _): (String, String) = self.client.view(request).await?;
        Ok(amount_out.parse()?)
    }

    async fn get_pool_reserves(&self, pool: &str) -> Result<(u64, u64), ViewError> {
        let request = ViewRequest::new(pool_function(FUNCTION_POOL_RESERVE_AMOUNT), vec![json!(pool)]);
        let (reserve_a, reserve_b): (String, String) = self.client.view(request).await?;
        Ok((reserve_a.parse()?, reserve_b.parse()?))
    }
}

fn vm_status(error: &ViewError) -> Option<u64> {
    match error.downcast_ref::<ClientError>() {
        Some(ClientError::Http { status: 400, body }) => serde_json::from_slice::<TransactionResponse>(body).ok()?.vm_error_code,
        _ => None,
    }
}

fn pool_function(function: &str) -> String {
    format!("{HYPERION_CONTRACT}::{MODULE_POOL}::{function}")
}

fn route_paths(from: &str, to: &str) -> Vec<RoutePath<String>> {
    let intermediates = INTERMEDIATE_TOKENS.iter().filter_map(|token| AccountAddress::from_hex(token).ok()).map(|token| token.to_string());
    RoutePath::all(&from.to_string(), &to.to_string(), intermediates, |left, right| left == right)
}

fn discovery_pairs(from: &str, to: &str) -> Vec<(String, String)> {
    route_paths(from, to).iter().flat_map(|path| path.pairs(&from.to_string(), &to.to_string())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hyperion::testkit::{mock_aptos_client, mock_aptos_client_with_calls};
    use primitives::asset_constants::{APTOS_USDC_TOKEN_ID, APTOS_USDT_TOKEN_ID};
    use std::sync::{Arc, Mutex};

    const USDT_TO_APT_VIEWS: &str = include_str!("../../testdata/hyperion/usdt_to_apt_views.json");

    #[test]
    fn test_route_paths() {
        let apt = AccountAddress::from_hex("0xa").unwrap().to_string();

        assert_eq!(route_paths(&apt, APTOS_USDC_TOKEN_ID), vec![RoutePath::Direct]);
        assert_eq!(route_paths(&apt, APTOS_USDT_TOKEN_ID), vec![RoutePath::Direct, RoutePath::Intermediary(APTOS_USDC_TOKEN_ID.to_string())]);
        assert_eq!(
            route_paths(APTOS_USDT_TOKEN_ID, "0x05fabd1b12e39967a3c24e91b7b8f67719a6dacee74f3c8b9fb7d93e855437d2"),
            vec![RoutePath::Direct, RoutePath::Intermediary(apt), RoutePath::Intermediary(APTOS_USDC_TOKEN_ID.to_string())]
        );
    }

    #[tokio::test]
    async fn test_discover_pools_records_each_probe() {
        let apt = AccountAddress::from_hex("0xa").unwrap().to_string();
        let reserves = |tier: u8, status: u16, response: serde_json::Value| json!({ "function": pool_function(FUNCTION_POOL_RESERVE_AMOUNT), "arguments": [Pool::new(&apt, APTOS_USDC_TOKEN_ID, tier).unwrap().address], "status": status, "response": response });
        let aborted = json!({ "message": "VMError with status ABORTED", "error_code": "invalid_input", "vm_error_code": VM_STATUS_ABORTED });
        let views = json!([
            reserves(0, 200, json!(["1000", "2000"])),
            reserves(1, 200, json!(["0", "2000"])),
            reserves(2, 400, aborted.clone()),
            reserves(3, 500, json!({ "message": "node unavailable" })),
            reserves(4, 400, json!({ "message": "invalid argument", "error_code": "invalid_input" })),
            reserves(5, 400, aborted),
        ]);
        let provider = Hyperion::with_client(mock_aptos_client(&views.to_string()));

        assert_eq!(provider.discover_pools(&apt, APTOS_USDC_TOKEN_ID).await, vec![Pool::new(&apt, APTOS_USDC_TOKEN_ID, 0).unwrap()]);
        assert_eq!(provider.discovery_cache.missing_probes(&apt, APTOS_USDC_TOKEN_ID, &HYPERION_FEE_TIERS), vec![3, 4]);
    }

    #[tokio::test]
    async fn test_find_route_skips_outpriced_pools() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let provider = Hyperion::with_client(mock_aptos_client_with_calls(USDT_TO_APT_VIEWS, calls.clone()));
        let apt = AccountAddress::from_hex("0xa").unwrap().to_string();
        let quotes = |calls: &Mutex<Vec<String>>| calls.lock().unwrap().drain(..).filter(|function| function.ends_with(FUNCTION_GET_AMOUNT_OUT)).count();

        let route = provider.find_route(APTOS_USDT_TOKEN_ID, &apt, 29_850).await;
        assert_eq!(quotes(&calls), 10);
        assert_eq!(provider.find_route(APTOS_USDT_TOKEN_ID, &apt, 29_850).await, route);
        assert_eq!(quotes(&calls), 2);
    }

    #[tokio::test]
    async fn test_find_route_skips_pool_without_output_reserve() {
        let provider = Hyperion::with_client(mock_aptos_client(USDT_TO_APT_VIEWS));
        let apt = AccountAddress::from_hex("0xa").unwrap().to_string();

        assert_eq!(
            provider.find_route(APTOS_USDT_TOKEN_ID, &apt, 29_850).await,
            Ok((
                PoolRoute {
                    pools: vec!["0x18269b1090d668fbbc01902fa6a5ac6e75565d61860ddae636ac89741c883cbc".to_string()],
                },
                3_548_211
            ))
        );
    }
}
