use std::collections::HashSet;
use std::sync::Arc;

use primitives::{Asset, AssetId, RecentActivityType, RecentAsset, WalletId};

use crate::day_section::GemDay;
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

    /// `days[i]` is the local day of `recents[i]`; the apps know the time zone, Core groups.
    pub fn view_state(&self, recents: Vec<RecentAsset>, days: Vec<GemDay>, query: String) -> GemRecentsViewState {
        let count = recents.len() as u32;
        let matching: HashSet<AssetId> = matching_assets(recents.iter().map(|recent| recent.asset.clone()).collect(), &query).into_iter().map(|asset| asset.id).collect();
        let dated: Vec<(GemDay, RecentAsset)> = days.into_iter().zip(recents).filter(|(_, recent)| matching.contains(&recent.asset.id)).collect();
        GemRecentsViewState {
            sections: GemRecentsCounts {
                recents: count,
                matching: dated.len() as u32,
            }
            .sections(!query.trim().is_empty()),
            days: recent_days(dated),
        }
    }
}

fn recent_days(mut dated: Vec<(GemDay, RecentAsset)>) -> Vec<GemRecentsDay> {
    dated.sort_by_key(|(_, recent)| std::cmp::Reverse(recent.created_at));
    dated.into_iter().fold(Vec::new(), |mut days: Vec<GemRecentsDay>, (day, recent)| {
        match days.iter_mut().find(|section| section.day == day) {
            Some(section) => section.recents.push(recent),
            None => days.push(GemRecentsDay { day, recents: vec![recent] }),
        }
        days
    })
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
    use chrono::DateTime;
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

    fn recent(chain: Chain, seconds: i64) -> RecentAsset {
        RecentAsset {
            asset: Asset::from_chain(chain),
            created_at: DateTime::from_timestamp(seconds, 0).unwrap(),
        }
    }

    #[test]
    fn test_a_view_state_matches_the_query_and_names_the_sections() {
        let service = GemRecentActivityService::mock(Arc::new(MemoryRecentActivityStore::default()), None);
        let today = GemDay { year: 2026, month: 3, day: 2 };
        let recents = vec![recent(Chain::Ethereum, 1), recent(Chain::Bitcoin, 2)];
        let days = vec![today, today];

        let listed = service.view_state(recents.clone(), days.clone(), String::new());
        assert_eq!(listed.days.iter().map(|day| day.recents.len()).sum::<usize>(), 2);
        assert!(listed.sections.shows_items && listed.sections.shows_clear);

        let searched = service.view_state(recents.clone(), days.clone(), "bitcoin".to_string());
        assert_eq!(searched.days[0].recents, vec![recent(Chain::Bitcoin, 2)]);
        assert!(!searched.sections.shows_clear, "a search hides the clear action");

        let missed = service.view_state(recents, days, "  nothing  ".to_string());
        assert_eq!(missed.sections.empty, Some(GemEmptyStateKind::SearchAssets));
        assert!(missed.days.is_empty());
        assert_eq!(listed.sections.empty, None);

        assert_eq!(service.view_state(vec![], vec![], String::new()).sections.empty, Some(GemEmptyStateKind::Recents));
    }

    #[test]
    fn test_recents_are_grouped_by_their_day_newest_first() {
        let service = GemRecentActivityService::mock(Arc::new(MemoryRecentActivityStore::default()), None);
        let today = GemDay { year: 2026, month: 3, day: 2 };
        let yesterday = GemDay { year: 2026, month: 3, day: 1 };
        let recents = vec![recent(Chain::Ethereum, 10), recent(Chain::Bitcoin, 30), recent(Chain::Solana, 20)];

        let days = service.view_state(recents, vec![yesterday, today, today], String::new()).days;

        assert_eq!(
            days.iter().map(|day| (day.day, day.recents.iter().map(|recent| recent.asset.chain()).collect::<Vec<_>>())).collect::<Vec<_>>(),
            vec![(today, vec![Chain::Bitcoin, Chain::Solana]), (yesterday, vec![Chain::Ethereum])]
        );
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemRecentsCounts {
    pub recents: u32,
    pub matching: u32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecentsDay {
    pub day: GemDay,
    pub recents: Vec<RecentAsset>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecentsViewState {
    pub sections: GemRecentsSections,
    pub days: Vec<GemRecentsDay>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemRecentsSections {
    pub shows_items: bool,
    pub shows_clear: bool,
    pub empty: Option<GemEmptyStateKind>,
}

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
