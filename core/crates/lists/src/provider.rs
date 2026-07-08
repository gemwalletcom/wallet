use std::error::Error;

use async_trait::async_trait;
use primitives::{AssetId, ListProviderName};

pub struct ListProviderData {
    pub name: String,
    pub asset_ids: Vec<AssetId>,
}

#[async_trait]
pub trait ListProvider: Send + Sync {
    fn provider(&self) -> ListProviderName;
    async fn get_list(&self, provider_list_id: &str) -> Result<Option<ListProviderData>, Box<dyn Error + Send + Sync>>;
}
