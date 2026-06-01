use super::{
    constants::{RouterInfo, STATIC_POOLS, StaticPool},
    model::{Router, SwapSimulation},
};
use crate::{SwapperError, SwapperQuoteAsset};
use gem_ton::{address::Address, constants::TON_PROXY_JETTON_ADDRESS};
use num_bigint::BigUint;
use std::str::FromStr;

const BPS_DENOMINATOR: u32 = 10_000;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct DiscoveredPool {
    pub pool_address: String,
    pub router: Router,
    pub asset0: String,
    pub asset1: String,
    pub wallet0: String,
    pub wallet1: String,
    pub lp_fee_bps: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PoolData {
    pub is_locked: bool,
    pub reserve0: BigUint,
    pub reserve1: BigUint,
    pub token0_wallet: String,
    pub token1_wallet: String,
    pub lp_fee: u32,
    pub protocol_fee: u32,
}

impl DiscoveredPool {
    pub(super) fn from_static(pool: &StaticPool) -> Self {
        Self {
            pool_address: pool.pool_address.to_string(),
            router: router_model(&pool.router),
            asset0: pool.token0.to_string(),
            asset1: pool.token1.to_string(),
            wallet0: pool.token0_wallet.to_string(),
            wallet1: pool.token1_wallet.to_string(),
            lp_fee_bps: pool.lp_fee_bps,
        }
    }

    pub(super) fn wallet_for(&self, token: &str) -> Option<&str> {
        if token == self.asset0 {
            Some(&self.wallet0)
        } else if token == self.asset1 {
            Some(&self.wallet1)
        } else {
            None
        }
    }
}

pub(super) fn static_candidates(from_token: &str, to_token: &str) -> Vec<DiscoveredPool> {
    STATIC_POOLS
        .iter()
        .filter(|pool| static_pool_matches(pool, from_token, to_token))
        .map(DiscoveredPool::from_static)
        .collect()
}

fn static_pool_matches(pool: &StaticPool, from_token: &str, to_token: &str) -> bool {
    (pool.token0 == from_token && pool.token1 == to_token) || (pool.token0 == to_token && pool.token1 == from_token)
}

pub(super) fn router_model(router: &RouterInfo) -> Router {
    Router {
        address: router.address.to_string(),
        major_version: router.major_version,
        minor_version: router.minor_version,
    }
}

pub(super) fn token_address(asset: &SwapperQuoteAsset) -> String {
    let asset_id = asset.asset_id();
    match asset_id.token_id {
        Some(token_id) => token_id,
        None => TON_PROXY_JETTON_ADDRESS.to_string(),
    }
}

pub(super) fn compute_amount_out(pool: &PoolData, offer_wallet: &str, amount: &BigUint) -> Result<BigUint, SwapperError> {
    let offer = Address::parse(offer_wallet)?;
    let token0 = Address::parse(&pool.token0_wallet)?;
    let token1 = Address::parse(&pool.token1_wallet)?;
    let (reserve_in, reserve_out) = if offer == token0 {
        (&pool.reserve0, &pool.reserve1)
    } else if offer == token1 {
        (&pool.reserve1, &pool.reserve0)
    } else {
        return Err(SwapperError::InvalidRoute);
    };
    let total_fee = pool
        .lp_fee
        .checked_add(pool.protocol_fee)
        .ok_or_else(|| SwapperError::ComputeQuoteError("STON.fi fee overflow".into()))?;
    if total_fee >= BPS_DENOMINATOR {
        return Err(SwapperError::ComputeQuoteError("STON.fi fee exceeds 100%".into()));
    }
    let amount_after_fee = (amount * BigUint::from(BPS_DENOMINATOR - total_fee)) / BigUint::from(BPS_DENOMINATOR);
    if amount_after_fee == BigUint::from(0u8) {
        return Ok(BigUint::from(0u8));
    }
    Ok((reserve_out * &amount_after_fee) / (reserve_in + amount_after_fee))
}

pub(super) fn apply_slippage(amount: &BigUint, bps: u32) -> BigUint {
    let slippage = BPS_DENOMINATOR - bps.min(BPS_DENOMINATOR);
    (amount * BigUint::from(slippage)) / BigUint::from(BPS_DENOMINATOR)
}

pub(super) fn scaled_next_min_ask_amount(first: &SwapSimulation, next: &SwapSimulation) -> Result<BigUint, SwapperError> {
    let first_ask = BigUint::from_str(&first.ask_units)?;
    if first_ask == BigUint::from(0u8) {
        return Err(SwapperError::InvalidRoute);
    }
    let first_min = BigUint::from_str(&first.min_ask_units)?;
    let next_min = BigUint::from_str(&next.min_ask_units)?;
    Ok((next_min * first_min) / first_ask)
}

#[cfg(test)]
mod tests {
    use super::super::constants::FALLBACK_ROUTERS;
    use super::*;
    use primitives::{AssetId, Chain, asset_constants::TON_USDT_TOKEN_ID};

    const PTON_WALLET: &str = "EQCSIMGBps_qzRG3uPYhON8bucyCtu0mYdL1-u4gSz77IBa3";
    const USDT_WALLET: &str = "EQCSLWJ9fY7b0A5OI72wxUp27l4fRlc6GvRBeFf6PiPpH4p3";

    fn pool_data() -> PoolData {
        PoolData {
            is_locked: false,
            reserve0: BigUint::from(3_809_436_784_065u64),
            reserve1: BigUint::from(1_784_561_670_122_756u64),
            token0_wallet: USDT_WALLET.to_string(),
            token1_wallet: PTON_WALLET.to_string(),
            lp_fee: 7,
            protocol_fee: 3,
        }
    }

    #[test]
    fn test_token_address() {
        assert_eq!(token_address(&SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton))), TON_PROXY_JETTON_ADDRESS);
        assert_eq!(
            token_address(&SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID))),
            TON_USDT_TOKEN_ID
        );
    }

    #[test]
    fn test_compute_amount_out() {
        let amount = BigUint::from(1_000_000_000u64);
        let out = compute_amount_out(&pool_data(), PTON_WALLET, &amount).unwrap();

        assert_eq!(out, BigUint::from(2_132_526u64));
        assert_eq!(apply_slippage(&out, 100), BigUint::from(2_111_200u64));
    }

    #[test]
    fn test_compute_amount_out_rejects_unknown_offer_wallet() {
        let amount = BigUint::from(1_000_000_000u64);
        assert_eq!(
            compute_amount_out(&pool_data(), "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c", &amount).unwrap_err(),
            SwapperError::InvalidRoute
        );
    }

    #[test]
    fn test_compute_amount_out_selects_pool_side() {
        let amount = BigUint::from(2_000_000u64);
        let out = compute_amount_out(&pool_data(), USDT_WALLET, &amount).unwrap();

        assert_eq!(out, BigUint::from(935_978_872u64));
    }

    #[test]
    fn test_static_candidates_for_ton_usdt() {
        let candidates = static_candidates(TON_PROXY_JETTON_ADDRESS, TON_USDT_TOKEN_ID);

        assert_eq!(candidates.len(), 2);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.pool_address == "EQCGScrZe1xbyWqWDvdI6mzP-GAcAWFv6ZXuaJOuSqemxku4" && candidate.lp_fee_bps == Some(7))
        );
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.pool_address == "EQD8TJ8xEWB1SpnRE4d89YO3jl0W0EiBnNS4IBaHaUmdfizE" && candidate.lp_fee_bps == Some(20))
        );
    }

    #[test]
    fn test_static_metadata_addresses_parse() {
        for router in FALLBACK_ROUTERS {
            Address::parse(router.address).unwrap();
            Address::parse(router.pton_wallet).unwrap();
        }
        for pool in STATIC_POOLS {
            Address::parse(pool.token0).unwrap();
            Address::parse(pool.token1).unwrap();
            Address::parse(pool.pool_address).unwrap();
            Address::parse(pool.router.address).unwrap();
            Address::parse(pool.token0_wallet).unwrap();
            Address::parse(pool.token1_wallet).unwrap();
        }
    }

    #[test]
    fn test_scaled_next_min_ask_amount() {
        let first = SwapSimulation::mock("", "", "260238", "257635");
        let next = SwapSimulation::mock("", "", "709", "702");

        assert_eq!(scaled_next_min_ask_amount(&first, &next).unwrap(), BigUint::from(694u32));
    }
}
