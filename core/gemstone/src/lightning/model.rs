use crate::models::copy::GemCopy;
use crate::services::amount::model::GemAmountField;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemLightningReceive {
    pub qr: String,
    pub copy: GemCopy,
    pub field: GemAmountField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemLightningReceiveState {
    Pending,
    Received,
    Expired,
}
