use std::error::Error;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::providers::MoonPayClient;

#[derive(Clone)]
pub struct IPCheckClient {
    client: MoonPayClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPAddressInfo {
    pub alpha2: String,
    pub state: String,
    pub ip_address: String,
}

#[async_trait]
pub trait IpAddressProvider: Send + Sync {
    async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>>;
}

impl IPCheckClient {
    pub fn new(client: MoonPayClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl IpAddressProvider for IPCheckClient {
    async fn get_ip_address(&self, ip_address: &str) -> Result<IPAddressInfo, Box<dyn Error + Send + Sync>> {
        let data = self.client.get_ip_address(ip_address).await?;
        Ok(IPAddressInfo {
            alpha2: data.alpha2,
            state: data.state,
            ip_address: ip_address.to_owned(),
        })
    }
}
