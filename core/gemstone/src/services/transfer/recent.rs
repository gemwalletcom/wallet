use std::sync::Arc;

use primitives::{AssetId, RecentActivityType, WalletId};

use crate::models::transaction::GemTransactionInputType;
use crate::services::error::GemServiceError;
use crate::services::transfer::{GemRecentActivity, GemRecentActivityStore};

#[derive(uniffi::Object)]
pub struct GemRecentActivityService {
    store: Arc<dyn GemRecentActivityStore>,
}

#[uniffi::export]
impl GemRecentActivityService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemRecentActivityStore>) -> Self {
        Self { store }
    }

    pub async fn add(&self, input_type: GemTransactionInputType, wallet_id: WalletId) -> Result<(), GemServiceError> {
        match input_type.recent_activity() {
            Some(activity) => self.store.add(activity, wallet_id).await,
            None => Ok(()),
        }
    }

    pub async fn add_asset(&self, activity_type: RecentActivityType, asset_id: AssetId, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.store
            .add(
                GemRecentActivity {
                    activity_type,
                    asset_id,
                    to_asset_id: None,
                },
                wallet_id,
            )
            .await
    }

    pub async fn clear(&self, wallet_id: WalletId, types: Vec<RecentActivityType>) -> Result<(), GemServiceError> {
        self.store.clear(wallet_id, types).await
    }
}

#[cfg(test)]
mod tests {
    use primitives::{Asset, Chain, StakeType};

    use super::*;
    use crate::services::transfer::testkit::MemoryRecentActivityStore;

    #[test]
    fn test_an_input_type_without_recent_activity_writes_nothing() {
        let store = Arc::new(MemoryRecentActivityStore::default());
        let service = GemRecentActivityService::new(store.clone());
        let asset = Asset::from_chain(Chain::Ethereum);
        let wallet_id = WalletId::Multicoin("address".to_string());

        futures::executor::block_on(service.add(GemTransactionInputType::Transfer { asset: asset.clone() }, wallet_id.clone())).unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);

        futures::executor::block_on(service.add(
            GemTransactionInputType::Stake {
                asset,
                stake_type: StakeType::Rewards(vec![]),
            },
            wallet_id,
        ))
        .unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);
    }
}
