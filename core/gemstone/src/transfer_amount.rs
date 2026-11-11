use crate::models::custom_types::GemBigInt;
use primitives::TransferAmount;

#[uniffi::remote(Record)]
pub struct TransferAmount {
    pub value: GemBigInt,
    pub network_fee: GemBigInt,
    pub is_max_amount: bool,
}
