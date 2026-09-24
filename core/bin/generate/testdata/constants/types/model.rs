#[derive(Clone, Copy, uniffi::Enum)]
pub enum GemLockPeriod {
    Immediate,
    OneMinute,
}

#[derive(Clone, uniffi::Record)]
pub struct GemAttachmentLimits {
    pub max_dimension: u32,
    pub jpeg_quality: u32,
}

#[derive(Clone, uniffi::Record)]
pub struct GemRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Clone, uniffi::Enum)]
pub enum GemLabel {
    None,
    Number { value: f64, digits: Option<u8> },
    Plain(String),
}

#[uniffi::remote(Enum)]
pub enum TransactionType {
    TransferNFT,
    Swap,
}
