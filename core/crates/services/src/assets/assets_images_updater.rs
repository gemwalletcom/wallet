use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use primitives::{AssetId, Chain};

use crate::StaticAssetsClient;
use crate::assets::repository::Repository;

pub struct AssetsImagesUpdater {
    client: StaticAssetsClient,
    repository: Arc<dyn Repository>,
}

impl AssetsImagesUpdater {
    pub(crate) fn new(client: StaticAssetsClient, repository: Arc<dyn Repository>) -> Self {
        Self { client, repository }
    }

    pub async fn update_chain(&self, chain: Chain) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        let mut assets = self.client.get_assets_list(chain).await?;
        assets.push(chain.as_asset_id());
        let with_images: HashSet<AssetId> = assets.into_iter().collect();
        Ok(self.repository.update_image_flags(chain, with_images).await?)
    }
}
