use primitives::{Asset, Chain, ChainAsset, NFTAsset};

use crate::payment::GemPaymentRecipient;
use crate::services::name::GemNameRecordState;
use crate::services::transfer::{GemRecipient, GemTransferData};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecipientValidation {
    pub is_valid: bool,
    pub address: String,
    pub error: Option<GemRecipientErrorDisplay>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemRecipientErrorDisplay {
    InvalidAddress { network: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemRecipientError {
    InvalidAddress { chain: Chain },
    NameRecordMismatch { chain: Chain },
}

impl std::fmt::Display for GemRecipientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress { .. } => write!(f, "invalid recipient address"),
            Self::NameRecordMismatch { .. } => write!(f, "name record does not match the input"),
        }
    }
}

impl std::error::Error for GemRecipientError {}

#[uniffi::export]
impl GemRecipientError {
    pub fn display(&self) -> GemRecipientErrorDisplay {
        match self {
            Self::InvalidAddress { chain } | Self::NameRecordMismatch { chain } => GemRecipientErrorDisplay::InvalidAddress {
                network: ChainAsset::from_chain(*chain).network_name,
            },
        }
    }
}

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

#[derive(Debug, Clone, PartialEq, Default, uniffi::Record)]
pub struct GemRecipientSession {
    pub address: String,
    pub memo: String,
    pub payment: Option<GemPaymentRecipient>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemRecipientSectionKind {
    Pinned,
    Contacts,
    Wallets,
    ViewWallets,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecipientRow {
    pub title: String,
    pub subtitle: String,
    pub recipient: GemRecipient,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecipientSection {
    pub kind: GemRecipientSectionKind,
    pub rows: Vec<GemRecipientRow>,
}

#[uniffi::export]
impl GemRecipientSession {
    pub fn on_address_changed(&self, address: String) -> Self {
        let payment = self.payment.clone().filter(|payment| payment.recipient.address == address);
        Self { address, payment, ..self.clone() }
    }

    pub fn on_memo_changed(&self, memo: String) -> Self {
        Self { memo, ..self.clone() }
    }

    pub fn on_payment(&self, payment: GemPaymentRecipient) -> Self {
        Self {
            address: payment.recipient.address.clone(),
            memo: payment.recipient.memo.clone().unwrap_or_else(|| self.memo.clone()),
            payment: Some(payment),
        }
    }

    pub fn next(&self, recipient_type: GemRecipientType, name_state: GemNameRecordState) -> Result<GemRecipientNext, GemRecipientError> {
        let references = self.payment.as_ref().map(|payment| payment.recipient.references.clone()).unwrap_or_default();
        let recipient = super::rules::recipient(recipient_type.asset().chain(), &self.address, &name_state, Some(self.memo.clone()), references)?;
        Ok(super::rules::next_step(
            recipient_type,
            GemPaymentRecipient {
                recipient,
                amount: self.payment.as_ref().and_then(|payment| payment.amount.clone()),
            },
        ))
    }
}
