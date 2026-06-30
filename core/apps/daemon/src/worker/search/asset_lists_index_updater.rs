use search_index::{ASSET_LISTS_INDEX_NAME, AssetListDocument, SearchIndexClient};
use storage::{
    Database, TagRepository,
    models::{AssetTagRow, PerpetualTagRow, TagRow},
};

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
        let tags = [self.database.tag()?.get_asset_list_tags()?, self.database.tag()?.get_perpetual_list_tags()?].concat();
        let assets_tags = self.database.tag()?.get_assets_tags()?;
        let perpetuals_tags = self.database.tag()?.get_perpetuals_tags()?;
        let documents = Self::build_documents(tags, &assets_tags, &perpetuals_tags);

        self.search_index.replace_documents(ASSET_LISTS_INDEX_NAME, documents).await
    }

    fn build_documents(tags: Vec<TagRow>, assets_tags: &[AssetTagRow], perpetuals_tags: &[PerpetualTagRow]) -> Vec<AssetListDocument> {
        tags.into_iter()
            .map(|tag| {
                let count = Self::count_items(&tag.id, assets_tags, perpetuals_tags);
                AssetListDocument::from(tag.as_primitive(count))
            })
            .collect()
    }

    fn count_items(tag_id: &str, assets_tags: &[AssetTagRow], perpetuals_tags: &[PerpetualTagRow]) -> u32 {
        let asset_count = assets_tags.iter().filter(|tag| tag.tag_id == tag_id).count();
        let perpetual_count = perpetuals_tags.iter().filter(|tag| tag.tag_id == tag_id).count();
        (asset_count + perpetual_count) as u32
    }
}
