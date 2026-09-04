use std::error::Error;

use async_trait::async_trait;
use gem_client::{ClientExt, ReqwestClient};

use super::model::IpApiResponse;
use super::target::IpApiTarget;
use crate::ip_check_provider::IpCheckProvider;
use crate::model::IpCheckResult;

#[derive(Clone)]
pub struct IpApiClient {
    client: ReqwestClient,
    api_key: String,
}

impl IpApiClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            client: ReqwestClient::new(url, gem_client::reqwest_client()),
            api_key,
        }
    }
}

#[async_trait]
impl IpCheckProvider for IpApiClient {
    fn name(&self) -> &'static str {
        "ipapi"
    }

    async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let response: IpApiResponse = self
            .client
            .get(IpApiTarget::Check {
                ip_address: ip_address.to_string(),
            })
            .query(&[("key", self.api_key.as_str())])
            .await?;

        Ok(response.as_ip_check_result())
    }
}
