use std::sync::Arc;

use primitives::{Asset, AssetId, RecentActivityType, WalletId};

use crate::services::assets::GemAssetAction;
use crate::services::empty_state::GemEmptyStateKind;
use crate::services::error::GemServiceError;
use crate::services::search::rules::matching_assets;
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

    pub fn view_state(&self, assets: Vec<Asset>, query: String) -> GemRecentsViewState {
        let matching = matching_assets(assets.clone(), &query);
        GemRecentsViewState {
            sections: GemRecentsCounts {
                recents: assets.len() as u32,
                matching: matching.len() as u32,
            }
            .sections(!query.trim().is_empty()),
            matching_asset_ids: matching.into_iter().map(|asset| asset.id).collect(),
        }
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

    #[test]
    fn test_add_recent_records_for_the_current_wallet_only() {
        let store = Arc::new(MemoryRecentActivityStore::default());
        let asset = Asset::from_chain(Chain::Ethereum);
        let wallet_id = WalletId::Multicoin("address".to_string());

        block_on(GemRecentActivityService::mock(store.clone(), Some(wallet_id.clone())).add_recent(GemAssetAction::Receive, asset.clone())).unwrap();
        assert_eq!(store.added.lock().unwrap()[0].1, wallet_id.clone());
        block_on(GemRecentActivityService::mock(store.clone(), Some(wallet_id)).add_recent(GemAssetAction::Send, asset.clone())).unwrap();
        assert_eq!(store.added.lock().unwrap().len(), 1);
        assert!(block_on(GemRecentActivityService::mock(store.clone(), None).add_recent(GemAssetAction::Receive, asset)).is_err());
        assert_eq!(store.added.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_a_view_state_matches_the_query_and_names_the_sections() {
        let service = GemRecentActivityService::mock(Arc::new(MemoryRecentActivityStore::default()), None);
        let assets = vec![Asset::from_chain(Chain::Ethereum), Asset::from_chain(Chain::Bitcoin)];

        let listed = service.view_state(assets.clone(), String::new());
        assert_eq!(listed.matching_asset_ids.len(), 2);
        assert!(listed.sections.shows_items && listed.sections.shows_clear);

        let searched = service.view_state(assets.clone(), "bitcoin".to_string());
        assert_eq!(searched.matching_asset_ids, vec![Asset::from_chain(Chain::Bitcoin).id]);
        assert!(!searched.sections.shows_clear, "a search hides the clear action");

        let missed = service.view_state(assets, "  nothing  ".to_string());
        assert_eq!(missed.sections.empty, Some(GemEmptyStateKind::SearchAssets));
        assert_eq!(listed.sections.empty, None);

        assert_eq!(service.view_state(vec![], String::new()).sections.empty, Some(GemEmptyStateKind::Recents));
    }

    #[test]
    fn test_an_input_type_without_recent_activity_writes_nothing() {
        let store = Arc::new(MemoryRecentActivityStore::default());
        let service = GemRecentActivityService::mock(store.clone(), None);
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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecentsViewState {
    pub matching_asset_ids: Vec<AssetId>,
    pub sections: GemRecentsSections,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemRecentsSections {
    pub shows_items: bool,
    pub shows_clear: bool,
    pub empty: Option<GemEmptyStateKind>,
}

#[uniffi::export]
impl GemRecentsCounts {
    pub fn sections(&self, is_searching: bool) -> GemRecentsSections {
        let empty = match (self.matching > 0, self.recents > 0 && is_searching) {
            (true, _) => None,
            (false, true) => Some(GemEmptyStateKind::SearchAssets),
            (false, false) => Some(GemEmptyStateKind::Recents),
        };
        GemRecentsSections {
            shows_items: self.matching > 0,
            shows_clear: self.recents > 0 && !is_searching,
            empty,
        }
    }
}

#[cfg(test)]
mod section_tests {
    use super::*;

    #[test]
    fn test_a_search_that_matches_nothing_is_not_the_same_as_having_no_recents() {
        let searched = GemRecentsCounts { recents: 5, matching: 0 }.sections(true);
        assert_eq!(searched.empty, Some(GemEmptyStateKind::SearchAssets));
        assert!(!searched.shows_clear);

        let none = GemRecentsCounts { recents: 0, matching: 0 }.sections(false);
        assert_eq!(none.empty, Some(GemEmptyStateKind::Recents));
        assert!(!none.shows_clear);

        let listed = GemRecentsCounts { recents: 5, matching: 5 }.sections(false);
        assert!(listed.shows_items);
        assert_eq!(listed.empty, None);
    }
}
