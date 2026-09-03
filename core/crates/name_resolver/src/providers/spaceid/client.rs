use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::ResolveRecord;

pub struct SpaceIdClient {
    client: ReqwestClient,
}

impl SpaceIdClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_address(&self, tld: &str, domain: &str) -> Result<ResolveRecord, Box<dyn Error + Send + Sync>> {
        let query = [("tld".to_string(), tld.to_string()), ("domain".to_string(), domain.to_string())];
        Ok(self.client.get("/v1/getAddress").query(&query).await?)
    }
}
