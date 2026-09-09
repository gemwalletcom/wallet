use std::error::Error;

use async_trait::async_trait;

use super::model::AssetImage;

#[async_trait]
pub trait ImageProvider: Send + Sync {
    async fn get_asset_images(&self, id: &str) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait ImageListProvider: ImageProvider {
    async fn get_top_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>>;
    async fn get_trending_asset_images(&self) -> Result<Vec<AssetImage>, Box<dyn Error + Send + Sync>>;
}
