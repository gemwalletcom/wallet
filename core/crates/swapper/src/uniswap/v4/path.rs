use alloy_primitives::{Address, Bytes};
use gem_evm::uniswap::{
    FeeTier,
    contracts::v4::{IV4Quoter::QuoteExactParams, PathKey, PoolKey},
    path::TokenPair,
};

use crate::{Route, SwapperError, error::INVALID_ADDRESS, eth_address, uniswap::swap_route::RouteData};

// return (currency0, currency1)
fn sort_addresses(token_in: &Address, token_out: &Address) -> (Address, Address) {
    if token_in.0 < token_out.0 { (*token_in, *token_out) } else { (*token_out, *token_in) }
}

pub fn build_pool_key(token_in: &Address, token_out: &Address, fee_tier: &FeeTier) -> (PoolKey, bool) {
    let (currency0, currency1) = sort_addresses(token_in, token_out);
    let zero_for_one = currency0.0 == token_in.0;
    let fee = fee_tier.as_u24();
    let tick_spacing = fee_tier.default_tick_spacing();
    (
        PoolKey {
            currency0,
            currency1,
            fee,
            tickSpacing: tick_spacing,
            hooks: Address::ZERO,
        },
        zero_for_one,
    )
}

pub fn build_pool_keys(token_in: &Address, token_out: &Address, fee_tiers: &[FeeTier]) -> Vec<(Vec<TokenPair>, PoolKey)> {
    fee_tiers
        .iter()
        .map(|fee_tier| {
            let (pool_key, _) = build_pool_key(token_in, token_out, fee_tier);
            (
                vec![TokenPair {
                    token_in: *token_in,
                    token_out: *token_out,
                    fee_tier: *fee_tier,
                }],
                pool_key,
            )
        })
        .collect()
}

pub fn build_quote_exact_params(
    amount_in: u128,
    token_in: &Address,
    token_out: &Address,
    fee_tiers: &[FeeTier],
    intermediaries: &[Address],
) -> Vec<Vec<(Vec<TokenPair>, QuoteExactParams)>> {
    intermediaries
        .iter()
        .map(|intermediary| {
            fee_tiers
                .iter()
                .map(|fee_tier| TokenPair::new_two_hop(token_in, intermediary, token_out, *fee_tier))
                .filter(|token_pairs| token_pairs.len() >= 2)
                .map(|token_pairs| {
                    let quote_exact_params = QuoteExactParams {
                        exactCurrency: token_pairs[0].token_in,
                        path: token_pairs
                            .iter()
                            .map(|token_pair| PathKey {
                                intermediateCurrency: token_pair.token_out,
                                fee: token_pair.fee_tier.as_u24(),
                                tickSpacing: token_pair.fee_tier.default_tick_spacing(),
                                hooks: Address::ZERO,
                                hookData: Bytes::new(),
                            })
                            .collect(),
                        exactAmount: amount_in,
                    };

                    (token_pairs, quote_exact_params)
                })
                .collect()
        })
        .collect()
}

pub fn get_intermediary_token(quote_exact_params: &[Vec<(Vec<TokenPair>, QuoteExactParams)>], batch_idx: usize) -> Option<Address> {
    if batch_idx == 0 {
        return None;
    }
    let batch = quote_exact_params.get(batch_idx - 1)?;
    let (token_pairs, _) = batch.first()?;
    let first_hop = token_pairs.first()?;
    Some(first_hop.token_out)
}

impl TryFrom<&Route> for PathKey {
    type Error = SwapperError;

    fn try_from(value: &Route) -> Result<Self, Self::Error> {
        let token_id = value
            .output
            .token_id
            .as_ref()
            .ok_or_else(|| SwapperError::ComputeQuoteError(format!("{}: {}", INVALID_ADDRESS, value.output)))?;
        let currency = eth_address::parse_str(token_id)?;

        let route_data: RouteData = serde_json::from_str(&value.route_data).map_err(|_| SwapperError::InvalidRoute)?;
        let fee_tier = FeeTier::try_from(route_data.fee_tier.as_str()).map_err(|_| SwapperError::ComputeQuoteError("invalid fee tier".into()))?;
        Ok(PathKey {
            intermediateCurrency: currency,
            fee: fee_tier.as_u24(),
            tickSpacing: fee_tier.default_tick_spacing(),
            hooks: Address::ZERO,
            hookData: Bytes::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uniswap::swap_route::get_intermediaries;
    use gem_evm::uniswap::path::get_base_pair;
    use primitives::EVMChain;
    use primitives::asset_constants::{ETHEREUM_UNI_TOKEN_ID, ETHEREUM_USDS_TOKEN_ID};
    use std::str::FromStr;

    #[test]
    fn test_get_intermediary_token() {
        let token_in = Address::from_str(ETHEREUM_UNI_TOKEN_ID).unwrap();
        let token_out = Address::from_str(ETHEREUM_USDS_TOKEN_ID).unwrap();
        let base_pair = get_base_pair(&EVMChain::Ethereum, true).unwrap();
        let intermediaries = get_intermediaries(&token_in, &token_out, &base_pair);
        let fee_tiers = vec![FeeTier::FiveHundred, FeeTier::ThreeThousand];
        let quote_exact_params = build_quote_exact_params(1_000_000, &token_in, &token_out, &fee_tiers, &intermediaries);

        assert_eq!(quote_exact_params.len(), intermediaries.len());
        assert_eq!(get_intermediary_token(&quote_exact_params, 0), None);
        assert_eq!(get_intermediary_token(&quote_exact_params, 1), Some(intermediaries[0]));
        assert_eq!(
            get_intermediary_token(&quote_exact_params, intermediaries.len()),
            Some(intermediaries[intermediaries.len() - 1])
        );
        assert_eq!(get_intermediary_token(&quote_exact_params, intermediaries.len() + 1), None);
    }
}
