pub mod rules;

use std::sync::Arc;

use primitives::{Asset, AssetId, AssetType, Deeplink, FiatQuoteType, Transaction, WalletId};

use crate::services::assets::GemAssetsService;
use crate::services::error::GemServiceError;
use crate::services::push_notification::GemPushNotification;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemNavigationTarget {
    Asset { asset: Asset, wallet_id: Option<WalletId>, is_perpetual: bool },
    Receive { asset: Asset },
    Fiat { asset: Asset, amount: Option<i32>, quote_type: FiatQuoteType },
    Swap { from: Asset, to: Option<Asset> },
    Perpetuals,
    Rewards { code: Option<String> },
    Support,
    Transaction { asset: Asset, wallet_id: WalletId, transaction: Transaction, is_perpetual: bool },
    None,
}

#[derive(uniffi::Object)]
pub struct GemNavigationService {
    assets: Arc<GemAssetsService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemNavigationService {
    #[uniffi::constructor]
    pub fn new(assets: Arc<GemAssetsService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { assets, session }
    }

    pub async fn open_deeplink(&self, deeplink: Deeplink) -> Result<GemNavigationTarget, GemServiceError> {
        match deeplink {
            Deeplink::Asset { asset_id } => self.open_asset(asset_id).await,
            Deeplink::Perpetuals => Ok(GemNavigationTarget::Perpetuals),
            Deeplink::Rewards { code } => Ok(GemNavigationTarget::Rewards { code: rules::code(code) }),
            Deeplink::Receive { asset_id } => Ok(GemNavigationTarget::Receive {
                asset: self.assets.ensure_asset(asset_id).await?,
            }),
            Deeplink::Buy { asset_id, amount } => self.fiat(asset_id, amount, FiatQuoteType::Buy).await,
            Deeplink::Sell { asset_id, amount } => self.fiat(asset_id, amount, FiatQuoteType::Sell).await,
            Deeplink::Swap { asset_id } => Ok(GemNavigationTarget::Swap {
                from: self.assets.ensure_asset(asset_id).await?,
                to: None,
            }),
        }
    }

    pub async fn open_notification(&self, notification: GemPushNotification) -> Result<GemNavigationTarget, GemServiceError> {
        match notification {
            GemPushNotification::Asset { asset_id } | GemPushNotification::PriceAlert { asset_id } => self.open_asset(asset_id).await,
            GemPushNotification::BuyAsset { asset_id } => self.fiat(asset_id, None, FiatQuoteType::Buy).await,
            GemPushNotification::SwapAsset { from_asset_id, to_asset_id } => Ok(GemNavigationTarget::Swap {
                from: self.assets.ensure_asset(from_asset_id).await?,
                to: Some(self.assets.ensure_asset(to_asset_id).await?),
            }),
            GemPushNotification::FiatTransaction { wallet_id, asset_id } | GemPushNotification::Stake { wallet_id, asset_id } => self.open_wallet_asset(wallet_id, asset_id).await,
            GemPushNotification::Transaction { wallet_id, asset_id, transaction } => match self.open_wallet_asset(wallet_id, asset_id).await? {
                GemNavigationTarget::Asset {
                    asset,
                    wallet_id: Some(wallet_id),
                    is_perpetual,
                } => Ok(GemNavigationTarget::Transaction { asset, wallet_id, transaction, is_perpetual }),
                _ => Ok(GemNavigationTarget::None),
            },
            GemPushNotification::Support => Ok(GemNavigationTarget::Support),
            GemPushNotification::Rewards => Ok(GemNavigationTarget::Rewards { code: None }),
            GemPushNotification::Test => Ok(GemNavigationTarget::None),
        }
    }
}

impl GemNavigationService {
    async fn open_asset(&self, asset_id: AssetId) -> Result<GemNavigationTarget, GemServiceError> {
        Ok(match self.assets.open_asset(asset_id).await? {
            Some(asset) => target(asset, None),
            None => GemNavigationTarget::None,
        })
    }

    async fn open_wallet_asset(&self, wallet_id: WalletId, asset_id: AssetId) -> Result<GemNavigationTarget, GemServiceError> {
        let Some(wallet) = self.session.get_wallet(wallet_id.clone()).await? else {
            return Ok(GemNavigationTarget::None);
        };
        Ok(match self.assets.open_wallet_asset(wallet, asset_id).await? {
            Some(asset) => target(asset, Some(wallet_id)),
            None => GemNavigationTarget::None,
        })
    }

    async fn fiat(&self, asset_id: AssetId, amount: Option<i32>, quote_type: FiatQuoteType) -> Result<GemNavigationTarget, GemServiceError> {
        Ok(GemNavigationTarget::Fiat {
            asset: self.assets.ensure_asset(asset_id).await?,
            amount,
            quote_type,
        })
    }
}

fn target(asset: Asset, wallet_id: Option<WalletId>) -> GemNavigationTarget {
    GemNavigationTarget::Asset {
        is_perpetual: asset.asset_type == AssetType::PERPETUAL,
        asset,
        wallet_id,
    }
}
