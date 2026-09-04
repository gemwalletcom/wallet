use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::ResolveDomain;

pub struct UdClient {
    client: ReqwestClient,
}

impl UdClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client }
    }

    pub async fn get_domain(&self, domain: &str) -> Result<ResolveDomain, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(&format!("/resolve/domains/{domain}")).await?)
    }
}
