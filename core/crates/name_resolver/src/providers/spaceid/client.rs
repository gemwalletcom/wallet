use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::ResolveRecord;
use super::target::SpaceIdTarget;

pub struct SpaceIdClient {
    client: ReqwestClient,
}

impl SpaceIdClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_address(&self, tld: &str, domain: &str) -> Result<ResolveRecord, Box<dyn Error + Send + Sync>> {
        Ok(self
            .client
            .get(SpaceIdTarget::Address {
                tld: tld.to_string(),
                domain: domain.to_string(),
            })
            .await?)
    }
}
