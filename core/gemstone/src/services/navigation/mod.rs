pub mod rules;

use std::sync::Arc;

use crate::services::transaction_state::GemTransactionStateService;

use primitives::{Asset, AssetId, AssetType, Chain, Deeplink, FiatQuoteType, Payment, Transaction, UrlAction, WalletConnectLink, WalletId};

use crate::services::assets::GemAssetsService;
use crate::services::error::GemServiceError;
use crate::services::error_text::GemErrorText;
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
    Address { chain: Chain, address: String },
    None,
}

/// What opening a link or a scanned code does: a payment link shows loading while it is prepared.
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemCodeOutcome {
    Open { target: GemNavigationTarget },
    WalletConnect { link: WalletConnectLink },
    Payment { payment: Payment, shows_loading: bool },
    Failure { text: GemErrorText },
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
            Self::Receive { .. } | Self::Fiat { .. } | Self::Swap { .. } | Self::Address { .. } | Self::None => None,
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
            Deeplink::Address { chain, address } => Ok(GemNavigationTarget::Address { chain, address }),
        }
    }

    pub async fn open_code(&self, code: String) -> GemCodeOutcome {
        match ::payment::classify_url(&code) {
            Some(action) => self.open_action(action).await,
            None => GemCodeOutcome::Failure { text: GemErrorText::NotSupported },
        }
    }

    pub async fn open_action(&self, action: UrlAction) -> GemCodeOutcome {
        match action {
            UrlAction::Deeplink { deeplink } => match self.open_deeplink(deeplink).await {
                Ok(target) => GemCodeOutcome::Open { target },
                Err(error) => GemCodeOutcome::Failure { text: error.text() },
            },
            UrlAction::Payment { payment } => GemCodeOutcome::Payment {
                shows_loading: matches!(payment, Payment::Link { .. }),
                payment,
            },
            UrlAction::WalletConnect { link } => GemCodeOutcome::WalletConnect { link },
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

    pub async fn open_asset(&self, asset_id: AssetId) -> Result<GemNavigationTarget, GemServiceError> {
        Ok(match self.assets.open_asset(asset_id).await? {
            Some(asset) => target(asset, None),
            None => GemNavigationTarget::None,
        })
    }

    pub fn asset_target(&self, asset: Asset) -> GemNavigationTarget {
        target(asset, None)
    }
}

impl GemNavigationService {
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
    fn test_only_a_perpetual_opens_the_perpetual_screen() {
        let testkit = DiscoveryTestkit::with_status(200);
        let service = GemNavigationService::new(testkit.assets.clone(), testkit.session.clone(), testkit.state.clone());
        let coin = Asset::from_chain(Chain::Ethereum);
        let perpetual = Asset {
            asset_type: AssetType::PERPETUAL,
            ..Asset::from_chain(Chain::HyperCore)
        };

        assert_eq!(
            service.asset_target(coin.clone()),
            GemNavigationTarget::Asset {
                asset: coin,
                wallet_id: None,
                is_perpetual: false
            }
        );
        assert!(matches!(service.asset_target(perpetual), GemNavigationTarget::Asset { is_perpetual: true, .. }));
    }

    #[test]
    fn test_a_code_opens_its_target_pairs_prepares_a_payment_or_says_why_not() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let wallet = Wallet::mock();
            *testkit.wallets.wallets.lock().unwrap() = vec![wallet.clone()];
            testkit.session.set_current_wallet_id(Some(wallet.id.clone())).unwrap();
            let service = GemNavigationService::new(testkit.assets.clone(), testkit.session.clone(), testkit.state.clone());

            assert_eq!(service.open_code("gem://perpetuals".to_string()).await, GemCodeOutcome::Open { target: GemNavigationTarget::Perpetuals });
            assert!(matches!(service.open_code("wc:abc@2?relay-protocol=irn".to_string()).await, GemCodeOutcome::WalletConnect { .. }));
            assert!(matches!(service.open_code("solana:https%3A%2F%2Fexample.com%2Fpay".to_string()).await, GemCodeOutcome::Payment { shows_loading: true, .. }));
            assert_eq!(service.open_code("https://example.com/unknown".to_string()).await, GemCodeOutcome::Failure { text: GemErrorText::NotSupported });
            assert_eq!(
                service.open_code("gem://tokens/bitcoin/receive".to_string()).await,
                GemCodeOutcome::Failure { text: GemErrorText::NoAccountForChain },
                "a link that cannot open says why instead of doing nothing"
            );
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
