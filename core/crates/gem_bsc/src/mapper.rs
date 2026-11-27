use std::collections::{BTreeMap, HashSet};

use num_bigint::BigUint;
use primitives::{AssetId, Balance, Chain, DateTime, DelegationBase, DelegationState, DelegationValidator};

use crate::model::{BscDelegation, BscUndelegation, BscValidator};

pub fn map_validators(validators: Vec<BscValidator>, elected: &HashSet<String>) -> Vec<DelegationValidator> {
    validators.into_iter().map(|validator| map_validator(validator, elected)).collect()
}

pub fn map_delegations(delegations: Vec<BscDelegation>, undelegations: Vec<BscUndelegation>, now: DateTime) -> Vec<DelegationBase> {
    let asset_id = AssetId::from_chain(Chain::SmartChain);
    let (matured, pending): (Vec<_>, Vec<_>) = undelegations.into_iter().partition(|undelegation| unlock_date(undelegation).is_some_and(|unlock_date| unlock_date <= now));

    delegations
        .into_iter()
        .map(|delegation| map_delegation(&asset_id, delegation))
        .chain(pending.into_iter().chain(merge_by_validator(matured)).map(|undelegation| map_undelegation(&asset_id, undelegation, now)))
        .collect()
}

pub fn map_staking_balance(delegations: &[BscDelegation], undelegations: &[BscUndelegation]) -> Balance {
    let staked = delegations.iter().map(|delegation| &delegation.amount).sum();
    let pending = undelegations.iter().map(|undelegation| &undelegation.amount).sum();
    Balance::stake_balance(staked, pending, None)
}

fn map_validator(validator: BscValidator, elected: &HashSet<String>) -> DelegationValidator {
    let is_active = elected.contains(&validator.operator_address);
    DelegationValidator::stake(
        Chain::SmartChain,
        validator.operator_address,
        validator.moniker,
        is_active,
        validator.commission as f64 / 10_000.0,
        validator.apy as f64 / 100.0,
    )
}

fn map_delegation(asset_id: &AssetId, delegation: BscDelegation) -> DelegationBase {
    DelegationBase {
        asset_id: asset_id.clone(),
        delegation_id: delegation.delegator_address,
        validator_id: delegation.validator_address,
        balance: delegation.amount,
        shares: delegation.shares,
        rewards: BigUint::from(0u32),
        completion_date: None,
        state: DelegationState::Active,
    }
}

fn merge_by_validator(undelegations: Vec<BscUndelegation>) -> Vec<BscUndelegation> {
    let mut merged: BTreeMap<String, BscUndelegation> = BTreeMap::new();
    for undelegation in undelegations {
        merged
            .entry(undelegation.validator_address.clone())
            .and_modify(|validator_undelegation| {
                validator_undelegation.amount += &undelegation.amount;
                validator_undelegation.shares += &undelegation.shares;
                validator_undelegation.unlock_time = validator_undelegation.unlock_time.max(undelegation.unlock_time);
            })
            .or_insert(undelegation);
    }
    merged.into_values().collect()
}

fn unlock_date(undelegation: &BscUndelegation) -> Option<DateTime> {
    undelegation.unlock_time.map(|unlock_time| DateTime::from_timestamp(unlock_time as i64))
}

fn map_undelegation(asset_id: &AssetId, undelegation: BscUndelegation, now: DateTime) -> DelegationBase {
    let completion_date = unlock_date(&undelegation);
    let state = match &completion_date {
        Some(completion_date) if *completion_date > now => DelegationState::Deactivating,
        Some(_) => DelegationState::AwaitingWithdrawal,
        None => DelegationState::Deactivating,
    };

    DelegationBase {
        asset_id: asset_id.clone(),
        delegation_id: undelegation.unlock_time.map(|unlock_time| unlock_time.to_string()).unwrap_or_default(),
        validator_id: undelegation.validator_address,
        balance: undelegation.amount,
        shares: undelegation.shares,
        rewards: BigUint::from(0u32),
        completion_date,
        state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_SMARTCHAIN_STAKING_ADDRESS, mock_undelegation};

    #[test]
    fn test_map_validators() {
        let elected = "0x773760b0708a5Cc369c346993a0c225D8e4043B1";
        let candidate = "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A";

        assert_eq!(
            map_validators(vec![BscValidator::mock(elected), BscValidator::mock(candidate)], &HashSet::from([elected.to_string()])),
            vec![
                DelegationValidator::stake(Chain::SmartChain, elected.to_string(), "Kraken".to_string(), true, 0.1, 0.4),
                DelegationValidator::stake(Chain::SmartChain, candidate.to_string(), "Kraken".to_string(), false, 0.1, 0.4),
            ]
        );
    }

    #[test]
    fn test_map_delegations() {
        let validator = "0x773760b0708a5Cc369c346993a0c225D8e4043B1";
        let other_validator = "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A";
        let active = BscDelegation {
            delegator_address: TEST_SMARTCHAIN_STAKING_ADDRESS.to_string(),
            validator_address: validator.to_string(),
            amount: BigUint::from(2_000_000_000_000_000_000u64),
            shares: BigUint::from(1_900_000_000_000_000_000u64),
        };
        let amount = 1_000_000_000_000_000_000u64;

        let result = map_delegations(
            vec![active],
            vec![
                mock_undelegation(validator, amount, Some(4_102_444_800)),
                mock_undelegation(validator, amount, Some(1_716_417_585)),
                mock_undelegation(other_validator, 2 * amount, Some(1_710_000_000)),
                mock_undelegation(validator, 3 * amount, Some(1_700_000_000)),
                mock_undelegation(validator, amount, None),
            ],
            DateTime::from_timestamp(1_800_000_000),
        );

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].state, DelegationState::Active);
        assert_eq!(result[0].balance, BigUint::from(2_000_000_000_000_000_000u64));
        assert_eq!(result[0].completion_date, None);
        assert_eq!(result[0].delegation_id, TEST_SMARTCHAIN_STAKING_ADDRESS);
        assert_eq!(result[1].state, DelegationState::Deactivating);
        assert_eq!(result[1].completion_date.unwrap().timestamp(), 4_102_444_800);
        assert_eq!(result[1].delegation_id, "4102444800");
        assert_eq!(result[2].state, DelegationState::Deactivating);
        assert_eq!(result[2].completion_date, None);
        assert_eq!(result[2].delegation_id, "");
        assert_eq!(result[3].state, DelegationState::AwaitingWithdrawal);
        assert_eq!(result[3].validator_id, other_validator);
        assert_eq!(result[3].balance, BigUint::from(2 * amount));
        assert_eq!(result[3].delegation_id, "1710000000");
        assert_eq!(result[4].state, DelegationState::AwaitingWithdrawal);
        assert_eq!(result[4].validator_id, validator);
        assert_eq!(result[4].balance, BigUint::from(4 * amount));
        assert_eq!(result[4].completion_date.unwrap().timestamp(), 1_716_417_585);
        assert_eq!(result[4].delegation_id, "1716417585");
        assert_eq!(result.iter().map(|delegation| (&delegation.validator_id, &delegation.delegation_id)).collect::<HashSet<_>>().len(), 5);
    }
}
