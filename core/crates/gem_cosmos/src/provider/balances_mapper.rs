use crate::models::staking::{Delegation, Rewards, UnbondingDelegation};
use num_bigint::{BigInt, BigUint};
use number_formatter::BigNumberFormatter;
use primitives::AssetBalance;
use std::str::FromStr;

pub fn map_balance_staking(delegations: Vec<Delegation>, unbonding: Vec<UnbondingDelegation>, rewards: Rewards, chain: primitives::Chain, denom: &str) -> AssetBalance {
    let staked = delegations
        .iter()
        .filter(|d| d.balance.denom == denom)
        .filter_map(|d| BigNumberFormatter::value_from_amount(&d.balance.amount, 0).ok())
        .filter_map(|v| BigInt::from_str(&v).ok())
        .fold(BigInt::from(0), |acc, amount| acc + amount);

    let pending = unbonding.iter().flat_map(|u| &u.entries).map(|entry| BigInt::from(entry.balance.clone())).fold(BigInt::from(0), |acc, amount| acc + amount);

    let rewards = rewards
        .rewards
        .iter()
        .flat_map(|r| &r.reward)
        .filter(|r| r.denom == denom)
        .filter_map(|r| {
            let integer_part = r.amount.split('.').next().unwrap_or("0");
            BigInt::from_str(integer_part).ok()
        })
        .fold(BigInt::from(0), |acc, amount| acc + amount);

    AssetBalance::new_staking(
        chain.as_asset_id(),
        BigUint::try_from(staked).unwrap_or_default(),
        BigUint::try_from(pending).unwrap_or_default(),
        BigUint::try_from(rewards).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::staking::{Delegations, UnbondingDelegations};
    use primitives::Chain;

    #[test]
    fn test_map_balance_staking() {
        let delegations: Vec<Delegation> = serde_json::from_str::<Delegations>(include_str!("../../testdata/staking_delegations.json")).unwrap().delegation_responses;
        let unbonding: Vec<UnbondingDelegation> = Vec::new();
        let rewards: Rewards = serde_json::from_str(include_str!("../../testdata/staking_rewards.json")).unwrap();

        let result = map_balance_staking(delegations, unbonding, rewards, Chain::Cosmos, "uatom");

        assert_eq!(result.asset_id.to_string(), "cosmos");
        assert_eq!(result.balance.staked, BigUint::from(10250000_u64));
        assert_eq!(result.balance.pending, BigUint::from(0u32));
        assert_eq!(result.balance.rewards, BigUint::from(307413_u64));
    }

    #[test]
    fn test_the_pending_total_keeps_every_digit_of_a_large_unbonding_principal() {
        let delegations: Vec<Delegation> = serde_json::from_str::<Delegations>(include_str!("../../testdata/staking_delegations.json")).unwrap().delegation_responses;
        let unbonding: Vec<UnbondingDelegation> = serde_json::from_str::<UnbondingDelegations>(include_str!("../../testdata/staking_unbonding_delegations_large.json"))
            .unwrap()
            .unbonding_responses;

        let result = map_balance_staking(delegations, unbonding, Rewards { rewards: vec![] }, Chain::Cosmos, "uatom");

        assert_eq!(result.balance.pending.to_string(), "9007199254740993");
    }
}
