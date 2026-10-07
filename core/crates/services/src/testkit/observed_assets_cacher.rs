use std::{error::Error, io};

use async_trait::async_trait;
use cacher::ObservedAssetsCacher;
use primitives::AssetId;

pub(crate) struct UnusedObservedCacher;

#[async_trait]
impl ObservedAssetsCacher for UnusedObservedCacher {
    async fn track_observed_assets(&self, _asset_ids: &[AssetId]) -> Result<(), Box<dyn Error + Send + Sync>> {
        Err(io::Error::other("unexpected observed assets write").into())
    }

    async fn observed_assets(&self, _min_observers: usize, _limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Err(io::Error::other("unexpected observed assets read").into())
    }
}
