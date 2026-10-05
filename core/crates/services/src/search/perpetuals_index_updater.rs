use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::repository::{PerpetualsIndexData, Repository};
use super::sync::{SearchSyncClient, SearchSyncResult};
use config_keys::ConfigKey;
use primitives::{Asset, Perpetual};
use search_index::{PERPETUALS_INDEX_NAME, PerpetualDocument, sanitize_index_primary_id};

pub struct PerpetualsIndexUpdater {
    repository: Arc<dyn Repository>,
    sync_client: SearchSyncClient,
}

impl PerpetualsIndexUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, sync_client: SearchSyncClient) -> Self {
        Self { repository, sync_client }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchPerpetualsLastUpdatedAt).await?;
        let PerpetualsIndexData { perpetuals, list_tags, perpetuals_tags } = self.repository.perpetuals().await?;
        let public_tag_ids = list_tags.into_iter().filter_map(|tag| tag.is_public().then_some(tag.id)).collect::<HashSet<_>>();

        if perpetuals.is_empty() {
            return sync.write(PERPETUALS_INDEX_NAME, Vec::<PerpetualDocument>::new()).await;
        }

        let asset_ids = perpetuals.iter().map(|p| p.asset_id.clone()).collect::<Vec<_>>();
        let assets = self.repository.assets(asset_ids).await?;

        let assets_map: HashMap<String, Asset> = assets.into_iter().map(|a| (a.id.to_string(), a)).collect();
        let perpetuals_tags_map: HashMap<String, Vec<String>> = perpetuals_tags.into_iter().filter(|tag| public_tag_ids.contains(&tag.tag_id)).fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.perpetual_id.to_string()).or_default().push(tag.tag_id);
            acc
        });

        let documents = Self::build_documents(perpetuals, &assets_map, &perpetuals_tags_map);

        sync.write(PERPETUALS_INDEX_NAME, documents).await
    }

    fn build_documents(perpetuals: Vec<Perpetual>, assets_map: &HashMap<String, Asset>, perpetuals_tags_map: &HashMap<String, Vec<String>>) -> Vec<PerpetualDocument> {
        perpetuals
            .into_iter()
            .filter_map(|perpetual| {
                let perpetual_id = perpetual.id.to_string();
                let asset = assets_map.get(&perpetual.asset_id.to_string())?.clone();
                Some(PerpetualDocument {
                    id: sanitize_index_primary_id(&perpetual_id),
                    tags: perpetuals_tags_map.get(&perpetual_id).cloned(),
                    perpetual,
                    asset,
                })
            })
            .collect()
    }
}
