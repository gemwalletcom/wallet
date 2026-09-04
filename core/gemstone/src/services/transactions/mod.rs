pub mod details;
pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use chrono::Utc;
use primitives::{AssetId, Chain, Currency, WalletId};

pub use details::GemTransactionDetailsService;
pub use model::{
    GemAmountSign, GemSwapAgain, GemSwapProgress, GemSwapProgressStep, GemTransactionDetails, GemTransactionHeaderKind, GemTransactionParticipant, GemTransactionParticipantRole,
    GemTransactionSubtitle, GemTransactionSummary, GemTransactionTitle, GemTransactionValue,
};
pub use store::GemTransactionStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::assets::GemAssetsService;
use crate::services::chain::rules as chain_rules;
use crate::services::name::GemAddressStore;
use crate::services::preferences::GemPreferencesService;
use crate::services::transaction_state::GemTransactionStatusService;
use crate::services::wallet_preferences::GemWalletPreferencesService;
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemTransactionsService {
    api: Arc<GemDeviceApiClient>,
    assets: Arc<GemAssetsService>,
    store: Arc<dyn GemTransactionStore>,
    address_store: Arc<dyn GemAddressStore>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
    preferences: Arc<GemPreferencesService>,
    session: Arc<GemWalletSessionService>,
    transaction_status: Arc<dyn GemTransactionStatusService>,
}

#[uniffi::export]
impl GemTransactionsService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        assets: Arc<GemAssetsService>,
        store: Arc<dyn GemTransactionStore>,
        address_store: Arc<dyn GemAddressStore>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
        preferences: Arc<GemPreferencesService>,
        session: Arc<GemWalletSessionService>,
        transaction_status: Arc<dyn GemTransactionStatusService>,
    ) -> Self {
        Self {
            api,
            assets,
            store,
            address_store,
            wallet_preferences,
            preferences,
            session,
            transaction_status,
        }
    }

    pub fn filter_chains(&self) -> Result<Vec<Chain>, GemServiceError> {
        Ok(chain_rules::wallet_chains_by_rank(&self.session.current_wallet()?))
    }

    pub fn get_currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub async fn sync(&self, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        self.sync_wallet(self.session.current_wallet_id()?, asset_id).await
    }
}

impl GemTransactionsService {
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
        self.store.save_transactions(wallet_id.clone(), response.transactions).await?;
        self.address_store.save_address_names(response.address_names).await?;
        self.wallet_preferences.set_transactions_timestamp(wallet_id.clone(), asset_id, timestamp)?;
        if !pending.is_empty() {
            self.transaction_status.track(wallet_id, pending);
        }
        Ok(())
    }
}
