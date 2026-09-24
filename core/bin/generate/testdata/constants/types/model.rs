#[derive(Clone, Copy, uniffi::Enum)]
pub enum GemLockPeriod {
    Immediate,
    OneMinute,
}

#[derive(Clone, uniffi::Record)]
pub struct GemAttachmentLimits {
    pub jpeg_quality: u32,
}

#[uniffi::remote(Enum)]
pub enum TransactionType {
    TransferNFT,
    Swap,
}
