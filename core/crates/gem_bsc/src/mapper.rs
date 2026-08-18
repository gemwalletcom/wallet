use std::error::Error;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use num_bigint::BigInt;
use num_bigint::BigUint;
use primitives::{AssetId, Balance, Chain, DelegationBase, DelegationState, DelegationValidator, StakeType};

use crate::stake_hub::{BscDelegation, BscUndelegation, BscValidator, STAKE_HUB_ADDRESS, encode_claim_call, encode_delegate_call, encode_redelegate_call, encode_undelegate_call};

const CLAIM_ALL_REQUEST_NUMBER: u64 = 0;

pub fn map_validators(validators: Vec<BscValidator>) -> Vec<DelegationValidator> {
    validators
        .into_iter()
        .map(|v| {
            DelegationValidator::stake(
                Chain::SmartChain,
                v.operator_address.clone(),
                v.moniker,
                !v.jailed,
                v.commission as f64 / 10000.0,
                v.apy as f64 / 100.0,
            )
        })
        .collect()
}

pub fn map_delegations(delegations: Vec<BscDelegation>, undelegations: Vec<BscUndelegation>) -> Vec<DelegationBase> {
    let mut result = Vec::new();

    let asset_id = AssetId {
        chain: Chain::SmartChain,
        token_id: None,
    };

    for delegation in delegations {
        if let Ok(balance) = BigUint::from_str(&delegation.amount) {
            let shares = BigUint::from_str(&delegation.shares).unwrap_or_else(|_| BigUint::from(0u32));

            result.push(DelegationBase {
                asset_id: asset_id.clone(),
                delegation_id: delegation.delegator_address.clone(),
                validator_id: delegation.validator_address,
                balance,
                shares,
                rewards: BigUint::from(0u32),
                completion_date: None,
                state: DelegationState::Active,
            });
        }
    }

    for undelegation in undelegations {
        if let Ok(balance) = BigUint::from_str(&undelegation.amount) {
            let shares = BigUint::from_str(&undelegation.shares).unwrap_or_else(|_| BigUint::from(0u32));

            let completion_date = undelegation
                .unlock_time
                .parse::<i64>()
                .ok()
                .and_then(|unlock_time| DateTime::from_timestamp(unlock_time, 0));

            let state = if let Some(ref completion_date) = completion_date {
                if *completion_date > Utc::now() {
                    DelegationState::Deactivating
                } else {
                    DelegationState::AwaitingWithdrawal
                }
            } else {
                DelegationState::Deactivating
            };

            result.push(DelegationBase {
                asset_id: asset_id.clone(),
                delegation_id: undelegation.delegator_address.clone(),
                validator_id: undelegation.validator_address,
                balance,
                shares,
                rewards: BigUint::from(0u32),
                completion_date,
                state,
            });
        }
    }

    result
}

pub fn map_staking_balance(delegations: &[BscDelegation], undelegations: &[BscUndelegation]) -> Balance {
    let staked = delegations
        .iter()
        .filter_map(|d| BigUint::from_str(&d.amount).ok())
        .fold(BigUint::from(0u32), |acc, amount| acc + amount);

    let pending = undelegations
        .iter()
        .filter_map(|u| BigUint::from_str(&u.amount).ok())
        .fold(BigUint::from(0u32), |acc, amount| acc + amount);

    Balance::stake_balance(staked, pending, None)
}

pub fn encode_stake_hub(stake_type: &StakeType, amount: &BigInt) -> Result<(&'static str, Vec<u8>, BigInt), Box<dyn Error + Send + Sync>> {
    let data = match stake_type {
        StakeType::Stake(validator) => encode_delegate_call(&validator.id, false).map_err(|e| e.to_string())?,
        StakeType::Unstake(delegation) => {
            let amount_uint = amount.magnitude().clone();
            let amount_shares = amount_uint * &delegation.base.shares / &delegation.base.balance;

            encode_undelegate_call(&delegation.validator.id, &amount_shares.to_string()).map_err(|e| e.to_string())?
        }
        StakeType::Redelegate(redelegate_data) => {
            let amount_uint = amount.magnitude().clone();
            let amount_shares = amount_uint * &redelegate_data.delegation.base.shares / &redelegate_data.delegation.base.balance;

            encode_redelegate_call(
                &redelegate_data.delegation.validator.id,
                &redelegate_data.to_validator.id,
                &amount_shares.to_string(),
                false,
            )
            .map_err(|e| e.to_string())?
        }
        StakeType::Withdraw(delegation) => encode_claim_call(&delegation.validator.id, CLAIM_ALL_REQUEST_NUMBER).map_err(|e| e.to_string())?,
        StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => return Err("Unsupported stake type for StakeHub".into()),
    };
    let value = match stake_type {
        StakeType::Stake(_) => amount.clone(),
        _ => BigInt::from(0),
    };
    Ok((STAKE_HUB_ADDRESS, data, value))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex;
    use num_bigint::{BigInt, BigUint};
    use primitives::{DelegationState, RedelegateData, StakeType};

    use super::*;
    use crate::testkit::mock_delegation;

    #[test]
    fn test_encode_stake_hub() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let one_bnb = BigInt::from(1_000_000_000_000_000_000u64);
        let validator = mock_delegation(
            "0x773760b0708a5Cc369c346993a0c225D8e4043B1",
            DelegationState::Active,
            2_000_000_000_000_000_000,
            1_900_000_000_000_000_000,
        )
        .validator;

        let (to, data, value) = encode_stake_hub(&StakeType::Stake(validator.clone()), &one_bnb)?;
        assert_eq!(to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&data[0..4]), "982ef0a7");
        assert_eq!(value, one_bnb);

        let unstake = mock_delegation(
            "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A",
            DelegationState::Active,
            2_000_000_000_000_000_000,
            1_900_000_000_000_000_000,
        );
        let (to, data, value) = encode_stake_hub(&StakeType::Unstake(unstake.clone()), &one_bnb)?;
        assert_eq!(to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&data[0..4]), "4d99dd16");
        assert_eq!(value, BigInt::from(0));

        let redelegate = RedelegateData {
            delegation: unstake,
            to_validator: validator,
        };
        let (to, data, value) = encode_stake_hub(&StakeType::Redelegate(redelegate), &one_bnb)?;
        assert_eq!(to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&data[0..4]), "59491871");
        assert_eq!(value, BigInt::from(0));

        let withdraw = mock_delegation(
            "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A",
            DelegationState::AwaitingWithdrawal,
            1_000_000_000_000_000_000,
            1_000_000_000_000_000_000,
        );
        let (to, data, value) = encode_stake_hub(&StakeType::Withdraw(withdraw), &BigInt::from(0))?;
        assert_eq!(to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&data[0..4]), "aad3ec96");
        assert_eq!(value, BigInt::from(0));

        Ok(())
    }

    #[test]
    fn test_map_delegations() {
        let active = BscDelegation {
            delegator_address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4".to_string(),
            validator_address: "0x773760b0708a5Cc369c346993a0c225D8e4043B1".to_string(),
            amount: "2000000000000000000".to_string(),
            shares: "1900000000000000000".to_string(),
        };

        let result = map_delegations(
            vec![active],
            vec![
                BscUndelegation::mock_with_unlock_time("4102444800"),
                BscUndelegation::mock_with_unlock_time("1716417585"),
                BscUndelegation::mock_with_unlock_time("invalid"),
            ],
        );

        assert_eq!(result.len(), 4);
        assert_eq!(result[0].state, DelegationState::Active);
        assert_eq!(result[0].balance, BigUint::from(2_000_000_000_000_000_000u64));
        assert_eq!(result[0].completion_date, None);
        assert_eq!(result[1].state, DelegationState::Deactivating);
        assert_eq!(result[1].completion_date.unwrap().timestamp(), 4102444800);
        assert_eq!(result[2].state, DelegationState::AwaitingWithdrawal);
        assert_eq!(result[2].completion_date.unwrap().timestamp(), 1716417585);
        assert_eq!(result[3].state, DelegationState::Deactivating);
        assert_eq!(result[3].completion_date, None);
    }
}
