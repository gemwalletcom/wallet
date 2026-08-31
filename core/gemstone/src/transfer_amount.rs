use crate::models::GemTransactionInputType;
use crate::models::custom_types::GemBigInt;
use primitives::{AssetId, TransactionInputType, TransferAmount, TransferAmountError, TransferAmountInput};

pub type GemTransferAmount = TransferAmount;
pub type GemTransferAmountError = TransferAmountError;

#[uniffi::remote(Record)]
pub struct GemTransferAmount {
    pub value: GemBigInt,
    pub network_fee: GemBigInt,
    pub is_max_amount: bool,
}

#[uniffi::remote(Error)]
pub enum GemTransferAmountError {
    InsufficientBalance { asset_id: AssetId, required: GemBigInt, available: GemBigInt },
    InsufficientNetworkFee { asset_id: AssetId, required: GemBigInt, available: GemBigInt },
    MinimumAccountBalanceTooLow { asset_id: AssetId, required: GemBigInt, available: GemBigInt },
}

#[derive(uniffi::Record, Debug, Clone)]
pub struct GemTransferAmountInput {
    pub input_type: GemTransactionInputType,
    pub value: GemBigInt,
    pub available_value: GemBigInt,
    pub fee_asset: AssetId,
    pub fee_asset_balance: GemBigInt,
    pub fee: GemBigInt,
    pub is_max_amount: bool,
    pub minimum_value: Option<GemBigInt>,
}

impl From<GemTransferAmountInput> for TransferAmountInput {
    fn from(value: GemTransferAmountInput) -> Self {
        Self {
            input_type: TransactionInputType::from(value.input_type),
            value: value.value,
            available_value: value.available_value,
            fee_asset: value.fee_asset,
            fee_asset_balance: value.fee_asset_balance,
            fee: value.fee,
            is_max_amount: value.is_max_amount,
            minimum_value: value.minimum_value,
        }
    }
}

pub fn calculate_transfer_amount(input: GemTransferAmountInput) -> Result<GemTransferAmount, GemTransferAmountError> {
    TransferAmountInput::from(input).calculate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use primitives::{Asset, Chain};

    fn input(value: u64, available_value: u64, fee_asset_balance: u64) -> GemTransferAmountInput {
        let asset = Asset::from_chain(Chain::Solana);
        GemTransferAmountInput {
            input_type: GemTransactionInputType::Transfer { asset: asset.clone() },
            value: BigInt::from(value),
            available_value: BigInt::from(available_value),
            fee_asset: asset.id,
            fee_asset_balance: BigInt::from(fee_asset_balance),
            fee: BigInt::from(5_000),
            is_max_amount: false,
            minimum_value: None,
        }
    }

    #[test]
    fn test_calculate_transfer_amount() {
        assert_eq!(
            calculate_transfer_amount(input(10_000_000, 100_000_000, 100_000_000)).unwrap(),
            GemTransferAmount {
                value: BigInt::from(10_000_000),
                network_fee: BigInt::from(5_000),
                is_max_amount: false,
            }
        );

        assert_eq!(
            calculate_transfer_amount(input(20_000_000, 10_000_000, 10_000_000)).unwrap_err(),
            GemTransferAmountError::InsufficientBalance {
                asset_id: Asset::from_chain(Chain::Solana).id,
                required: BigInt::from(20_005_000u64),
                available: BigInt::from(10_000_000u64),
            }
        );
    }
}
