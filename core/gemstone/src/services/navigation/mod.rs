pub mod rules;

use std::sync::Arc;

use crate::services::transaction_state::GemTransactionStateService;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemNavigationTab {
    Wallet,
    Settings,
}

#[uniffi::export]
impl GemNavigationTarget {
    pub fn tab(&self) -> Option<GemNavigationTab> {
        match self {
            Self::Asset { .. } | Self::Transaction { .. } | Self::Perpetuals => Some(GemNavigationTab::Wallet),
            Self::Rewards { .. } | Self::Support => Some(GemNavigationTab::Settings),
            Self::Receive { .. } | Self::Fiat { .. } | Self::Swap { .. } | Self::None => None,
        }
    }
}

#[derive(uniffi::Object)]
pub struct GemNavigationService {
    assets: Arc<GemAssetsService>,
    session: Arc<GemWalletSessionService>,
    transaction_state: Arc<GemTransactionStateService>,
}

#[uniffi::export]
impl GemNavigationService {
    #[uniffi::constructor]
    pub fn new(assets: Arc<GemAssetsService>, session: Arc<GemWalletSessionService>, transaction_state: Arc<GemTransactionStateService>) -> Self {
        Self { assets, session, transaction_state }
    }

    pub async fn open_deeplink(&self, deeplink: Deeplink) -> Result<GemNavigationTarget, GemServiceError> {
        match deeplink {
            Deeplink::Asset { asset_id } => self.open_asset(asset_id).await,
            Deeplink::Perpetuals => Ok(GemNavigationTarget::Perpetuals),
            Deeplink::Rewards { code } => Ok(GemNavigationTarget::Rewards { code: rules::code(code) }),
            Deeplink::Receive { asset_id } => Ok(GemNavigationTarget::Receive { asset: self.account_asset(asset_id).await? }),
            Deeplink::Buy { asset_id, amount } => self.fiat(asset_id, amount, FiatQuoteType::Buy).await,
            Deeplink::Sell { asset_id, amount } => self.fiat(asset_id, amount, FiatQuoteType::Sell).await,
            Deeplink::Swap { asset_id } => Ok(GemNavigationTarget::Swap {
                from: self.account_asset(asset_id).await?,
                to: None,
            }),
        }
    }

    pub async fn open_notification(&self, notification: GemPushNotification) -> Result<GemNavigationTarget, GemServiceError> {
        match notification {
            GemPushNotification::Asset { asset_id } | GemPushNotification::PriceAlert { asset_id } => self.open_asset(asset_id).await,
            GemPushNotification::BuyAsset { asset_id } => self.fiat(asset_id, None, FiatQuoteType::Buy).await,
            GemPushNotification::SwapAsset { from_asset_id, to_asset_id } => Ok(GemNavigationTarget::Swap {
                from: self.account_asset(from_asset_id).await?,
                to: Some(self.account_asset(to_asset_id).await?),
            }),
            GemPushNotification::FiatTransaction { wallet_id, asset_id } | GemPushNotification::Stake { wallet_id, asset_id } => self.open_wallet_asset(wallet_id, asset_id).await,
            GemPushNotification::Transaction { wallet_id, asset_id, transaction } => self.open_transaction(wallet_id, asset_id, transaction).await,
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

    async fn open_transaction(&self, wallet_id: WalletId, asset_id: AssetId, transaction: Transaction) -> Result<GemNavigationTarget, GemServiceError> {
        let Some(wallet) = self.session.get_wallet(wallet_id.clone()).await? else {
            return Ok(GemNavigationTarget::None);
        };
        Ok(match self.transaction_state.add_notification_transaction(wallet, asset_id, transaction.clone()).await? {
            Some(asset) => GemNavigationTarget::Transaction {
                is_perpetual: asset.asset_type == AssetType::PERPETUAL,
                asset,
                wallet_id,
                transaction,
            },
            None => GemNavigationTarget::None,
        })
    }

    async fn fiat(&self, asset_id: AssetId, amount: Option<i32>, quote_type: FiatQuoteType) -> Result<GemNavigationTarget, GemServiceError> {
        Ok(GemNavigationTarget::Fiat {
            asset: self.account_asset(asset_id).await?,
            amount,
            quote_type,
        })
    }
}

impl GemNavigationService {
    async fn account_asset(&self, asset_id: AssetId) -> Result<Asset, GemServiceError> {
        if let Some(wallet) = self.session.get_current_wallet().await?
            && wallet.account(asset_id.chain).is_none()
        {
            return Err(GemServiceError::NoAccountForChain { chain: asset_id.chain });
        }
        self.assets.ensure_asset(asset_id).await
    }
}

fn target(asset: Asset, wallet_id: Option<WalletId>) -> GemNavigationTarget {
    GemNavigationTarget::Asset {
        is_perpetual: asset.asset_type == AssetType::PERPETUAL,
        asset,
        wallet_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_target_opens_on_the_tab_that_holds_it() {
        let asset = Asset::from_chain(primitives::Chain::Ethereum);
        assert_eq!(GemNavigationTarget::Perpetuals.tab(), Some(GemNavigationTab::Wallet));
        assert_eq!(GemNavigationTarget::Support.tab(), Some(GemNavigationTab::Settings));
        assert_eq!(GemNavigationTarget::Rewards { code: None }.tab(), Some(GemNavigationTab::Settings));
        assert_eq!(GemNavigationTarget::Receive { asset }.tab(), None, "a sheet keeps the tab it opens over");
        assert_eq!(GemNavigationTarget::None.tab(), None);
    }
    use crate::services::asset_discovery::testkit::DiscoveryTestkit;
    use crate::services::assets::GemAssetStore;
    use crate::services::assets::rules::default_asset_basic;
    use futures::executor::block_on;
    use primitives::{Chain, Wallet};

    #[test]
    fn test_a_pushed_transaction_is_stored_and_tracked_before_it_is_opened() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let wallet = Wallet::mock();
            *testkit.wallets.wallets.lock().unwrap() = vec![wallet.clone()];
            let asset = Asset::from_chain(Chain::Ethereum);
            testkit.asset_store.save_assets(vec![default_asset_basic(asset.clone())]).await.unwrap();
            let transaction = Transaction {
                asset_id: asset.id.clone(),
                ..Transaction::mock()
            };
            let service = GemNavigationService::new(testkit.assets.clone(), testkit.session.clone(), testkit.state.clone());

            let target = service
                .open_notification(GemPushNotification::Transaction {
                    wallet_id: wallet.id.clone(),
                    asset_id: asset.id.clone(),
                    transaction: transaction.clone(),
                })
                .await
                .unwrap();

            assert_eq!(
                target,
                GemNavigationTarget::Transaction {
                    asset,
                    wallet_id: wallet.id,
                    transaction: transaction.clone(),
                    is_perpetual: false
                }
            );
            assert_eq!(*testkit.status.tracked.lock().unwrap(), vec![vec![transaction]]);
        });
    }

    #[test]
    fn test_a_link_for_a_chain_the_wallet_lacks_names_the_missing_account() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let wallet = Wallet::mock();
            *testkit.wallets.wallets.lock().unwrap() = vec![wallet.clone()];
            testkit.session.set_current_wallet_id(Some(wallet.id.clone())).unwrap();
            let bitcoin = Asset::from_chain(Chain::Bitcoin);
            testkit.asset_store.save_assets(vec![default_asset_basic(bitcoin.clone())]).await.unwrap();
            let service = GemNavigationService::new(testkit.assets.clone(), testkit.session.clone(), testkit.state.clone());

            let result = service.open_deeplink(Deeplink::Receive { asset_id: bitcoin.id.clone() }).await;

            assert!(matches!(result, Err(GemServiceError::NoAccountForChain { chain: Chain::Bitcoin })));
        });
    }
}
