use std::collections::HashMap;

use super::sync::{SearchSyncClient, SearchSyncResult};
use primitives::ConfigKey;
use search_index::{PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::models::{AssetRow, PerpetualRow};
use storage::{AssetsRepository, Database, PerpetualsRepository, TagRepository};

pub struct PerpetualsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
}

impl PerpetualsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            sync_client: SearchSyncClient::new(database.clone(), search_index),
            database,
        }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchPerpetualsLastUpdatedAt)?;
        let perpetuals = self.database.perpetuals()?.get_perpetual_rows()?;
        let perpetuals_tags = self.database.tag()?.get_perpetuals_tags()?;

        if perpetuals.is_empty() {
            return sync.write(PERPETUALS_INDEX_NAME, Vec::<PerpetualDocument>::new()).await;
        }

        let asset_ids = perpetuals.iter().map(|p| p.asset_id.0.clone()).collect::<Vec<_>>();
        let assets = self.database.assets()?.get_assets_rows(asset_ids)?;

        let assets_map: HashMap<String, AssetRow> = assets.into_iter().map(|a| (a.id.to_string(), a)).collect();
        let perpetuals_tags_map: HashMap<String, Vec<String>> = perpetuals_tags.into_iter().fold(HashMap::new(), |mut acc, tag| {
            acc.entry(tag.perpetual_id.to_string()).or_default().push(tag.tag_id);
            acc
        });

        let documents = Self::build_documents(perpetuals.iter(), &assets_map, &perpetuals_tags_map);

        sync.write(PERPETUALS_INDEX_NAME, documents).await
    }

    fn build_documents<'a>(
        perpetuals: impl IntoIterator<Item = &'a PerpetualRow>,
        assets_map: &HashMap<String, AssetRow>,
        perpetuals_tags_map: &HashMap<String, Vec<String>>,
    ) -> Vec<PerpetualDocument> {
        perpetuals
            .into_iter()
            .filter_map(|p| {
                let perpetual_id = p.id.to_string();
                assets_map.get(&p.asset_id.to_string()).map(|a| PerpetualDocument {
                    id: sanitize_index_primary_id(&perpetual_id),
                    perpetual: p.as_primitive(),
                    asset: a.as_primitive(),
                    tags: perpetuals_tags_map.get(&perpetual_id).cloned(),
                })
            })
            .collect()
    }
}
