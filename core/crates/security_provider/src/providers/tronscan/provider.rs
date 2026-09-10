use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use gem_client::{Client, ClientExt};
use primitives::Chain;

use super::models::{AddressSecurity, TokenSecurity};
use super::target::TronscanTarget;
use crate::{AddressScanProvider, AddressTarget, ScanResult, TokenScanProvider, TokenTarget};

pub struct TronscanProvider<C: Client> {
    client: C,
    api_key: String,
}

impl<C: Client> TronscanProvider<C> {
    pub const NAME: &'static str = "Tronscan";

    pub fn new(client: C, api_key: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([("TRON-PRO-API-KEY".to_string(), self.api_key.clone())])
    }
}

#[async_trait]
impl<C: Client> AddressScanProvider for TronscanProvider<C> {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        chain == Chain::Tron
    }

    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn Error + Send + Sync>> {
        let security: AddressSecurity = self
            .client
            .get(TronscanTarget::AddressSecurity { address: target.address.clone() })
            .headers(self.headers())
            .await?;
        let reason = security.malicious_reason();

        Ok(ScanResult {
            target: target.clone(),
            is_malicious: reason.is_some(),
            reason,
            provider: Self::NAME.into(),
        })
    }
}

#[async_trait]
impl<C: Client> TokenScanProvider for TronscanProvider<C> {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        chain == Chain::Tron
    }

    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn Error + Send + Sync>> {
        let security: TokenSecurity = self
            .client
            .get(TronscanTarget::TokenSecurity { address: target.token_id.clone() })
            .headers(self.headers())
            .await?;
        let reason = security.malicious_reason()?;

        Ok(ScanResult {
            target: target.clone(),
            is_malicious: reason.is_some(),
            reason,
            provider: Self::NAME.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use gem_client::{ClientError, testkit::MockClient};

    use super::*;

    const TEST_ADDRESS: &str = "TT2T17KZhoDu47i2E4FWxfG79zdkEWkU9N";
    const TEST_TOKEN: &str = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t";

    #[tokio::test]
    async fn test_scan_address() {
        let client = MockClient::new().with_get_with_headers(|path, headers| {
            assert_eq!(path, format!("/api/security/account/data?address={TEST_ADDRESS}"));
            assert_eq!(headers.get("TRON-PRO-API-KEY").map(String::as_str), Some("test-key"));
            Ok(br#"{"send_ad_by_memo":false,"has_fraud_transaction":true,"fraud_token_creator":false,"is_black_list":false}"#.to_vec())
        });
        let target = AddressTarget {
            address: TEST_ADDRESS.to_string(),
            chain: Chain::Tron,
        };
        let result = TronscanProvider::new(client, "test-key").scan_address(&target).await.unwrap();
        assert_eq!(result.target, target);
        assert!(result.is_malicious);
        assert_eq!(result.reason.as_deref(), Some("has_fraud_transaction"));
        assert_eq!(result.provider, "Tronscan");
    }

    #[tokio::test]
    async fn test_scan_token() {
        let client = MockClient::new().with_get_with_headers(|path, headers| {
            assert_eq!(path, format!("/api/security/token/data?address={TEST_TOKEN}"));
            assert_eq!(headers.get("TRON-PRO-API-KEY").map(String::as_str), Some("test-key"));
            Ok(br#"{"token_level":"4"}"#.to_vec())
        });
        let target = TokenTarget {
            token_id: TEST_TOKEN.to_string(),
            chain: Chain::Tron,
        };
        let result = TronscanProvider::new(client, "test-key").scan_token(&target).await.unwrap();
        assert_eq!(result.target, target);
        assert!(result.is_malicious);
        assert_eq!(result.reason.as_deref(), Some("token_level:unsafe"));
        assert_eq!(result.provider, "Tronscan");
    }

    #[tokio::test]
    async fn test_trc20_blacklist_capability_is_not_malicious() {
        let client = MockClient::new().with_get(|path| {
            if path == format!("/api/security/account/data?address={TEST_TOKEN}") {
                Ok(br#"{"send_ad_by_memo":false,"has_fraud_transaction":false,"fraud_token_creator":false,"is_black_list":false}"#.to_vec())
            } else {
                assert_eq!(path, format!("/api/security/token/data?address={TEST_TOKEN}"));
                Ok(br#"{"token_level":"2","black_list_type":1,"increase_total_supply":1}"#.to_vec())
            }
        });
        let provider = TronscanProvider::new(client, "test-key");
        let address = provider
            .scan_address(&AddressTarget {
                address: TEST_TOKEN.into(),
                chain: Chain::Tron,
            })
            .await
            .unwrap();
        let token = provider
            .scan_token(&TokenTarget {
                token_id: TEST_TOKEN.into(),
                chain: Chain::Tron,
            })
            .await
            .unwrap();
        assert_eq!((address.is_malicious, address.reason), (false, None));
        assert_eq!((token.is_malicious, token.reason), (false, None));
    }

    #[tokio::test]
    async fn test_scan_failures_are_not_successful_verdicts() {
        let address = AddressTarget {
            address: TEST_ADDRESS.into(),
            chain: Chain::Tron,
        };
        let token = TokenTarget {
            token_id: TEST_TOKEN.into(),
            chain: Chain::Tron,
        };
        for status in [401, 429, 503] {
            let client = MockClient::new().with_get(move |_| Err(ClientError::Http { status, body: vec![] }));
            let provider = TronscanProvider::new(client, "test-key");
            assert!(provider.scan_address(&address).await.is_err());
            assert!(provider.scan_token(&token).await.is_err());
        }
        for response in ["{}", "null", r#"{"message":"invalid api key"}"#] {
            let client = MockClient::new().with_get(move |_| Ok(response.as_bytes().to_vec()));
            let provider = TronscanProvider::new(client, "test-key");
            assert!(provider.scan_address(&address).await.is_err());
            assert!(provider.scan_token(&token).await.is_err());
        }
        let client = MockClient::new().with_get(|_| Ok(br#"{"token_level":"0"}"#.to_vec()));
        assert!(TronscanProvider::new(client, "test-key").scan_token(&token).await.is_err());
    }
}

#[cfg(all(test, feature = "security_integration_tests"))]
mod integration_tests {
    use std::env;

    use gem_client::ReqwestClient;

    use super::*;

    #[tokio::test]
    async fn test_tronscan_security() -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = env::var("TRONSCAN_API_KEY")?;
        let provider = TronscanProvider::new(ReqwestClient::new("https://apilist.tronscanapi.com".into(), gem_client::builder().build()?), &key);
        let address = AddressTarget {
            address: "TT2T17KZhoDu47i2E4FWxfG79zdkEWkU9N".into(),
            chain: Chain::Tron,
        };
        assert_eq!(provider.scan_address(&address).await?.target, address);
        let token = TokenTarget {
            token_id: "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".into(),
            chain: Chain::Tron,
        };
        assert_eq!(provider.scan_token(&token).await?.target, token);
        Ok(())
    }
}
