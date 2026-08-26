pub mod error;
pub mod rules;
pub mod store;

use std::sync::Arc;

use chrono::Utc;
use primitives::AssetId;

pub use error::GemTransactionsError;
pub use store::GemTransactionStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::assets::GemAssetsService;
use crate::services::name::GemAddressStore;

#[derive(uniffi::Object)]
pub struct GemTransactionsService {
    api: Arc<GemDeviceApiClient>,
    assets: Arc<GemAssetsService>,
    store: Arc<dyn GemTransactionStore>,
    address_store: Arc<dyn GemAddressStore>,
}

#[uniffi::export]
impl GemTransactionsService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, assets: Arc<GemAssetsService>, store: Arc<dyn GemTransactionStore>, address_store: Arc<dyn GemAddressStore>) -> Self {
        Self {
            api,
            assets,
            store,
            address_store,
        }
    }

    pub async fn sync(&self, wallet_id: String, asset_id: Option<AssetId>) -> Result<(), GemTransactionsError> {
        let from_timestamp = self.store.get_sync_timestamp(wallet_id.clone(), asset_id.clone()).await?;
        let timestamp = Utc::now().timestamp() as u64;
        let response = self
            .api
            .client
            .get_transactions(wallet_id.clone(), asset_id.as_ref().map(|asset_id| asset_id.to_string()), from_timestamp)
            .await
            .map_err(GemApiError::from)?;

        let new_asset_ids = self.assets.prefetch_assets(rules::transaction_asset_ids(&response.transactions)).await?;
        if !new_asset_ids.is_empty() {
            self.assets.add_missing_balances(wallet_id.clone(), new_asset_ids).await?;
        }
        self.store.add_transactions(wallet_id.clone(), response.transactions).await?;
        self.address_store.save_address_names(response.address_names).await?;
        self.store.set_sync_timestamp(wallet_id, asset_id, timestamp).await
    }

    pub async fn get_assets_list(&self, wallet_id: String, from_timestamp: u64) -> Result<Vec<String>, GemApiError> {
        Ok(self.api.client.get_assets_list(wallet_id, from_timestamp).await?)
    }
}
