use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use gem_client::{ClientExt, ReqwestClient};

use super::model::AbuseIPDBResponse;
use super::target::AbuseIpDbTarget;
use crate::ip_check_provider::IpCheckProvider;
use crate::model::IpCheckResult;

#[derive(Clone)]
pub struct AbuseIPDBClient {
    client: ReqwestClient,
    api_key: String,
}

impl AbuseIPDBClient {
    pub fn new(url: String, api_key: String) -> Self {
        Self {
            client: ReqwestClient::new(url, gem_client::reqwest_client()),
            api_key,
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([("Key".to_string(), self.api_key.clone()), ("Accept".to_string(), "application/json".to_string())])
    }
}

#[async_trait]
impl IpCheckProvider for AbuseIPDBClient {
    fn name(&self) -> &'static str {
        "abuseipdb"
    }

    async fn check_ip(&self, ip_address: &str) -> Result<IpCheckResult, Box<dyn Error + Send + Sync>> {
        let response: AbuseIPDBResponse = self
            .client
            .get(AbuseIpDbTarget::Check {
                ip_address: ip_address.to_string(),
            })
            .headers(self.headers())
            .await?;

        Ok(response.data.as_ip_check_result())
    }
}
