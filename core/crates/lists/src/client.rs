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
        let Some(provider) = self.providers.get(&list_id.provider) else {
            return Ok(None);
        };
        let Some(list) = provider.get_list(&list_id.provider_list_id).await? else {
            return Ok(None);
        };
        self.database.tag()?.add_list_tag(&id, &list.name, &list_id.id())?;
        if !list.asset_ids.is_empty() {
            self.database.tag()?.set_assets_tags_for_tag(&id, list.asset_ids)?;
        }
        let count = self.database.tag()?.get_assets_tags_for_tag(&id)?.len() as u32;

        Ok(Some(AssetList { id, name: list.name, count }))
    }
}
