use std::sync::Arc;

use primitives::{Asset, RecentActivityType, WalletId};

use crate::services::assets::GemAssetAction;
use crate::services::error::GemServiceError;
use crate::services::transfer::rules::TransferInput;
use crate::services::transfer::{GemRecentActivity, GemRecentActivityStore};
use crate::services::wallet_session::GemWalletSessionService;
use primitives::TransactionInputType;

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

    pub async fn clear(&self, types: Vec<RecentActivityType>) -> Result<(), GemServiceError> {
        self.store.clear(self.session.current_wallet_id()?, types).await
    }
}

impl GemRecentActivityService {
    pub async fn add(&self, input_type: TransactionInputType, wallet_id: WalletId) -> Result<(), GemServiceError> {
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

        block_on(service.add(TransactionInputType::Transfer { asset: asset.clone() }, wallet_id.clone())).unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);

        block_on(service.add(
            TransactionInputType::Stake {
                asset,
                stake_type: StakeType::Rewards(vec![]),
            },
            wallet_id,
        ))
        .unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemRecentsCounts {
    pub recents: u32,
    pub matching: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemRecentsSections {
    pub shows_items: bool,
    pub shows_clear: bool,
    pub shows_no_results: bool,
    pub shows_empty: bool,
}

#[uniffi::export]
impl GemRecentsCounts {
    pub fn sections(&self, is_searching: bool) -> GemRecentsSections {
        let no_results = self.recents > 0 && is_searching && self.matching == 0;
        GemRecentsSections {
            shows_items: self.matching > 0,
            shows_clear: self.recents > 0 && !is_searching,
            shows_no_results: no_results,
            shows_empty: self.recents == 0,
        }
    }
}

#[cfg(test)]
mod section_tests {
    use super::*;

    fn sections(recents: u32, matching: u32, is_searching: bool) -> GemRecentsSections {
        GemRecentsCounts { recents, matching }.sections(is_searching)
    }

    #[test]
    fn test_a_search_that_matches_nothing_is_not_the_same_as_having_no_recents() {
        let searched = sections(5, 0, true);
        assert!(searched.shows_no_results);
        assert!(!searched.shows_empty);
        assert!(!searched.shows_clear);

        let none = sections(0, 0, false);
        assert!(none.shows_empty);
        assert!(!none.shows_no_results);
        assert!(!none.shows_clear);

        let listed = sections(5, 5, false);
        assert!(listed.shows_items);
        assert!(!listed.shows_empty);
    }
}
