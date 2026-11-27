use gem_evm::u256::u256_to_biguint;
use num_bigint::BigUint;
use num_traits::Zero;
use primitives::{AssetId, Balance, Chain, DelegationBase, DelegationState};

use crate::constants::EVERSTAKE_POOL_ADDRESS;
use crate::contracts::WithdrawRequest;
use crate::models::AccountState;

fn delegation_id(validator_id: &str, state: DelegationState) -> String {
    format!("{}-{}", validator_id, state.as_ref())
}

pub fn map_account_state_to_delegations(state: &AccountState) -> Vec<DelegationBase> {
    let mut delegations = Vec::new();

    if state.autocompound_balance > BigUint::zero() {
        delegations.push(map_balance_to_delegation(&state.autocompound_balance, &state.restaked_reward, DelegationState::Active));
    }

    let pending_balance = &state.pending_balance + &state.pending_deposited_balance;
    if pending_balance > BigUint::zero() {
        delegations.push(map_balance_to_delegation(&pending_balance, &BigUint::zero(), DelegationState::Activating));
    }

    delegations.extend(map_withdraw_request_to_delegations(&state.withdraw_request));
    delegations
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

pub fn map_staking_balance(state: &AccountState) -> Balance {
    let pending = &state.pending_balance + &state.pending_deposited_balance + u256_to_biguint(&state.withdraw_request.requested);
    Balance::stake_balance(state.deposited_balance.clone(), pending, Some(state.restaked_reward.clone()))
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

    use super::*;

    #[test]
    fn test_map_withdraw_request_to_delegations() {
        let requested = 1_000_000_000_000_000_000u64;
        let cases = [
            (
                300_000_000_000_000_000,
                vec![
                    map_balance_to_delegation(&BigUint::from(700_000_000_000_000_000u64), &BigUint::zero(), DelegationState::Deactivating),
                    map_balance_to_delegation(&BigUint::from(300_000_000_000_000_000u64), &BigUint::zero(), DelegationState::AwaitingWithdrawal),
                ],
            ),
            (requested, vec![map_balance_to_delegation(&BigUint::from(requested), &BigUint::zero(), DelegationState::AwaitingWithdrawal)]),
            (0, vec![map_balance_to_delegation(&BigUint::from(requested), &BigUint::zero(), DelegationState::Deactivating)]),
        ];

        for (ready_for_claim, expected) in cases {
            let delegations = map_withdraw_request_to_delegations(&WithdrawRequest {
                requested: U256::from(requested),
                readyForClaim: U256::from(ready_for_claim),
            });

            assert_eq!(delegations, expected);
        }
    }

    #[test]
    fn test_map_account_state_to_delegations() {
        let state = AccountState {
            deposited_balance: BigUint::from(5u32),
            pending_balance: BigUint::from(2u32),
            pending_deposited_balance: BigUint::from(1u32),
            withdraw_request: WithdrawRequest {
                requested: U256::from(4),
                readyForClaim: U256::from(1),
            },
            restaked_reward: BigUint::from(7u32),
            autocompound_balance: BigUint::from(12u32),
        };

        assert_eq!(
            map_account_state_to_delegations(&state),
            vec![
                map_balance_to_delegation(&BigUint::from(12u32), &BigUint::from(7u32), DelegationState::Active),
                map_balance_to_delegation(&BigUint::from(3u32), &BigUint::zero(), DelegationState::Activating),
                map_balance_to_delegation(&BigUint::from(3u32), &BigUint::zero(), DelegationState::Deactivating),
                map_balance_to_delegation(&BigUint::from(1u32), &BigUint::zero(), DelegationState::AwaitingWithdrawal),
            ]
        );
        assert_eq!(map_staking_balance(&state), Balance::stake_balance(BigUint::from(5u32), BigUint::from(7u32), Some(BigUint::from(7u32))));
        assert_eq!(
            map_account_state_to_delegations(&AccountState {
                deposited_balance: BigUint::zero(),
                pending_balance: BigUint::zero(),
                pending_deposited_balance: BigUint::zero(),
                withdraw_request: WithdrawRequest {
                    requested: U256::ZERO,
                    readyForClaim: U256::ZERO,
                },
                restaked_reward: BigUint::zero(),
                autocompound_balance: BigUint::zero(),
            }),
            vec![]
        );
    }

    #[test]
    fn test_map_account_state_active_balance_is_autocompound_balance() {
        let deposited = BigUint::from(18_991_489_398_350_502_488u128);
        let restaked = BigUint::from(879_162_057_515_466_311u128);
        let autocompound_balance = BigUint::from(19_870_651_455_865_968_799u128);
        let state = AccountState {
            deposited_balance: deposited.clone(),
            pending_balance: BigUint::zero(),
            pending_deposited_balance: BigUint::zero(),
            withdraw_request: WithdrawRequest {
                requested: U256::ZERO,
                readyForClaim: U256::ZERO,
            },
            restaked_reward: restaked.clone(),
            autocompound_balance: autocompound_balance.clone(),
        };

        assert_eq!(map_account_state_to_delegations(&state), vec![map_balance_to_delegation(&autocompound_balance, &restaked, DelegationState::Active)]);
        assert_eq!(map_staking_balance(&state), Balance::stake_balance(deposited, BigUint::zero(), Some(restaked)));
    }
}
