pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub mod testkit;

use crate::services::error::GemServiceError;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use futures::future::join_all;
use futures::lock::Mutex as AsyncMutex;
use primitives::{Asset, AssetBalance, AssetId, Wallet, WalletId};
use std::mem::{Discriminant, discriminant};

pub use model::{GemAssetBalance, GemAssetBalanceRow, GemBalanceRecord, GemBalanceRequirement, GemBalanceRow, GemBalanceRowValue, GemBalanceUpdate, GemBalanceUpdateType, GemBalanceValue};
pub use store::GemBalanceStore;

use crate::gateway::GemGateway;
use crate::services::assets::GemAssetsService;
use crate::services::stream::GemStreamSubscriptionService;
use crate::services::wallet::rules as wallet_rules;
use crate::services::wallet_session::GemWalletSessionService;
use rules::{BalanceKind, BalanceRequest};

type PublishedSequences = HashMap<(AssetId, Discriminant<GemBalanceUpdateType>), u64>;

#[derive(uniffi::Object)]
pub struct GemBalanceService {
    gateway: Arc<GemGateway>,
    store: Arc<dyn GemBalanceStore>,
    assets: Arc<GemAssetsService>,
    session: Arc<GemWalletSessionService>,
    stream: Arc<GemStreamSubscriptionService>,
    sequence: AtomicU64,
    published: Mutex<HashMap<WalletId, Arc<AsyncMutex<PublishedSequences>>>>,
}

#[uniffi::export]
impl GemBalanceService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, store: Arc<dyn GemBalanceStore>, assets: Arc<GemAssetsService>, session: Arc<GemWalletSessionService>, stream: Arc<GemStreamSubscriptionService>) -> Self {
        Self {
            gateway,
            store,
            assets,
            session,
            stream,
            sequence: AtomicU64::new(0),
            published: Mutex::new(HashMap::new()),
        }
    }
}

impl GemBalanceService {
    pub async fn balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
        self.store.get_available_balances(wallet_id, asset_ids).await
    }

    pub async fn set_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<(), GemServiceError> {
        let newly_enabled = self.store_assets_enabled(wallet_id.clone(), asset_ids, enabled).await?;
        self.refresh_enabled_assets(wallet_id, newly_enabled).await
    }

    pub async fn enable_assets(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let newly_enabled = self.store_assets_enabled(wallet_id.clone(), asset_ids, true).await?;
        let _ = self.refresh_enabled_assets(wallet_id, newly_enabled).await;
        Ok(())
    }

    async fn store_assets_enabled(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>, enabled: bool) -> Result<Vec<AssetId>, GemServiceError> {
        let asset_ids = rules::unique_asset_ids(asset_ids);
        let asset_ids = if enabled { rules::exclude_native_mirrors(asset_ids) } else { asset_ids };
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }
        if enabled {
            self.assets.sync_missing_assets(asset_ids.clone()).await?;
        }
        let enabled_ids = self.store.get_enabled_asset_ids(wallet_id.clone()).await?;
        self.add_missing_balances(wallet_id.clone(), asset_ids.clone()).await?;
        self.store.set_asset_configuration(wallet_id, asset_ids.clone(), rules::enabled_configuration(enabled)).await?;
        if !enabled {
            let _ = self.stream.resubscribe().await;
            return Ok(vec![]);
        }
        Ok(rules::missing_asset_ids(&asset_ids, &enabled_ids))
    }

    pub async fn set_asset_pinned(&self, wallet_id: WalletId, asset_id: AssetId, pinned: bool) -> Result<(), GemServiceError> {
        if pinned {
            self.enable_assets(wallet_id.clone(), vec![asset_id.clone()]).await?;
        }
        self.store.set_asset_configuration(wallet_id, vec![asset_id], rules::pinned_configuration(pinned)).await
    }

    pub async fn update(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let Some(wallet) = self.session.get_wallet(wallet_id.clone()).await? else {
            return Ok(());
        };
        let sequence = self.next_sequence();
        let requests = rules::balance_requests(&wallet.accounts, &asset_ids);
        let results = join_all(requests.iter().map(|request| self.chain_balances(request))).await;
        let (balances, failure) = rules::published_balances(results);
        if !balances.is_empty() {
            let assets = self.assets.assets(balances.iter().map(|(_, balance)| balance.asset_id.clone()).collect()).await?;
            self.write_balances(wallet_id, sequence, rules::balance_updates(balances), &assets).await?;
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    pub async fn update_enabled_balances(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        let asset_ids = self.store.get_enabled_asset_ids(wallet_id.clone()).await?;
        self.update(wallet_id, asset_ids).await
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        let (enabled, disabled) = crate::services::assets::rules::default_balances(&wallet);
        let stored_ids = self.store.get_balance_asset_ids(wallet.id.clone(), [enabled.clone(), disabled.clone()].concat()).await?;
        let has_synced = !stored_ids.is_empty();
        let enabled = rules::missing_asset_ids(&enabled, &stored_ids);
        let disabled = rules::missing_asset_ids(&disabled, &stored_ids);
        self.assets.add_balances(wallet.id.clone(), enabled.clone(), true).await?;
        self.assets.add_balances(wallet.id.clone(), disabled, false).await?;
        if wallet_rules::is_new_wallet(&wallet.source, has_synced) {
            let _ = self.stream.resubscribe().await;
        } else {
            let _ = self.refresh_enabled_assets(wallet.id, enabled).await;
        }
        Ok(())
    }

    async fn add_missing_balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        let stored_ids = self.store.get_balance_asset_ids(wallet_id.clone(), asset_ids.clone()).await?;
        self.assets.add_missing_balances(wallet_id, rules::missing_asset_ids(&asset_ids, &stored_ids)).await
    }

    async fn refresh_enabled_assets(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<(), GemServiceError> {
        if asset_ids.is_empty() {
            return Ok(());
        }
        let _ = self.stream.resubscribe().await;
        self.update(wallet_id, asset_ids).await
    }

    pub async fn update_balances(&self, wallet_id: WalletId, updates: Vec<GemBalanceUpdate>) -> Result<(), GemServiceError> {
        let sequence = self.next_sequence();
        let assets = self.assets.assets(updates.iter().map(|update| update.asset_id.clone()).collect()).await?;
        self.write_balances(wallet_id, sequence, updates, &assets).await
    }

    fn next_sequence(&self) -> u64 {
        self.sequence.fetch_add(1, Ordering::SeqCst)
    }

    fn newer_updates(published: &mut PublishedSequences, sequence: u64, updates: Vec<GemBalanceUpdate>) -> Vec<GemBalanceUpdate> {
        updates
            .into_iter()
            .filter(|update| {
                let key = (update.asset_id.clone(), discriminant(&update.update_type));
                if published.get(&key).is_some_and(|applied| *applied > sequence) {
                    return false;
                }
                published.insert(key, sequence);
                true
            })
            .collect()
    }

    fn wallet_publication(&self, wallet_id: &WalletId) -> Arc<AsyncMutex<PublishedSequences>> {
        self.published.lock().unwrap().entry(wallet_id.clone()).or_default().clone()
    }

    async fn write_balances(&self, wallet_id: WalletId, sequence: u64, updates: Vec<GemBalanceUpdate>, assets: &[Asset]) -> Result<(), GemServiceError> {
        let publication = self.wallet_publication(&wallet_id);
        let mut published = publication.lock().await;
        let updates = Self::newer_updates(&mut published, sequence, updates);
        if updates.is_empty() {
            return Ok(());
        }
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
                    self.gateway.get_balance_staking(request.chain, request.address.clone()).await.map(|balance| balance.into_iter().collect())
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
    use super::model::GemAssetConfiguration;
    use super::*;
    use crate::services::asset_discovery::testkit::DiscoveryTestkit;
    use crate::services::assets::rules::{default_asset_basic, default_balances};
    use crate::testkit::TestAlienProvider;
    use futures::executor::block_on;
    use num_bigint::BigUint;
    use primitives::{Chain, WalletSource};
    use testkit::{BalanceTestkit, MemoryBalanceStore};

    use crate::services::assets::GemAssetStore;

    #[test]
    fn test_hiding_an_asset_unpins_it_in_the_same_write() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let testkit = BalanceTestkit::new(MemoryBalanceStore::default());
            let ethereum = AssetId::from_chain(Chain::Ethereum);
            testkit.assets.save_assets(vec![default_asset_basic(Asset::from_chain(Chain::Ethereum))]).await.unwrap();

            testkit.service.set_assets_enabled(wallet.id.clone(), vec![ethereum.clone()], false).await.unwrap();
            testkit.service.set_asset_pinned(wallet.id.clone(), ethereum.clone(), false).await.unwrap();

            let writes = testkit.balances.configuration_writes.lock().unwrap().clone();
            assert_eq!(
                writes.iter().map(|(_, configuration)| *configuration).collect::<Vec<_>>(),
                vec![
                    GemAssetConfiguration {
                        is_enabled: Some(false),
                        is_pinned: Some(false)
                    },
                    GemAssetConfiguration { is_enabled: None, is_pinned: Some(false) }
                ],
                "hiding an asset unpins it in one patch, and unpinning leaves it enabled"
            );
        })
    }

    #[test]
    fn test_a_coin_and_a_stake_refresh_of_the_same_asset_keep_each_other() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let ethereum = AssetId::from_chain(Chain::Ethereum);
            let testkit = BalanceTestkit::new(MemoryBalanceStore {
                balances: std::sync::Mutex::new(HashMap::from([(wallet.id.clone(), vec![GemAssetBalance::zero(ethereum.clone())])])),
                yields_between_read_and_write: true,
                ..Default::default()
            });
            testkit.assets.save_assets(vec![default_asset_basic(Asset::from_chain(Chain::Ethereum))]).await.unwrap();

            let coin = GemBalanceUpdate::mock(GemBalanceUpdateType::Coin {
                available: BigUint::from(7u32),
                frozen: BigUint::ZERO,
                reserved: BigUint::ZERO,
                pending_unconfirmed: BigUint::ZERO,
            });
            let stake = GemBalanceUpdate::mock(GemBalanceUpdateType::Stake {
                staked: BigUint::from(3u32),
                pending: BigUint::ZERO,
                rewards: BigUint::ZERO,
                locked: BigUint::ZERO,
                frozen: BigUint::ZERO,
                metadata: None,
            });
            let (first, second) = futures::future::join(testkit.service.update_balances(wallet.id.clone(), vec![stake]), testkit.service.update_balances(wallet.id.clone(), vec![coin])).await;
            first.unwrap();
            second.unwrap();

            let stored = testkit.service.balances(wallet.id, vec![ethereum.clone()]).await.unwrap();
            assert_eq!(
                stored,
                vec![GemAssetBalance {
                    available: BigUint::from(7u32),
                    staked: BigUint::from(3u32),
                    ..GemAssetBalance::zero(ethereum)
                }]
            );
        });
    }

    #[test]
    fn test_a_response_that_arrives_after_a_newer_one_is_dropped() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let ethereum = AssetId::from_chain(Chain::Ethereum);
            let testkit = BalanceTestkit::new(MemoryBalanceStore::with_balances(wallet.id.clone(), vec![GemAssetBalance::zero(ethereum.clone())]));
            testkit.assets.save_assets(vec![default_asset_basic(Asset::from_chain(Chain::Ethereum))]).await.unwrap();

            let token = |available: u32| GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: BigUint::from(available) });
            let older = testkit.next_sequence();
            let newer = testkit.next_sequence();

            testkit.service.write_balances(wallet.id.clone(), newer, vec![token(9)], &[Asset::from_chain(Chain::Ethereum)]).await.unwrap();
            testkit.service.write_balances(wallet.id.clone(), older, vec![token(4)], &[Asset::from_chain(Chain::Ethereum)]).await.unwrap();

            let stored = testkit.service.balances(wallet.id, vec![ethereum.clone()]).await.unwrap();
            assert_eq!(
                stored,
                vec![GemAssetBalance {
                    available: BigUint::from(9u32),
                    ..GemAssetBalance::zero(ethereum)
                }]
            );
            assert_eq!(testkit.balances.balance_writes.lock().unwrap().len(), 1);
        });
    }

    #[test]
    fn test_setup_of_a_new_wallet_adds_every_default_balance() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Cosmos, Chain::Ethereum]);
            let (enabled, disabled) = default_balances(&wallet);
            let testkit = BalanceTestkit::new(MemoryBalanceStore::default());

            testkit.service.setup_wallet(wallet.clone()).await.unwrap();

            let added = testkit.assets.added_balances.lock().unwrap();
            assert_eq!(added.len(), 2);
            assert_eq!(added[0], (wallet.id.clone(), enabled, true));
            assert_eq!(added[1], (wallet.id, disabled, false));
        });
    }

    #[test]
    fn test_setup_of_a_created_wallet_adds_its_balances_without_asking_the_chains() {
        block_on(async {
            let imported = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let created = Wallet {
                source: WalletSource::Create,
                ..imported.clone()
            };
            let setup = |wallet: Wallet| async move {
                let testkit = DiscoveryTestkit::with_provider(Arc::new(TestAlienProvider::with_status(503)), wallet.clone());
                testkit.asset_store.save_assets(vec![default_asset_basic(Asset::from_chain(Chain::Ethereum))]).await.unwrap();
                testkit.balance.setup_wallet(wallet).await.unwrap();
                testkit
            };

            let created_kit = setup(created.clone()).await;
            let imported_kit = setup(imported).await;

            assert_eq!(
                created_kit.asset_store.added_balances.lock().unwrap()[0],
                (created.id.clone(), default_balances(&created).0, true),
                "a created wallet still gets its default rows"
            );
            assert!(created_kit.provider.requested_paths().is_empty(), "a created wallet has nothing to fetch yet");
            assert!(!imported_kit.provider.requested_paths().is_empty(), "an imported wallet asks the chains for what it holds");
        });
    }

    #[test]
    fn test_a_toggle_reports_a_failed_first_balance_fetch_and_a_side_effect_enable_does_not() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Ethereum]);
            let ethereum = AssetId::from_chain(Chain::Ethereum);
            let kit = || async {
                let testkit = DiscoveryTestkit::with_provider(Arc::new(TestAlienProvider::with_status(503)), wallet.clone());
                testkit.asset_store.save_assets(vec![default_asset_basic(Asset::from_chain(Chain::Ethereum))]).await.unwrap();
                testkit
            };

            let toggled = kit().await;
            let toggle = toggled.balance.set_assets_enabled(wallet.id.clone(), vec![ethereum.clone()], true).await;
            let side_effect = kit().await;
            let enable = side_effect.balance.enable_assets(wallet.id.clone(), vec![ethereum.clone()]).await;

            assert!(toggle.is_err(), "the user sees that the first balance fetch failed");
            assert!(enable.is_ok(), "a side-effect caller does not fail its own action on the fetch");
            for testkit in [&toggled, &side_effect] {
                let writes = testkit.balances.configuration_writes.lock().unwrap().clone();
                assert_eq!(writes.last().map(|(_, configuration)| configuration.is_enabled), Some(Some(true)), "the asset is enabled either way");
            }
        });
    }

    #[test]
    fn test_setup_of_a_complete_wallet_writes_nothing() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Cosmos, Chain::Ethereum]);
            let (enabled, disabled) = default_balances(&wallet);
            let rows = [enabled, disabled].concat().into_iter().map(GemAssetBalance::zero).collect();
            let testkit = BalanceTestkit::new(MemoryBalanceStore::with_balances(wallet.id.clone(), rows));

            testkit.service.setup_wallet(wallet).await.unwrap();

            assert!(testkit.assets.added_balances.lock().unwrap().is_empty());
            assert!(testkit.balances.balance_writes.lock().unwrap().is_empty());
        });
    }

    #[test]
    fn test_a_balance_update_creates_only_the_rows_it_lacks() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Cosmos, Chain::Ethereum]);
            let ethereum = AssetId::from_chain(Chain::Ethereum);
            let cosmos = AssetId::from_chain(Chain::Cosmos);
            let testkit = BalanceTestkit::new(MemoryBalanceStore::with_balances(wallet.id.clone(), vec![GemAssetBalance::zero(ethereum.clone())]));
            testkit
                .assets
                .save_assets(vec![default_asset_basic(Asset::from_chain(Chain::Ethereum)), default_asset_basic(Asset::from_chain(Chain::Cosmos))])
                .await
                .unwrap();
            let update = GemBalanceUpdate::mock(GemBalanceUpdateType::Token { available: num_bigint::BigUint::ZERO });

            testkit
                .service
                .update_balances(wallet.id.clone(), vec![GemBalanceUpdate { asset_id: ethereum, ..update.clone() }, GemBalanceUpdate { asset_id: cosmos.clone(), ..update }])
                .await
                .unwrap();

            assert_eq!(*testkit.assets.added_balances.lock().unwrap(), vec![(wallet.id, vec![cosmos], false)]);
            assert_eq!(testkit.balances.balance_writes.lock().unwrap().len(), 1);
        });
    }

    #[test]
    fn test_setup_adds_only_the_balances_the_wallet_lacks() {
        block_on(async {
            let wallet = Wallet::mock_with_chains(&[Chain::Cosmos, Chain::Ethereum]);
            let (enabled, disabled) = default_balances(&wallet);
            let missing_enabled = enabled[0].clone();
            let missing_disabled = disabled[0].clone();
            let rows = [enabled, disabled].concat().into_iter().filter(|id| id != &missing_enabled && id != &missing_disabled).map(GemAssetBalance::zero).collect();
            let testkit = BalanceTestkit::new(MemoryBalanceStore::with_balances(wallet.id.clone(), rows));

            testkit.service.setup_wallet(wallet.clone()).await.unwrap();

            let added = testkit.assets.added_balances.lock().unwrap();
            assert_eq!(*added, vec![(wallet.id.clone(), vec![missing_enabled], true), (wallet.id, vec![missing_disabled], false)]);
        });
    }
}
