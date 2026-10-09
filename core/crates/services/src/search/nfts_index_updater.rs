use super::sync::{SearchSyncClient, SearchSyncResult};
use config_keys::ConfigKey;
use primitives::NFTCollection;
use search_index::{NFTDocument, NFTS_INDEX_NAME};
use std::sync::Arc;

use storage::NftCollectionFilter;

use super::repository::Repository;

pub struct NftsIndexUpdater {
    repository: Arc<dyn Repository>,
    sync_client: SearchSyncClient,
}

impl NftsIndexUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, sync_client: SearchSyncClient) -> Self {
        Self { repository, sync_client }
    }

    pub async fn update(&self) -> Result<SearchSyncResult, Box<dyn std::error::Error + Send + Sync>> {
        let sync = self.sync_client.for_key(ConfigKey::SearchNftsLastUpdatedAt).await?;
        let filters = sync.since().map(NftCollectionFilter::UpdatedSince).into_iter().collect();
        let collections = self.repository.get_nft_collections(filters).await?;

        let documents = Self::build_documents(collections);

        sync.write(NFTS_INDEX_NAME, documents).await
    }

    fn build_documents(collections: Vec<NFTCollection>) -> Vec<NFTDocument> {
        collections.into_iter().map(|collection| NFTDocument::from(NFTCollection { links: vec![], ..collection })).collect()
    }
}
