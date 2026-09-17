use crate::GemstoneError;
use crate::gateway::GatewayError;
use crate::models::custom_types::GemBigInt;
use crate::payment::GemPaymentError;
use crate::services::balance::GemBalanceRequirement;
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
        requirement: GemBalanceRequirement,
    },
    NetworkFeeRequired {
        asset: Asset,
        requirement: GemBalanceRequirement,
    },
    NetworkFeeMissing {
        asset: Asset,
    },
    MinimumAccountBalance {
        asset: Asset,
        required: GemBigInt,
    },
    SwapMinimum {
        asset: Asset,
        provider: SwapProvider,
        provider_name: String,
        requirement: GemBalanceRequirement,
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
                requirement: requirement.clone(),
            },
            Self::InsufficientNetworkFee { asset, requirement } => match requirement {
                Some(requirement) => GemConfirmErrorDisplay::NetworkFeeRequired {
                    asset: asset.clone(),
                    requirement: requirement.clone(),
                },
                None => GemConfirmErrorDisplay::NetworkFeeMissing { asset: asset.clone() },
            },
            Self::MinimumAccountBalanceTooLow { asset, requirement } => GemConfirmErrorDisplay::MinimumAccountBalance {
                asset: asset.clone(),
                required: requirement.required.clone(),
            },
            Self::BelowSwapMinimum {
                asset,
                provider,
                provider_name,
                requirement,
            } => GemConfirmErrorDisplay::SwapMinimum {
                asset: asset.clone(),
                provider: *provider,
                provider_name: provider_name.clone(),
                requirement: requirement.clone(),
            },
            Self::Sign { error, chain, msg } => match error {
                GemSignerError::DustThreshold => GemConfirmErrorDisplay::DustThreshold { chain: *chain },
                GemSignerError::InsufficientFunds => GemConfirmErrorDisplay::InsufficientFunds,
                GemSignerError::InvalidInput(_) | GemSignerError::SigningError(_) | GemSignerError::SwapValueBelowMinimum { .. } => {
                    GemConfirmErrorDisplay::Message { msg: msg.clone() }
                }
            },
            Self::Payment { status } => GemConfirmErrorDisplay::Payment { status: *status },
            Self::BalanceMissing { .. } | Self::Network { .. } | Self::Load { .. } | Self::Broadcast { .. } | Self::Record { .. } | Self::ApprovalInvalid { .. } => {
                GemConfirmErrorDisplay::Message { msg: self.to_string() }
            }
        }
    }
}

#[uniffi::export]
impl GemConfirmErrorDisplay {
    pub fn has_info_sheet(&self) -> bool {
        match self {
            Self::Malicious
            | Self::MemoRequired { .. }
            | Self::BalanceRequired { .. }
            | Self::NetworkFeeRequired { .. }
            | Self::NetworkFeeMissing { .. }
            | Self::MinimumAccountBalance { .. }
            | Self::SwapMinimum { .. }
            | Self::DustThreshold { .. } => true,
            Self::Offline
            | Self::FeeRatesMissing
            | Self::Cancelled
            | Self::AccountMissing
            | Self::Unknown
            | Self::InsufficientFunds
            | Self::Payment { .. }
            | Self::Message { .. } => false,
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
            Self::BelowSwapMinimum {
                asset,
                provider_name,
                requirement,
                ..
            } => {
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
        assert!(matches!(
            GemConfirmError::from(GemServiceError::Store { msg: "x".to_string() }),
            GemConfirmError::Load { .. }
        ));
    }

    #[test]
    fn test_the_display_collapses_the_branches_both_apps_would_re_derive() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let requirement = GemBalanceRequirement::new(GemBigInt::from(10), GemBigInt::from(4));

        let with_requirement = GemConfirmError::InsufficientNetworkFee {
            asset: asset.clone(),
            requirement: Some(requirement.clone()),
        };
        let without = GemConfirmError::InsufficientNetworkFee {
            asset: asset.clone(),
            requirement: None,
        };
        assert!(matches!(with_requirement.display(), GemConfirmErrorDisplay::NetworkFeeRequired { .. }));
        assert!(matches!(without.display(), GemConfirmErrorDisplay::NetworkFeeMissing { .. }));

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
