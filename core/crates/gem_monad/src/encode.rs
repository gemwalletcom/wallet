use std::error::Error;
use std::str::FromStr;

use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use gem_evm::transaction_params::TransactionParams;
use gem_evm::u256::{bigint_to_u256, u256_to_biguint};
use num_bigint::BigInt;
use num_traits::Zero;
use primitives::StakeType;

use crate::constants::{DEFAULT_WITHDRAW_ID, STAKING_CONTRACT};
use crate::contracts::{IMonadStaking, IMonadStakingLens};
use crate::model::{LensBalance, LensDelegation, LensValidator};

pub fn encode_balance(delegator: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IMonadStakingLens::getBalanceCall { delegator }.abi_encode())
}

pub fn decode_balance(data: &[u8]) -> Result<LensBalance, Box<dyn Error + Send + Sync>> {
    let decoded = IMonadStakingLens::getBalanceCall::abi_decode_returns(data)?;
    Ok(LensBalance {
        staked: u256_to_biguint(&decoded.staked),
        pending: u256_to_biguint(&decoded.pending),
        rewards: u256_to_biguint(&decoded.rewards),
    })
}

pub fn encode_delegations(delegator: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IMonadStakingLens::getDelegationsCall { delegator }.abi_encode())
}

pub fn encode_apys(validator_ids: &[u64]) -> Vec<u8> {
    IMonadStakingLens::getAPYsCall {
        validatorIds: validator_ids.to_vec(),
    }
    .abi_encode()
}

pub fn decode_apys(data: &[u8]) -> Result<Vec<u64>, Box<dyn Error + Send + Sync>> {
    Ok(IMonadStakingLens::getAPYsCall::abi_decode_returns(data)?)
}

pub fn decode_delegations(data: &[u8]) -> Result<Vec<LensDelegation>, Box<dyn Error + Send + Sync>> {
    let decoded = IMonadStakingLens::getDelegationsCall::abi_decode_returns(data)?;

    Ok(decoded
        .into_iter()
        .map(|position| LensDelegation {
            validator_id: position.validatorId,
            withdraw_id: position.withdrawId,
            state: position.state,
            amount: u256_to_biguint(&position.amount),
            rewards: u256_to_biguint(&position.rewards),
            completion_timestamp: position.completionTimestamp,
        })
        .collect())
}

pub fn encode_validators(validator_ids: &[u64]) -> Vec<u8> {
    IMonadStakingLens::getValidatorsCall {
        validatorIds: validator_ids.to_vec(),
    }
    .abi_encode()
}

pub fn decode_validators(data: &[u8]) -> Result<(Vec<LensValidator>, u64), Box<dyn Error + Send + Sync>> {
    let decoded = IMonadStakingLens::getValidatorsCall::abi_decode_returns(data)?;

    Ok((
        decoded
            .validators
            .into_iter()
            .map(|validator| LensValidator {
                validator_id: validator.validatorId,
                commission: u256_to_biguint(&validator.commission),
                apy_bps: validator.apyBps,
                is_active: validator.isActive,
            })
            .collect(),
        decoded.networkApyBps,
    ))
}

pub fn encode_stake(stake_type: &StakeType, amount: &BigInt) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    let (data, value) = match stake_type {
        StakeType::Stake(validator) => {
            let validator_id = validator.id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            (IMonadStaking::delegateCall { validatorId: validator_id }.abi_encode(), amount.clone())
        }
        StakeType::Unstake(delegation) => {
            let validator_id = delegation.base.validator_id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            let current_withdraw_id = withdraw_id(&delegation.base.delegation_id).unwrap_or(DEFAULT_WITHDRAW_ID);
            let next_withdraw_id = current_withdraw_id.saturating_add(1);
            (
                IMonadStaking::undelegateCall {
                    validatorId: validator_id,
                    amount: bigint_to_u256(amount)?,
                    withdrawId: next_withdraw_id,
                }
                .abi_encode(),
                BigInt::zero(),
            )
        }
        StakeType::Withdraw(delegation) => {
            let validator_id = delegation.base.validator_id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            let withdraw_id = withdraw_id(&delegation.base.delegation_id).ok_or("Invalid withdraw id for Monad")?;

            (
                IMonadStaking::withdrawCall {
                    validatorId: validator_id,
                    withdrawId: withdraw_id,
                }
                .abi_encode(),
                BigInt::zero(),
            )
        }
        StakeType::Rewards(validators) => {
            let validator = validators.first().ok_or("Missing validator for rewards")?;
            let validator_id = validator.id.parse::<u64>().map_err(|_| "Invalid validator id for Monad")?;
            (IMonadStaking::claimRewardsCall { validatorId: validator_id }.abi_encode(), BigInt::zero())
        }
        StakeType::Redelegate(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => return Err("Unsupported stake type for Monad".into()),
    };
    Ok(TransactionParams::new(STAKING_CONTRACT, data, value))
}

fn withdraw_id(delegation_id: &str) -> Option<u8> {
    delegation_id.rsplit(':').next()?.parse::<u8>().ok()
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use primitives::{Delegation, DelegationBase, StakeType};

    use super::*;
    use crate::testkit::TEST_ADDRESS;

    #[test]
    fn test_withdraw_id_supports_legacy_and_extended_ids() {
        assert_eq!(withdraw_id(&format!("{TEST_ADDRESS}:1")), Some(1));
        assert_eq!(withdraw_id(&format!("{TEST_ADDRESS}:10:active:7")), Some(7));
        assert_eq!(withdraw_id("invalid"), None);
    }

    #[test]
    fn test_encode_stake_reads_last_id_segment() {
        let cases = [
            (
                StakeType::Unstake(Delegation::mock_base(DelegationBase {
                    validator_id: "10".to_string(),
                    delegation_id: format!("{TEST_ADDRESS}:10:active:1"),
                    ..DelegationBase::mock()
                })),
                BigInt::from(5u32),
                IMonadStaking::undelegateCall {
                    validatorId: 10,
                    amount: U256::from(5u32),
                    withdrawId: 2,
                }
                .abi_encode(),
            ),
            (
                StakeType::Withdraw(Delegation::mock_base(DelegationBase {
                    validator_id: "9".to_string(),
                    delegation_id: format!("{TEST_ADDRESS}:9:awaitingwithdrawal:1"),
                    ..DelegationBase::mock()
                })),
                BigInt::zero(),
                IMonadStaking::withdrawCall { validatorId: 9, withdrawId: 1 }.abi_encode(),
            ),
        ];

        for (stake_type, amount, expected_data) in cases {
            let params = encode_stake(&stake_type, &amount).unwrap();

            assert_eq!(params.to, STAKING_CONTRACT);
            assert_eq!(params.value, BigInt::zero());
            assert_eq!(params.data, expected_data);
        }
    }
}
