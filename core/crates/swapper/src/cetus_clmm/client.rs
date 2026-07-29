use super::{
    constants::{CETUS_ALL_TICK_SPACINGS, CETUS_PRIMARY_TICK_SPACINGS, KNOWN_POOLS},
    model::{DiscoveredPool, Hop, INTERMEDIATE_COIN_TYPES},
    tx_builder,
};
use crate::{
    ProviderType, RpcProvider, SwapperError, SwapperProvider,
    client_factory::create_sui_client,
    fees::{ReferralFee, default_referral_fees},
    route_cache::DiscoveryCache,
};
use gem_sui::{EMPTY_ADDRESS, SUI_COIN_TYPE, SuiClient, coin_type_matches, full_coin_type, models::InspectResult, tx_builder::ObjectResolver};
use primitives::AssetId;
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub(super) struct QuoteResult {
    pub amount_out: u64,
    pub current_sqrt_price: u128,
    pub after_sqrt_price: u128,
    pub is_exceed: bool,
}

const DIRECT_PRICE_IMPACT_THRESHOLD_BPS: u32 = 50;

#[derive(Debug, Default)]
struct PhaseResult {
    acceptable_direct: Option<(Vec<Hop>, u32)>,
    best_route: Option<(Vec<Hop>, u32)>,
}

pub struct CetusClmm {
    pub(super) provider: ProviderType,
    pub(super) sui_client: SuiClient,
    discovery_cache: DiscoveryCache<DiscoveredPool, u32>,
}

impl std::fmt::Debug for CetusClmm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CetusClmm")
    }
}

impl CetusClmm {
    pub fn new(rpc_provider: Arc<dyn RpcProvider>) -> Self {
        let sui_client = create_sui_client(rpc_provider).expect("failed to create Sui gRPC client");
        Self::with_client(sui_client)
    }

    pub fn with_client(sui_client: SuiClient) -> Self {
        Self {
            provider: ProviderType::new(SwapperProvider::CetusClmm),
            sui_client,
            discovery_cache: DiscoveryCache::default(),
        }
    }

    pub(super) fn referral_fee() -> ReferralFee {
        default_referral_fees().sui
    }

    pub(super) fn coin_type(asset_id: &AssetId) -> String {
        full_coin_type(asset_id.token_id.as_deref().unwrap_or(SUI_COIN_TYPE))
    }

    pub(super) async fn find_route_hops(&self, from: &str, to: &str, swap_amount: u64) -> Result<Vec<Hop>, SwapperError> {
        let discovery_complete = self.route_discovery_complete(from, to);
        let initial_ticks = if discovery_complete { CETUS_ALL_TICK_SPACINGS } else { CETUS_PRIMARY_TICK_SPACINGS };
        let PhaseResult { acceptable_direct, best_route } = self.try_route_with_ticks(from, to, swap_amount, initial_ticks).await;
        if let Some((hops, _)) = acceptable_direct {
            return Ok(hops);
        }
        if discovery_complete {
            return best_route.map(|(hops, _)| hops).ok_or(SwapperError::NoQuoteAvailable);
        }
        match best_route {
            Some((hops, impact)) if impact <= DIRECT_PRICE_IMPACT_THRESHOLD_BPS => Ok(hops),
            best_route => {
                let expanded = self.try_route_with_ticks(from, to, swap_amount, CETUS_ALL_TICK_SPACINGS).await;
                expanded
                    .acceptable_direct
                    .or(expanded.best_route)
                    .or(best_route)
                    .map(|(hops, _)| hops)
                    .ok_or(SwapperError::NoQuoteAvailable)
            }
        }
    }

    pub(super) async fn preload_pair(&self, from: &str, to: &str) {
        let discoveries = discovery_pairs(from, to)
            .into_iter()
            .map(|(pair_from, pair_to)| async move { self.discover_direct_pools(&pair_from, &pair_to, CETUS_ALL_TICK_SPACINGS).await });
        futures::future::join_all(discoveries).await;
    }

    async fn try_route_with_ticks(&self, from: &str, to: &str, swap_amount: u64, ticks: &[u32]) -> PhaseResult {
        let direct_candidates: Vec<Vec<DiscoveredPool>> = self.discover_direct_pools(from, to, ticks).await.into_iter().map(|pool| vec![pool]).collect();
        let direct_quotes = self.quote_candidates_batched(direct_candidates, from, swap_amount).await;
        let acceptable_direct = direct_quotes
            .iter()
            .filter_map(|q| q.as_ref())
            .filter(|(_, impact)| *impact < DIRECT_PRICE_IMPACT_THRESHOLD_BPS)
            .max_by_key(|(hops, _)| hops.last().map(|h| h.amount_out).unwrap_or_default())
            .cloned();
        if acceptable_direct.is_some() {
            return PhaseResult {
                acceptable_direct,
                best_route: None,
            };
        }

        let multi_hop_discoveries = INTERMEDIATE_COIN_TYPES.iter().filter_map(|raw_intermediate| {
            let intermediate = full_coin_type(raw_intermediate);
            if coin_type_matches(from, &intermediate) || coin_type_matches(to, &intermediate) {
                None
            } else {
                Some(async move { futures::future::join(self.discover_direct_pools(from, &intermediate, ticks), self.discover_direct_pools(&intermediate, to, ticks)).await })
            }
        });
        let multi_hop_candidates = futures::future::join_all(multi_hop_discoveries)
            .await
            .into_iter()
            .flat_map(|(firsts, seconds)| {
                firsts
                    .into_iter()
                    .flat_map(move |first| seconds.clone().into_iter().map(move |second| vec![first.clone(), second]))
            })
            .collect();
        let multi_hop_quotes = self.quote_candidates_batched(multi_hop_candidates, from, swap_amount).await;
        let best_route = direct_quotes
            .into_iter()
            .chain(multi_hop_quotes)
            .flatten()
            .max_by_key(|(hops, _)| hops.last().map(|h| h.amount_out).unwrap_or_default());
        PhaseResult {
            acceptable_direct: None,
            best_route,
        }
    }

    fn route_discovery_complete(&self, from: &str, to: &str) -> bool {
        discovery_pairs(from, to)
            .into_iter()
            .all(|(pair_from, pair_to)| self.discovery_cache.missing_probes(pair_from, pair_to, CETUS_ALL_TICK_SPACINGS).is_empty())
    }

    fn known_pools(from: &str, to: &str) -> Vec<DiscoveredPool> {
        KNOWN_POOLS
            .iter()
            .filter(|known| {
                (coin_type_matches(from, known.coin_a) && coin_type_matches(to, known.coin_b)) || (coin_type_matches(from, known.coin_b) && coin_type_matches(to, known.coin_a))
            })
            .map(|known| DiscoveredPool {
                pool_id: known.pool_id.to_string(),
                pool_init_version: known.pool_init_version,
                coin_a: known.coin_a.to_string(),
                coin_b: known.coin_b.to_string(),
            })
            .collect()
    }

    fn discovered_pools(&self, from: &str, to: &str, ticks: &[u32]) -> Vec<DiscoveredPool> {
        let known = Self::known_pools(from, to);
        known
            .iter()
            .cloned()
            .chain(
                self.discovery_cache
                    .candidates_for_probes(from, to, ticks)
                    .into_iter()
                    .filter(|candidate| known.iter().all(|pool| pool.pool_id != candidate.pool_id)),
            )
            .collect()
    }

    async fn discover_direct_pools(&self, from: &str, to: &str, ticks: &[u32]) -> Vec<DiscoveredPool> {
        let missing = self.discovery_cache.missing_probes(from, to, ticks);
        if missing.is_empty() {
            return self.discovered_pools(from, to, ticks);
        }
        let discoveries = self.query_direct_pools(from, to, &missing).await;
        self.discovery_cache.record_discovery(from, to, discoveries);
        self.discovered_pools(from, to, ticks)
    }

    async fn query_direct_pools(&self, from: &str, to: &str, ticks: &[u32]) -> Vec<(u32, Option<DiscoveredPool>)> {
        let (coin_a, coin_b) = canonical_pair_order(from, to);
        let inspects = ticks
            .iter()
            .map(|tick| async move { self.inspect_pool_id(coin_a, coin_b, *tick).await.map(|pool_id| (*tick, pool_id)) });
        let results = futures::future::join_all(inspects).await.into_iter().filter_map(Result::ok).collect::<Vec<_>>();
        let pool_ids = results.iter().filter_map(|(_, pool_id)| pool_id.clone()).collect::<Vec<_>>();
        let pool_ids = pool_ids
            .iter()
            .enumerate()
            .filter(|(index, pool_id)| !pool_ids[..*index].contains(pool_id))
            .map(|(_, pool_id)| pool_id.clone())
            .collect::<Vec<_>>();
        if pool_ids.is_empty() {
            return results.into_iter().map(|(tick, _)| (tick, None)).collect();
        }
        let Ok(resolver) = ObjectResolver::prefetch(&self.sui_client, pool_ids, &HashMap::new()).await else {
            return results.into_iter().filter_map(|(tick, pool_id)| pool_id.is_none().then_some((tick, None))).collect();
        };
        results
            .into_iter()
            .filter_map(|(tick, pool_id)| match pool_id {
                Some(pool_id) => resolver.initial_shared_version(&pool_id).map(|pool_init_version| {
                    (
                        tick,
                        Some(DiscoveredPool {
                            pool_id,
                            pool_init_version,
                            coin_a: coin_a.to_string(),
                            coin_b: coin_b.to_string(),
                        }),
                    )
                }),
                None => Some((tick, None)),
            })
            .collect()
    }

    async fn quote_candidates_batched(&self, candidates: Vec<Vec<DiscoveredPool>>, from: &str, swap_amount: u64) -> Vec<Option<(Vec<Hop>, u32)>> {
        if candidates.is_empty() {
            return Vec::new();
        }
        if candidates[0].len() == 1 {
            self.quote_direct_batched(candidates, from, swap_amount).await
        } else {
            self.quote_multi_hop_fused(candidates, from, swap_amount).await
        }
    }

    async fn quote_direct_batched(&self, candidates: Vec<Vec<DiscoveredPool>>, from: &str, swap_amount: u64) -> Vec<Option<(Vec<Hop>, u32)>> {
        let hops: Vec<Hop> = candidates.iter().map(|pools| pools[0].clone().into_hop(from, swap_amount)).collect();
        let inputs: Vec<(&Hop, u64)> = hops.iter().map(|hop| (hop, swap_amount)).collect();
        let quote_results = self.inspect_batch_quotes(&inputs).await.unwrap_or_else(|_| vec![None; candidates.len()]);

        hops.into_iter()
            .zip(quote_results)
            .map(|(hop, quote)| {
                let q = quote?;
                if q.amount_out == 0 || q.is_exceed {
                    return None;
                }
                let impact = price_impact_bps(q.current_sqrt_price, q.after_sqrt_price);
                let hop = Hop {
                    amount_out: q.amount_out,
                    after_sqrt_price: q.after_sqrt_price,
                    ..hop
                };
                Some((vec![hop], impact))
            })
            .collect()
    }

    async fn quote_multi_hop_fused(&self, candidates: Vec<Vec<DiscoveredPool>>, from: &str, swap_amount: u64) -> Vec<Option<(Vec<Hop>, u32)>> {
        let hop_pairs: Vec<(Hop, Hop)> = candidates
            .iter()
            .map(|pools| {
                let hop1 = pools[0].clone().into_hop(from, swap_amount);
                let intermediate = hop1.output_coin_type().to_string();
                let hop2 = pools[1].clone().into_hop(&intermediate, 0);
                (hop1, hop2)
            })
            .collect();
        let inputs: Vec<(&Hop, &Hop, u64)> = hop_pairs.iter().map(|(h1, h2)| (h1, h2, swap_amount)).collect();
        let fused_results = self.inspect_batch_multi_hop_quotes(&inputs).await.unwrap_or_else(|_| vec![(None, None); candidates.len()]);

        hop_pairs
            .into_iter()
            .zip(fused_results)
            .map(|((hop1, hop2), (q1, q2))| {
                let q1 = q1?;
                if q1.amount_out == 0 || q1.is_exceed {
                    return None;
                }
                let q2 = q2?;
                if q2.amount_out == 0 || q2.is_exceed {
                    return None;
                }
                let max_impact = price_impact_bps(q1.current_sqrt_price, q1.after_sqrt_price).max(price_impact_bps(q2.current_sqrt_price, q2.after_sqrt_price));
                let hop1 = Hop {
                    amount_out: q1.amount_out,
                    after_sqrt_price: q1.after_sqrt_price,
                    ..hop1
                };
                let hop2 = Hop {
                    amount_in: q1.amount_out,
                    amount_out: q2.amount_out,
                    after_sqrt_price: q2.after_sqrt_price,
                    ..hop2
                };
                Some((vec![hop1, hop2], max_impact))
            })
            .collect()
    }

    async fn inspect_pool_id(&self, coin_a: &str, coin_b: &str, tick_spacing: u32) -> Result<Option<String>, SwapperError> {
        let transaction = tx_builder::build_pool_id_inspect(coin_a, coin_b, tick_spacing)?;
        let result = self
            .sui_client
            .inspect_transaction_block(EMPTY_ADDRESS, &transaction, None)
            .await
            .map_err(SwapperError::compute_quote_error)?;
        if let Some(error) = result.error.as_deref() {
            return if is_missing_pool_error(error) {
                Ok(None)
            } else {
                Err(SwapperError::ComputeQuoteError(format!("Cetus CLMM pool discovery failed: {error}")))
            };
        }
        let bytes = result
            .results
            .last()
            .and_then(|command| command.return_values.first())
            .map(|(bytes, _)| bytes)
            .ok_or_else(|| SwapperError::ComputeQuoteError("Cetus CLMM pool discovery returned no value".into()))?;
        if bytes.len() != 32 {
            return Err(SwapperError::ComputeQuoteError("Cetus CLMM pool discovery returned invalid id".into()));
        }
        Ok(Some(format!("0x{}", hex::encode(bytes))))
    }

    async fn inspect_batch_quotes(&self, quotes: &[(&Hop, u64)]) -> Result<Vec<Option<QuoteResult>>, SwapperError> {
        if quotes.is_empty() {
            return Ok(Vec::new());
        }
        let result = self.inspect_quote(tx_builder::build_batch_quote_inspect(quotes)?).await?;
        Ok((0..quotes.len()).map(|i| quote_result_at(&result, i)).collect())
    }

    async fn inspect_batch_multi_hop_quotes(&self, routes: &[(&Hop, &Hop, u64)]) -> Result<Vec<(Option<QuoteResult>, Option<QuoteResult>)>, SwapperError> {
        if routes.is_empty() {
            return Ok(Vec::new());
        }
        let result = self.inspect_quote(tx_builder::build_batch_multi_hop_quote_inspect(routes)?).await?;
        Ok((0..routes.len()).map(|i| (quote_result_at(&result, i * 3), quote_result_at(&result, i * 3 + 2))).collect())
    }

    async fn inspect_quote(&self, transaction: Vec<u8>) -> Result<InspectResult, SwapperError> {
        let result = self
            .sui_client
            .inspect_transaction_block(EMPTY_ADDRESS, &transaction, None)
            .await
            .map_err(SwapperError::compute_quote_error)?;
        if let Some(error) = result.error.as_deref() {
            return Err(SwapperError::ComputeQuoteError(format!("Cetus CLMM quote simulation failed: {error}")));
        }
        Ok(result)
    }
}

fn discovery_pairs(from: &str, to: &str) -> Vec<(String, String)> {
    std::iter::once((from.to_string(), to.to_string()))
        .chain(
            INTERMEDIATE_COIN_TYPES
                .iter()
                .map(|raw_intermediate| full_coin_type(raw_intermediate))
                .filter(|intermediate| !coin_type_matches(from, intermediate) && !coin_type_matches(to, intermediate))
                .flat_map(|intermediate| [(from.to_string(), intermediate.clone()), (intermediate, to.to_string())]),
        )
        .collect()
}

fn canonical_pair_order<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a > b { (a, b) } else { (b, a) }
}

fn is_missing_pool_error(error: &str) -> bool {
    let error = error.to_ascii_lowercase();
    error.contains("moveabort")
        && error.contains("factory")
        && error.contains("pool_simple_info")
        && (error.contains(", 10)") || error.contains("abort code 10") || error.contains("abort_code: 10"))
}

fn decode_quote_result_bytes(bytes: &[u8]) -> Result<QuoteResult, SwapperError> {
    if bytes.len() < 66 {
        return Err(SwapperError::ComputeQuoteError("Cetus CLMM quote inspect returned truncated CalculatedSwapResult".into()));
    }
    let amount_out = u64::from_le_bytes(
        bytes[8..16]
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError("Cetus CLMM amount_out decode failed".into()))?,
    );
    let after_sqrt_price = u128::from_le_bytes(
        bytes[32..48]
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError("Cetus CLMM after_sqrt_price decode failed".into()))?,
    );
    let is_exceed = bytes[48] != 0;
    let current_sqrt_price = u128::from_le_bytes(
        bytes[50..66]
            .try_into()
            .map_err(|_| SwapperError::ComputeQuoteError("Cetus CLMM current_sqrt_price decode failed".into()))?,
    );
    Ok(QuoteResult {
        amount_out,
        current_sqrt_price,
        after_sqrt_price,
        is_exceed,
    })
}

fn price_impact_bps(current_sqrt_price: u128, after_sqrt_price: u128) -> u32 {
    if current_sqrt_price == 0 {
        return u32::MAX;
    }
    let (high, low) = if current_sqrt_price >= after_sqrt_price {
        (current_sqrt_price, after_sqrt_price)
    } else {
        (after_sqrt_price, current_sqrt_price)
    };
    let delta = high - low;
    let bps = delta.saturating_mul(20_000) / current_sqrt_price;
    u32::try_from(bps).unwrap_or(u32::MAX)
}

fn quote_result_at(result: &InspectResult, cmd_idx: usize) -> Option<QuoteResult> {
    let bytes = result.results.get(cmd_idx).and_then(|cmd| cmd.return_values.first()).map(|(bytes, _)| bytes)?;
    decode_quote_result_bytes(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alien::mock::ProviderMock;
    use primitives::asset_constants::SUI_USDC_TOKEN_ID;
    use std::sync::Arc;

    fn inspect_result_many(per_command: Vec<Vec<u8>>) -> InspectResult {
        InspectResult {
            effects: gem_sui::models::InspectEffects {
                gas_used: gem_sui::models::InspectGasUsed {
                    computation_cost: 0,
                    storage_cost: 0,
                    storage_rebate: 0,
                },
            },
            events: serde_json::Value::Null,
            error: None,
            results: per_command
                .into_iter()
                .map(|bytes| gem_sui::models::InspectCommandResult {
                    return_values: vec![(bytes, "CalculatedSwapResult".into())],
                })
                .collect(),
        }
    }

    fn calc_swap_bytes(amount_out: u64, current_sqrt: u128, after_sqrt: u128, is_exceed: bool) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(66);
        bytes.extend_from_slice(&997_500_u64.to_le_bytes());
        bytes.extend_from_slice(&amount_out.to_le_bytes());
        bytes.extend_from_slice(&2_500_u64.to_le_bytes());
        bytes.extend_from_slice(&2_500_u64.to_le_bytes());
        bytes.extend_from_slice(&after_sqrt.to_le_bytes());
        bytes.push(if is_exceed { 1 } else { 0 });
        bytes.push(1);
        bytes.extend_from_slice(&current_sqrt.to_le_bytes());
        bytes
    }

    #[test]
    fn test_price_impact_bps() {
        assert_eq!(price_impact_bps(1_000_000, 1_000_000), 0);
        assert_eq!(price_impact_bps(1_000_000, 995_000), 100);
        assert_eq!(price_impact_bps(995_000, 1_000_000), 100);
        assert_eq!(price_impact_bps(1_000_000, 990_000), 200);
        assert_eq!(price_impact_bps(0, 1_000_000), u32::MAX);
    }

    #[test]
    fn test_canonical_pair_order() {
        let sui = gem_sui::SUI_COIN_TYPE_FULL;
        let usdc = "0xdba34672e30cb065b1f93e3ab55318768fd6fef66c15942c9f7cb846e2f900e7::usdc::USDC";
        let blue = "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE";

        assert_eq!(canonical_pair_order(usdc, sui), (usdc, sui));
        assert_eq!(canonical_pair_order(sui, usdc), (usdc, sui));
        assert_eq!(canonical_pair_order(blue, sui), (blue, sui));
        assert_eq!(canonical_pair_order(sui, blue), (blue, sui));
        assert_eq!(canonical_pair_order(blue, usdc), (blue, usdc));
        assert_eq!(canonical_pair_order(usdc, blue), (blue, usdc));
    }

    #[test]
    fn test_discovery_pairs() {
        let sui = gem_sui::SUI_COIN_TYPE_FULL;
        let usdc = SUI_USDC_TOKEN_ID;
        let blue = "0xe1b45a0e641b9955a20aa0ad1c1f4ad86aad8afb07296d4085e349a50e90bdca::blue::BLUE";
        let other = "0x123::coin::COIN";

        assert_eq!(discovery_pairs(sui, usdc), vec![(sui.to_string(), usdc.to_string())]);
        assert_eq!(discovery_pairs(sui, blue).len(), 3);
        assert_eq!(discovery_pairs(blue, other).len(), 5);
    }

    #[test]
    fn test_known_pools_do_not_suppress_discovery() {
        let provider = CetusClmm::new(Arc::new(ProviderMock::new(String::new())));

        assert!(!provider.route_discovery_complete(SUI_USDC_TOKEN_ID, gem_sui::SUI_COIN_TYPE_FULL));

        provider
            .discovery_cache
            .record_discovery(SUI_USDC_TOKEN_ID, gem_sui::SUI_COIN_TYPE_FULL, CETUS_ALL_TICK_SPACINGS.iter().map(|tick| (*tick, None)));

        assert!(provider.route_discovery_complete(SUI_USDC_TOKEN_ID, gem_sui::SUI_COIN_TYPE_FULL));
        assert_eq!(
            provider.discovered_pools(SUI_USDC_TOKEN_ID, gem_sui::SUI_COIN_TYPE_FULL, CETUS_ALL_TICK_SPACINGS),
            CetusClmm::known_pools(SUI_USDC_TOKEN_ID, gem_sui::SUI_COIN_TYPE_FULL)
        );
    }

    #[test]
    fn test_missing_pool_error_requires_exact_cetus_abort() {
        let missing = r#"MoveAbort(MoveLocation { module: ModuleId { name: Identifier("factory") }, function_name: Some("pool_simple_info") }, 10) in command 1"#;

        assert!(is_missing_pool_error(missing));
        assert!(!is_missing_pool_error(&missing.replace(", 10)", ", 11)")));
        assert!(!is_missing_pool_error(&missing.replace("pool_simple_info", "pool_id")));
        assert!(!is_missing_pool_error("RPC timeout"));
    }

    #[test]
    fn test_quote_result_at_extracts_per_command() {
        let current = 521_723_622_374_070_550_528_u128;
        let after = 521_460_761_563_383_315_264_u128;
        let bytes_a = calc_swap_bytes(100_000, current, after, false);
        let bytes_b = calc_swap_bytes(200_000, current, after, true);
        let bytes_c = calc_swap_bytes(300_000, current, after, false);
        let result = inspect_result_many(vec![bytes_a, bytes_b, bytes_c]);

        assert_eq!(quote_result_at(&result, 0).unwrap().amount_out, 100_000);
        assert!(!quote_result_at(&result, 0).unwrap().is_exceed);
        assert_eq!(quote_result_at(&result, 1).unwrap().amount_out, 200_000);
        assert!(quote_result_at(&result, 1).unwrap().is_exceed);
        assert_eq!(quote_result_at(&result, 2).unwrap().amount_out, 300_000);
        assert!(quote_result_at(&result, 3).is_none());
    }

    #[test]
    fn test_decode_quote_result_bytes() {
        let current = 521_723_622_374_070_550_528_u128;
        let after = 521_460_761_563_383_315_264_u128;
        let bytes = calc_swap_bytes(796_985_864, current, after, false);
        let decoded = decode_quote_result_bytes(&bytes).unwrap();
        assert_eq!(decoded.amount_out, 796_985_864);
        assert_eq!(decoded.after_sqrt_price, after);
        assert!(!decoded.is_exceed);

        let exceeded = calc_swap_bytes(796_985_864, current, after, true);
        assert!(decode_quote_result_bytes(&exceeded).unwrap().is_exceed);

        let truncated = decode_quote_result_bytes(&[0u8; 16]);
        match truncated {
            Err(SwapperError::ComputeQuoteError(_)) => {}
            other => panic!("expected ComputeQuoteError, got {other:?}"),
        }
    }
}
