use crate::models::custom_types::GemBigInt;
use primitives::{AssetId, BannerEvent, BannerState, Chain, WalletId};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBannerContext {
    pub has_wallet: bool,
    pub has_asset: bool,
    pub is_stakeable: bool,
    pub has_stake_balance: bool,
    pub has_available_balance: bool,
    pub is_asset_activated: bool,
    pub asset_rank_score: Option<i32>,
    pub has_perpetuals_support: bool,
    pub is_wallet_empty: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBannerItem {
    pub event: BannerEvent,
    pub state: BannerState,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBannerKey {
    pub wallet_id: Option<WalletId>,
    pub asset_id: Option<AssetId>,
    pub event: BannerEvent,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBannerIcon {
    MoneyBag,
    Network { chain: Chain },
    Warning,
    Suspicious,
    Bitcoin,
    Perpetuals,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBannerAmount {
    pub value: GemBigInt,
    pub decimals: i32,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBannerTitle {
    Stake { asset_name: String },
    AccountActivation,
    Warning,
    ActivateAsset,
    SuspiciousAsset,
    Onboarding,
    TradePerpetuals,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemBannerDescription {
    Stake { asset_symbol: String },
    AccountActivation { network_name: String, fee: GemBannerAmount },
    MultiSignatureBlocked { network_name: String },
    ActivateAsset { asset_symbol: String, network_name: String },
    SuspiciousAsset,
    Onboarding,
    TradePerpetuals,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBannerContent {
    pub icon: Option<GemBannerIcon>,
    pub title: Option<GemBannerTitle>,
    pub description: Option<GemBannerDescription>,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemBannerAction {
    Event { event: BannerEvent },
    Button,
    Close,
}

impl GemBannerAction {
    pub fn is_dismissal(&self) -> bool {
        matches!(self, Self::Close)
    }
}

#[uniffi::export]
pub fn banner_identifier(key: GemBannerKey) -> String {
    [
        key.wallet_id.map(|wallet_id| wallet_id.id()),
        key.asset_id.map(|asset_id| asset_id.to_string()),
        Some(key.event.as_ref().to_string()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_only_the_close_action_dismisses_a_banner() {
        assert!(GemBannerAction::Close.is_dismissal());
        assert!(!GemBannerAction::Button.is_dismissal());
        assert!(!GemBannerAction::Event { event: BannerEvent::Stake }.is_dismissal());
    }

    #[test]
    fn test_banner_identifier() {
        let key = |wallet_id: Option<&str>, asset_id: Option<AssetId>, event: BannerEvent| GemBannerKey {
            wallet_id: wallet_id.map(|id| WalletId::Multicoin(id.to_string())),
            asset_id,
            event,
        };

        assert_eq!(
            banner_identifier(key(Some("wallet-1"), Some(AssetId::from_chain(Chain::Bitcoin)), BannerEvent::Stake)),
            "multicoin_wallet-1_bitcoin_stake"
        );
        assert_eq!(banner_identifier(key(None, None, BannerEvent::SuspiciousAsset)), "suspiciousAsset");
        assert_eq!(banner_identifier(key(Some("wallet-1"), None, BannerEvent::Onboarding)), "multicoin_wallet-1_onboarding");
        assert_eq!(
            banner_identifier(key(None, Some(AssetId::from_chain(Chain::Ethereum)), BannerEvent::ActivateAsset)),
            "ethereum_activateAsset"
        );
    }
}
