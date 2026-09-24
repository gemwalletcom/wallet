use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::GemBigUint;
use crate::services::balance::GemAssetBalance;
use crate::services::transfer::GemTransferData;
use primitives::{Asset, AssetId, AssetMetaData, Banner, BannerEvent, BannerState, Chain, Platform, Wallet, WalletId};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBannerContext {
    pub wallet: Option<Wallet>,
    pub asset: Option<Asset>,
    pub is_stakeable: bool,
    pub has_stake_balance: bool,
    pub has_available_balance: bool,
    pub is_asset_activated: bool,
    pub asset_rank_score: Option<i32>,
    pub is_wallet_empty: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBannerRow {
    pub key: GemBannerKey,
    pub banner: Banner,
    pub content: GemBannerContent,
}

impl GemBannerRow {
    pub fn new(banner: Banner, platform: Platform) -> Self {
        Self {
            key: GemBannerKey::from(&banner),
            content: super::rules::banner_content(banner.event, banner.asset.as_ref(), banner.state, platform),
            banner,
        }
    }
}

#[uniffi::export]
impl GemBannerContext {
    pub fn visible_banners(&self, stored: Vec<Banner>, platform: Platform) -> Vec<GemBannerRow> {
        super::rules::visible_banners(stored, self).into_iter().map(|banner| GemBannerRow::new(banner, platform)).collect()
    }
}

#[uniffi::export]
pub fn wallet_banner_events() -> Vec<BannerEvent> {
    super::rules::wallet_banner_events()
}

impl GemBannerContext {
    pub fn asset(wallet: Option<Wallet>, asset: Asset, metadata: &AssetMetaData, balance: &GemAssetBalance) -> Self {
        Self {
            wallet,
            is_stakeable: metadata.is_stake_enabled,
            has_stake_balance: balance.staked_value(asset.chain()) > GemBigUint::ZERO,
            has_available_balance: balance.available > GemBigUint::ZERO,
            is_asset_activated: balance.is_active,
            asset_rank_score: Some(metadata.rank_score),
            asset: Some(asset),
            is_wallet_empty: false,
        }
    }

    pub fn wallet(wallet: Wallet, is_wallet_empty: bool) -> Self {
        Self {
            wallet: Some(wallet),
            asset: None,
            is_stakeable: false,
            has_stake_balance: false,
            has_available_balance: false,
            is_asset_activated: true,
            asset_rank_score: None,
            is_wallet_empty,
        }
    }

    pub(super) fn asset_id(&self) -> Option<AssetId> {
        self.asset.as_ref().map(|asset| asset.id.clone())
    }

    pub(super) fn banner(&self, item: GemBannerItem) -> Banner {
        Banner {
            wallet_id: self.wallet.as_ref().map(|wallet| wallet.id.clone()),
            asset: self.asset.clone(),
            event: item.event,
            state: item.state,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemBannerItem {
    pub event: BannerEvent,
    pub state: BannerState,
    pub asset_id: Option<AssetId>,
}

#[derive(PartialEq)]
pub(super) enum BannerScope {
    Asset,
    Chain,
    Wallet,
}

pub(super) fn banner_scope(event: BannerEvent) -> BannerScope {
    match event {
        BannerEvent::AccountBlockedMultiSignature => BannerScope::Chain,
        BannerEvent::Onboarding => BannerScope::Wallet,
        BannerEvent::Stake | BannerEvent::AccountActivation | BannerEvent::ActivateAsset | BannerEvent::SuspiciousAsset | BannerEvent::TradePerpetuals => BannerScope::Asset,
    }
}

impl GemBannerItem {
    pub(super) fn applies_to_asset(&self, asset_id: &AssetId) -> bool {
        match banner_scope(self.event) {
            BannerScope::Asset => self.asset_id.as_ref() == Some(asset_id),
            BannerScope::Chain => self.asset_id.as_ref().is_some_and(|id| id.chain == asset_id.chain),
            BannerScope::Wallet => false,
        }
    }

    pub(super) fn applies_to_wallet(&self) -> bool {
        banner_scope(self.event) != BannerScope::Asset
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemBannerKey {
    pub wallet_id: Option<WalletId>,
    pub asset_id: Option<AssetId>,
    pub event: BannerEvent,
}

impl From<&Banner> for GemBannerKey {
    fn from(banner: &Banner) -> Self {
        Self {
            wallet_id: banner.wallet_id.clone(),
            asset_id: banner.asset.as_ref().map(|asset| asset.id.clone()),
            event: banner.event,
        }
    }
}

#[uniffi::export]
impl GemBannerKey {
    pub fn identifier(&self) -> String {
        [self.wallet_id.as_ref().map(WalletId::id), self.asset_id.as_ref().map(ToString::to_string), Some(self.event.as_ref().to_string())]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("_")
    }
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
    AccountActivation { network_name: String, fee: GemFormattedNumber },
    ExternallyControlledAccount { network_name: String },
    ActivateAsset { asset_symbol: String, network_name: String },
    SuspiciousAsset,
    Onboarding,
    TradePerpetuals,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemBannerDestination {
    Stake,
    ActivateAsset { transfer: GemTransferData },
    Perpetuals,
    Url { url: String },
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemBannerContent {
    pub icon: Option<GemBannerIcon>,
    pub title: Option<GemBannerTitle>,
    pub description: Option<GemBannerDescription>,
    pub destination: Option<GemBannerDestination>,
    pub style: GemBannerStyle,
    pub buttons: Vec<GemBannerButton>,
    pub can_close: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemBannerStyle {
    List,
    Welcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemBannerButton {
    Buy,
    Receive,
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_asset_context_decides_the_balance_facts_itself() {
        let asset = Asset::from_chain(Chain::Ethereum);
        let metadata = AssetMetaData {
            is_stake_enabled: true,
            rank_score: 42,
            ..AssetMetaData::mock()
        };
        let empty = GemAssetBalance { is_active: false, ..GemAssetBalance::mock() };
        let funded = GemAssetBalance {
            staked: GemBigUint::from(5u32),
            is_active: true,
            ..GemAssetBalance::mock_with_available(10)
        };

        let context = GemBannerContext::asset(None, asset.clone(), &metadata, &empty);
        assert!(context.is_stakeable);
        assert!(!context.has_stake_balance);
        assert!(!context.has_available_balance);
        assert!(!context.is_asset_activated);
        assert_eq!(context.asset_rank_score, Some(42));
        assert!(!context.is_wallet_empty);

        let context = GemBannerContext::asset(None, asset, &metadata, &funded);
        assert!(context.has_stake_balance);
        assert!(context.has_available_balance);
        assert!(context.is_asset_activated);
    }

    #[test]
    fn test_a_banner_row_carries_the_key_that_closes_it() {
        let wallet_id = WalletId::Multicoin("wallet-1".to_string());
        let banner = Banner {
            wallet_id: Some(wallet_id.clone()),
            asset: Some(Asset::from_chain(Chain::Bitcoin)),
            event: BannerEvent::Stake,
            state: BannerState::Active,
        };

        let row = GemBannerRow::new(banner, Platform::IOS);

        assert_eq!(
            row.key,
            GemBannerKey {
                wallet_id: Some(wallet_id),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                event: BannerEvent::Stake,
            }
        );
    }

    #[test]
    fn test_banner_identifier() {
        let wallet_id = WalletId::Multicoin("wallet-1".to_string());

        assert_eq!(
            GemBannerKey {
                wallet_id: Some(wallet_id.clone()),
                asset_id: Some(AssetId::from_chain(Chain::Bitcoin)),
                event: BannerEvent::Stake
            }
            .identifier(),
            "multicoin_wallet-1_bitcoin_stake"
        );
        assert_eq!(
            GemBannerKey {
                wallet_id: None,
                asset_id: None,
                event: BannerEvent::SuspiciousAsset
            }
            .identifier(),
            "suspiciousAsset"
        );
        assert_eq!(
            GemBannerKey {
                wallet_id: Some(wallet_id),
                asset_id: None,
                event: BannerEvent::Onboarding
            }
            .identifier(),
            "multicoin_wallet-1_onboarding"
        );
        assert_eq!(
            GemBannerKey {
                wallet_id: None,
                asset_id: Some(AssetId::from_chain(Chain::Ethereum)),
                event: BannerEvent::ActivateAsset
            }
            .identifier(),
            "ethereum_activateAsset"
        );
    }
}
