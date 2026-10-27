pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{Asset, AssetId, AssetType, Chain, Deeplink, FiatQuoteType, Payment, TransactionId, UrlAction, Wallet, WalletConnectLink, WalletId};

use crate::services::assets::{GemAssetAction, GemAssetsService};
use crate::services::balance::GemBalanceService;
use crate::services::error::{GemServiceError, required_account};
use crate::services::error_text::GemErrorText;
use crate::services::fiat::rules::quote_action;
use crate::services::preferences::GemPreferencesService;
use crate::services::push_notification::GemPushNotification;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetDestination {
    Asset { asset: Asset },
    Perpetual { asset: Asset },
}

impl From<Asset> for GemAssetDestination {
    fn from(asset: Asset) -> Self {
        match asset.asset_type {
            AssetType::PERPETUAL => Self::Perpetual { asset },
            AssetType::NATIVE
            | AssetType::ERC20
            | AssetType::BEP20
            | AssetType::SPL
            | AssetType::SPL2022
            | AssetType::TRC20
            | AssetType::TIP20
            | AssetType::TOKEN
            | AssetType::IBC
            | AssetType::JETTON
            | AssetType::SYNTH
            | AssetType::ASA
            | AssetType::SPOT => Self::Asset { asset },
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemNavigationTarget {
    Asset { destination: GemAssetDestination, wallet_id: Option<WalletId> },
    Receive { asset: Asset },
    Fiat { asset: Asset, amount: Option<i32>, quote_type: FiatQuoteType },
    Swap { from: Asset, to: Option<Asset> },
    Perpetuals,
    Rewards { code: Option<String> },
    Support,
    Transaction { destination: GemAssetDestination, wallet_id: WalletId, transaction_id: TransactionId },
    Address { chain: Chain, address: String },
    None,
}

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
    balance: Arc<GemBalanceService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemNavigationService {
    #[uniffi::constructor]
    pub fn new(assets: Arc<GemAssetsService>, balance: Arc<GemBalanceService>, preferences: Arc<GemPreferencesService>, session: Arc<GemWalletSessionService>) -> Self {
        Self { assets, balance, preferences, session }
    }

    pub async fn open_deeplink(&self, deeplink: Deeplink) -> Result<GemNavigationTarget, GemServiceError> {
        match deeplink {
            Deeplink::Asset { asset_id } => self.open_asset(asset_id).await,
            Deeplink::Perpetuals => self.perpetuals().await,
            Deeplink::Rewards { code } => Ok(GemNavigationTarget::Rewards { code: rules::code(code) }),
            Deeplink::Receive { asset_id } => self.receive(asset_id).await,
            Deeplink::Buy { asset_id, amount } => self.fiat(asset_id, amount, FiatQuoteType::Buy).await,
            Deeplink::Sell { asset_id, amount } => self.fiat(asset_id, amount, FiatQuoteType::Sell).await,
            Deeplink::Swap { asset_id } => self.swap(asset_id, None).await,
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
            GemPushNotification::SwapAsset { from_asset_id, to_asset_id } => self.swap(from_asset_id, Some(to_asset_id)).await,
            GemPushNotification::FiatTransaction { wallet_id, asset_id } | GemPushNotification::Stake { wallet_id, asset_id } => self.open_wallet_asset(wallet_id, asset_id).await,
            GemPushNotification::Transaction { wallet_id, asset_id, transaction_id } => self.open_transaction(wallet_id, asset_id, transaction_id).await,
            GemPushNotification::Support => Ok(GemNavigationTarget::Support),
            GemPushNotification::Rewards => Ok(GemNavigationTarget::Rewards { code: None }),
            GemPushNotification::Test => Ok(GemNavigationTarget::None),
        }
    }

    pub async fn open_asset(&self, asset_id: AssetId) -> Result<GemNavigationTarget, GemServiceError> {
        let wallet = self.session.require_current_wallet().await?;
        Ok(match self.balance.open_wallet_asset(wallet.clone(), asset_id).await? {
            Some(asset) => self.wallet_asset_target(&wallet, asset, None),
            None => GemNavigationTarget::None,
        })
    }

    pub async fn open_swap(&self, from_asset_id: AssetId, to_asset_id: AssetId) -> Result<GemNavigationTarget, GemServiceError> {
        self.swap(from_asset_id, Some(to_asset_id)).await
    }

    pub fn asset_target(&self, asset: Asset) -> GemNavigationTarget {
        target(asset, None)
    }
}

impl GemNavigationService {
    async fn perpetuals(&self) -> Result<GemNavigationTarget, GemServiceError> {
        Ok(match self.session.get_current_wallet().await? {
            Some(wallet) if self.shows_perpetuals(&wallet) => GemNavigationTarget::Perpetuals,
            _ => GemNavigationTarget::None,
        })
    }

    fn shows_perpetuals(&self, wallet: &Wallet) -> bool {
        self.preferences.show_perpetuals(wallet.wallet_type, wallet.chains())
    }

    fn hides_asset(&self, wallet: &Wallet, asset: &Asset) -> bool {
        asset.asset_type == AssetType::PERPETUAL && !self.shows_perpetuals(wallet)
    }

    fn wallet_asset_target(&self, wallet: &Wallet, asset: Asset, wallet_id: Option<WalletId>) -> GemNavigationTarget {
        match self.hides_asset(wallet, &asset) {
            true => GemNavigationTarget::None,
            false => target(asset, wallet_id),
        }
    }

    async fn open_wallet_asset(&self, wallet_id: WalletId, asset_id: AssetId) -> Result<GemNavigationTarget, GemServiceError> {
        let Some(wallet) = self.session.get_wallet(wallet_id.clone()).await? else {
            return Ok(GemNavigationTarget::None);
        };
        Ok(match self.balance.open_wallet_asset(wallet.clone(), asset_id).await? {
            Some(asset) => self.wallet_asset_target(&wallet, asset, Some(wallet_id)),
            None => GemNavigationTarget::None,
        })
    }

    async fn open_transaction(&self, wallet_id: WalletId, asset_id: AssetId, transaction_id: TransactionId) -> Result<GemNavigationTarget, GemServiceError> {
        Ok(match self.open_wallet_asset(wallet_id.clone(), asset_id).await? {
            GemNavigationTarget::Asset { destination, .. } => GemNavigationTarget::Transaction { destination, wallet_id, transaction_id },
            _ => GemNavigationTarget::None,
        })
    }

    async fn receive(&self, asset_id: AssetId) -> Result<GemNavigationTarget, GemServiceError> {
        let Some(wallet) = self.session.get_current_wallet().await? else {
            return Ok(GemNavigationTarget::None);
        };
        Ok(match self.offered_asset(&wallet, asset_id, GemAssetAction::Receive).await? {
            Some(asset) => GemNavigationTarget::Receive { asset },
            None => GemNavigationTarget::None,
        })
    }

    async fn fiat(&self, asset_id: AssetId, amount: Option<i32>, quote_type: FiatQuoteType) -> Result<GemNavigationTarget, GemServiceError> {
        let Some(wallet) = self.session.get_current_wallet().await? else {
            return Ok(GemNavigationTarget::None);
        };
        Ok(match self.offered_asset(&wallet, asset_id, quote_action(&quote_type)).await? {
            Some(asset) => GemNavigationTarget::Fiat { asset, amount, quote_type },
            None => GemNavigationTarget::None,
        })
    }

    async fn swap(&self, from_asset_id: AssetId, to_asset_id: Option<AssetId>) -> Result<GemNavigationTarget, GemServiceError> {
        let Some(wallet) = self.session.get_current_wallet().await? else {
            return Ok(GemNavigationTarget::None);
        };
        let Some(from) = self.offered_asset(&wallet, from_asset_id, GemAssetAction::SwapPay).await? else {
            return Ok(GemNavigationTarget::None);
        };
        let to = match to_asset_id {
            Some(asset_id) if wallet.account(asset_id.chain).is_some() => self.offered_asset(&wallet, asset_id, GemAssetAction::SwapReceive).await?,
            _ => None,
        };
        Ok(GemNavigationTarget::Swap { from, to })
    }
}

impl GemNavigationService {
    async fn offered_asset(&self, wallet: &Wallet, asset_id: AssetId, action: GemAssetAction) -> Result<Option<Asset>, GemServiceError> {
        required_account(wallet, asset_id.chain)?;
        let asset = self.assets.ensure_asset(asset_id.clone()).await?;
        let properties = self.assets.asset_properties(asset_id).await?;
        Ok(properties.is_some_and(|properties| action.is_offered(wallet.wallet_type, &properties)).then_some(asset))
    }
}

fn target(asset: Asset, wallet_id: Option<WalletId>) -> GemNavigationTarget {
    GemNavigationTarget::Asset {
        destination: GemAssetDestination::from(asset),
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
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;
    use primitives::{AssetBasic, AssetProperties, Chain, Transaction, Wallet, WalletType};

    #[test]
    fn test_a_transaction_push_opens_its_details_at_once_and_leaves_loading_to_the_screen() {
        block_on(async {
            let wallet = Wallet::mock();
            let testkit = DiscoveryTestkit::with_provider(Arc::new(TestAlienProvider::with_status(500)), wallet.clone());
            let asset = Asset::from_chain(Chain::Ethereum);
            testkit.asset_store.save_assets(vec![default_asset_basic(asset.clone())]).await.unwrap();
            let service = GemNavigationService::mock(&testkit);
            let transaction_id = Transaction::mock().id;

            let target = service
                .open_notification(GemPushNotification::Transaction {
                    wallet_id: wallet.id.clone(),
                    asset_id: asset.id.clone(),
                    transaction_id: transaction_id.clone(),
                })
                .await
                .unwrap();

            assert_eq!(
                target,
                GemNavigationTarget::Transaction {
                    destination: GemAssetDestination::Asset { asset },
                    wallet_id: wallet.id,
                    transaction_id,
                }
            );
            assert!(!testkit.provider.requested_paths().iter().any(|path| path.contains("/v3/devices/transactions/")));
            assert!(testkit.transaction_store.added.lock().unwrap().is_empty());
        });
    }

    #[test]
    fn test_only_a_perpetual_opens_the_perpetual_screen() {
        let testkit = DiscoveryTestkit::with_status(200);
        let service = GemNavigationService::mock(&testkit);
        let coin = Asset::from_chain(Chain::Ethereum);
        let perpetual = Asset {
            asset_type: AssetType::PERPETUAL,
            ..Asset::from_chain(Chain::HyperCore)
        };

        assert_eq!(
            service.asset_target(coin.clone()),
            GemNavigationTarget::Asset {
                destination: GemAssetDestination::Asset { asset: coin },
                wallet_id: None,
            }
        );
        assert_eq!(
            service.asset_target(perpetual.clone()),
            GemNavigationTarget::Asset {
                destination: GemAssetDestination::Perpetual { asset: perpetual },
                wallet_id: None,
            }
        );
    }

    #[test]
    fn test_a_code_opens_its_target_pairs_prepares_a_payment_or_says_why_not() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Arbitrum]);
            testkit.select_wallet(&wallet);
            testkit.preferences.set_perpetual_enabled(true).unwrap();
            let service = GemNavigationService::mock(&testkit);

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
    fn test_a_perpetuals_link_opens_only_where_the_wallet_shows_perpetuals() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let solana = Wallet::mock_with_type(WalletType::Single, &[Chain::Solana]);
            let multicoin = Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Arbitrum]);
            let service = GemNavigationService::mock(&testkit);
            let open = |wallet: Wallet| {
                testkit.select_wallet(&wallet);
                service.open_deeplink(Deeplink::Perpetuals)
            };

            assert_eq!(open(multicoin.clone()).await.unwrap(), GemNavigationTarget::None, "Perpetuals are switched off");
            testkit.preferences.set_perpetual_enabled(true).unwrap();
            assert_eq!(open(solana).await.unwrap(), GemNavigationTarget::None, "a single chain wallet never shows Perpetuals");
            assert_eq!(open(multicoin).await.unwrap(), GemNavigationTarget::Perpetuals);
        });
    }

    async fn save_asset(testkit: &DiscoveryTestkit, asset: Asset, is_buyable: bool, is_swapable: bool) {
        let basic = default_asset_basic(asset);
        let basic = AssetBasic {
            properties: AssetProperties {
                is_buyable,
                is_sellable: is_buyable,
                is_swapable,
                ..basic.properties
            },
            ..basic
        };
        testkit.asset_store.save_assets(vec![basic]).await.unwrap();
    }

    #[test]
    fn test_links_open_only_the_actions_the_asset_screen_offers() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let service = GemNavigationService::mock(&testkit);
            let ethereum = Asset::from_chain(Chain::Ethereum);
            let buy = || service.open_deeplink(Deeplink::Buy { asset_id: ethereum.id.clone(), amount: None });
            let sell = || service.open_deeplink(Deeplink::Sell { asset_id: ethereum.id.clone(), amount: None });
            let swap = || service.open_deeplink(Deeplink::Swap { asset_id: ethereum.id.clone() });
            let receive = || service.open_deeplink(Deeplink::Receive { asset_id: ethereum.id.clone() });
            testkit.select_wallet(&Wallet::mock());

            save_asset(&testkit, ethereum.clone(), false, false).await;
            assert_eq!(buy().await.unwrap(), GemNavigationTarget::None, "the asset cannot be bought");
            assert_eq!(sell().await.unwrap(), GemNavigationTarget::None, "the asset cannot be sold");
            assert_eq!(swap().await.unwrap(), GemNavigationTarget::None, "the asset cannot be swapped");

            save_asset(&testkit, ethereum.clone(), true, true).await;
            assert_eq!(
                buy().await.unwrap(),
                GemNavigationTarget::Fiat {
                    asset: ethereum.clone(),
                    amount: None,
                    quote_type: FiatQuoteType::Buy
                }
            );
            assert_eq!(
                sell().await.unwrap(),
                GemNavigationTarget::Fiat {
                    asset: ethereum.clone(),
                    amount: None,
                    quote_type: FiatQuoteType::Sell
                }
            );
            assert_eq!(swap().await.unwrap(), GemNavigationTarget::Swap { from: ethereum.clone(), to: None });
            assert_eq!(receive().await.unwrap(), GemNavigationTarget::Receive { asset: ethereum.clone() });

            testkit.select_wallet(&Wallet::mock_with_type(WalletType::View, &[Chain::Ethereum]));
            assert_eq!(buy().await.unwrap(), GemNavigationTarget::None, "a watch-only wallet shows a notice instead of buttons");
            assert_eq!(sell().await.unwrap(), GemNavigationTarget::None);
            assert_eq!(swap().await.unwrap(), GemNavigationTarget::None);
            assert_eq!(receive().await.unwrap(), GemNavigationTarget::None);
        });
    }

    #[test]
    fn test_a_swap_drops_a_receive_asset_the_wallet_cannot_take() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let service = GemNavigationService::mock(&testkit);
            let ethereum = Asset::from_chain(Chain::Ethereum);
            let arbitrum = Asset::from_chain(Chain::Arbitrum);
            let bitcoin = Asset::from_chain(Chain::Bitcoin);
            testkit.select_wallet(&Wallet::mock_with_chains(&[Chain::Ethereum, Chain::Arbitrum]));
            save_asset(&testkit, ethereum.clone(), false, true).await;
            save_asset(&testkit, arbitrum.clone(), false, false).await;
            save_asset(&testkit, bitcoin.clone(), false, true).await;

            let target = service
                .open_notification(GemPushNotification::SwapAsset {
                    from_asset_id: ethereum.id.clone(),
                    to_asset_id: arbitrum.id,
                })
                .await
                .unwrap();

            assert_eq!(target, GemNavigationTarget::Swap { from: ethereum.clone(), to: None }, "the asset cannot be swapped");
            assert_eq!(
                service.open_swap(ethereum.id.clone(), bitcoin.id).await.unwrap(),
                GemNavigationTarget::Swap { from: ethereum, to: None },
                "the wallet has no account on the network"
            );
        });
    }

    #[test]
    fn test_a_perpetual_opens_only_where_the_wallet_shows_perpetuals() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let service = GemNavigationService::mock(&testkit);
            let perpetual = Asset {
                asset_type: AssetType::PERPETUAL,
                ..Asset::from_chain(Chain::HyperCore)
            };
            testkit.select_wallet(&Wallet::mock_with_chains(&[Chain::HyperCore]));
            save_asset(&testkit, perpetual.clone(), false, false).await;
            let open = || service.open_deeplink(Deeplink::Asset { asset_id: perpetual.id.clone() });

            assert_eq!(open().await.unwrap(), GemNavigationTarget::None, "Perpetuals are switched off");
            testkit.preferences.set_perpetual_enabled(true).unwrap();
            assert_eq!(
                open().await.unwrap(),
                GemNavigationTarget::Asset {
                    destination: GemAssetDestination::Perpetual { asset: perpetual },
                    wallet_id: None,
                }
            );
        });
    }

    #[test]
    fn test_a_perpetual_transaction_push_opens_only_where_the_wallet_shows_perpetuals() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let service = GemNavigationService::mock(&testkit);
            let wallet = Wallet::mock_with_chains(&[Chain::HyperCore]);
            let perpetual = Asset {
                asset_type: AssetType::PERPETUAL,
                ..Asset::from_chain(Chain::HyperCore)
            };
            let transaction_id = Transaction::mock().id;
            testkit.select_wallet(&wallet);
            save_asset(&testkit, perpetual.clone(), false, false).await;
            let open = || {
                service.open_notification(GemPushNotification::Transaction {
                    wallet_id: wallet.id.clone(),
                    asset_id: perpetual.id.clone(),
                    transaction_id: transaction_id.clone(),
                })
            };

            assert_eq!(open().await.unwrap(), GemNavigationTarget::None, "Perpetuals are switched off");
            testkit.preferences.set_perpetual_enabled(true).unwrap();
            assert_eq!(
                open().await.unwrap(),
                GemNavigationTarget::Transaction {
                    destination: GemAssetDestination::Perpetual { asset: perpetual },
                    wallet_id: wallet.id,
                    transaction_id,
                }
            );
        });
    }

    #[test]
    fn test_a_link_for_a_chain_the_wallet_lacks_names_the_missing_account() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let wallet = Wallet::mock();
            testkit.select_wallet(&wallet);
            let bitcoin = Asset::from_chain(Chain::Bitcoin);
            testkit.asset_store.save_assets(vec![default_asset_basic(bitcoin.clone())]).await.unwrap();
            let service = GemNavigationService::mock(&testkit);

            let result = service.open_deeplink(Deeplink::Receive { asset_id: bitcoin.id.clone() }).await;

            assert_eq!(result.unwrap_err(), GemServiceError::NoAccountForChain { chain: Chain::Bitcoin });
        });
    }
}
