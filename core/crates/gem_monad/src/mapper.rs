use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};
use primitives::{AssetId, Chain, DateTime, DelegationBase, DelegationState, DelegationValidator};

use crate::constants::{COMMISSION_SCALE, FIRST_WITHDRAW_ID, VALIDATOR_NAMES};
use crate::contracts::IMonadStakingLens;
use crate::model::{LensDelegation, LensValidator};

fn delegation_id(address: &str, validator_id: u64, state: DelegationState, withdraw_id: u8) -> String {
    format!("{}:{}:{}:{}", address, validator_id, state.as_ref(), withdraw_id)
}

pub fn map_validator(validator: &LensValidator, network_apy: f64) -> DelegationValidator {
    let name = VALIDATOR_NAMES.iter().find(|(id, _)| *id == validator.validator_id).map(|(_, name)| name.to_string()).unwrap_or_default();

    DelegationValidator::stake(
        Chain::Monad,
        validator.validator_id.to_string(),
        name,
        validator.is_active,
        validator.commission.to_f64().unwrap_or(0.0) / COMMISSION_SCALE,
        if validator.apy_bps > 0 { validator.apy_bps as f64 / 100.0 } else { network_apy },
    )
}

pub fn map_delegations(address: &str, positions: Vec<LensDelegation>) -> Vec<DelegationBase> {
    let pending_withdrawals: Vec<(u64, u8)> = positions.iter().filter(|position| is_pending_withdrawal(position)).map(|position| (position.validator_id, position.withdraw_id)).collect();

    positions
        .into_iter()
        .filter(|position| !position.amount.is_zero() || !position.rewards.is_zero())
        .map(|position| LensDelegation {
            withdraw_id: if is_pending_withdrawal(&position) {
                position.withdraw_id
            } else {
                last_withdraw_id_before_free(&pending_withdrawals, &position)
            },
            ..position
        })
        .map(|position| map_delegation(address, position))
        .collect()
}

fn is_pending_withdrawal(position: &LensDelegation) -> bool {
    match position.state {
        IMonadStakingLens::DelegationState::Deactivating | IMonadStakingLens::DelegationState::AwaitingWithdrawal => true,
        IMonadStakingLens::DelegationState::Active | IMonadStakingLens::DelegationState::Activating | IMonadStakingLens::DelegationState::__Invalid => false,
    }
}

fn last_withdraw_id_before_free(pending_withdrawals: &[(u64, u8)], position: &LensDelegation) -> u8 {
    (FIRST_WITHDRAW_ID..=u8::MAX)
        .find(|withdraw_id| !pending_withdrawals.contains(&(position.validator_id, *withdraw_id)))
        .map_or(position.withdraw_id, |free_withdraw_id| free_withdraw_id - 1)
}

fn map_delegation(address: &str, position: LensDelegation) -> DelegationBase {
    let state = map_state(&position);
    let completion_date = (position.completion_timestamp != 0).then(|| DateTime::from_timestamp(position.completion_timestamp as i64));

    DelegationBase {
        asset_id: AssetId::from_chain(Chain::Monad),
        state,
        balance: position.amount,
        shares: BigUint::zero(),
        rewards: position.rewards,
        completion_date,
        delegation_id: delegation_id(address, position.validator_id, state, position.withdraw_id),
        validator_id: position.validator_id.to_string(),
    }
}

fn map_state(position: &LensDelegation) -> DelegationState {
    match position.state {
        IMonadStakingLens::DelegationState::Active => DelegationState::Active,
        IMonadStakingLens::DelegationState::Activating => DelegationState::Activating,
        IMonadStakingLens::DelegationState::Deactivating => DelegationState::Deactivating,
        IMonadStakingLens::DelegationState::AwaitingWithdrawal => DelegationState::AwaitingWithdrawal,
        IMonadStakingLens::DelegationState::__Invalid => DelegationState::Inactive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TEST_ADDRESS;

    #[test]
    fn test_delegation_id_is_unique_per_validator_and_state() {
        let everstake = delegation_id(TEST_ADDRESS, 9, DelegationState::AwaitingWithdrawal, 1);
        let stakin_withdraw = delegation_id(TEST_ADDRESS, 10, DelegationState::Deactivating, 1);
        let stakin_active = delegation_id(TEST_ADDRESS, 10, DelegationState::Active, 1);

        assert_ne!(everstake, stakin_withdraw);
        assert_ne!(stakin_withdraw, stakin_active);
        assert_eq!(everstake, format!("{TEST_ADDRESS}:9:awaitingwithdrawal:1"));
    }

    #[test]
    fn test_map_validator_names_only_the_validators_gem_lists() {
        let validator = |validator_id: u64| LensValidator {
            validator_id,
            commission: BigUint::zero(),
            apy_bps: 1_146,
            is_active: true,
        };

        let listed = map_validator(&validator(5), 12.73);
        let unlisted = map_validator(&validator(7), 12.73);

        assert_eq!((listed.name.as_str(), listed.apr), ("Alchemy", 11.46));
        assert_eq!((unlisted.name.as_str(), unlisted.apr), ("", 11.46));
    }

    #[test]
    fn test_map_delegations() {
        let active = IMonadStakingLens::DelegationState::Active;
        let deactivating = IMonadStakingLens::DelegationState::Deactivating;
        let awaiting = IMonadStakingLens::DelegationState::AwaitingWithdrawal;
        let positions = vec![
            LensDelegation::mock_with(5, 255, deactivating),
            LensDelegation::mock_with(5, 255, active),
            LensDelegation::mock_with(10, 1, awaiting),
            LensDelegation::mock_with(10, 2, deactivating),
            LensDelegation::mock_with(10, 2, active),
            LensDelegation::mock_with(16, 2, awaiting),
            LensDelegation::mock_with(16, 2, active),
            LensDelegation::mock_with(9, 0, active),
            LensDelegation {
                amount: BigUint::zero(),
                ..LensDelegation::mock_with(7, 0, active)
            },
        ];

        assert_eq!(
            map_delegations(TEST_ADDRESS, positions).into_iter().map(|delegation| delegation.delegation_id).collect::<Vec<_>>(),
            vec![
                format!("{TEST_ADDRESS}:5:deactivating:255"),
                format!("{TEST_ADDRESS}:5:active:0"),
                format!("{TEST_ADDRESS}:10:awaitingwithdrawal:1"),
                format!("{TEST_ADDRESS}:10:deactivating:2"),
                format!("{TEST_ADDRESS}:10:active:2"),
                format!("{TEST_ADDRESS}:16:awaitingwithdrawal:2"),
                format!("{TEST_ADDRESS}:16:active:0"),
                format!("{TEST_ADDRESS}:9:active:0"),
            ]
        );
    }

    #[test]
    fn test_map_delegation_awaiting_withdrawal() {
        let position = LensDelegation {
            validator_id: 10,
            withdraw_id: 1,
            state: IMonadStakingLens::DelegationState::AwaitingWithdrawal,
            amount: BigUint::from(5u32),
            rewards: BigUint::zero(),
            completion_timestamp: 1_700_000_000,
        };

        let delegation = map_delegation(TEST_ADDRESS, position);

        assert_eq!(delegation.state, DelegationState::AwaitingWithdrawal);
        assert_eq!(delegation.balance, BigUint::from(5u32));
        assert_eq!(delegation.validator_id, "10");
        assert_eq!(delegation.delegation_id, format!("{TEST_ADDRESS}:10:awaitingwithdrawal:1"));
        assert_eq!(delegation.completion_date, Some(DateTime::from_timestamp(1_700_000_000)));
    }
}
