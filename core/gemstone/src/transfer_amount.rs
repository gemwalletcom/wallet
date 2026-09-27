use crate::models::custom_types::GemBigInt;
use primitives::{TransferAmount, TransferAmountError};

pub type GemTransferAmount = TransferAmount;
pub type GemTransferAmountError = TransferAmountError;

#[uniffi::remote(Record)]
pub struct GemTransferAmount {
    pub value: GemBigInt,
    pub network_fee: GemBigInt,
    pub is_max_amount: bool,
}
