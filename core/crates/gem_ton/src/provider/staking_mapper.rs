use num_bigint::BigUint;
use primitives::{AssetBalance, AssetId, Chain, DelegationBase, DelegationState, DelegationValidator};

use crate::{constants::TONSTAKERS_POOL_ADDRESS, models::PoolFullData};

const TONSTAKERS_NAME: &str = "Tonstakers";
const SHARE_BASIS: f64 = 16_777_216.0;
const VALIDATION_ROUND_SECONDS: f64 = 65_536.0;
const SECONDS_PER_YEAR: f64 = 365.25 * 24.0 * 60.0 * 60.0;
const ROUNDS_PER_LOAN: f64 = 2.0;

pub fn map_staking_apy(pool: &PoolFullData) -> f64 {
    let rounds_per_year = SECONDS_PER_YEAR / VALIDATION_ROUND_SECONDS;
    share_ratio(pool.interest_rate) * (1.0 - share_ratio(pool.governance_fee)) * rounds_per_year / ROUNDS_PER_LOAN * 100.0
}

pub fn map_validator(pool: &PoolFullData) -> DelegationValidator {
    DelegationValidator::stake(
        Chain::Ton,
        TONSTAKERS_POOL_ADDRESS.to_string(),
        TONSTAKERS_NAME.to_string(),
        true,
        share_ratio(pool.governance_fee) * 100.0,
        map_staking_apy(pool),
    )
}

pub fn map_delegations(staked_shares: BigUint, pool: &PoolFullData, deposit_bills: &[BigUint], withdrawal_bills: &[BigUint]) -> Vec<DelegationBase> {
    let withdrawal_shares: BigUint = withdrawal_bills.iter().sum();
    let deposit_value: BigUint = deposit_bills.iter().sum();

    [
        (DelegationState::Active, shares_to_value(&staked_shares, pool), staked_shares),
        (DelegationState::Activating, deposit_value, BigUint::ZERO),
        (DelegationState::Deactivating, shares_to_value(&withdrawal_shares, pool), withdrawal_shares),
    ]
    .into_iter()
    .filter(|(_, balance, _)| *balance > BigUint::ZERO)
    .map(|(state, balance, shares)| map_delegation(state, balance, shares))
    .collect()
}

pub fn map_balance_staking(delegations: &[DelegationBase]) -> AssetBalance {
    let (staked, pending) = delegations.iter().fold((BigUint::ZERO, BigUint::ZERO), |(staked, pending), delegation| match delegation.state {
        DelegationState::Active => (staked + &delegation.balance, pending),
        _ => (staked, pending + &delegation.balance),
    });
    AssetBalance::new_staking(AssetId::from_chain(Chain::Ton), staked, pending, BigUint::ZERO)
}

fn map_delegation(state: DelegationState, balance: BigUint, shares: BigUint) -> DelegationBase {
    DelegationBase {
        asset_id: AssetId::from_chain(Chain::Ton),
        state,
        balance,
        shares,
        rewards: BigUint::ZERO,
        completion_date: None,
        delegation_id: format!("{}_{}", state.as_ref().to_lowercase(), TONSTAKERS_POOL_ADDRESS),
        validator_id: TONSTAKERS_POOL_ADDRESS.to_string(),
    }
}

fn shares_to_value(shares: &BigUint, pool: &PoolFullData) -> BigUint {
    if pool.supply == BigUint::ZERO {
        return BigUint::ZERO;
    }
    shares * &pool.total_balance / &pool.supply
}

fn share_ratio(value: u32) -> f64 {
    f64::from(value) / SHARE_BASIS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_staking_apy() {
        assert_eq!((map_staking_apy(&PoolFullData::mock()) * 100.0).round() / 100.0, 11.41);
        assert_eq!(map_staking_apy(&PoolFullData { interest_rate: 0, ..PoolFullData::mock() }), 0.0);
        assert_eq!((map_validator(&PoolFullData::mock()).commission * 100.0).round() / 100.0, 16.0);
    }

    #[test]
    fn test_map_delegations() {
        let pool = PoolFullData {
            total_balance: BigUint::from(1_100u32),
            supply: BigUint::from(1_000u32),
            ..PoolFullData::mock()
        };

        assert_eq!(
            map_delegations(BigUint::from(500u32), &pool, &[BigUint::from(30u32)], &[BigUint::from(100u32), BigUint::from(200u32)]),
            vec![
                map_delegation(DelegationState::Active, BigUint::from(550u32), BigUint::from(500u32)),
                map_delegation(DelegationState::Activating, BigUint::from(30u32), BigUint::ZERO),
                map_delegation(DelegationState::Deactivating, BigUint::from(330u32), BigUint::from(300u32)),
            ]
        );
        assert_eq!(map_delegations(BigUint::ZERO, &pool, &[], &[]), vec![]);
        assert_eq!(map_delegations(BigUint::from(500u32), &PoolFullData { supply: BigUint::ZERO, ..pool }, &[], &[]), vec![]);
    }

    #[test]
    fn test_map_balance_staking() {
        let delegations = vec![
            map_delegation(DelegationState::Active, BigUint::from(550u32), BigUint::from(500u32)),
            map_delegation(DelegationState::Activating, BigUint::from(30u32), BigUint::ZERO),
            map_delegation(DelegationState::Deactivating, BigUint::from(330u32), BigUint::from(300u32)),
        ];

        assert_eq!(
            map_balance_staking(&delegations),
            AssetBalance::new_staking(AssetId::from_chain(Chain::Ton), BigUint::from(550u32), BigUint::from(360u32), BigUint::ZERO)
        );
    }
}
