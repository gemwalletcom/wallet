use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;

use lists::ListProvider;
use primitives::{AssetId, AssetIdVecExt, AssetList, ListId, ListProviderName};
use storage::{AssetFilter, AssetsRepository, Database, DatabaseClient, DatabaseError, TagRepository};

pub struct ListsClient {
    database: Database,
    providers: HashMap<ListProviderName, Arc<dyn ListProvider>>,
}

impl ListsClient {
    pub fn new(database: Database, providers: Vec<Arc<dyn ListProvider>>) -> Self {
        Self {
            database,
            providers: providers.into_iter().map(|provider| (provider.provider(), provider)).collect(),
        }
    }

    pub async fn add_list(&self, id: String, list_id: ListId) -> Result<Option<AssetList>, Box<dyn Error + Send + Sync>> {
        let tag_id = id.clone();
        let tag = self.database.run(move |client| client.get_tag(&tag_id)).await?;
        if tag.as_ref().is_some_and(|tag| tag.list_id.as_ref() != Some(&list_id)) {
            return Ok(None);
        }

        let Some(provider) = self.providers.get(&list_id.provider) else {
            return Ok(None);
        };
        let Some(list) = provider.get_list(&list_id.provider_list_id).await? else {
            return Ok(None);
        };
        let tag_id = id.clone();
        let list_name = list.name.clone();
        let candidate_asset_ids = list.asset_ids;
        let is_new_tag = tag.is_none();
        let count = self
            .database
            .run(move |client| -> Result<Option<usize>, DatabaseError> {
                let asset_ids = known_asset_ids(client, candidate_asset_ids)?;
                if is_new_tag && client.add_list_tag(&tag_id, &list_name, list_id)? == 0 {
                    return Ok(None);
                }
                if !asset_ids.is_empty() {
                    client.set_assets_tags_for_tag(&tag_id, asset_ids)?;
                }
                Ok(Some(client.get_asset_ids_for_tag(&tag_id)?.len()))
            })
            .await?;
        let Some(count) = count else {
            return Ok(None);
        };

        Ok(Some(AssetList {
            id,
            name: list.name,
            count: count.try_into()?,
        }))
    }

    pub async fn update_lists(&self, provider: ListProviderName) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let tags = self.database.run(TagRepository::get_list_tags).await?;
        let mut count = 0;
        for tag in tags {
            let Some(list_id) = tag.list_id else {
                continue;
            };
            if list_id.provider == provider && self.add_list(tag.id, list_id).await?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }
}

fn known_asset_ids(client: &mut DatabaseClient, asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, DatabaseError> {
    if asset_ids.is_empty() {
        return Ok(asset_ids);
    }
    let existing = client.get_asset_ids_by_filter(vec![AssetFilter::Ids(asset_ids.ids())])?.into_iter().collect::<HashSet<_>>();
    let mut seen = HashSet::new();
    Ok(asset_ids.into_iter().filter(|asset_id| existing.contains(asset_id) && seen.insert(asset_id.clone())).collect())
}
