use std::error::Error;

use alloy_sol_types::SolCall;
use gem_evm::u256::bigint_to_u256;
use num_bigint::{BigInt, BigUint};
use num_traits::Zero;
use primitives::{AssetId, Chain, DelegationBase, DelegationState, StakeType};

use crate::constants::{DEFAULT_ALLOWED_INTERCHANGE_NUM, EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS, EVERSTAKE_SOURCE};
use crate::contracts::{IAccounting, IPool, WithdrawRequest};

pub fn encode_everstake(stake_type: &StakeType, amount: &BigInt) -> Result<(&'static str, Vec<u8>, BigInt), Box<dyn Error + Send + Sync>> {
    match stake_type {
        StakeType::Stake(_) => Ok((EVERSTAKE_POOL_ADDRESS, IPool::stakeCall { source: EVERSTAKE_SOURCE }.abi_encode(), amount.clone())),
        StakeType::Unstake(_) => {
            let value = bigint_to_u256(amount)?;
            let data = IPool::unstakeCall {
                value,
                allowedInterchangeNum: DEFAULT_ALLOWED_INTERCHANGE_NUM,
                source: EVERSTAKE_SOURCE,
            }
            .abi_encode();
            Ok((EVERSTAKE_POOL_ADDRESS, data, BigInt::from(0)))
        }
        StakeType::Withdraw(_) => Ok((EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::claimWithdrawRequestCall {}.abi_encode(), BigInt::from(0))),
        StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => Err("Unsupported stake type for Everstake".into()),
    }
}

fn delegation_id(validator_id: &str, state: DelegationState) -> String {
    format!("{}-{}", validator_id, state.as_ref())
}

pub fn map_withdraw_request_to_delegations(withdraw_request: &WithdrawRequest) -> Vec<DelegationBase> {
    let requested = BigUint::from_bytes_be(&withdraw_request.requested.to_be_bytes::<32>());
    let ready_for_claim = BigUint::from_bytes_be(&withdraw_request.readyForClaim.to_be_bytes::<32>());

    let mut delegations = Vec::new();
    let pending_amount = if requested > ready_for_claim { requested - &ready_for_claim } else { BigUint::zero() };

    let asset_id = AssetId::from_chain(Chain::Ethereum);
    let validator_id = EVERSTAKE_POOL_ADDRESS;

    if pending_amount > BigUint::zero() {
        delegations.push(DelegationBase {
            asset_id: asset_id.clone(),
            state: DelegationState::Deactivating,
            balance: pending_amount,
            shares: BigUint::zero(),
            rewards: BigUint::zero(),
            completion_date: None,
            delegation_id: delegation_id(validator_id, DelegationState::Deactivating),
            validator_id: validator_id.to_string(),
        });
    }

    if ready_for_claim > BigUint::zero() {
        delegations.push(DelegationBase {
            asset_id,
            state: DelegationState::AwaitingWithdrawal,
            balance: ready_for_claim,
            shares: BigUint::zero(),
            rewards: BigUint::zero(),
            completion_date: None,
            delegation_id: delegation_id(validator_id, DelegationState::AwaitingWithdrawal),
            validator_id: validator_id.to_string(),
        });
    }

    delegations
}

pub fn map_balance_to_delegation(balance: &BigUint, restaked_reward: &BigUint, state: DelegationState) -> DelegationBase {
    DelegationBase {
        asset_id: AssetId::from_chain(Chain::Ethereum),
        state,
        balance: balance.clone(),
        shares: BigUint::zero(),
        rewards: restaked_reward.clone(),
        completion_date: None,
        delegation_id: delegation_id(EVERSTAKE_POOL_ADDRESS, state),
        validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{U256, hex};
    use num_bigint::BigInt;

    use super::*;
    use crate::testkit::mock_delegation;

    #[test]
    fn test_map_withdraw_request_to_delegations() {
        let withdraw_request = WithdrawRequest {
            requested: U256::from_str_radix("1000000000000000000", 10).unwrap(),
            readyForClaim: U256::from_str_radix("500000000000000000", 10).unwrap(),
        };

        let delegations = map_withdraw_request_to_delegations(&withdraw_request);

        assert_eq!(delegations.len(), 2);

        let pending = delegations.iter().find(|d| matches!(d.state, DelegationState::Deactivating)).unwrap();
        assert_eq!(pending.balance, BigUint::from(500000000000000000_u64));
        assert_eq!(pending.delegation_id, delegation_id(EVERSTAKE_POOL_ADDRESS, DelegationState::Deactivating));

        let awaiting = delegations.iter().find(|d| matches!(d.state, DelegationState::AwaitingWithdrawal)).unwrap();
        assert_eq!(awaiting.balance, BigUint::from(500000000000000000_u64));
        assert_eq!(awaiting.delegation_id, delegation_id(EVERSTAKE_POOL_ADDRESS, DelegationState::AwaitingWithdrawal));
    }

    #[test]
    fn test_encode_everstake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (to, data, value) = encode_everstake(
            &StakeType::Stake(mock_delegation(DelegationState::Active).validator),
            &BigInt::from(1_000_000_000_000_000_000u64),
        )?;
        assert_eq!(to, EVERSTAKE_POOL_ADDRESS);
        assert_eq!(hex::encode(&data), "3a29dbae0000000000000000000000000000000000000000000000000000000000000017");
        assert_eq!(value, BigInt::from(1_000_000_000_000_000_000u64));

        let (to, data, value) = encode_everstake(&StakeType::Unstake(mock_delegation(DelegationState::Active)), &BigInt::from(500_000_000_000_000_000u64))?;
        assert_eq!(to, EVERSTAKE_POOL_ADDRESS);
        assert_eq!(
            hex::encode(&data),
            "76ec871c00000000000000000000000000000000000000000000000006f05b59d3b2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000017"
        );
        assert_eq!(value, BigInt::from(0));

        let (to, data, value) = encode_everstake(&StakeType::Withdraw(mock_delegation(DelegationState::AwaitingWithdrawal)), &BigInt::from(0))?;
        assert_eq!(to, EVERSTAKE_ACCOUNTING_ADDRESS);
        assert_eq!(hex::encode(&data), "33986ffa");
        assert_eq!(value, BigInt::from(0));

        Ok(())
    }
}
