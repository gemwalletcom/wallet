pub mod details;
pub mod filter;
pub mod model;
pub mod rules;

use crate::models::state::GemLoadState;
use crate::services::error::GemServiceError;
use crate::services::transaction_state::GemTransactionStateStore;
use std::sync::Arc;

use chrono::Utc;
use primitives::{AssetId, Chain, Transaction, Wallet, WalletId};

pub use details::GemTransactionDetailsService;
pub use model::{
    GemAmountSign, GemSwapAgain, GemSwapProgress, GemSwapProgressStep, GemTransactionAmount, GemTransactionDetailRow, GemTransactionDetailRows, GemTransactionDetailSection, GemTransactionFilter, GemTransactionHeader,
    GemTransactionHeaderAction, GemTransactionHeaderKind, GemTransactionParticipantRole, GemTransactionRow, GemTransactionRowSubtitle, GemTransactionRowValue, GemTransactionStateTone, GemTransactionStatus, GemTransactionTitle,
};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::assets::GemAssetsService;
use crate::services::chain::rules as chain_rules;
use crate::services::name::GemNameService;
use crate::services::swap::GemSwapPair;
use crate::services::transaction_state::GemTransactionStatusService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemTransactionsService {
    api: Arc<GemDeviceApiClient>,
    assets: Arc<GemAssetsService>,
    store: Arc<dyn GemTransactionStateStore>,
    names: Arc<GemNameService>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
    session: Arc<GemWalletSessionService>,
    transaction_status: Arc<dyn GemTransactionStatusService>,
}

#[uniffi::export]
pub fn transaction_swap_pair(transaction: Transaction) -> Option<GemSwapPair> {
    transaction.swap_metadata().map(|metadata| GemSwapPair {
        from_asset_id: metadata.from_asset,
        to_asset_id: metadata.to_asset,
    })
}

#[uniffi::export]
impl GemTransactionsService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        assets: Arc<GemAssetsService>,
        store: Arc<dyn GemTransactionStateStore>,
        names: Arc<GemNameService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
        session: Arc<GemWalletSessionService>,
        transaction_status: Arc<dyn GemTransactionStatusService>,
    ) -> Self {
        Self {
            api,
            assets,
            store,
            names,
            wallet_preferences,
            session,
            transaction_status,
        }
    }

    pub fn filter_chains(&self, wallet: Wallet) -> Vec<Chain> {
        chain_rules::wallet_chains_by_rank(&wallet)
    }

    pub async fn refresh(&self, asset_id: Option<AssetId>, has_transactions: bool) -> GemLoadState {
        GemLoadState::refreshed(self.sync(asset_id).await, has_transactions)
    }
}

impl GemTransactionsService {
    pub async fn save_transactions(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        self.store.add_transactions(wallet_id, transactions).await
    }

    async fn sync(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        self.sync_wallet(self.session.current_wallet_id()?, asset_id).await
    }

    pub async fn sync_wallet(&self, wallet_id: WalletId, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        let from_timestamp = self.wallet_preferences.get_transactions_timestamp(wallet_id.clone(), asset_id.clone());
        let timestamp = Utc::now().timestamp() as u64;
        let response = self
            .api
            .client
            .get_transactions(wallet_id.id(), asset_id.as_ref().map(|asset_id| asset_id.to_string()), from_timestamp)
            .await
            .map_err(GemApiError::from)?;

        let new_asset_ids = self.assets.sync_missing_assets(rules::transaction_asset_ids(&response.transactions)).await?;
        if !new_asset_ids.is_empty() {
            self.assets.add_missing_balances(wallet_id.clone(), new_asset_ids).await?;
        }
        let pending = rules::pending_transactions(&response.transactions);
        self.store.add_transactions(wallet_id.clone(), response.transactions).await?;
        self.names.save_names(response.address_names).await?;
        self.wallet_preferences.set_transactions_timestamp(wallet_id.clone(), asset_id, timestamp)?;
        if !pending.is_empty() {
            self.transaction_status.track(wallet_id, pending);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, TransactionSwapMetadata};

    #[test]
    fn test_a_swap_leg_the_build_cannot_read_leaves_the_transaction_without_a_pair() {
        let swap = Transaction::mock_swap();
        assert_eq!(
            transaction_swap_pair(swap.clone()),
            Some(GemSwapPair {
                from_asset_id: TransactionSwapMetadata::mock().from_asset,
                to_asset_id: TransactionSwapMetadata::mock().to_asset,
            })
        );
        let unreadable = Transaction {
            metadata: Some(serde_json::json!({ "fromAsset": "gemchain", "toAsset": Chain::Ethereum.as_ref() })),
            ..swap
        };
        assert_eq!(transaction_swap_pair(unreadable), None);
    }
}
