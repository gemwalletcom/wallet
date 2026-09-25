use crate::GemstoneError;
use crate::formatted_number::GemFormattedNumber;
use crate::gateway::GatewayError;
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemNoticeKind, suspicious_address_notice};
use crate::payment::GemPaymentError;
use crate::precision::GemValueStyle;
use crate::services::balance::GemBalanceRequirement;
use crate::services::confirm::model::GemAcquireAsset;
use crate::services::error::GemServiceError;
use crate::signer::GemSignerError;
use primitives::{Asset, AssetId, Chain, PaymentStatus, SwapProvider};

#[derive(Debug, Clone, uniffi::Error)]
pub enum GemConfirmError {
    ScanMalicious,
    ScanMemoRequired {
        symbol: String,
    },
    FeeRatesMissing,
    Offline,
    Network {
        msg: String,
    },
    Load {
        msg: String,
    },
    Broadcast {
        hashes: Vec<String>,
        msg: String,
    },
    Record {
        msg: String,
    },
    AccountMissing {
        chain: Chain,
    },
    BalanceMissing {
        asset_id: AssetId,
    },
    InsufficientBalance {
        asset: Asset,
        requirement: GemBalanceRequirement,
    },
    InsufficientNetworkFee {
        asset: Asset,
        requirement: Option<GemBalanceRequirement>,
    },
    MinimumAccountBalanceTooLow {
        asset: Asset,
        requirement: GemBalanceRequirement,
    },
    DestinationAccountActivation {
        asset: Asset,
        required: GemBigInt,
    },
    BelowSwapMinimum {
        asset: Asset,
        provider: SwapProvider,
        provider_name: String,
        requirement: GemBalanceRequirement,
    },
    SenderMismatch {
        from: String,
        signer: String,
    },
    Sign {
        error: GemSignerError,
        chain: Chain,
        msg: String,
    },
    ApprovalInvalid {
        msg: String,
    },
    Payment {
        status: PaymentStatus,
    },
    Cancelled,
}

impl GemConfirmError {
    pub(crate) fn is_account_missing(&self) -> bool {
        match self {
            Self::AccountMissing { .. } => true,
            Self::ScanMalicious
            | Self::ScanMemoRequired { .. }
            | Self::FeeRatesMissing
            | Self::Offline
            | Self::Network { .. }
            | Self::Load { .. }
            | Self::Broadcast { .. }
            | Self::Record { .. }
            | Self::BalanceMissing { .. }
            | Self::InsufficientBalance { .. }
            | Self::InsufficientNetworkFee { .. }
            | Self::MinimumAccountBalanceTooLow { .. }
            | Self::DestinationAccountActivation { .. }
            | Self::BelowSwapMinimum { .. }
            | Self::SenderMismatch { .. }
            | Self::Sign { .. }
            | Self::ApprovalInvalid { .. }
            | Self::Payment { .. }
            | Self::Cancelled => false,
        }
    }

    pub(crate) fn notice(&self) -> Option<GemListRow> {
        match self {
            Self::ScanMalicious => Some(suspicious_address_notice(GemNoticeKind::Error)),
            Self::ScanMemoRequired { .. }
            | Self::FeeRatesMissing
            | Self::Offline
            | Self::Network { .. }
            | Self::Load { .. }
            | Self::Broadcast { .. }
            | Self::Record { .. }
            | Self::AccountMissing { .. }
            | Self::BalanceMissing { .. }
            | Self::InsufficientBalance { .. }
            | Self::InsufficientNetworkFee { .. }
            | Self::MinimumAccountBalanceTooLow { .. }
            | Self::DestinationAccountActivation { .. }
            | Self::BelowSwapMinimum { .. }
            | Self::SenderMismatch { .. }
            | Self::Sign { .. }
            | Self::ApprovalInvalid { .. }
            | Self::Payment { .. }
            | Self::Cancelled => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmErrorDisplay {
    Offline,
    Malicious,
    MemoRequired {
        symbol: String,
    },
    FeeRatesMissing,
    Cancelled,
    AccountMissing,
    Unknown,
    BalanceRequired {
        asset: Asset,
        requirement: GemConfirmRequirement,
    },
    NetworkFeeRequired {
        asset: Asset,
        title: String,
        requirement: GemConfirmRequirement,
    },
    NetworkFeeMissing {
        asset: Asset,
        title: String,
    },
    MinimumAccountBalance {
        asset: Asset,
        required: GemFormattedNumber,
    },
    DestinationAccountActivation {
        asset: Asset,
        required: GemFormattedNumber,
    },
    SwapMinimum {
        asset: Asset,
        provider: SwapProvider,
        provider_name: String,
        requirement: GemConfirmRequirement,
    },
    DustThreshold {
        chain: Chain,
    },
    InsufficientFunds,
    Payment {
        status: PaymentStatus,
    },
    Message {
        msg: String,
    },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemConfirmRequirement {
    pub required: GemFormattedNumber,
    pub available: GemFormattedNumber,
    pub shortfall: GemFormattedNumber,
}

impl GemConfirmRequirement {
    pub fn new(requirement: &GemBalanceRequirement, asset: &Asset) -> Self {
        let amount = |value: &GemBigInt| GemFormattedNumber::asset_amount(value, asset, GemValueStyle::Auto);
        Self {
            required: amount(&requirement.required),
            available: amount(&requirement.available),
            shortfall: amount(&requirement.shortfall),
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmErrorSheet {
    BalanceRequired,
    NetworkFeeRequired,
    NetworkFeeMissing,
    MinimumAccountBalance,
    SwapMinimum { provider: SwapProvider, provider_name: String },
    DustThreshold { chain: Chain },
    Malicious,
    MemoRequired { symbol: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemConfirmErrorInfo {
    pub sheet: GemConfirmErrorSheet,
    pub asset: Option<Asset>,
    pub title: String,
    pub required: Option<GemFormattedNumber>,
    pub required_fiat: Option<GemFormattedNumber>,
    pub available: Option<GemFormattedNumber>,
    pub shortfall: Option<GemFormattedNumber>,
    pub acquire: Option<GemAcquireAsset>,
}

/// The one rule behind `GemConfirmation::error_info`, exported so a test double can answer it faithfully.
#[uniffi::export]
pub fn confirm_error_info(error: GemConfirmError, prices: Vec<primitives::AssetPrice>, currency: primitives::currency::Currency, input_asset_id: AssetId, fee_asset_id: AssetId) -> Option<GemConfirmErrorInfo> {
    super::rules::error_info(&error.display(), &prices, currency, &input_asset_id, &fee_asset_id)
}

#[uniffi::export]
impl GemConfirmError {
    pub fn display(&self) -> GemConfirmErrorDisplay {
        match self {
            Self::Offline => GemConfirmErrorDisplay::Offline,
            Self::ScanMalicious => GemConfirmErrorDisplay::Malicious,
            Self::ScanMemoRequired { symbol } => GemConfirmErrorDisplay::MemoRequired { symbol: symbol.clone() },
            Self::FeeRatesMissing => GemConfirmErrorDisplay::FeeRatesMissing,
            Self::Cancelled => GemConfirmErrorDisplay::Cancelled,
            Self::AccountMissing { .. } => GemConfirmErrorDisplay::AccountMissing,
            Self::SenderMismatch { .. } => GemConfirmErrorDisplay::Unknown,
            Self::InsufficientBalance { asset, requirement } => GemConfirmErrorDisplay::BalanceRequired {
                asset: asset.clone(),
                requirement: GemConfirmRequirement::new(requirement, asset),
            },
            Self::InsufficientNetworkFee { asset, requirement } => match requirement {
                Some(requirement) => GemConfirmErrorDisplay::NetworkFeeRequired {
                    title: asset.display_title(),
                    asset: asset.clone(),
                    requirement: GemConfirmRequirement::new(requirement, asset),
                },
                None => GemConfirmErrorDisplay::NetworkFeeMissing {
                    title: asset.display_title(),
                    asset: asset.clone(),
                },
            },
            Self::MinimumAccountBalanceTooLow { asset, requirement } => GemConfirmErrorDisplay::MinimumAccountBalance {
                asset: asset.clone(),
                required: GemFormattedNumber::asset_amount(&requirement.required, asset, GemValueStyle::Auto),
            },
            Self::DestinationAccountActivation { asset, required } => GemConfirmErrorDisplay::DestinationAccountActivation {
                asset: asset.clone(),
                required: GemFormattedNumber::asset_amount(required, asset, GemValueStyle::Auto),
            },
            Self::BelowSwapMinimum { asset, provider, provider_name, requirement } => GemConfirmErrorDisplay::SwapMinimum {
                asset: asset.clone(),
                provider: *provider,
                provider_name: provider_name.clone(),
                requirement: GemConfirmRequirement::new(requirement, asset),
            },
            Self::Sign { error, chain, msg } => match error {
                GemSignerError::DustThreshold => GemConfirmErrorDisplay::DustThreshold { chain: *chain },
                GemSignerError::InsufficientFunds => GemConfirmErrorDisplay::InsufficientFunds,
                GemSignerError::InvalidInput(_) | GemSignerError::SigningError(_) | GemSignerError::SwapValueBelowMinimum { .. } => GemConfirmErrorDisplay::Message { msg: msg.clone() },
            },
            Self::Payment { status } => GemConfirmErrorDisplay::Payment { status: *status },
            Self::BalanceMissing { .. } | Self::Network { .. } | Self::Load { .. } | Self::Broadcast { .. } | Self::Record { .. } | Self::ApprovalInvalid { .. } => GemConfirmErrorDisplay::Message { msg: self.to_string() },
        }
    }
}

#[uniffi::export]
impl GemConfirmErrorDisplay {
    pub fn has_info_sheet(&self) -> bool {
        self.sheet().is_some()
    }
}

impl GemConfirmErrorDisplay {
    pub(crate) fn sheet(&self) -> Option<GemConfirmErrorSheet> {
        match self {
            Self::Malicious => Some(GemConfirmErrorSheet::Malicious),
            Self::MemoRequired { symbol } => Some(GemConfirmErrorSheet::MemoRequired { symbol: symbol.clone() }),
            Self::BalanceRequired { .. } => Some(GemConfirmErrorSheet::BalanceRequired),
            Self::NetworkFeeRequired { .. } => Some(GemConfirmErrorSheet::NetworkFeeRequired),
            Self::NetworkFeeMissing { .. } => Some(GemConfirmErrorSheet::NetworkFeeMissing),
            Self::MinimumAccountBalance { .. } => Some(GemConfirmErrorSheet::MinimumAccountBalance),
            Self::SwapMinimum { provider, provider_name, .. } => Some(GemConfirmErrorSheet::SwapMinimum {
                provider: *provider,
                provider_name: provider_name.clone(),
            }),
            Self::DustThreshold { chain } => Some(GemConfirmErrorSheet::DustThreshold { chain: *chain }),
            Self::Offline | Self::FeeRatesMissing | Self::Cancelled | Self::AccountMissing | Self::Unknown | Self::InsufficientFunds | Self::DestinationAccountActivation { .. } | Self::Payment { .. } | Self::Message { .. } => None,
        }
    }
}

impl std::fmt::Display for GemConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ScanMalicious => write!(f, "transaction flagged as malicious"),
            Self::ScanMemoRequired { symbol } => write!(f, "{symbol} transfer requires a memo"),
            Self::FeeRatesMissing => write!(f, "fee rates not found"),
            Self::Offline => write!(f, "network offline"),
            Self::AccountMissing { chain } => write!(f, "wallet has no {chain} account"),
            Self::BalanceMissing { asset_id } => write!(f, "no stored balance for {asset_id}"),
            Self::InsufficientBalance { asset, .. } => write!(f, "not enough {} balance", asset.symbol),
            Self::InsufficientNetworkFee { asset, .. } => write!(f, "not enough {} to pay the network fee", asset.symbol),
            Self::MinimumAccountBalanceTooLow { asset, requirement } => write!(f, "{} balance must stay above {}", asset.symbol, requirement.required),
            Self::DestinationAccountActivation { asset, required } => write!(f, "{} destination activation requires {}", asset.symbol, required),
            Self::BelowSwapMinimum { asset, provider_name, requirement, .. } => {
                write!(f, "{} amount is below the {} minimum {}", asset.symbol, provider_name, requirement.required)
            }
            Self::SenderMismatch { from, signer } => write!(f, "transaction was built for {from} but would be signed by {signer}"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Payment { status } => write!(f, "payment is {status:?}"),
            Self::Network { msg } | Self::Load { msg } | Self::Broadcast { msg, .. } | Self::Record { msg } | Self::Sign { msg, .. } | Self::ApprovalInvalid { msg } => {
                write!(f, "{msg}")
            }
        }
    }
}

impl std::error::Error for GemConfirmError {}

pub(super) fn sign_error(chain: Chain, error: GemstoneError) -> GemConfirmError {
    match error {
        GemstoneError::Cancelled => GemConfirmError::Cancelled,
        GemstoneError::SignerError { error, msg } => GemConfirmError::Sign { error, chain, msg },
        GemstoneError::AnyError { msg } => GemConfirmError::Sign {
            error: GemSignerError::SigningError(msg.clone()),
            chain,
            msg,
        },
    }
}

impl From<GemPaymentError> for GemConfirmError {
    fn from(error: GemPaymentError) -> Self {
        match error {
            GemPaymentError::Status { status } => Self::Payment { status },
            error => Self::Load { msg: error.to_string() },
        }
    }
}

impl From<GemServiceError> for GemConfirmError {
    fn from(error: GemServiceError) -> Self {
        match error {
            GemServiceError::Cancelled => Self::Cancelled,
            GemServiceError::Offline => Self::Offline,
            error => Self::Load { msg: error.to_string() },
        }
    }
}

pub(super) fn load_error(error: GatewayError) -> GemConfirmError {
    match error {
        GatewayError::Offline => GemConfirmError::Offline,
        GatewayError::NetworkError { msg } => GemConfirmError::Network { msg },
        error => GemConfirmError::Load { msg: error.to_string() },
    }
}

pub(super) fn broadcast_error(hashes: Vec<String>, error: GatewayError) -> GemConfirmError {
    match error {
        GatewayError::Offline if hashes.is_empty() => GemConfirmError::Offline,
        GatewayError::NetworkError { msg } if hashes.is_empty() => GemConfirmError::Network { msg },
        error => GemConfirmError::Broadcast { hashes, msg: error.to_string() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_a_flagged_recipient_shows_the_suspicious_address_notice() {
        assert_eq!(GemConfirmError::ScanMalicious.notice(), Some(suspicious_address_notice(GemNoticeKind::Error)));
        assert_eq!(GemConfirmError::ScanMemoRequired { symbol: "XRP".to_string() }.notice(), None);
        assert_eq!(GemConfirmError::Offline.notice(), None);
    }

    #[test]
    fn test_a_cancelled_signer_is_a_cancel_not_a_failure() {
        assert!(matches!(sign_error(Chain::Ethereum, GemstoneError::Cancelled), GemConfirmError::Cancelled));
        assert!(matches!(
            sign_error(Chain::Ethereum, GemstoneError::AnyError { msg: "boom".into() }),
            GemConfirmError::Sign { error: GemSignerError::SigningError(msg), chain: Chain::Ethereum, .. } if msg == "boom"
        ));
    }

    #[test]
    fn test_a_cancelled_keystore_prompt_is_a_cancel_not_a_load_failure() {
        assert!(matches!(GemConfirmError::from(GemServiceError::Cancelled), GemConfirmError::Cancelled));
        assert!(matches!(GemConfirmError::from(GemServiceError::Offline), GemConfirmError::Offline));
        assert!(matches!(GemConfirmError::from(GemServiceError::Store { msg: "x".to_string() }), GemConfirmError::Load { .. }));
    }

    #[test]
    fn test_the_display_collapses_the_branches_both_apps_would_re_derive() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let requirement = GemBalanceRequirement::new(GemBigInt::from(10), GemBigInt::from(4));

        let with_requirement = GemConfirmError::InsufficientNetworkFee {
            asset: asset.clone(),
            requirement: Some(requirement.clone()),
        };
        let without = GemConfirmError::InsufficientNetworkFee { asset: asset.clone(), requirement: None };
        assert!(matches!(with_requirement.display(), GemConfirmErrorDisplay::NetworkFeeRequired { .. }));
        assert!(matches!(
            without.display(),
            GemConfirmErrorDisplay::NetworkFeeMissing { ref title, .. } if *title == asset.display_title()
        ));

        let dust = GemConfirmError::Sign {
            error: GemSignerError::DustThreshold,
            chain: Chain::Bitcoin,
            msg: "dust".to_string(),
        };
        let signing = GemConfirmError::Sign {
            error: GemSignerError::SigningError("boom".to_string()),
            chain: Chain::Bitcoin,
            msg: "boom".to_string(),
        };
        assert!(matches!(dust.display(), GemConfirmErrorDisplay::DustThreshold { chain: Chain::Bitcoin }));
        assert!(matches!(signing.display(), GemConfirmErrorDisplay::Message { msg } if msg == "boom"));

        assert!(dust.display().has_info_sheet());
        assert!(!signing.display().has_info_sheet());
        assert!(!GemConfirmError::Cancelled.display().has_info_sheet());
    }

    #[test]
    fn test_gateway_errors_keep_their_kind() {
        match load_error(GatewayError::NetworkError { msg: "timeout".to_string() }) {
            GemConfirmError::Network { msg } => assert_eq!(msg, "timeout"),
            error => panic!("expected a network error, got {error:?}"),
        }
        assert!(matches!(load_error(GatewayError::Offline), GemConfirmError::Offline));
        assert!(matches!(broadcast_error(vec![], GatewayError::Offline), GemConfirmError::Offline));
        assert!(matches!(broadcast_error(vec!["h1".to_string()], GatewayError::Offline), GemConfirmError::Broadcast { .. }));
        match load_error(GatewayError::PlatformError { msg: "dust".to_string() }) {
            GemConfirmError::Load { msg } => assert_eq!(msg, "Platform error: dust"),
            error => panic!("expected a load error, got {error:?}"),
        }
        match broadcast_error(vec![], GatewayError::NetworkError { msg: "offline".to_string() }) {
            GemConfirmError::Network { msg } => assert_eq!(msg, "offline"),
            error => panic!("expected a network error, got {error:?}"),
        }
        match broadcast_error(vec!["h1".to_string()], GatewayError::NetworkError { msg: "offline".to_string() }) {
            GemConfirmError::Broadcast { hashes, .. } => assert_eq!(hashes, vec!["h1".to_string()]),
            error => panic!("expected a partial broadcast error, got {error:?}"),
        }
    }
}
