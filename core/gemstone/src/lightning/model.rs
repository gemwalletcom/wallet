use crate::models::copy::GemCopy;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemLightningReceive {
    pub qr: String,
    pub copy: GemCopy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemLightningReceiveState {
    Pending,
    Received,
    Expired,
}
