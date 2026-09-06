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

pub(crate) struct GemTransferAmountInput {
    pub(crate) input_type: TransactionInputType,
    pub(crate) value: GemBigInt,
    pub(crate) available_value: GemBigInt,
    pub(crate) fee_asset: AssetId,
    pub(crate) fee_asset_balance: GemBigInt,
    pub(crate) fee: GemBigInt,
    pub(crate) is_max_amount: bool,
    pub(crate) minimum_value: Option<GemBigInt>,
}

impl From<GemTransferAmountInput> for TransferAmountInput {
    fn from(value: GemTransferAmountInput) -> Self {
        Self {
            input_type: value.input_type,
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

impl GemTransferAmountInput {
    pub(crate) fn calculate(self) -> Result<GemTransferAmount, GemTransferAmountError> {
        TransferAmountInput::from(self).calculate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use primitives::{Asset, Chain};

    fn input(value: u64, available_value: u64, fee_asset_balance: u64) -> GemTransferAmountInput {
        let asset = Asset::from_chain(Chain::Solana);
        GemTransferAmountInput {
            input_type: TransactionInputType::Transfer { asset: asset.clone() },
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
            input(10_000_000, 100_000_000, 100_000_000).calculate().unwrap(),
            GemTransferAmount {
                value: BigInt::from(10_000_000),
                network_fee: BigInt::from(5_000),
                is_max_amount: false,
            }
        );

        assert_eq!(
            input(20_000_000, 10_000_000, 10_000_000).calculate().unwrap_err(),
            GemTransferAmountError::InsufficientBalance {
                asset_id: Asset::from_chain(Chain::Solana).id,
                required: BigInt::from(20_005_000u64),
                available: BigInt::from(10_000_000u64),
            }
        );
    }
}
