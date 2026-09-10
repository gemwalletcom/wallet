pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use futures::future::join_all;
use primitives::{Asset, AssetBalance, AssetId, Wallet, WalletId};

pub use model::{GemAssetBalance, GemBalanceRecord, GemBalanceRequirement, GemBalanceRow, GemBalanceUpdate, GemBalanceUpdateType, GemBalanceValue};
pub use store::GemBalanceStore;

use crate::gateway::GemGateway;
use crate::services::assets::{GemAssetStore, GemAssetsService};
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::wallet::GemWalletStore;
use rules::{BalanceKind, BalanceRequest};

#[derive(uniffi::Object)]
pub struct GemBalanceService {
    gateway: Arc<GemGateway>,
    wallet_store: Arc<dyn GemWalletStore>,
    asset_store: Arc<dyn GemAssetStore>,
    store: Arc<dyn GemBalanceStore>,
    assets: Arc<GemAssetsService>,
    stream: Arc<GemStreamSubscriptionService>,
}

#[uniffi::export]
impl GemBalanceService {
    pub async fn balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
        self.store.get_available_balances(wallet_id, asset_ids).await
    }

    #[uniffi::constructor]
    pub fn new(
        gateway: Arc<GemGateway>,
        wallet_store: Arc<dyn GemWalletStore>,
        asset_store: Arc<dyn GemAssetStore>,
        store: Arc<dyn GemBalanceStore>,
        assets: Arc<GemAssetsService>,
        stream: Arc<GemStreamSubscriptionService>,
    ) -> Self {
        Self {
            gateway,
            wallet_store,
            asset_store,
            store,
            assets,
            stream,
        }
    }

    pub async fn set_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        let asset_ids = rules::unique_asset_ids(asset_ids);
        if asset_ids.is_empty() {
            return Ok(());
        }
        if enabled {
            self.assets.sync_missing_assets(asset_ids.clone()).await?;
        }
        let enabled_ids = self.store.get_enabled_asset_ids(wallet_id.clone()).await?;
        self.add_missing_balances(wallet_id.clone(), asset_ids.clone()).await?;
        self.store.set_assets_enabled(wallet_id.clone(), asset_ids.clone(), enabled).await?;
        if enabled {
            self.refresh_enabled_assets(wallet_id, rules::missing_asset_ids(&asset_ids, &enabled_ids)).await;
        } else {
            let _ = self.stream.resubscribe().await;
        }
        Ok(())
    }

    pub async fn set_asset_pinned(&self, wallet_id: WalletId, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        if pinned {
            self.set_assets_enabled(wallet_id.clone(), vec![asset_id.clone()], true).await?;
        }
        self.store.set_asset_pinned(wallet_id, asset_id, pinned).await
    }

    pub async fn update(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let Some(wallet) = self
            .wallet_store
            .get_wallet(wallet_id.clone())
            .await
            .map_err(|error| GemServiceError::Store { msg: error.to_string() })?
        else {
            return Ok(());
        };
        let requests = rules::balance_requests(&wallet.accounts, &asset_ids);
        let (balances, failures): (Vec<_>, Vec<_>) = join_all(requests.iter().map(|request| self.chain_balances(request)))
            .await
            .into_iter()
            .partition(Result::is_ok);
        let balances: Vec<(BalanceKind, AssetBalance)> = balances.into_iter().flatten().flatten().collect();
        if !balances.is_empty() {
            let assets = self
                .asset_store
                .get_assets(balances.iter().map(|(_, balance)| balance.asset_id.clone()).collect())
                .await
                .map_err(|error| GemServiceError::Store { msg: error.to_string() })?;
            self.write_balances(wallet_id, rules::balance_updates(balances), &assets).await?;
        }
        match failures.into_iter().find_map(Result::err) {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl GemBalanceService {
    pub async fn update_enabled_balances(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let asset_ids = self.store.get_enabled_asset_ids(wallet_id.clone()).await?;
        self.update(wallet_id, asset_ids).await
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        let (enabled, disabled) = crate::services::assets::rules::default_balances(&wallet);
        let stored = self.store.get_available_balances(wallet.id.clone(), [enabled.clone(), disabled.clone()].concat()).await?;
        let stored_ids: Vec<AssetId> = stored.into_iter().map(|balance| balance.asset_id).collect();
        let enabled = rules::missing_asset_ids(&enabled, &stored_ids);
        let disabled = rules::missing_asset_ids(&disabled, &stored_ids);
        self.assets.add_balances(wallet.id.clone(), enabled.clone(), true).await?;
        self.assets.add_balances(wallet.id.clone(), disabled, false).await?;
        self.refresh_enabled_assets(wallet.id, enabled).await;
        Ok(())
    }

    async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let stored = self.store.get_available_balances(wallet_id.clone(), asset_ids.clone()).await?;
        let stored_ids: Vec<AssetId> = stored.into_iter().map(|balance| balance.asset_id).collect();
        self.assets.add_missing_balances(wallet_id, rules::missing_asset_ids(&asset_ids, &stored_ids)).await
    }

    async fn refresh_enabled_assets(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) {
        if asset_ids.is_empty() {
            return;
        }
        let _ = self.stream.resubscribe().await;
        let _ = self.update(wallet_id, asset_ids).await;
    }

    pub async fn update_balances(&self, wallet_id: WalletId, updates: Vec<GemBalanceUpdate>) -> Result<(), GemServiceError> {
        let assets = self
            .asset_store
            .get_assets(updates.iter().map(|update| update.asset_id.clone()).collect())
            .await
            .map_err(|error| GemServiceError::Store { msg: error.to_string() })?;
        self.write_balances(wallet_id, updates, &assets).await
    }

    async fn write_balances(&self, wallet_id: WalletId, updates: Vec<GemBalanceUpdate>, assets: &[Asset]) -> Result<(), GemServiceError> {
        let asset_ids: Vec<AssetId> = rules::unique_asset_ids(updates.iter().map(|update| update.asset_id.clone()).collect());
        let stored = self.store.get_available_balances(wallet_id.clone(), asset_ids.clone()).await?;
        let stored_ids: Vec<AssetId> = stored.iter().map(|balance| balance.asset_id.clone()).collect();
        self.assets.add_missing_balances(wallet_id.clone(), rules::missing_asset_ids(&asset_ids, &stored_ids)).await?;
        let records = rules::balance_records(rules::changed_balances(stored, updates), assets);
        if records.is_empty() {
            return Ok(());
        }
        self.store.update_balances(wallet_id, records).await
    }

    async fn chain_balances(&self, request: &BalanceRequest) -> Result<Vec<(BalanceKind, AssetBalance)>, GemServiceError> {
        let token_ids = rules::request_token_ids(&request.token_ids);
        let (coin, stake, tokens, earn) = futures::join!(
            async {
                if request.coin {
                    self.gateway.get_balance_coin(request.chain, request.address.clone()).await.map(|balance| vec![balance])
                } else {
                    Ok(Vec::new())
                }
            },
            async {
                if request.coin {
                    self.gateway
                        .get_balance_staking(request.chain, request.address.clone())
                        .await
                        .map(|balance| balance.into_iter().collect())
                } else {
                    Ok(Vec::new())
                }
            },
            async {
                if token_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    self.gateway.get_balance_tokens(request.chain, request.address.clone(), token_ids.clone()).await
                }
            },
            async {
                if token_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    self.gateway.get_balance_earn(request.chain, request.address.clone(), token_ids.clone()).await
                }
            },
        );
        Ok(rules::chain_balances(coin?, stake?, tokens?, earn?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use crate::api::GemApiClient;
    use crate::gateway::{EmptyPreferences, GemGateway};
    use crate::services::assets::testkit::MemoryAssetStore;
    use crate::services::preferences::{GemPreferencesService, testkit::MemoryPreferencesStore};
    use crate::services::price::{GemPriceService, testkit::MemoryPriceStore};
    use crate::services::stream::testkit::SubscriptionTestkit;
    use crate::services::wallet::testkit::MemoryWalletStore;
    use crate::services::wallet_session::{GemWalletSessionService, testkit::MemoryWalletSessionStore};
    use crate::testkit::TestAlienProvider;
    use async_trait::async_trait;
    use primitives::{Account, Chain, WalletType};
    use std::sync::Mutex;

    struct MemoryBalanceStore {
        rows: Mutex<Vec<AssetId>>,
        updates: Mutex<usize>,
    }

    #[async_trait]
    impl GemBalanceStore for MemoryBalanceStore {
        async fn get_available_balances(&self, _wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
            Ok(self.rows.lock().unwrap().iter().filter(|id| asset_ids.contains(id)).cloned().map(GemAssetBalance::zero).collect())
        }
        async fn update_balances(&self, _wallet_id: WalletId, _balances: Vec<GemBalanceRecord>) -> Result<(), GemServiceError> {
            *self.updates.lock().unwrap() += 1;
            Ok(())
        }
        async fn get_enabled_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
            Ok(Vec::new())
        }
        async fn set_assets_enabled(&self, _wallet_id: WalletId, _asset_ids: Vec<AssetId>, _enabled: bool) -> Result<(), GemServiceError> {
            Ok(())
        }
        async fn set_asset_pinned(&self, _wallet_id: WalletId, _asset_id: AssetId, _pinned: bool) -> Result<(), GemServiceError> {
            Ok(())
        }
    }

    fn service(rows: Vec<AssetId>) -> (GemBalanceService, Arc<MemoryAssetStore>, Arc<MemoryBalanceStore>) {
        let provider = Arc::new(TestAlienProvider::with_status(503));
        let preferences_store = Arc::new(MemoryPreferencesStore::default());
        let preferences = Arc::new(GemPreferencesService::new(preferences_store.clone()));
        let gateway = Arc::new(GemGateway::new(provider.clone(), preferences_store, Arc::new(EmptyPreferences)));
        let wallets = Arc::new(MemoryWalletStore::default());
        let session = Arc::new(GemWalletSessionService::new(Arc::new(MemoryWalletSessionStore { current: Mutex::new(None) }), wallets.clone()));
        let asset_store = Arc::new(MemoryAssetStore::default());
        let assets = Arc::new(GemAssetsService::new(
            Arc::new(GemApiClient::new(provider)),
            gateway.clone(),
            asset_store.clone(),
            Arc::new(GemPriceService::new(Arc::new(MemoryPriceStore::default()))),
            preferences,
            session,
        ));
        let store = Arc::new(MemoryBalanceStore {
            rows: Mutex::new(rows),
            updates: Mutex::new(0),
        });
        let service = GemBalanceService::new(gateway, wallets, asset_store.clone(), store.clone(), assets, Arc::new(SubscriptionTestkit::new(&[], &[]).service));
        (service, asset_store, store)
    }

    fn wallet() -> Wallet {
        Wallet {
            wallet_type: WalletType::Multicoin,
            ..Wallet::mock_with_accounts(Account::mock_chains(&[Chain::Cosmos, Chain::Ethereum], "address"))
        }
    }

    #[test]
    fn test_setup_of_a_new_wallet_adds_every_default_balance() {
        block_on(async {
            let (enabled, disabled) = crate::services::assets::rules::default_balances(&wallet());
            let (service, asset_store, _) = service(Vec::new());

            service.setup_wallet(wallet()).await.unwrap();

            let added = asset_store.added_balances.lock().unwrap();
            assert_eq!(added.len(), 2);
            assert_eq!(added[0], (wallet().id, enabled, true));
            assert_eq!(added[1], (wallet().id, disabled, false));
        });
    }

    #[test]
    fn test_setup_of_a_complete_wallet_writes_nothing() {
        block_on(async {
            let (enabled, disabled) = crate::services::assets::rules::default_balances(&wallet());
            let (service, asset_store, store) = service([enabled, disabled].concat());

            service.setup_wallet(wallet()).await.unwrap();

            assert!(asset_store.added_balances.lock().unwrap().is_empty());
            assert_eq!(*store.updates.lock().unwrap(), 0);
        });
    }

    #[test]
    fn test_a_balance_update_creates_only_the_rows_it_lacks() {
        block_on(async {
            let ethereum = AssetId::from_chain(Chain::Ethereum);
            let cosmos = AssetId::from_chain(Chain::Cosmos);
            let (service, asset_store, store) = service(vec![ethereum.clone()]);
            asset_store
                .save_assets(vec![
                    crate::services::assets::rules::default_asset_basic(Asset::from_chain(Chain::Ethereum)),
                    crate::services::assets::rules::default_asset_basic(Asset::from_chain(Chain::Cosmos)),
                ])
                .await
                .unwrap();
            let update = |asset_id: AssetId| GemBalanceUpdate {
                asset_id,
                update_type: GemBalanceUpdateType::Token { available: num_bigint::BigUint::ZERO },
                is_active: true,
            };

            service.update_balances(wallet().id, vec![update(ethereum), update(cosmos.clone())]).await.unwrap();

            assert_eq!(*asset_store.added_balances.lock().unwrap(), vec![(wallet().id, vec![cosmos], false)]);
            assert_eq!(*store.updates.lock().unwrap(), 1);
        });
    }

    #[test]
    fn test_setup_adds_only_the_balances_the_wallet_lacks() {
        block_on(async {
            let (enabled, disabled) = crate::services::assets::rules::default_balances(&wallet());
            let missing_enabled = enabled[0].clone();
            let missing_disabled = disabled[0].clone();
            let rows = [enabled, disabled].concat().into_iter().filter(|id| id != &missing_enabled && id != &missing_disabled).collect();
            let (service, asset_store, _) = service(rows);

            service.setup_wallet(wallet()).await.unwrap();

            let added = asset_store.added_balances.lock().unwrap();
            assert_eq!(*added, vec![(wallet().id, vec![missing_enabled], true), (wallet().id, vec![missing_disabled], false)]);
        });
    }
}
