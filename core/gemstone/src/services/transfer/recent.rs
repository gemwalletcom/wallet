use std::sync::Arc;

use primitives::WalletId;

use crate::models::transaction::GemTransactionInputType;
use crate::services::error::GemServiceError;
use crate::services::transfer::GemRecentActivityStore;

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

    pub fn record(&self, input_type: GemTransactionInputType, wallet_id: WalletId) -> Result<(), GemServiceError> {
        match input_type.recent_activity() {
            Some(activity) => self.store.add(activity, wallet_id),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use primitives::{Asset, Chain, StakeType};

    use super::*;
    use crate::services::transfer::GemRecentActivity;

    #[derive(Default)]
    struct RecordingStore {
        added: Mutex<Vec<GemRecentActivity>>,
    }

    impl GemRecentActivityStore for RecordingStore {
        fn add(&self, activity: GemRecentActivity, _wallet_id: WalletId) -> Result<(), GemServiceError> {
            self.added.lock().unwrap().push(activity);
            Ok(())
        }
    }

    #[test]
    fn test_an_input_type_without_recent_activity_writes_nothing() {
        let store = Arc::new(RecordingStore::default());
        let service = GemRecentActivityService::new(store.clone());
        let asset = Asset::from_chain(Chain::Ethereum);
        let wallet_id = WalletId::Multicoin("address".to_string());

        service.record(GemTransactionInputType::Transfer { asset: asset.clone() }, wallet_id.clone()).unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);

        service
            .record(
                GemTransactionInputType::Stake {
                    asset,
                    stake_type: StakeType::Rewards(vec![]),
                },
                wallet_id,
            )
            .unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);
    }
}
