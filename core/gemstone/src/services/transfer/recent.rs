use std::sync::Arc;

use primitives::{Asset, RecentActivityType, WalletId};

use crate::models::transaction::GemTransactionInputType;
use crate::services::assets::GemAssetAction;
use crate::services::error::GemServiceError;
use crate::services::transfer::{GemRecentActivity, GemRecentActivityStore};
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemRecentActivityService {
    store: Arc<dyn GemRecentActivityStore>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemRecentActivityService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemRecentActivityStore>, session: Arc<GemWalletSessionService>) -> Self {
        Self { store, session }
    }

    pub async fn add_recent(&self, action: GemAssetAction, asset: Asset) -> Result<(), GemServiceError> {
        let Some(activity_type) = action.recent_activity_type(&asset) else {
            return Ok(());
        };
        let wallet_id = self.session.current_wallet_id()?;
        self.store
            .add(
                GemRecentActivity {
                    activity_type,
                    asset_id: asset.id,
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

impl GemRecentActivityService {
    pub async fn add(&self, input_type: GemTransactionInputType, wallet_id: WalletId) -> Result<(), GemServiceError> {
        match input_type.recent_activity() {
            Some(activity) => self.store.add(activity, wallet_id).await,
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::block_on;
    use primitives::{Asset, Chain, StakeType};

    use super::*;
    use crate::services::transfer::testkit::MemoryRecentActivityStore;
    use crate::services::wallet::testkit::MemoryWalletStore;
    use crate::services::wallet_session::testkit::MemoryWalletSessionStore;

    fn service(store: Arc<MemoryRecentActivityStore>, wallet_id: Option<WalletId>) -> GemRecentActivityService {
        let session = Arc::new(GemWalletSessionService::new(
            Arc::new(MemoryWalletSessionStore::default()),
            Arc::new(MemoryWalletStore::default()),
        ));
        session.set_current_wallet_id(wallet_id).unwrap();
        GemRecentActivityService::new(store, session)
    }

    #[test]
    fn test_add_recent_records_for_the_current_wallet_only() {
        let store = Arc::new(MemoryRecentActivityStore::default());
        let asset = Asset::from_chain(Chain::Ethereum);
        let wallet_id = WalletId::Multicoin("address".to_string());

        block_on(service(store.clone(), Some(wallet_id.clone())).add_recent(GemAssetAction::Receive, asset.clone())).unwrap();
        assert_eq!(store.added.lock().unwrap()[0].1, wallet_id.clone());
        block_on(service(store.clone(), Some(wallet_id)).add_recent(GemAssetAction::Send, asset.clone())).unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);
        assert!(block_on(service(store.clone(), None).add_recent(GemAssetAction::Receive, asset)).is_err());
        assert_eq!(store.added.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_an_input_type_without_recent_activity_writes_nothing() {
        let store = Arc::new(MemoryRecentActivityStore::default());
        let service = service(store.clone(), None);
        let asset = Asset::from_chain(Chain::Ethereum);
        let wallet_id = WalletId::Multicoin("address".to_string());

        block_on(service.add(GemTransactionInputType::Transfer { asset: asset.clone() }, wallet_id.clone())).unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);

        block_on(service.add(
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
