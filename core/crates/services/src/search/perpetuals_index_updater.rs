use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::sync::{SearchSyncClient, SearchSyncResult};
use crate::ConfigCacher;
use config_keys::ConfigKey;
use primitives::{Asset, Perpetual};
use search_index::{PERPETUALS_INDEX_NAME, PerpetualDocument, SearchIndexClient, sanitize_index_primary_id};
use storage::{AssetsRepository, Database, DatabaseError, PerpetualsRepository, TagRepository};

pub struct PerpetualsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
}

impl PerpetualsIndexUpdater {
    pub fn new(database: Database, config: Arc<ConfigCacher>, search_index: &SearchIndexClient) -> Self {
        Self {
            sync_client: SearchSyncClient::new(config, search_index),
            database,
        }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchPerpetualsLastUpdatedAt).await?;
        let (perpetuals, public_tag_ids, perpetuals_tags) = self
            .database
            .run(|client| -> Result<_, DatabaseError> {
                let perpetuals = client.get_perpetuals()?;
                let public_tag_ids = client.get_perpetual_list_tags()?.into_iter().filter_map(|tag| tag.is_public().then_some(tag.id)).collect::<HashSet<_>>();
                let perpetuals_tags = client.get_perpetuals_tags()?;
                Ok((perpetuals, public_tag_ids, perpetuals_tags))
            })
            .await?;

        if perpetuals.is_empty() {
            return sync.write(PERPETUALS_INDEX_NAME, Vec::<PerpetualDocument>::new()).await;
        }

        let asset_ids = perpetuals.iter().map(|p| p.asset_id.clone()).collect::<Vec<_>>();
        let assets = self.database.run(move |client| client.get_assets(asset_ids)).await?;

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
