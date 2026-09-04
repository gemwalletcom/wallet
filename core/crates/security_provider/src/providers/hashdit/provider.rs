use std::collections::HashMap;

use async_trait::async_trait;
use gem_client::{Client, ClientExt};
use primitives::Chain;

use crate::providers::hashdit::{
    mapper,
    models::{SecurityData, SecurityRequest, SecurityResponse},
    target::HashDitTarget,
};
use crate::{AddressScanProvider, AddressTarget, ScanResult, TokenScanProvider, TokenTarget};

const PROVIDER_NAME: &str = "HashDit";
const X_API_KEY: &str = "X-API-KEY";

pub struct HashDitProvider<C: Client> {
    client: C,
    api_key: String,
}

impl<C: Client> HashDitProvider<C> {
    pub fn new(client: C, api_key: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([(X_API_KEY.to_string(), self.api_key.clone())])
    }

    async fn security(&self, target: HashDitTarget, body: &SecurityRequest) -> Result<SecurityData, Box<dyn std::error::Error + Send + Sync>> {
        let response: SecurityResponse = self.client.post(target, body).headers(self.headers()).await?;
        Ok(response.data)
    }

    async fn scan<T: Clone + Send + Sync + 'static>(
        &self,
        target: &T,
        request_target: HashDitTarget,
        body: &SecurityRequest,
    ) -> Result<ScanResult<T>, Box<dyn std::error::Error + Send + Sync>> {
        let risk_level = self.security(request_target, body).await?.overall_risk_level;
        let is_malicious = risk_level.is_malicious();
        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason: Some(risk_level.as_str().to_string()),
            provider: PROVIDER_NAME.into(),
        })
    }
}

#[async_trait]
impl<C: Client> AddressScanProvider for HashDitProvider<C> {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        mapper::map_chain(chain).is_ok()
    }

    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let body = SecurityRequest {
            chain_id: mapper::map_chain(target.chain)?,
            address: target.address.clone(),
            sync: true,
        };
        self.scan(target, HashDitTarget::AddressSecurity, &body).await
    }
}

#[async_trait]
impl<C: Client> TokenScanProvider for HashDitProvider<C> {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        mapper::map_chain(chain).is_ok()
    }

    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let body = SecurityRequest {
            chain_id: mapper::map_chain(target.chain)?,
            address: target.token_id.clone(),
            sync: true,
        };
        self.scan(target, HashDitTarget::TokenSecurity, &body).await
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_scan_address() {
        let client = MockClient::new().with_post_with_headers(|path, body, headers| {
            assert_eq!(path, "/v2/hashdit/address-security-v2");
            assert_eq!(body, br#"{"chainId":"56","address":"0x123","sync":true}"#);
            assert_eq!(headers.get(X_API_KEY).map(String::as_str), Some("api-key"));
            Ok(br#"{"code":"0","status":"ok","data":{"overall_risk_level":"High Risk"}}"#.to_vec())
        });
        let target = AddressTarget {
            chain: Chain::SmartChain,
            address: "0x123".to_string(),
        };
        let result = HashDitProvider::new(client, "api-key").scan_address(&target).await.unwrap();

        assert!(result.is_malicious);
        assert_eq!(result.reason.as_deref(), Some("High Risk"));
    }

    #[tokio::test]
    async fn test_scan_token() {
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/v2/hashdit/token-security");
            assert_eq!(body, br#"{"chainId":"56","address":"0x456","sync":true}"#);
            Ok(br#"{"code":"0","status":"ok","data":{"overall_risk_level":"No Obvious Risk"}}"#.to_vec())
        });
        let target = TokenTarget {
            chain: Chain::SmartChain,
            token_id: "0x456".to_string(),
        };
        let result = HashDitProvider::new(client, "api-key").scan_token(&target).await.unwrap();

        assert!(!result.is_malicious);
    }
}
