pub mod collectible;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::future::Future;
use std::sync::Arc;

use primitives::{NFTAssetData, NFTAssetId, NFTData, ReportNft, WalletId};

pub use collectible::{GemCollectibleLinks, GemCollectibleService};
pub use store::GemNftStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::wallet_session::GemWalletSessionService;

#[derive(uniffi::Object)]
pub struct GemNftService {
    api: Arc<GemDeviceApiClient>,
    store: Arc<dyn GemNftStore>,
    session: Arc<GemWalletSessionService>,
}

#[uniffi::export]
impl GemNftService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, store: Arc<dyn GemNftStore>, session: Arc<GemWalletSessionService>) -> Self {
        Self { api, store, session }
    }

    pub async fn sync(&self) -> Result<u32, GemServiceError> {
        self.sync_wallet(self.session.current_wallet_id()?).await
    }

    pub async fn ensure_asset(&self, asset_id: NFTAssetId) -> Result<NFTAssetData, GemServiceError> {
        cached_or_loaded(self.store.as_ref(), asset_id.clone(), async move {
            Ok(self.api.client.get_nft_asset(asset_id).await.map_err(GemApiError::from)?)
        })
        .await
    }

    pub async fn refresh_asset(&self, wallet_id: WalletId, asset_id: NFTAssetId) -> Result<(), GemServiceError> {
        self.api.client.refresh_nft_asset(wallet_id.id(), asset_id).await.map_err(GemApiError::from)?;
        Ok(())
    }

    pub async fn report(&self, report: ReportNft) -> Result<(), GemServiceError> {
        self.api.client.report_nft(report).await.map_err(GemApiError::from)?;
        Ok(())
    }

    pub fn sorted_collections(&self, data: Vec<NFTData>) -> Vec<NFTData> {
        rules::sorted_collections(data)
    }

    pub fn verified_collections(&self, data: Vec<NFTData>) -> Vec<NFTData> {
        rules::verified_collections(data)
    }

    pub fn unverified_collections(&self, data: Vec<NFTData>) -> Vec<NFTData> {
        rules::unverified_collections(data)
    }
}

async fn cached_or_loaded<F>(store: &dyn GemNftStore, asset_id: NFTAssetId, load: F) -> Result<NFTAssetData, GemServiceError>
where
    F: Future<Output = Result<NFTAssetData, GemServiceError>>,
{
    if let Some(data) = store.get_asset_data(asset_id).await? {
        return Ok(data);
    }
    let data = load.await?;
    store.save_asset(data.clone()).await?;
    Ok(data)
}

impl GemNftService {
    pub async fn sync_wallet(&self, wallet_id: WalletId) -> Result<u32, GemServiceError> {
        let data = self.api.client.get_nft_assets(wallet_id.id()).await.map_err(GemApiError::from)?;
        let count = data.len() as u32;
        self.store.save_nfts(wallet_id, data).await?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::MemoryNftStore;
    use super::*;
    use primitives::{Chain, NFTAsset, NFTCollection, NFTCollectionId, NFTImages, NFTResource, NFTType, VerificationStatus};
    use std::sync::Mutex;

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
    fn test_cached_asset_skips_loading() {
        let store = MemoryNftStore {
            cached: Some(asset_data("cached")),
            ..Default::default()
        };
        let loaded = Mutex::new(false);

        let data = futures::executor::block_on(cached_or_loaded(&store, asset_data("cached").asset.id, async {
            *loaded.lock().unwrap() = true;
            Ok(asset_data("remote"))
        }))
        .unwrap();

        assert_eq!(data.collection.name, "cached");
        assert!(!*loaded.lock().unwrap());
        assert!(store.added.lock().unwrap().is_empty());
    }

    #[test]
    fn test_missing_asset_is_loaded_and_added() {
        let store = MemoryNftStore::default();

        let data = futures::executor::block_on(cached_or_loaded(&store, asset_data("remote").asset.id, async { Ok(asset_data("remote")) })).unwrap();

        assert_eq!(data.collection.name, "remote");
        assert_eq!(store.added.lock().unwrap().len(), 1);
    }
}
