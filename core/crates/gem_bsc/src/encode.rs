use std::error::Error;
use std::str::FromStr;

use alloy_primitives::{Address, U256};
use alloy_sol_types::SolCall;
use gem_evm::transaction_params::TransactionParams;
use gem_evm::u256::{biguint_to_u256, u256_to_biguint};
use num_bigint::{BigInt, BigUint};
use primitives::StakeType;

use crate::constants::STAKE_HUB_ADDRESS;
use crate::contracts::{IHubReader, IStakeHub};
use crate::model::{BscDelegation, BscUndelegation, BscValidator};

pub fn encode_validators_call(offset: u16, limit: u16) -> Vec<u8> {
    IHubReader::getValidatorsCall { offset, limit }.abi_encode()
}

pub fn decode_validators(result: &[u8]) -> Result<Vec<BscValidator>, Box<dyn Error + Send + Sync>> {
    Ok(IHubReader::getValidatorsCall::abi_decode_returns(result)?
        .into_iter()
        .map(|validator| BscValidator {
            operator_address: validator.operatorAddress.to_string(),
            moniker: validator.moniker,
            commission: validator.commission,
            apy: validator.apy,
            jailed: validator.jailed,
        })
        .collect())
}

pub fn encode_delegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IHubReader::getDelegationsCall { delegator, offset, limit }.abi_encode())
}

pub fn decode_delegations(result: &[u8]) -> Result<Vec<BscDelegation>, Box<dyn Error + Send + Sync>> {
    Ok(IHubReader::getDelegationsCall::abi_decode_returns(result)?
        .into_iter()
        .map(|delegation| BscDelegation {
            delegator_address: delegation.delegatorAddress.to_string(),
            validator_address: delegation.validatorAddress.to_string(),
            amount: u256_to_biguint(&delegation.amount),
            shares: u256_to_biguint(&delegation.shares),
        })
        .collect())
}

pub fn encode_undelegations_call(delegator: &str, offset: u16, limit: u16) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let delegator = Address::from_str(delegator)?;
    Ok(IHubReader::getUndelegationsCall { delegator, offset, limit }.abi_encode())
}

pub fn decode_undelegations(result: &[u8]) -> Result<Vec<BscUndelegation>, Box<dyn Error + Send + Sync>> {
    Ok(IHubReader::getUndelegationsCall::abi_decode_returns(result)?
        .into_iter()
        .map(|undelegation| BscUndelegation {
            delegator_address: undelegation.delegatorAddress.to_string(),
            validator_address: undelegation.validatorAddress.to_string(),
            amount: u256_to_biguint(&undelegation.amount),
            shares: u256_to_biguint(&undelegation.shares),
            unlock_time: u64::try_from(undelegation.unlockTime).ok(),
        })
        .collect())
}

pub fn encode_stake(stake_type: &StakeType, amount: &BigInt) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    let (data, value) = match stake_type {
        StakeType::Stake(validator) => (encode_delegate_call(&validator.id, false)?, amount.clone()),
        StakeType::Unstake(delegation) => (
            encode_undelegate_call(&delegation.validator.id, amount_shares(amount, &delegation.base.balance, &delegation.base.shares)?)?,
            BigInt::from(0),
        ),
        StakeType::Redelegate(redelegate_data) => (
            encode_redelegate_call(
                &redelegate_data.delegation.validator.id,
                &redelegate_data.to_validator.id,
                amount_shares(amount, &redelegate_data.delegation.base.balance, &redelegate_data.delegation.base.shares)?,
                false,
            )?,
            BigInt::from(0),
        ),
        StakeType::Withdraw(delegation) => (encode_claim_call(&delegation.validator.id, 0)?, BigInt::from(0)),
        StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => return Err("Unsupported stake type for StakeHub".into()),
    };
    Ok(TransactionParams::new(STAKE_HUB_ADDRESS, data, value))
}

fn amount_shares(amount: &BigInt, balance: &BigUint, shares: &BigUint) -> Result<U256, Box<dyn Error + Send + Sync>> {
    biguint_to_u256(&(amount.magnitude() * shares / balance)).ok_or("Shares value does not fit in U256".into())
}

fn encode_delegate_call(operator_address: &str, delegate_vote_power: bool) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let operator_address = Address::from_str(operator_address)?;
    Ok(IStakeHub::delegateCall {
        operatorAddress: operator_address,
        delegateVotePower: delegate_vote_power,
    }
    .abi_encode())
}

fn encode_undelegate_call(operator_address: &str, shares: U256) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let operator_address = Address::from_str(operator_address)?;
    Ok(IStakeHub::undelegateCall {
        operatorAddress: operator_address,
        shares,
    }
    .abi_encode())
}

fn encode_redelegate_call(src_validator: &str, dst_validator: &str, shares: U256, delegate_vote_power: bool) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let src_validator = Address::from_str(src_validator)?;
    let dst_validator = Address::from_str(dst_validator)?;
    Ok(IStakeHub::redelegateCall {
        srcValidator: src_validator,
        dstValidator: dst_validator,
        shares,
        delegateVotePower: delegate_vote_power,
    }
    .abi_encode())
}

fn encode_claim_call(operator_address: &str, request_number: u64) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let operator_address = Address::from_str(operator_address)?;
    Ok(IStakeHub::claimCall {
        operatorAddress: operator_address,
        requestNumber: U256::from(request_number),
    }
    .abi_encode())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::hex;
    use num_bigint::{BigInt, BigUint};
    use primitives::{DelegationState, RedelegateData};

    use super::*;
    use crate::testkit::mock_delegation;

    #[test]
    fn test_decode_validators() {
        let result = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000001400000000000000000000000000000000000000000000000000000000000000220000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000003e000000000000000000000000000000000000000000000000000000000000004c000000000000000000000000000000000000000000000000000000000000005a00000000000000000000000000000000000000000000000000000000000000680000000000000000000000000000000000000000000000000000000000000076000000000000000000000000000000000000000000000000000000000000008400000000000000000000000000000000000000000000000000000000000000920000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b1000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc000000000000000000000000000000000000000000000000000000000000017400000000000000000000000000000000000000000000000000000000000000064c6567656e640000000000000000000000000000000000000000000000000000000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000000c900000000000000000000000000000000000000000000000000000000000000084c6567656e644949000000000000000000000000000000000000000000000000000000000000000000000000f2b1d86dc7459887b1f7ce8d840db1d87613ce7f000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000002bc00000000000000000000000000000000000000000000000000000000000001d300000000000000000000000000000000000000000000000000000000000000094c6567656e644949490000000000000000000000000000000000000000000000000000000000000000000000eace91702b20bc6ee62034ec7f5162d9a94bfbe4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000004416e6b72000000000000000000000000000000000000000000000000000000000000000000000000000000005ce21461e6472914f5e4d5b296c72125f26ed462000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000008b00000000000000000000000000000000000000000000000000000000000000095472616e636865737300000000000000000000000000000000000000000000000000000000000000000000005c38ff8ca2b16099c086bf36546e99b13d152c4c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e80000000000000000000000000000000000000000000000000000000000000057000000000000000000000000000000000000000000000000000000000000000954575374616b696e6700000000000000000000000000000000000000000000000000000000000000000000001ae5f5c3cb452e042b0b7b9dc60596c9cd84baf6000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000007b000000000000000000000000000000000000000000000000000000000000000446756a6900000000000000000000000000000000000000000000000000000000000000000000000000000000b12e8137ef499a1d81552db11664a9e617fd350a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009f00000000000000000000000000000000000000000000000000000000000000054d617468570000000000000000000000000000000000000000000000000000000000000000000000000000004dc1bf52da103452097df48505a6d01020ffb22b000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009a000000000000000000000000000000000000000000000000000000000000000744656669626974000000000000000000000000000000000000000000000000000000000000000000000000007d0f8a6d1c8fbf929dcf4847a31e30d14923fa31000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000003e8000000000000000000000000000000000000000000000000000000000000009f00000000000000000000000000000000000000000000000000000000000000084e6f64655265616c000000000000000000000000000000000000000000000000").unwrap();
        let validators = decode_validators(&result).unwrap();
        assert_eq!(validators.len(), 10);
        assert_eq!(validators[0].operator_address, "0x773760b0708a5Cc369c346993a0c225D8e4043B1");
        assert_eq!(validators[0].moniker, "Legend");
        assert_eq!(validators[0].commission, 700);
        assert_eq!(validators[0].apy, 372);
    }

    #[test]
    fn test_decode_delegations() {
        let result = hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000002000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000773760b0708a5cc369c346993a0c225d8e4043b10000000000000000000000000000000000000000000000000de0b6b3b015a6430000000000000000000000000000000000000000000000000dd62dce1850f388000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a0000000000000000000000000000000000000000000000000e09ef1d9101a1740000000000000000000000000000000000000000000000000e028d70463b87f8").unwrap();
        let delegations = decode_delegations(&result).unwrap();
        assert_eq!(delegations.len(), 2);
        assert_eq!(delegations[1].delegator_address, "0xee448667ffc3D15ca023A6deEf2D0fAf084C0716");
        assert_eq!(delegations[1].validator_address, "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A");
        assert_eq!(delegations[1].amount, BigUint::from_str("1011602501587280244").unwrap());
        assert_eq!(delegations[1].shares, BigUint::from_str("1009524779838572536").unwrap());
    }

    #[test]
    fn test_decode_undelegations() {
        let result = hex::decode("00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000ee448667ffc3d15ca023a6deef2d0faf084c0716000000000000000000000000343da7ff0446247ca47aa41e2a25c5bbb230ed0a000000000000000000000000000000000000000000000000016345785d89ffff00000000000000000000000000000000000000000000000001628aab7a64b3dc00000000000000000000000000000000000000000000000000000000664e7431").unwrap();
        let undelegations = decode_undelegations(&result).unwrap();
        assert_eq!(undelegations.len(), 1);
        assert_eq!(undelegations[0].delegator_address, "0xee448667ffc3D15ca023A6deEf2D0fAf084C0716");
        assert_eq!(undelegations[0].validator_address, "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A");
        assert_eq!(undelegations[0].amount, BigUint::from_str("99999999999999999").unwrap());
        assert_eq!(undelegations[0].shares, BigUint::from_str("99794610853032924").unwrap());
        assert_eq!(undelegations[0].unlock_time, Some(1_716_417_585));
    }

    #[test]
    fn test_encode_stake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let one_bnb = BigInt::from(1_000_000_000_000_000_000u64);
        let validator = mock_delegation(
            "0x773760b0708a5Cc369c346993a0c225D8e4043B1",
            DelegationState::Active,
            2_000_000_000_000_000_000,
            1_900_000_000_000_000_000,
        )
        .validator;

        let params = encode_stake(&StakeType::Stake(validator.clone()), &one_bnb)?;
        assert_eq!(params.to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&params.data[0..4]), "982ef0a7");
        assert_eq!(params.value, one_bnb);

        let unstake = mock_delegation(
            "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A",
            DelegationState::Active,
            2_000_000_000_000_000_000,
            1_900_000_000_000_000_000,
        );
        let params = encode_stake(&StakeType::Unstake(unstake.clone()), &one_bnb)?;
        assert_eq!(params.to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&params.data[0..4]), "4d99dd16");
        assert_eq!(params.value, BigInt::from(0));

        let params = encode_stake(
            &StakeType::Redelegate(RedelegateData {
                delegation: unstake,
                to_validator: validator,
            }),
            &one_bnb,
        )?;
        assert_eq!(params.to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&params.data[0..4]), "59491871");
        assert_eq!(params.value, BigInt::from(0));

        let withdraw = mock_delegation(
            "0x343dA7Ff0446247ca47AA41e2A25c5Bbb230ED0A",
            DelegationState::AwaitingWithdrawal,
            1_000_000_000_000_000_000,
            1_000_000_000_000_000_000,
        );
        let params = encode_stake(&StakeType::Withdraw(withdraw), &BigInt::from(0))?;
        assert_eq!(params.to, STAKE_HUB_ADDRESS);
        assert_eq!(hex::encode(&params.data[0..4]), "aad3ec96");
        assert_eq!(params.value, BigInt::from(0));

        Ok(())
    }
}
