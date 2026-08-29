pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use chrono::Utc;
use primitives::{AssetId, Transaction, WalletId};

pub use model::{GemAmountSign, GemTransactionSubtitle, GemTransactionTitle, GemTransactionValue};
pub use store::GemTransactionStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::assets::GemAssetsService;
use crate::services::name::GemAddressStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemTransactionsService {
    api: Arc<GemDeviceApiClient>,
    assets: Arc<GemAssetsService>,
    store: Arc<dyn GemTransactionStore>,
    address_store: Arc<dyn GemAddressStore>,
    preferences: Arc<GemWalletPreferencesService>,
}

#[uniffi::export]
impl GemTransactionsService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        assets: Arc<GemAssetsService>,
        store: Arc<dyn GemTransactionStore>,
        address_store: Arc<dyn GemAddressStore>,
        preferences: Arc<GemWalletPreferencesService>,
    ) -> Self {
        Self {
            api,
            assets,
            store,
            address_store,
            preferences,
        }
    }

    pub async fn sync(&self, wallet_id: WalletId, asset_id: Option<AssetId>) -> Result<(), GemServiceError> {
        let from_timestamp = self.preferences.get_transactions_timestamp(wallet_id.clone(), asset_id.clone());
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
        self.store.save_transactions(wallet_id.clone(), response.transactions).await?;
        self.address_store.save_address_names(response.address_names).await?;
        self.preferences.set_transactions_timestamp(wallet_id, asset_id, timestamp)
    }
}

#[derive(Default, uniffi::Object)]
pub struct GemTransactionFormatter {}

#[uniffi::export]
impl GemTransactionFormatter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn title(&self, transaction: Transaction) -> GemTransactionTitle {
        rules::transaction_title(&transaction)
    }

    pub fn subtitle(&self, transaction: Transaction) -> GemTransactionSubtitle {
        rules::transaction_subtitle(&transaction)
    }

    pub fn value(&self, transaction: Transaction) -> GemTransactionValue {
        rules::transaction_value(&transaction)
    }

    pub fn equivalent_value(&self, transaction: Transaction) -> GemTransactionValue {
        rules::transaction_equivalent_value(&transaction)
    }
}
