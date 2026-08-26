pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use futures::future::join_all;
use primitives::{AssetBalance, AssetId, WalletId};

pub use model::{GemBalanceUpdate, GemBalanceUpdateType, GemBalanceValue};
pub use store::GemBalanceStore;

use crate::gateway::GemGateway;
use crate::services::assets::GemAssetStore;
use crate::services::subscription::GemWalletStore;
use rules::{BalanceKind, BalanceRequest};

#[derive(uniffi::Object)]
pub struct GemBalanceService {
    gateway: Arc<GemGateway>,
    wallet_store: Arc<dyn GemWalletStore>,
    asset_store: Arc<dyn GemAssetStore>,
    store: Arc<dyn GemBalanceStore>,
}

#[uniffi::export]
impl GemBalanceService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, wallet_store: Arc<dyn GemWalletStore>, asset_store: Arc<dyn GemAssetStore>, store: Arc<dyn GemBalanceStore>) -> Self {
        Self {
            gateway,
            wallet_store,
            asset_store,
            store,
        }
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
        let balances: Vec<(BalanceKind, AssetBalance)> = join_all(requests.iter().map(|request| self.fetch(request))).await.into_iter().flatten().collect();
        if balances.is_empty() {
            return Ok(());
        }
        let assets = self
            .asset_store
            .get_assets(balances.iter().map(|(_, balance)| balance.asset_id.clone()).collect())
            .await
            .map_err(|error| GemServiceError::Store { msg: error.to_string() })?;
        let updates = rules::balance_updates(&assets, balances);
        self.store.update_balances(wallet_id, updates).await
    }
}

impl GemBalanceService {
    async fn fetch(&self, request: &BalanceRequest) -> Vec<(BalanceKind, AssetBalance)> {
        let token_ids: Vec<String> = request.token_ids.iter().filter_map(|asset_id| asset_id.token_id.clone()).collect();
        let (coin, stake, tokens, earn) = futures::join!(
            async {
                if request.coin {
                    self.gateway
                        .get_balance_coin(request.chain, request.address.clone())
                        .await
                        .ok()
                        .map(|balance| vec![balance])
                } else {
                    None
                }
            },
            async {
                if request.coin {
                    self.gateway
                        .get_balance_staking(request.chain, request.address.clone())
                        .await
                        .ok()
                        .flatten()
                        .map(|balance| vec![balance])
                } else {
                    None
                }
            },
            async {
                if token_ids.is_empty() {
                    None
                } else {
                    self.gateway.get_balance_tokens(request.chain, request.address.clone(), token_ids.clone()).await.ok()
                }
            },
            async {
                if token_ids.is_empty() {
                    None
                } else {
                    self.gateway.get_balance_earn(request.chain, request.address.clone(), token_ids.clone()).await.ok()
                }
            },
        );
        [
            (BalanceKind::Coin, coin),
            (BalanceKind::Stake, stake),
            (BalanceKind::Token, tokens),
            (BalanceKind::Earn, earn),
        ]
        .into_iter()
        .flat_map(|(kind, balances)| balances.unwrap_or_default().into_iter().map(move |balance| (kind, balance)))
        .collect()
    }
}
