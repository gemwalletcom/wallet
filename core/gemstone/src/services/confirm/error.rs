use crate::GemstoneError;
use crate::gateway::GatewayError;
use crate::models::custom_types::GemBigInt;
use crate::services::error::GemServiceError;
use crate::signer::GemSignerError;
use primitives::{Asset, AssetId, Chain};

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
        required: GemBigInt,
        available: GemBigInt,
    },
    InsufficientNetworkFee {
        asset: Asset,
        required: Option<GemBigInt>,
        available: Option<GemBigInt>,
    },
    MinimumAccountBalanceTooLow {
        asset: Asset,
        required: GemBigInt,
        available: GemBigInt,
    },
    SenderMismatch {
        from: String,
        signer: String,
    },
    Sign {
        error: GemSignerError,
        msg: String,
    },
    ApprovalInvalid {
        msg: String,
    },
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
            Self::MinimumAccountBalanceTooLow { asset, required, .. } => write!(f, "{} balance must stay above {required}", asset.symbol),
            Self::SenderMismatch { from, signer } => write!(f, "transaction was built for {from} but would be signed by {signer}"),
            Self::Network { msg } | Self::Load { msg } | Self::Broadcast { msg, .. } | Self::Record { msg } | Self::Sign { msg, .. } | Self::ApprovalInvalid { msg } => {
                write!(f, "{msg}")
            }
        }
    }
}

impl std::error::Error for GemConfirmError {}

impl From<GemstoneError> for GemConfirmError {
    fn from(error: GemstoneError) -> Self {
        match error {
            GemstoneError::SignerError { error, msg } => Self::Sign { error, msg },
            GemstoneError::AnyError { msg } => Self::Sign {
                error: GemSignerError::SigningError(msg.clone()),
                msg,
            },
        }
    }
}

impl From<GemServiceError> for GemConfirmError {
    fn from(error: GemServiceError) -> Self {
        Self::Load { msg: error.to_string() }
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
