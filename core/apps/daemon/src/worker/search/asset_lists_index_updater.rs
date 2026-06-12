use search_index::{ASSET_LISTS_INDEX_NAME, AssetListDocument, SearchIndexClient};
use storage::{Database, TagRepository};

pub struct AssetListsIndexUpdater {
    database: Database,
    search_index: SearchIndexClient,
}

impl AssetListsIndexUpdater {
    pub fn new(database: Database, search_index: &SearchIndexClient) -> Self {
        Self {
            database,
            search_index: search_index.clone(),
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let tags = self.database.tag()?.get_asset_list_tags()?;
        let documents = tags.iter().map(|tag| AssetListDocument::from(tag.as_primitive())).collect::<Vec<_>>();

        self.search_index.replace_documents(ASSET_LISTS_INDEX_NAME, documents).await
    }
}
