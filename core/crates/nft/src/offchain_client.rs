use std::error::Error;
use std::net::IpAddr;
use std::time::Duration;

use reqwest::{Client, Url};
use serde::de::DeserializeOwned;

use crate::config::OffchainClientConfig;

#[derive(Debug)]
pub(crate) struct OffchainClient {
    client: Client,
    limit: usize,
}

impl OffchainClient {
    pub(crate) fn new(config: OffchainClientConfig) -> Self {
        let timeout = Duration::from_secs(config.timeout);
        let client = gem_client::builder()
            .timeout(timeout)
            .connect_timeout(timeout)
            .no_proxy()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .build()
            .expect("Failed to build offchain client");
        Self { client, limit: config.limit }
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T, Box<dyn Error + Send + Sync>> {
        let url = Self::validated_url(url)?;
        let mut response = self.client.get(url).send().await?.error_for_status()?;
        if response.content_length().is_some_and(|length| length > self.limit as u64) {
            return Err(format!("Offchain response exceeds {} bytes", self.limit).into());
        }

        let mut body = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if body.len() + chunk.len() > self.limit {
                return Err(format!("Offchain response exceeds {} bytes", self.limit).into());
            }
            body.extend_from_slice(&chunk);
        }
        Ok(serde_json::from_slice(&body)?)
    }

    fn validated_url(url: &str) -> Result<Url, Box<dyn Error + Send + Sync>> {
        let url = Url::parse(url)?;
        if url.scheme() != "https" {
            return Err("Offchain URL must use HTTPS".into());
        }
        let host = url.host_str().ok_or("Offchain URL host is required")?;
        if host.trim_matches(['[', ']']).parse::<IpAddr>().is_ok() {
            return Err("Offchain URL IP address hosts are not allowed".into());
        }
        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_url() {
        assert_eq!(OffchainClient::validated_url("https://example.com/metadata.json").unwrap().host_str(), Some("example.com"));

        for url in ["http://example.com/metadata.json", "https://127.0.0.1/metadata.json", "https://[::1]/metadata.json"] {
            assert!(OffchainClient::validated_url(url).is_err());
        }
    }
}
