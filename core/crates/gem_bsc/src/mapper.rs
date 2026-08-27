use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::{AssetId, Balance, Chain, DelegationBase, DelegationState, DelegationValidator};

use crate::model::{BscDelegation, BscUndelegation, BscValidator};

pub fn map_validators(validators: Vec<BscValidator>) -> Vec<DelegationValidator> {
    validators.into_iter().map(map_validator).collect()
}

pub fn map_delegations(delegations: Vec<BscDelegation>, undelegations: Vec<BscUndelegation>) -> Vec<DelegationBase> {
    let asset_id = AssetId::from_chain(Chain::SmartChain);
    delegations
        .into_iter()
        .map(|delegation| map_delegation(&asset_id, delegation))
        .chain(undelegations.into_iter().map(|undelegation| map_undelegation(&asset_id, undelegation)))
        .collect()
}

pub fn map_staking_balance(delegations: &[BscDelegation], undelegations: &[BscUndelegation]) -> Balance {
    let staked = delegations.iter().map(|delegation| &delegation.amount).sum();
    let pending = undelegations.iter().map(|undelegation| &undelegation.amount).sum();
    Balance::stake_balance(staked, pending, None)
}

fn map_validator(validator: BscValidator) -> DelegationValidator {
    DelegationValidator::stake(
        Chain::SmartChain,
        validator.operator_address,
        validator.moniker,
        !validator.jailed,
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

fn map_undelegation(asset_id: &AssetId, undelegation: BscUndelegation) -> DelegationBase {
    let completion_date = undelegation.unlock_time.and_then(|unlock_time| DateTime::from_timestamp(unlock_time as i64, 0));
    let state = match &completion_date {
        Some(completion_date) if *completion_date > Utc::now() => DelegationState::Deactivating,
        Some(_) => DelegationState::AwaitingWithdrawal,
        None => DelegationState::Deactivating,
    };

    DelegationBase {
        asset_id: asset_id.clone(),
        delegation_id: undelegation.delegator_address,
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
    fn test_map_delegations() {
        let active = BscDelegation {
            delegator_address: TEST_SMARTCHAIN_STAKING_ADDRESS.to_string(),
            validator_address: "0x773760b0708a5Cc369c346993a0c225D8e4043B1".to_string(),
            amount: BigUint::from(2_000_000_000_000_000_000u64),
            shares: BigUint::from(1_900_000_000_000_000_000u64),
        };

        let result = map_delegations(
            vec![active],
            vec![mock_undelegation(Some(4_102_444_800)), mock_undelegation(Some(1_716_417_585)), mock_undelegation(None)],
        );

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].state, DelegationState::Active);
        assert_eq!(result[0].balance, BigUint::from(2_000_000_000_000_000_000u64));
        assert_eq!(result[0].completion_date, None);
        assert_eq!(result[1].state, DelegationState::Deactivating);
        assert_eq!(result[1].completion_date.unwrap().timestamp(), 4_102_444_800);
        assert_eq!(result[2].state, DelegationState::AwaitingWithdrawal);
        assert_eq!(result[2].completion_date.unwrap().timestamp(), 1_716_417_585);
        assert_eq!(result[3].state, DelegationState::Deactivating);
        assert_eq!(result[3].completion_date, None);
    }
}
