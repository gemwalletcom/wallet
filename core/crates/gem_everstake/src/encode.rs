use std::error::Error;

use alloy_sol_types::SolCall;
use gem_evm::transaction_params::TransactionParams;
use gem_evm::u256::bigint_to_u256;
use num_bigint::BigInt;
use primitives::StakeType;

use crate::constants::{DEFAULT_ALLOWED_INTERCHANGE_NUM, EVERSTAKE_ACCOUNTING_ADDRESS, EVERSTAKE_POOL_ADDRESS, EVERSTAKE_SOURCE};
use crate::contracts::{IAccounting, IPool};

pub fn encode_stake(stake_type: &StakeType, amount: &BigInt) -> Result<TransactionParams, Box<dyn Error + Send + Sync>> {
    let (to, data, value) = match stake_type {
        StakeType::Stake(_) => (EVERSTAKE_POOL_ADDRESS, IPool::stakeCall { source: EVERSTAKE_SOURCE }.abi_encode(), amount.clone()),
        StakeType::Unstake(_) => {
            let data = IPool::unstakeCall {
                value: bigint_to_u256(amount)?,
                allowedInterchangeNum: DEFAULT_ALLOWED_INTERCHANGE_NUM,
                source: EVERSTAKE_SOURCE,
            }
            .abi_encode();
            (EVERSTAKE_POOL_ADDRESS, data, BigInt::from(0))
        }
        StakeType::Withdraw(_) => (EVERSTAKE_ACCOUNTING_ADDRESS, IAccounting::claimWithdrawRequestCall {}.abi_encode(), BigInt::from(0)),
        StakeType::Redelegate(_) | StakeType::Rewards(_) | StakeType::Freeze(_) | StakeType::Unfreeze(_) => return Err("Unsupported stake type for Everstake".into()),
    };
    Ok(TransactionParams::new(to, data, value))
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex;
    use num_bigint::BigInt;
    use primitives::DelegationState;

    use super::*;
    use crate::testkit::mock_delegation;

    #[test]
    fn test_encode_stake() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let params = encode_stake(
            &StakeType::Stake(mock_delegation(DelegationState::Active).validator),
            &BigInt::from(1_000_000_000_000_000_000u64),
        )?;
        assert_eq!(params.to, EVERSTAKE_POOL_ADDRESS);
        assert_eq!(hex::encode(&params.data), "3a29dbae0000000000000000000000000000000000000000000000000000000000000017");
        assert_eq!(params.value, BigInt::from(1_000_000_000_000_000_000u64));

        let params = encode_stake(&StakeType::Unstake(mock_delegation(DelegationState::Active)), &BigInt::from(500_000_000_000_000_000u64))?;
        assert_eq!(params.to, EVERSTAKE_POOL_ADDRESS);
        assert_eq!(
            hex::encode(&params.data),
            "76ec871c00000000000000000000000000000000000000000000000006f05b59d3b2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000017"
        );
        assert_eq!(params.value, BigInt::from(0));

        let params = encode_stake(&StakeType::Withdraw(mock_delegation(DelegationState::AwaitingWithdrawal)), &BigInt::from(0))?;
        assert_eq!(params.to, EVERSTAKE_ACCOUNTING_ADDRESS);
        assert_eq!(hex::encode(&params.data), "33986ffa");
        assert_eq!(params.value, BigInt::from(0));

        Ok(())
    }
}
