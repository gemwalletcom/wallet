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
        }
    }
}

impl GemTransferAmountInput {
    pub(crate) fn calculate(self) -> Result<GemTransferAmount, GemTransferAmountError> {
        TransferAmountInput::from(self).calculate()
    }
}
