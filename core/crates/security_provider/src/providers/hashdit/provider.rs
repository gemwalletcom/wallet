use std::collections::HashMap;

use async_trait::async_trait;
use gem_client::{Client, ClientExt};
use primitives::Chain;
use serde::Serialize;

use crate::providers::hashdit::{
    mapper,
    models::{
        AddressPoisoningRequest, AddressPoisoningResponse, DomainSecurityRequest, DomainSecurityResponse, SecurityData, SecurityRequest, SecurityResponse,
        SolanaTokenSecurityRequest,
    },
    target::HashDitTarget,
};
use crate::{AddressPoisoningProvider, AddressPoisoningTarget, AddressScanProvider, AddressTarget, ScanResult, TokenScanProvider, TokenTarget, WebsiteScanProvider, WebsiteTarget};

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

    async fn security<B: Serialize + Send + Sync>(&self, target: HashDitTarget, body: &B) -> Result<SecurityData, Box<dyn std::error::Error + Send + Sync>> {
        let response: SecurityResponse = self.client.post(target, body).headers(self.headers()).await?;
        Ok(response.into_data()?)
    }

    async fn scan<T: Clone + Send + Sync, B: Serialize + Send + Sync>(
        &self,
        target: &T,
        request_target: HashDitTarget,
        body: &B,
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
impl<C: Client> AddressPoisoningProvider for HashDitProvider<C> {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        mapper::map_poisoning_chain(chain).is_ok()
    }

    async fn scan_address_poisoning(&self, target: &AddressPoisoningTarget) -> Result<ScanResult<AddressPoisoningTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let body = AddressPoisoningRequest {
            chain_id: mapper::map_poisoning_chain(target.target.chain)?,
            address: target.target.address.clone(),
            user_address: target.user_address.clone(),
        };
        let response: AddressPoisoningResponse = self.client.post(HashDitTarget::AddressPoisoning, &body).headers(self.headers()).await?;
        let is_malicious = response.data.target_address.is_poisoning.is_risk();
        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason: is_malicious.then(|| "is_poisoning".to_string()),
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
        match chain {
            Chain::Solana => true,
            chain => mapper::map_chain(chain).is_ok(),
        }
    }

    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn std::error::Error + Send + Sync>> {
        match target.chain {
            Chain::Solana => {
                let body = SolanaTokenSecurityRequest {
                    address: target.token_id.clone(),
                    sync: true,
                };
                self.scan(target, HashDitTarget::SolanaTokenSecurity, &body).await
            }
            chain => {
                let body = SecurityRequest {
                    chain_id: mapper::map_chain(chain)?,
                    address: target.token_id.clone(),
                    sync: true,
                };
                self.scan(target, HashDitTarget::TokenSecurity, &body).await
            }
        }
    }
}

#[async_trait]
impl<C: Client> WebsiteScanProvider for HashDitProvider<C> {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn scan_website(&self, target: &WebsiteTarget) -> Result<ScanResult<WebsiteTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let body = DomainSecurityRequest { url: target.website.clone() };
        let response: DomainSecurityResponse = self.client.post(HashDitTarget::DomainSecurity, &body).headers(self.headers()).await?;
        let is_malicious = response.data.is_malicious()?;
        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason: Some(format!("risk_level={}", response.data.risk_level)),
            provider: PROVIDER_NAME.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use serde_json::Value;

    use super::*;

    fn assert_json(actual: &[u8], expected: &str) {
        assert_eq!(serde_json::from_slice::<Value>(actual).unwrap(), serde_json::from_str::<Value>(expected).unwrap());
    }

    #[tokio::test]
    async fn test_scan_address() {
        let client = MockClient::new().with_post_with_headers(|path, body, headers| {
            assert_eq!(path, "/v2/hashdit/address-security-v2");
            assert_json(body, include_str!("../../../testdata/hashdit/address_security_request.json"));
            assert_eq!(headers.get(X_API_KEY).map(String::as_str), Some("api-key"));
            Ok(include_str!("../../../testdata/hashdit/address_security_high_risk_response.json").as_bytes().to_vec())
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
    async fn test_scan_address_significant_risk() {
        let client = MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/v2/hashdit/address-security-v2");
            let request: Value = serde_json::from_slice(body).unwrap();
            assert_eq!(request["chainId"], "56");
            assert_eq!(request["address"], "0x0f9adaaccd7caecc5019194e15ad19624fed95fa");
            assert_eq!(request["sync"], true);
            Ok(include_str!("../../../testdata/hashdit/address_security_significant_risk_response.json")
                .as_bytes()
                .to_vec())
        });
        let target = AddressTarget {
            chain: Chain::SmartChain,
            address: "0x0f9adaaccd7caecc5019194e15ad19624fed95fa".to_string(),
        };
        let result = HashDitProvider::new(client, "api-key").scan_address(&target).await.unwrap();

        assert!(result.is_malicious);
        assert_eq!(result.target, target);
        assert_eq!(result.reason.as_deref(), Some("Significant Risk"));
    }

    #[tokio::test]
    async fn test_scan_address_in_progress() {
        let client = MockClient::new().with_post(|_, _| Ok(include_str!("../../../testdata/hashdit/security_in_progress_response.json").as_bytes().to_vec()));
        let target = AddressTarget {
            chain: Chain::SmartChain,
            address: "0x0f9adaaccd7caecc5019194e15ad19624fed95fa".to_string(),
        };
        let error = HashDitProvider::new(client, "api-key").scan_address(&target).await.unwrap_err();

        assert_eq!(error.to_string(), "HashDit scan is in progress; retry after 10 seconds");
    }

    #[tokio::test]
    async fn test_scan_token() {
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/v2/hashdit/token-security");
            assert_json(body, include_str!("../../../testdata/hashdit/token_security_request.json"));
            Ok(include_str!("../../../testdata/hashdit/token_security_safe_response.json").as_bytes().to_vec())
        });
        let target = TokenTarget {
            chain: Chain::SmartChain,
            token_id: "0x456".to_string(),
        };
        let result = HashDitProvider::new(client, "api-key").scan_token(&target).await.unwrap();

        assert!(!result.is_malicious);
    }

    #[tokio::test]
    async fn test_scan_solana_token() {
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/v2/hashdit/solana-token-security");
            assert_json(body, include_str!("../../../testdata/hashdit/solana_token_security_request.json"));
            Ok(include_str!("../../../testdata/hashdit/solana_token_security_medium_risk_response.json")
                .as_bytes()
                .to_vec())
        });
        let target = TokenTarget {
            chain: Chain::Solana,
            token_id: "mint".to_string(),
        };
        let result = HashDitProvider::new(client, "api-key").scan_token(&target).await.unwrap();

        assert!(result.is_malicious);
        assert_eq!(result.reason.as_deref(), Some("Medium Risk"));
    }

    #[tokio::test]
    async fn test_scan_address_poisoning() {
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/v2/hashdit/address-poisoning");
            assert_json(body, include_str!("../../../testdata/hashdit/address_poisoning_request.json"));
            Ok(include_str!("../../../testdata/hashdit/address_poisoning_response.json").as_bytes().to_vec())
        });
        let target = AddressPoisoningTarget {
            target: AddressTarget {
                chain: Chain::Tron,
                address: "recipient".to_string(),
            },
            user_address: "sender".to_string(),
        };
        let result = HashDitProvider::new(client, "api-key").scan_address_poisoning(&target).await.unwrap();

        assert!(result.is_malicious);
        assert_eq!(result.reason.as_deref(), Some("is_poisoning"));
    }

    #[tokio::test]
    async fn test_scan_website() {
        let client = MockClient::new().with_post(move |path, body| {
            assert_eq!(path, "/v2/hashdit/domain-security");
            assert_json(body, include_str!("../../../testdata/hashdit/domain_security_request.json"));
            Ok(include_str!("../../../testdata/hashdit/domain_security_malicious_response.json").as_bytes().to_vec())
        });
        let target = WebsiteTarget {
            website: "https://malicious.example".to_string(),
        };
        let result = HashDitProvider::new(client, "api-key").scan_website(&target).await.unwrap();

        assert!(result.is_malicious);
        assert_eq!(result.reason.as_deref(), Some("risk_level=3"));
    }
}
