use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;

sol! {
    interface IOkxDexRouter {
        event OrderRecord(address fromToken, address toToken, address sender, uint256 fromAmount, uint256 returnAmount);
        event CommissionFromTokenRecord(address fromTokenAddress, uint256 commissionAmount, address referrerAddress, uint256 commissionRate);
        event CommissionToTokenRecord(address toTokenAddress, uint256 commissionAmount, address referrerAddress, uint256 commissionRate);
    }
}

pub struct OkxCommission {
    pub token: Address,
    pub amount: U256,
    pub referrer: Address,
}

impl From<IOkxDexRouter::CommissionFromTokenRecord> for OkxCommission {
    fn from(event: IOkxDexRouter::CommissionFromTokenRecord) -> Self {
        Self {
            token: event.fromTokenAddress,
            amount: event.commissionAmount,
            referrer: event.referrerAddress,
        }
    }
}

impl From<IOkxDexRouter::CommissionToTokenRecord> for OkxCommission {
    fn from(event: IOkxDexRouter::CommissionToTokenRecord) -> Self {
        Self {
            token: event.toTokenAddress,
            amount: event.commissionAmount,
            referrer: event.referrerAddress,
        }
    }
}
