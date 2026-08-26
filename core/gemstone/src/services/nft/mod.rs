pub mod error;
pub mod store;

use primitives::WalletId;
use std::future::Future;
use std::sync::Arc;

use primitives::{NFTAssetData, NFTAssetId, ReportNft};

pub use error::GemNftError;
pub use store::GemNftStore;

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(uniffi::Object)]
pub struct GemNftService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemNftStore>,
}

#[uniffi::export]
impl GemNftService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemNftStore>) -> Self {
        Self { api, store }
    }

    pub async fn sync(&self, wallet_id: WalletId) -> Result<u32, GemNftError> {
        let data = self.api.client.get_nft_assets(wallet_id.id()).await.map_err(GemApiError::from)?;
        let count = data.len() as u32;
        self.store.save(wallet_id, data).await?;
        Ok(count)
    }

    pub async fn get_or_fetch_asset(&self, asset_id: NFTAssetId) -> Result<NFTAssetData, GemNftError> {
        cached_or_fetched(self.store.as_ref(), asset_id.clone(), async move {
            Ok(self.api.client.get_nft_asset(asset_id).await.map_err(GemApiError::from)?)
        })
        .await
    }

    pub async fn refresh_asset(&self, wallet_id: WalletId, asset_id: NFTAssetId) -> Result<(), GemNftError> {
        self.api.client.refresh_nft_asset(wallet_id.id(), asset_id).await.map_err(GemApiError::from)?;
        Ok(())
    }

    pub async fn report(&self, report: ReportNft) -> Result<(), GemNftError> {
        self.api.client.report_nft(report).await.map_err(GemApiError::from)?;
        Ok(())
    }
}

async fn cached_or_fetched<F>(store: &dyn GemNftStore, asset_id: NFTAssetId, fetch: F) -> Result<NFTAssetData, GemNftError>
where
    F: Future<Output = Result<NFTAssetData, GemNftError>>,
{
    if let Some(data) = store.get_asset_data(asset_id).await? {
        return Ok(data);
    }
    let data = fetch.await?;
    store.save_asset(data.clone()).await?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{Chain, NFTAsset, NFTCollection, NFTCollectionId, NFTImages, NFTResource, NFTType, VerificationStatus};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStore {
        cached: Option<NFTAssetData>,
        added: Mutex<Vec<NFTAssetData>>,
    }

    #[async_trait::async_trait]
    impl GemNftStore for MemoryStore {
        async fn save(&self, _wallet_id: WalletId, _data: Vec<primitives::NFTData>) -> Result<(), GemNftError> {
            Ok(())
        }
        async fn get_asset_data(&self, _asset_id: NFTAssetId) -> Result<Option<NFTAssetData>, GemNftError> {
            Ok(self.cached.clone())
        }
        async fn save_asset(&self, data: NFTAssetData) -> Result<(), GemNftError> {
            self.added.lock().unwrap().push(data);
            Ok(())
        }
    }

    fn asset_data(name: &str) -> NFTAssetData {
        let collection_id = NFTCollectionId::new(Chain::Ethereum, "0xcollection");
        let asset_id = NFTAssetId::new(Chain::Ethereum, "0xcollection", "1");
        let images = NFTImages {
            preview: NFTResource {
                url: "".into(),
                mime_type: "".into(),
            },
        };
        NFTAssetData {
            collection: NFTCollection {
                id: collection_id.clone(),
                name: name.into(),
                symbol: None,
                description: None,
                chain: Chain::Ethereum,
                contract_address: "0xcollection".into(),
                images: images.clone(),
                is_verified: true,
                status: VerificationStatus::Verified,
                links: vec![],
            },
            asset: NFTAsset {
                id: asset_id,
                collection_id,
                contract_address: Some("0xcollection".into()),
                token_id: "1".into(),
                token_type: NFTType::ERC721,
                name: name.into(),
                description: None,
                chain: Chain::Ethereum,
                resource: NFTResource {
                    url: "".into(),
                    mime_type: "".into(),
                },
                images,
                attributes: vec![],
            },
        }
    }

    #[test]
    fn test_cached_asset_skips_fetch() {
        let store = MemoryStore {
            cached: Some(asset_data("cached")),
            ..Default::default()
        };
        let fetched = Mutex::new(false);

        let data = futures::executor::block_on(cached_or_fetched(&store, asset_data("cached").asset.id, async {
            *fetched.lock().unwrap() = true;
            Ok(asset_data("remote"))
        }))
        .unwrap();

        assert_eq!(data.collection.name, "cached");
        assert!(!*fetched.lock().unwrap());
        assert!(store.added.lock().unwrap().is_empty());
    }

    #[test]
    fn test_missing_asset_is_fetched_and_added() {
        let store = MemoryStore::default();

        let data = futures::executor::block_on(cached_or_fetched(&store, asset_data("remote").asset.id, async { Ok(asset_data("remote")) })).unwrap();

        assert_eq!(data.collection.name, "remote");
        assert_eq!(store.added.lock().unwrap().len(), 1);
    }
}
