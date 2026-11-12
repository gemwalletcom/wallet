use super::rules::ConfirmInput;
use crate::GemstoneError;
use crate::formatted_number::GemFormattedNumber;
use crate::gateway::GatewayError;
use crate::models::custom_types::GemBigInt;
use crate::models::list::{GemListRow, GemNoticeKind, suspicious_address_notice};
use crate::services::balance::GemBalanceRequirement;
use crate::services::confirm::model::GemAcquireAsset;
use crate::services::error::GemServiceError;
use crate::services::info_sheet::{GemInfoAmount, GemInfoDescription};
use number_formatter::ValueStyle;
use payment::PaymentError;
use primitives::{Asset, AssetId, Chain, ChainAsset, PaymentStatus, SignerError, SwapProvider, TransactionInputType};
use swapper::SwapperError;

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
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
        msg: String,
    },
    PartialBroadcast {
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
        error: SignerError,
        chain: Chain,
        msg: String,
    },
    InvalidRequest {
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
            | Self::PartialBroadcast { .. }
            | Self::Record { .. }
            | Self::BalanceMissing { .. }
            | Self::InsufficientBalance { .. }
            | Self::InsufficientNetworkFee { .. }
            | Self::MinimumAccountBalanceTooLow { .. }
            | Self::DestinationAccountActivation { .. }
            | Self::BelowSwapMinimum { .. }
            | Self::SenderMismatch { .. }
            | Self::Sign { .. }
            | Self::InvalidRequest { .. }
            | Self::ApprovalInvalid { .. }
            | Self::Payment { .. }
            | Self::Cancelled => false,
        }
    }

    pub(crate) fn has_sent_transactions(&self) -> bool {
        match self {
            Self::PartialBroadcast { .. } => true,
            Self::ScanMalicious
            | Self::Broadcast { .. }
            | Self::ScanMemoRequired { .. }
            | Self::FeeRatesMissing
            | Self::Offline
            | Self::Network { .. }
            | Self::Load { .. }
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
            | Self::InvalidRequest { .. }
            | Self::ApprovalInvalid { .. }
            | Self::Payment { .. }
            | Self::Cancelled => false,
        }
    }

    pub(crate) fn includes_network_fee(&self, fee_asset_id: &AssetId) -> bool {
        match self {
            Self::InsufficientBalance { asset, .. } | Self::MinimumAccountBalanceTooLow { asset, .. } => &asset.id == fee_asset_id,
            Self::InsufficientNetworkFee { .. } => true,
            Self::ScanMalicious
            | Self::ScanMemoRequired { .. }
            | Self::FeeRatesMissing
            | Self::Offline
            | Self::Network { .. }
            | Self::Load { .. }
            | Self::Broadcast { .. }
            | Self::PartialBroadcast { .. }
            | Self::Record { .. }
            | Self::AccountMissing { .. }
            | Self::BalanceMissing { .. }
            | Self::DestinationAccountActivation { .. }
            | Self::BelowSwapMinimum { .. }
            | Self::SenderMismatch { .. }
            | Self::Sign { .. }
            | Self::InvalidRequest { .. }
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
            | Self::PartialBroadcast { .. }
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
            | Self::InvalidRequest { .. }
            | Self::ApprovalInvalid { .. }
            | Self::Payment { .. }
            | Self::Cancelled => None,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmErrorDisplay {
    Offline,
    FeeRatesMissing,
    Cancelled,
    AccountMissing,
    Unknown,
    Description { description: GemInfoDescription },
    DestinationAccountActivation { asset: Asset, required: GemFormattedNumber },
    DustThreshold { chain: Chain },
    InsufficientFunds,
    Payment { status: PaymentStatus },
    InvalidRequest,
    Message { msg: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemConfirmRequirement {
    pub required: GemFormattedNumber,
    pub available: GemFormattedNumber,
    pub shortfall: GemFormattedNumber,
}

impl GemConfirmRequirement {
    pub fn new(requirement: &GemBalanceRequirement, asset: &Asset) -> Self {
        let amount = |value: &GemBigInt| GemFormattedNumber::asset_amount(value, asset, ValueStyle::Auto);
        let (required, shortfall) = (amount(&requirement.required), amount(&requirement.shortfall));
        let fraction_places = |value: f64| number_formatter::precision::value(ValueStyle::Auto, value).fraction_places(value);
        let places = fraction_places(shortfall.value);
        match fraction_places(required.value) < places {
            true => {
                let exact = |value: &GemBigInt| GemFormattedNumber::asset_amount(&truncated(value, asset.decimals, places), asset, ValueStyle::Full);
                Self {
                    required: exact(&requirement.required),
                    available: exact(&requirement.available),
                    shortfall,
                }
            }
            false => Self {
                required,
                available: amount(&requirement.available),
                shortfall,
            },
        }
    }
}

fn truncated(value: &GemBigInt, decimals: u32, places: usize) -> GemBigInt {
    let unit = GemBigInt::from(10).pow(decimals.saturating_sub(places as u32));
    value - value % unit
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
    pub description: GemInfoDescription,
    pub acquire: Option<GemAcquireAsset>,
}

#[uniffi::export]
pub fn confirm_error_info(error: GemConfirmError, prices: Vec<primitives::AssetPrice>, currency: primitives::currency::Currency, input_asset_id: AssetId, fee_asset_id: AssetId) -> Option<GemConfirmErrorInfo> {
    super::rules::error_info(&error, &prices, currency, &input_asset_id, &fee_asset_id)
}

#[uniffi::export]
impl GemConfirmError {
    pub fn display(&self) -> GemConfirmErrorDisplay {
        let no_fiat = |_: &GemFormattedNumber| None;
        let described = |description: GemInfoDescription| GemConfirmErrorDisplay::Description { description };
        match self {
            Self::Offline => GemConfirmErrorDisplay::Offline,
            Self::ScanMalicious => described(GemInfoDescription::MaliciousTransaction),
            Self::ScanMemoRequired { symbol } => described(GemInfoDescription::MemoRequired { symbol: symbol.clone() }),
            Self::FeeRatesMissing => GemConfirmErrorDisplay::FeeRatesMissing,
            Self::Cancelled => GemConfirmErrorDisplay::Cancelled,
            Self::AccountMissing { .. } => GemConfirmErrorDisplay::AccountMissing,
            Self::SenderMismatch { .. } => GemConfirmErrorDisplay::Unknown,
            Self::InsufficientBalance { asset, requirement } => described(balance_required_description(asset, requirement)),
            Self::InsufficientNetworkFee { asset, requirement } => described(network_fee_description(asset, requirement.as_ref(), no_fiat)),
            Self::MinimumAccountBalanceTooLow { asset, requirement } => described(minimum_account_balance_description(asset, requirement)),
            Self::DestinationAccountActivation { asset, required } => GemConfirmErrorDisplay::DestinationAccountActivation {
                asset: asset.clone(),
                required: GemFormattedNumber::asset_amount(required, asset, ValueStyle::Auto),
            },
            Self::BelowSwapMinimum { asset, provider_name, requirement, .. } => described(swap_minimum_description(asset, provider_name, requirement, no_fiat)),
            Self::Sign { error, chain, msg } => match error {
                SignerError::DustThreshold => GemConfirmErrorDisplay::DustThreshold { chain: *chain },
                SignerError::InsufficientFunds => GemConfirmErrorDisplay::InsufficientFunds,
                SignerError::InvalidInput(_) | SignerError::SigningError(_) => GemConfirmErrorDisplay::Message { msg: msg.clone() },
            },
            Self::Payment { status } => GemConfirmErrorDisplay::Payment { status: *status },
            Self::InvalidRequest { .. } => GemConfirmErrorDisplay::InvalidRequest,
            Self::BalanceMissing { .. } | Self::Network { .. } | Self::Load { .. } | Self::Broadcast { .. } | Self::PartialBroadcast { .. } | Self::Record { .. } | Self::ApprovalInvalid { .. } => {
                GemConfirmErrorDisplay::Message { msg: self.to_string() }
            }
        }
    }
}

impl GemConfirmError {
    pub(crate) fn sheet(&self) -> Option<GemConfirmErrorSheet> {
        match self {
            Self::ScanMalicious => Some(GemConfirmErrorSheet::Malicious),
            Self::ScanMemoRequired { symbol } => Some(GemConfirmErrorSheet::MemoRequired { symbol: symbol.clone() }),
            Self::InsufficientBalance { .. } => Some(GemConfirmErrorSheet::BalanceRequired),
            Self::InsufficientNetworkFee { requirement: Some(_), .. } => Some(GemConfirmErrorSheet::NetworkFeeRequired),
            Self::InsufficientNetworkFee { requirement: None, .. } => Some(GemConfirmErrorSheet::NetworkFeeMissing),
            Self::MinimumAccountBalanceTooLow { .. } => Some(GemConfirmErrorSheet::MinimumAccountBalance),
            Self::BelowSwapMinimum { provider, provider_name, .. } => Some(GemConfirmErrorSheet::SwapMinimum {
                provider: *provider,
                provider_name: provider_name.clone(),
            }),
            Self::Sign { error, chain, .. } => match error {
                SignerError::DustThreshold => Some(GemConfirmErrorSheet::DustThreshold { chain: *chain }),
                SignerError::InsufficientFunds | SignerError::InvalidInput(_) | SignerError::SigningError(_) => None,
            },
            Self::Offline
            | Self::FeeRatesMissing
            | Self::Cancelled
            | Self::AccountMissing { .. }
            | Self::SenderMismatch { .. }
            | Self::DestinationAccountActivation { .. }
            | Self::Payment { .. }
            | Self::BalanceMissing { .. }
            | Self::Network { .. }
            | Self::Load { .. }
            | Self::Broadcast { .. }
            | Self::PartialBroadcast { .. }
            | Self::Record { .. }
            | Self::InvalidRequest { .. }
            | Self::ApprovalInvalid { .. } => None,
        }
    }
}

pub(super) fn balance_required_description(asset: &Asset, requirement: &GemBalanceRequirement) -> GemInfoDescription {
    let requirement = GemConfirmRequirement::new(requirement, asset);
    GemInfoDescription::BalanceRequired {
        required: Some(requirement.required),
        available: Some(requirement.available),
        shortfall: Some(requirement.shortfall),
    }
}

pub(super) fn network_fee_description(asset: &Asset, requirement: Option<&GemBalanceRequirement>, required_fiat: impl Fn(&GemFormattedNumber) -> Option<GemFormattedNumber>) -> GemInfoDescription {
    match requirement {
        Some(requirement) => {
            let requirement = GemConfirmRequirement::new(requirement, asset);
            GemInfoDescription::InsufficientNetworkFeeBalance {
                required: Some(GemInfoAmount {
                    fiat: required_fiat(&requirement.required),
                    amount: requirement.required,
                }),
                network: ChainAsset::from_chain(asset.chain()).network_name,
                available: requirement.available,
                shortfall: Some(requirement.shortfall),
            }
        }
        None => GemInfoDescription::InsufficientNetworkFee { title: asset.display_title() },
    }
}

pub(super) fn minimum_account_balance_description(asset: &Asset, requirement: &GemBalanceRequirement) -> GemInfoDescription {
    GemInfoDescription::AccountMinimumBalance {
        amount: Some(GemFormattedNumber::asset_amount(&requirement.required, asset, ValueStyle::Auto)),
    }
}

pub(super) fn swap_minimum_description(asset: &Asset, provider_name: &str, requirement: &GemBalanceRequirement, required_fiat: impl Fn(&GemFormattedNumber) -> Option<GemFormattedNumber>) -> GemInfoDescription {
    let requirement = GemConfirmRequirement::new(requirement, asset);
    GemInfoDescription::SwapMinimumAmount {
        provider: provider_name.to_string(),
        required: Some(GemInfoAmount {
            fiat: required_fiat(&requirement.required),
            amount: requirement.required,
        }),
        available: Some(requirement.available),
        shortfall: Some(requirement.shortfall),
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
            Self::Network { msg } | Self::Load { msg } | Self::Broadcast { msg } | Self::PartialBroadcast { msg, .. } | Self::Record { msg } | Self::Sign { msg, .. } | Self::InvalidRequest { msg } | Self::ApprovalInvalid { msg } => {
                write!(f, "{msg}")
            }
        }
    }
}

impl std::error::Error for GemConfirmError {}

pub(super) fn sign_error(input_type: &TransactionInputType, error: GemstoneError) -> GemConfirmError {
    match error {
        GemstoneError::Cancelled => GemConfirmError::Cancelled,
        GemstoneError::SignerError { error, msg } => signer_error(input_type, msg, error),
        GemstoneError::AnyError { msg } => GemConfirmError::Sign {
            error: SignerError::SigningError(msg.clone()),
            chain: input_type.get_asset().chain(),
            msg,
        },
    }
}

fn signer_error(input_type: &TransactionInputType, msg: String, error: SignerError) -> GemConfirmError {
    match error {
        SignerError::InvalidInput(_) if input_type.is_dapp_request() => GemConfirmError::InvalidRequest { msg },
        error => GemConfirmError::Sign {
            error,
            chain: input_type.get_asset().chain(),
            msg,
        },
    }
}

impl From<PaymentError> for GemConfirmError {
    fn from(error: PaymentError) -> Self {
        match error {
            PaymentError::Status { status } => Self::Payment { status },
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

impl From<SwapperError> for GemConfirmError {
    fn from(error: SwapperError) -> Self {
        match error {
            SwapperError::Offline => GemConfirmError::Offline,
            error => GemConfirmError::Load { msg: error.to_string() },
        }
    }
}

pub(super) fn load_error(input_type: &TransactionInputType, error: GatewayError) -> GemConfirmError {
    match error {
        GatewayError::Offline => GemConfirmError::Offline,
        GatewayError::NetworkError { msg } => GemConfirmError::Network { msg },
        GatewayError::SignerError { error } => signer_error(input_type, error.to_string(), error),
        error => GemConfirmError::Load { msg: error.to_string() },
    }
}

pub(super) fn broadcast_error(hashes: Vec<String>, error: GatewayError) -> GemConfirmError {
    if !hashes.is_empty() {
        return GemConfirmError::PartialBroadcast { hashes, msg: error.to_string() };
    }
    match error {
        GatewayError::Offline => GemConfirmError::Offline,
        GatewayError::NetworkError { msg } => GemConfirmError::Network { msg },
        error => GemConfirmError::Broadcast { msg: error.to_string() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::map_network_error;
    use primitives::TransferDataExtra;

    #[test]
    fn test_only_a_flagged_recipient_shows_the_suspicious_address_notice() {
        assert_eq!(GemConfirmError::ScanMalicious.notice(), Some(suspicious_address_notice(GemNoticeKind::Error)));
        assert_eq!(GemConfirmError::ScanMemoRequired { symbol: "XRP".to_string() }.notice(), None);
        assert_eq!(GemConfirmError::Offline.notice(), None);
    }

    #[test]
    fn test_a_cancelled_signer_is_a_cancel_not_a_failure() {
        let transfer = TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Ethereum) };
        assert_eq!(sign_error(&transfer, GemstoneError::Cancelled), GemConfirmError::Cancelled);
        assert_eq!(
            sign_error(&transfer, GemstoneError::AnyError { msg: "boom".into() }),
            GemConfirmError::Sign {
                error: SignerError::SigningError("boom".to_string()),
                chain: Chain::Ethereum,
                msg: "boom".to_string(),
            }
        );
    }

    #[test]
    fn test_a_cancelled_keystore_prompt_is_a_cancel_not_a_load_failure() {
        assert_eq!(GemConfirmError::from(GemServiceError::Cancelled), GemConfirmError::Cancelled);
        assert_eq!(GemConfirmError::from(GemServiceError::Offline), GemConfirmError::Offline);
        assert_eq!(GemConfirmError::from(GemServiceError::Store { msg: "x".to_string() }), GemConfirmError::Load { msg: "x".to_string() });
    }

    #[test]
    fn test_a_requirement_prints_the_required_and_available_amounts_apart() {
        let asset = Asset::from_chain(Chain::Blast);
        let wei = |value: u64| GemBigInt::from(value);
        let formatted = |value: u64, style: ValueStyle| GemFormattedNumber::asset_amount(&wei(value), &asset, style);

        assert_eq!(
            GemConfirmRequirement::new(&GemBalanceRequirement::new(wei(66_488_177_805_216_601), wei(66_488_046_605_216_601)), &asset),
            GemConfirmRequirement {
                required: formatted(66_488_177_800_000_000, ValueStyle::Full),
                available: formatted(66_488_046_600_000_000, ValueStyle::Full),
                shortfall: formatted(131_200_000_000, ValueStyle::Auto),
            },
            "0.0664881778 and 0.0664880466 instead of 0.06648 twice: cut to the shortfall's places, so the three amounts add up"
        );

        let apart = GemConfirmRequirement::new(&GemBalanceRequirement::new(wei(3_000_000_000_000_000_000), wei(1_000_000_000_000_000_000)), &asset);
        assert_eq!(apart.required, formatted(3_000_000_000_000_000_000, ValueStyle::Auto), "amounts the usual precision already shows apart keep it");
        assert_eq!(apart.available, formatted(1_000_000_000_000_000_000, ValueStyle::Auto));

        let one_wei = GemConfirmRequirement::new(&GemBalanceRequirement::new(wei(66_488_000_000_000_001), wei(66_488_000_000_000_000)), &asset);
        assert_eq!(one_wei.required.exact.as_deref(), Some("0.066488000000000001"), "past what a double holds, the digits stay exact");
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
        assert!(
            matches!(with_requirement.display(), GemConfirmErrorDisplay::Description { description: GemInfoDescription::InsufficientNetworkFeeBalance { ref network, .. } } if *network == ChainAsset::from_chain(Chain::Ethereum).network_name),
            "Core names the network, so neither app works it out"
        );
        assert_eq!(
            without.display(),
            GemConfirmErrorDisplay::Description {
                description: GemInfoDescription::InsufficientNetworkFee { title: asset.display_title() }
            }
        );

        let dust = GemConfirmError::Sign {
            error: SignerError::DustThreshold,
            chain: Chain::Bitcoin,
            msg: "dust".to_string(),
        };
        let signing = GemConfirmError::Sign {
            error: SignerError::SigningError("boom".to_string()),
            chain: Chain::Bitcoin,
            msg: "boom".to_string(),
        };
        assert_eq!(dust.display(), GemConfirmErrorDisplay::DustThreshold { chain: Chain::Bitcoin });
        assert_eq!(signing.display(), GemConfirmErrorDisplay::Message { msg: "boom".to_string() });

        assert!(dust.sheet().is_some());
        assert!(signing.sheet().is_none());
        assert!(GemConfirmError::Cancelled.sheet().is_none());
    }

    #[test]
    fn test_a_dapp_request_gem_cannot_read_shows_invalid_request() {
        let unreadable = SignerError::invalid_input("missing field `ref_block_bytes` at line 1 column 1160");
        let msg = unreadable.to_string();
        let dapp = TransactionInputType::mock_generic(Asset::from_chain(Chain::Tron), TransferDataExtra::mock());
        let payment = TransactionInputType::mock_payment(Asset::from_chain(Chain::Tron), TransferDataExtra::mock());
        let transfer = TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Tron) };

        let invalid_request = sign_error(&dapp, GemstoneError::from(unreadable.clone()));
        assert_eq!(invalid_request, GemConfirmError::InvalidRequest { msg: msg.clone() });
        assert_eq!(invalid_request.display(), GemConfirmErrorDisplay::InvalidRequest);
        assert_eq!(invalid_request.to_string(), msg, "the parser's text stays in the log");
        assert_eq!(
            load_error(&dapp, map_network_error(Box::new(unreadable.clone()))).display(),
            GemConfirmErrorDisplay::InvalidRequest,
            "found while loading the fee, as at signing"
        );
        assert_eq!(
            sign_error(&dapp, GemstoneError::from(SignerError::InsufficientFunds)).display(),
            GemConfirmErrorDisplay::InsufficientFunds,
            "a dapp request the balance can't cover still says so"
        );
        assert_eq!(
            sign_error(&payment, GemstoneError::from(unreadable.clone())).display(),
            GemConfirmErrorDisplay::Message { msg: msg.clone() },
            "a payment that can't be prepared shows the reason"
        );
        assert_eq!(sign_error(&transfer, GemstoneError::from(unreadable)).display(), GemConfirmErrorDisplay::Message { msg });
    }

    #[test]
    fn test_gateway_errors_keep_their_kind() {
        let transfer = TransactionInputType::Transfer { asset: Asset::from_chain(Chain::Bitcoin) };
        assert_eq!(load_error(&transfer, GatewayError::NetworkError { msg: "timeout".to_string() }), GemConfirmError::Network { msg: "timeout".to_string() });
        assert_eq!(load_error(&transfer, GatewayError::Offline), GemConfirmError::Offline);
        assert_eq!(
            load_error(&transfer, map_network_error(Box::new(SignerError::DustThreshold))).display(),
            GemConfirmErrorDisplay::DustThreshold { chain: Chain::Bitcoin },
            "dust found while loading the fee shows the dust explanation, as it does at signing"
        );
        assert_eq!(broadcast_error(vec![], GatewayError::Offline), GemConfirmError::Offline);
        assert_eq!(
            broadcast_error(vec!["h1".to_string()], GatewayError::Offline),
            GemConfirmError::PartialBroadcast {
                hashes: vec!["h1".to_string()],
                msg: GatewayError::Offline.to_string(),
            }
        );
        assert_eq!(
            load_error(&transfer, GatewayError::PlatformError { msg: "dust".to_string() }),
            GemConfirmError::Load { msg: "Platform error: dust".to_string() }
        );
        assert_eq!(broadcast_error(vec![], GatewayError::NetworkError { msg: "offline".to_string() }), GemConfirmError::Network { msg: "offline".to_string() });
        assert_eq!(
            broadcast_error(vec![], GatewayError::PlatformError { msg: "rejected".to_string() }),
            GemConfirmError::Broadcast {
                msg: GatewayError::PlatformError { msg: "rejected".to_string() }.to_string(),
            },
            "a send the network refused before accepting anything is a plain broadcast failure"
        );
        assert_eq!(
            broadcast_error(vec!["h1".to_string()], GatewayError::NetworkError { msg: "offline".to_string() }),
            GemConfirmError::PartialBroadcast {
                hashes: vec!["h1".to_string()],
                msg: GatewayError::NetworkError { msg: "offline".to_string() }.to_string(),
            }
        );
    }
}
