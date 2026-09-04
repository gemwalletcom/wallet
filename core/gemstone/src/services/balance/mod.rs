pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use futures::future::join_all;
use primitives::{AssetBalance, AssetId, Wallet, WalletId};

pub use model::{GemAssetBalance, GemBalanceRequirement, GemBalanceRow, GemBalanceUpdate, GemBalanceUpdateType, GemBalanceValue};
pub use store::GemBalanceStore;

use crate::gateway::GemGateway;
use crate::services::assets::rules::default_balances;
use crate::services::assets::{GemAssetStore, GemAssetsService};
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;
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
    price: Arc<GemPriceService>,
    stream: Arc<GemStreamSubscriptionService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemBalanceService {
    pub fn balances(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) -> Result<Vec<GemAssetBalance>, GemServiceError> {
        self.store.get_available_balances(wallet_id, asset_ids)
    }

    #[uniffi::constructor]
    pub fn new(
        gateway: Arc<GemGateway>,
        wallet_store: Arc<dyn GemWalletStore>,
        asset_store: Arc<dyn GemAssetStore>,
        store: Arc<dyn GemBalanceStore>,
        assets: Arc<GemAssetsService>,
        price: Arc<GemPriceService>,
        stream: Arc<GemStreamSubscriptionService>,
        preferences: Arc<GemPreferencesService>,
    ) -> Self {
        Self {
            gateway,
            wallet_store,
            asset_store,
            store,
            assets,
            price,
            stream,
            preferences,
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
        let enabled_ids = self.store.get_enabled_asset_ids(wallet_id.clone(), asset_ids.clone()).await?;
        self.assets.add_missing_balances(wallet_id.clone(), asset_ids.clone()).await?;
        self.store.set_assets_enabled(wallet_id.clone(), asset_ids.clone(), enabled).await?;
        if enabled {
            self.refresh_enabled_assets(wallet_id, rules::newly_enabled_asset_ids(&asset_ids, &enabled_ids)).await;
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
                .map_err(|error| GemServiceError::Store { msg: error.to_string() })?;
            let updates = rules::balance_updates(&assets, balances);
            self.update_balances(wallet_id, updates).await?;
        }
        match failures.into_iter().find_map(Result::err) {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

impl GemBalanceService {
    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        let (defaults, _) = default_balances(&wallet);
        let stored = self.store.get_enabled_asset_ids(wallet.id.clone(), defaults).await?;
        let enabled = self.assets.setup_wallet(wallet.clone()).await?;
        self.refresh_enabled_assets(wallet.id, rules::newly_enabled_asset_ids(&enabled, &stored)).await;
        Ok(())
    }

    async fn refresh_enabled_assets(&self, wallet_id: WalletId, asset_ids: Vec<AssetId>) {
        if asset_ids.is_empty() {
            return;
        }
        let currency = self.preferences.get_currency();
        if let Ok(prices) = self.price.get_prices(currency.clone(), asset_ids.clone()).await {
            let _ = self.price.update_prices(prices, currency).await;
        }
        let _ = self.stream.add_prices(asset_ids.clone()).await;
        let _ = self.update(wallet_id, asset_ids).await;
    }

    pub async fn update_balances(&self, wallet_id: WalletId, updates: Vec<GemBalanceUpdate>) -> Result<(), GemServiceError> {
        let asset_ids = updates.iter().map(|update| update.asset_id.clone()).collect();
        self.assets.add_missing_balances(wallet_id.clone(), asset_ids).await?;
        self.store.update_balances(wallet_id, updates).await
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
