use super::{
    constants::{RouterInfo, STATIC_POOLS, StaticPool},
    model::Router,
};
use crate::{SwapperError, SwapperQuoteAsset};
use gem_ton::{address::Address, constants::TON_PROXY_JETTON_ADDRESS};
use num_bigint::BigUint;
use primitives::swap::HUNDRED_PERCENT_IN_BPS;

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
    pub referral_fee: Option<u32>,
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
    STATIC_POOLS.iter().filter(|pool| static_pool_matches(pool, from_token, to_token)).map(DiscoveredPool::from_static).collect()
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

pub(super) fn compute_amount_out(pool: &PoolData, offer_wallet: &str, amount: &BigUint, referral_bps: u32) -> Result<BigUint, SwapperError> {
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
    let referral_bps = pool.referral_fee.unwrap_or(referral_bps);
    if pool.lp_fee.saturating_add(pool.protocol_fee).saturating_add(referral_bps) >= HUNDRED_PERCENT_IN_BPS {
        return Err(SwapperError::ComputeQuoteError("STON.fi fee exceeds 100%".into()));
    }
    let divider = BigUint::from(HUNDRED_PERCENT_IN_BPS);
    let amount_in_with_fee = amount * (HUNDRED_PERCENT_IN_BPS - pool.lp_fee);
    let base_out = reserve_out * &amount_in_with_fee / (reserve_in * &divider + &amount_in_with_fee);
    let fee_out = |bps: u32| (&base_out * bps + &divider - 1u32) / &divider;
    let fees_out = fee_out(pool.protocol_fee) + fee_out(referral_bps);
    Ok(if base_out > fees_out { &base_out - fees_out } else { BigUint::ZERO })
}

#[cfg(test)]
mod tests {
    use super::super::constants::DISCOVERY_ROUTERS;
    use super::super::testkit::{TEST_PTON_WALLET, TEST_USDT_WALLET};
    use super::*;
    use crate::fees::subtract_bps;
    use primitives::{AssetId, Chain, asset_constants::TON_USDT_TOKEN_ID};

    #[test]
    fn test_token_address() {
        assert_eq!(token_address(&SwapperQuoteAsset::from(AssetId::from_chain(Chain::Ton))), TON_PROXY_JETTON_ADDRESS);
        assert_eq!(token_address(&SwapperQuoteAsset::from(AssetId::from_token(Chain::Ton, TON_USDT_TOKEN_ID))), TON_USDT_TOKEN_ID);
    }

    #[test]
    fn test_compute_amount_out() {
        let amount = BigUint::from(1_000_000_000u64);
        let out = compute_amount_out(&PoolData::mock(), TEST_PTON_WALLET, &amount, 0).unwrap();

        assert_eq!(out, BigUint::from(2_132_526u64));
        assert_eq!(compute_amount_out(&PoolData::mock(), TEST_PTON_WALLET, &amount, 50).unwrap(), BigUint::from(2_121_860u64));
        assert_eq!(
            compute_amount_out(&PoolData { referral_fee: Some(10), ..PoolData::mock() }, TEST_PTON_WALLET, &amount, 50).unwrap(),
            compute_amount_out(&PoolData::mock(), TEST_PTON_WALLET, &amount, 10).unwrap()
        );
        assert_eq!(subtract_bps(&out, 100), BigUint::from(2_111_200u64));
    }

    #[test]
    fn test_compute_amount_out_rejects_unknown_offer_wallet() {
        let amount = BigUint::from(1_000_000_000u64);
        assert_eq!(compute_amount_out(&PoolData::mock(), "EQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM9c", &amount, 0).unwrap_err(), SwapperError::InvalidRoute);
    }

    #[test]
    fn test_compute_amount_out_selects_pool_side() {
        let amount = BigUint::from(2_000_000u64);
        let out = compute_amount_out(&PoolData::mock(), TEST_USDT_WALLET, &amount, 0).unwrap();

        assert_eq!(out, BigUint::from(935_979_069u64));
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
        for router in DISCOVERY_ROUTERS {
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
}
