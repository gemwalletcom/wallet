use super::sync::{SearchSyncClient, SearchSyncResult};
use config_keys::ConfigKey;
use primitives::NFTCollection;
use search_index::{NFTDocument, NFTS_INDEX_NAME};
use storage::{Database, NftCollectionFilter, NftRepository};

pub struct NftsIndexUpdater {
    database: Database,
    sync_client: SearchSyncClient,
}

impl NftsIndexUpdater {
    pub fn new(database: Database, sync_client: SearchSyncClient) -> Self {
        Self { database, sync_client }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchNftsLastUpdatedAt).await?;
        let filters = sync.since().map(NftCollectionFilter::UpdatedSince).into_iter().collect();
        let collections = self.database.run(move |client| client.get_nft_collections(filters)).await?;

        let documents = Self::build_documents(collections);

        sync.write(NFTS_INDEX_NAME, documents).await
    }

    fn build_documents(collections: Vec<NFTCollection>) -> Vec<NFTDocument> {
        collections.into_iter().map(|collection| NFTDocument::from(NFTCollection { links: vec![], ..collection })).collect()
    }
}
