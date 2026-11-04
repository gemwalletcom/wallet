use num_bigint::BigUint;

use crate::{address::Address, models::RunGetMethodResult, tvm::TvmError};

const POOL_TOTAL_BALANCE_INDEX: usize = 2;
const POOL_INTEREST_RATE_INDEX: usize = 3;
const POOL_GOVERNANCE_FEE_INDEX: usize = 11;
const POOL_SUPPLY_INDEX: usize = 13;
const POOL_DEPOSIT_PAYOUT_INDEX: usize = 14;
const POOL_WITHDRAWAL_PAYOUT_INDEX: usize = 16;

#[derive(Debug, Clone, PartialEq)]
pub struct PoolFullData {
    pub total_balance: BigUint,
    pub interest_rate: u32,
    pub governance_fee: u32,
    pub supply: BigUint,
    pub deposit_payout: Option<Address>,
    pub withdrawal_payout: Option<Address>,
}

impl PoolFullData {
    pub fn from_stack(result: &RunGetMethodResult) -> Result<Self, TvmError> {
        Ok(Self {
            total_balance: result.stack_num(POOL_TOTAL_BALANCE_INDEX)?,
            interest_rate: stack_u32(result, POOL_INTEREST_RATE_INDEX)?,
            governance_fee: stack_u32(result, POOL_GOVERNANCE_FEE_INDEX)?,
            supply: result.stack_num(POOL_SUPPLY_INDEX)?,
            deposit_payout: result.stack_maybe_address(POOL_DEPOSIT_PAYOUT_INDEX)?,
            withdrawal_payout: result.stack_maybe_address(POOL_WITHDRAWAL_PAYOUT_INDEX)?,
        })
    }
}

fn stack_u32(result: &RunGetMethodResult, index: usize) -> Result<u32, TvmError> {
    u32::try_from(result.stack_num(index)?).map_err(|_| TvmError::new("TON pool rate does not fit in u32"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_stack() {
        let result: RunGetMethodResult = serde_json::from_str(include_str!("../../testdata/tonstakers_pool_full_data.json")).unwrap();

        assert_eq!(
            PoolFullData::from_stack(&result).unwrap(),
            PoolFullData {
                withdrawal_payout: Some(Address::parse("0:2ca6ad3f72fffa324d825d318985164ba14e0c1daf4992ba1312976333a5b857").unwrap()),
                ..PoolFullData::mock()
            }
        );
    }
}
