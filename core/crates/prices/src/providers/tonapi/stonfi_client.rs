use std::error::Error;

use gem_client::{Client, ClientExt};

use super::stonfi_model::{StonfiAssetsQuery, StonfiAssetsResponse};
use super::target::StonfiTarget;

pub struct StonfiClient<C: Client> {
    client: C,
}

impl<C: Client> StonfiClient<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_assets(&self, limit: usize) -> Result<StonfiAssetsResponse, Box<dyn Error + Send + Sync>> {
        let query = StonfiAssetsQuery {
            condition: "true",
            sort_by: ["popularity_index:desc"],
            limit,
        };
        Ok(self.client.post(StonfiTarget::Assets, &query).await?)
    }
}
