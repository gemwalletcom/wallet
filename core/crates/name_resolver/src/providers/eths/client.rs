use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::ResolveRecord;

pub struct EthsClient {
    client: ReqwestClient,
}

impl EthsClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_record(&self, domain: &str) -> Result<ResolveRecord, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/resolve/{domain}")).await?)
    }
}
