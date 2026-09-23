use num_bigint::BigInt;

use crate::{AssetId, TransactionInputType, TransferAmountInput};

impl TransferAmountInput {
    pub fn mock(input_type: TransactionInputType, value: u64, available_value: u64, fee_asset_balance: u64) -> Self {
        Self {
            fee_asset: AssetId::from_chain(input_type.get_asset().chain()),
            input_type,
            value: BigInt::from(value),
            available_value: BigInt::from(available_value),
            fee_asset_balance: BigInt::from(fee_asset_balance),
            fee: BigInt::from(5_000u64),
            is_max_amount: false,
            destination_account_exists: None,
        }
    }
}
