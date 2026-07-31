use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::stonfi_model::{StonfiAssetsQuery, StonfiAssetsResponse};

pub struct StonfiClient {
    client: ReqwestClient,
}

impl StonfiClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_assets(&self, limit: usize) -> Result<StonfiAssetsResponse, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .post(
                "/v1/assets/query",
                &StonfiAssetsQuery {
                    condition: "true",
                    sort_by: ["popularity_index:desc"],
                    limit,
                },
            )
            .await?)
    }
}
