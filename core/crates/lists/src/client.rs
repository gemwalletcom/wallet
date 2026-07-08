use std::{collections::HashMap, error::Error, sync::Arc};

use primitives::{AssetList, ListId, ListProviderName};
use storage::{Database, TagRepository};

use crate::provider::ListProvider;

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
        let tag = self.database.tag()?.get_tag(&id)?;
        if tag.as_ref().is_some_and(|tag| tag.list_id.as_deref() != Some(&list_id)) {
            return Ok(None);
        }

        let Some(provider) = self.providers.get(&list_id.provider) else {
            return Ok(None);
        };
        let Some(list) = provider.get_list(&list_id.provider_list_id).await? else {
            return Ok(None);
        };
        if tag.is_none() && self.database.tag()?.add_list_tag(&id, &list.name, list_id)? == 0 {
            return Ok(None);
        }
        if !list.asset_ids.is_empty() {
            self.database.tag()?.set_assets_tags_for_tag(&id, list.asset_ids)?;
        }
        let count = self.database.tag()?.get_assets_tags_for_tag(&id)?.len().try_into()?;

        Ok(Some(AssetList { id, name: list.name, count }))
    }

    pub async fn update_lists(&self, provider: ListProviderName) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let tags = self.database.tag()?.get_list_tags()?;
        let mut count = 0;
        for tag in tags {
            let Some(list_id) = tag.list_id.map(ListId::from) else {
                continue;
            };
            if list_id.provider == provider && self.add_list(tag.id, list_id).await?.is_some() {
                count += 1;
            }
        }
        Ok(count)
    }
}
