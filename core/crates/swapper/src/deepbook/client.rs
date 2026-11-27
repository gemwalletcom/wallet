use super::{
    constants::{EPACKAGE_VERSION_DISABLED, EPOOL_DOES_NOT_EXIST, FUNCTION_GET_POOL_ID, FUNCTION_LOAD_INNER, MODULE_POOL, MODULE_REGISTRY},
    model::{OrderSide, Pool, PoolQuote, Route, route_legs, route_out},
    tx_builder,
};
use crate::{
    ProviderType, RpcProvider, SwapperError, SwapperProvider,
    client_factory::create_sui_client,
    route_cache::DiscoveryCache,
    sui_swap::{MoveAbort, is_move_abort, lookup_pools, upgraded_error},
};
use futures::future::join_all;
use gem_sui::{EMPTY_ADDRESS, SuiClient};
use std::{collections::HashMap, sync::Arc};

const MISSING_POOL: MoveAbort = (MODULE_REGISTRY, FUNCTION_GET_POOL_ID, EPOOL_DOES_NOT_EXIST);
pub(super) const PACKAGE_VERSION_DISABLED: MoveAbort = (MODULE_POOL, FUNCTION_LOAD_INNER, EPACKAGE_VERSION_DISABLED);

#[derive(Debug)]
pub struct Deepbook {
    pub(super) provider: ProviderType,
    pub(super) sui_client: SuiClient,
    discovery_cache: DiscoveryCache<Pool, String>,
}

impl Deepbook {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Option<Self> {
        let sui_client = create_sui_client(rpc_provider).ok()?;
        Some(Self {
            provider: ProviderType::new(SwapperProvider::DeepbookV3),
            sui_client,
            discovery_cache: DiscoveryCache::default(),
        })
    }

    pub(super) async fn find_pool(&self, from: &str, to: &str) -> Option<Pool> {
        let bases = [from.to_string(), to.to_string()];
        if let Some(pool) = self.discovery_cache.candidates_for_probes(from, to, &bases).into_iter().next() {
            return Some(pool);
        }
        let missing = self.discovery_cache.missing_probes(from, to, &bases);
        if !missing.is_empty() {
            let counterpart = |base: &str| (if base == from { to } else { from }).to_string();
            let lookups = missing
                .into_iter()
                .filter_map(|base| tx_builder::build_pool_id_inspect(&base, &counterpart(&base)).ok().map(|transaction| (base, transaction)))
                .collect();
            let discoveries = lookup_pools(&self.sui_client, lookups, MISSING_POOL, &HashMap::new()).await.into_iter().map(|(base, pool)| {
                let pool = pool.map(|(pool_id, initial_shared_version)| Pool {
                    pool_id,
                    initial_shared_version,
                    quote_coin_type: counterpart(&base),
                    base_coin_type: base.clone(),
                });
                (base, pool)
            });
            self.discovery_cache.record_discovery(from, to, discoveries);
        }
        self.discovery_cache.candidates_for_probes(from, to, &bases).into_iter().next()
    }

    pub(super) async fn find_route(&self, pairs: Vec<(String, String)>) -> Option<Vec<Pool>> {
        join_all(pairs.iter().map(|(from, to)| self.find_pool(from, to))).await.into_iter().collect()
    }

    pub(super) async fn quote_pools(&self, pools: Vec<Pool>, from: &str, to: &str, amount_in: u64) -> Result<Route, SwapperError> {
        let legs = route_legs(&pools, from, to).ok_or(SwapperError::NoQuoteAvailable)?;
        let quotes = match self.quote_route(&legs, amount_in).await {
            Err(error) if legs.len() > 1 => return Err(below_minimum(&self.quote_route(&legs[..1], amount_in).await?, amount_in).unwrap_or(error)),
            result => result?,
        };
        if let Some(error) = below_minimum(&quotes, amount_in) {
            return Err(error);
        }
        let amount_out = route_out(&quotes, amount_in).ok_or(SwapperError::NoQuoteAvailable)?;
        Ok(Route { pools, amount_out })
    }

    async fn quote_route(&self, legs: &[(&Pool, OrderSide)], amount_in: u64) -> Result<Vec<(OrderSide, PoolQuote)>, SwapperError> {
        let transaction = tx_builder::build_quote_inspect(legs, amount_in)?;
        let result = self.sui_client.inspect_transaction_block(EMPTY_ADDRESS, &transaction).await.map_err(SwapperError::compute_quote_error)?;
        if let Some(error) = result.error.as_deref() {
            return Err(match is_move_abort(error, PACKAGE_VERSION_DISABLED) {
                true => upgraded_error(&self.provider.name),
                false => SwapperError::ComputeQuoteError(format!("DeepBook quote failed: {error}")),
            });
        }
        let values = result.results.iter().map(|command| command.return_values.iter().map(|(bytes, _)| decode_u64(bytes)).collect()).collect::<Result<Vec<_>, _>>()?;
        if values.len() != legs.len() * 3 {
            return Err(SwapperError::ComputeQuoteError("DeepBook quote returned no quantities".into()));
        }
        values.chunks(3).zip(legs).map(|(values, (_, side))| PoolQuote::from_return_values(*side, values).map(|quote| (*side, quote))).collect()
    }
}

fn below_minimum(quotes: &[(OrderSide, PoolQuote)], amount_in: u64) -> Option<SwapperError> {
    quotes.first().filter(|(_, quote)| amount_in < quote.minimum_in).map(|(_, quote)| SwapperError::InputAmountError {
        min_amount: Some(quote.minimum_in.to_string()),
    })
}

fn decode_u64(bytes: &[u8]) -> Result<u64, SwapperError> {
    bytes.try_into().map(u64::from_le_bytes).map_err(|_| SwapperError::ComputeQuoteError("DeepBook quote returned an invalid quantity".into()))
}
