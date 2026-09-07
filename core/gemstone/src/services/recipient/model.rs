use primitives::{Asset, NFTAsset};

use crate::payment::GemPaymentRecipient;
use crate::services::transfer::GemTransferData;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecipientValidation {
    pub is_valid: bool,
    pub address: String,
    pub shows_error: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemRecipientError {
    InvalidAddress,
    NameRecordMismatch,
}

impl std::fmt::Display for GemRecipientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress => write!(f, "invalid recipient address"),
            Self::NameRecordMismatch => write!(f, "name record does not match the input"),
        }
    }
}

impl std::error::Error for GemRecipientError {}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemRecipientType {
    Asset { asset: Asset },
    Nft { nft_asset: NFTAsset },
}

#[uniffi::export]
impl GemRecipientType {
    pub fn identifier(&self) -> String {
        match self {
            Self::Asset { asset } => asset.id.to_string(),
            Self::Nft { nft_asset } => nft_asset.id.to_string(),
        }
    }
}

impl GemRecipientType {
    pub fn asset(&self) -> Asset {
        match self {
            Self::Asset { asset } => asset.clone(),
            Self::Nft { nft_asset } => Asset::from_chain(nft_asset.chain),
        }
    }
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemRecipientScan {
    Confirm { transfer: GemTransferData },
    Recipient { payment: GemPaymentRecipient },
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemRecipientNext {
    Amount { payment: GemPaymentRecipient },
    Confirm { transfer: GemTransferData },
}
