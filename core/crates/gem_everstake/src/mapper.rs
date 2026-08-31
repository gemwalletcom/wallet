use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::{AssetId, Balance, Chain, DelegationBase, DelegationState};

use crate::constants::EVERSTAKE_POOL_ADDRESS;
use crate::contracts::WithdrawRequest;

fn delegation_id(validator_id: &str, state: DelegationState) -> String {
    format!("{}-{}", validator_id, state.as_ref())
}

pub fn map_withdraw_request_to_delegations(withdraw_request: &WithdrawRequest) -> Vec<DelegationBase> {
    let requested = u256_to_biguint(&withdraw_request.requested);
    let ready_for_claim = u256_to_biguint(&withdraw_request.readyForClaim);

    let mut delegations = Vec::new();
    let pending_amount = if requested > ready_for_claim { requested - &ready_for_claim } else { BigUint::zero() };

    if pending_amount > BigUint::zero() {
        delegations.push(map_balance_to_delegation(&pending_amount, &BigUint::zero(), DelegationState::Deactivating));
    }

    if ready_for_claim > BigUint::zero() {
        delegations.push(map_balance_to_delegation(&ready_for_claim, &BigUint::zero(), DelegationState::AwaitingWithdrawal));
    }

    delegations
}

pub fn map_staking_balance(delegations: &[DelegationBase]) -> Balance {
    let mut staked = BigUint::zero();
    let mut rewards = BigUint::zero();
    let mut pending = BigUint::zero();
    for delegation in delegations {
        match delegation.state {
            DelegationState::Active => {
                staked += &delegation.balance;
                rewards += &delegation.rewards;
            }
            DelegationState::Activating | DelegationState::Deactivating | DelegationState::AwaitingWithdrawal => {
                pending += &delegation.balance;
            }
            DelegationState::Pending | DelegationState::Inactive => {}
        }
    }
    Balance::stake_balance(staked, pending, Some(rewards))
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
    use alloy_primitives::U256;
    use num_bigint::BigUint;

    use super::*;

    #[test]
    fn test_map_withdraw_request_to_delegations() {
        let withdraw_request = WithdrawRequest {
            requested: U256::from_str_radix("1000000000000000000", 10).unwrap(),
            readyForClaim: U256::from_str_radix("500000000000000000", 10).unwrap(),
        };

        let delegations = map_withdraw_request_to_delegations(&withdraw_request);

        assert_eq!(delegations.len(), 2);

        let pending = delegations.iter().find(|d| d.state == DelegationState::Deactivating).unwrap();
        assert_eq!(pending.balance, BigUint::from(500000000000000000_u64));
        assert_eq!(pending.delegation_id, delegation_id(EVERSTAKE_POOL_ADDRESS, DelegationState::Deactivating));

        let awaiting = delegations.iter().find(|d| d.state == DelegationState::AwaitingWithdrawal).unwrap();
        assert_eq!(awaiting.balance, BigUint::from(500000000000000000_u64));
        assert_eq!(awaiting.delegation_id, delegation_id(EVERSTAKE_POOL_ADDRESS, DelegationState::AwaitingWithdrawal));

        let balance = map_staking_balance(&delegations);
        assert_eq!(balance.pending, BigUint::from(1_000_000_000_000_000_000u64));
        assert_eq!(balance.staked, BigUint::from(0u32));
        assert_eq!(balance.rewards, BigUint::from(0u32));
    }
}
