use crate::providers::goplus::{
    mapper,
    models::{AccessToken, AccessTokenRequest, Response, SecurityAddress, SecurityToken},
};
use crate::{AddressTarget, ScanProvider, ScanResult, TokenTarget};
use async_trait::async_trait;
use gem_client::{Client, ClientExt, build_path_with_query};
use primitives::{AccessTokenCacher, Chain};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct GoPlusProvider<C: Client> {
    client: C,
    app_key: String,
    app_secret: String,
    access_token_cacher: Option<Arc<dyn AccessTokenCacher>>,
}

impl<C: Client> GoPlusProvider<C> {
    pub const NAME: &'static str = "GoPlus";

    pub fn new(client: C, app_key: &str, app_secret: &str, access_token_cacher: Option<Arc<dyn AccessTokenCacher>>) -> Self {
        Self {
            client,
            app_key: app_key.to_string(),
            app_secret: app_secret.to_string(),
            access_token_cacher,
        }
    }

    fn sign(app_key: &str, time: u64, app_secret: &str) -> String {
        hex::encode(Sha1::digest(format!("{app_key}{time}{app_secret}").as_bytes()))
    }

    async fn access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.access_token_cacher
            .as_ref()
            .ok_or("GoPlus access token cacher is missing")?
            .get_or_refresh(Box::pin(self.refresh_access_token()))
            .await
    }

    async fn refresh_access_token(&self) -> Result<(String, Duration), Box<dyn Error + Send + Sync>> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let request = AccessTokenRequest {
            app_key: self.app_key.clone(),
            sign: Self::sign(&self.app_key, time, &self.app_secret),
            time,
        };
        let response: Response<Option<AccessToken>> = self.client.post("/api/v1/token", &request).await?;
        if response.code != 1 {
            return Err(response.message.into());
        }
        let token = response.result.ok_or("GoPlus access token is missing")?;
        Ok((token.access_token, Duration::from_secs(token.expires_in)))
    }

    async fn headers(&self) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        if self.app_key.is_empty() || self.app_secret.is_empty() {
            return Ok(HashMap::new());
        }
        Ok(HashMap::from([("Authorization".to_string(), self.access_token().await?)]))
    }
}

#[async_trait]
impl<C: Client> ScanProvider for GoPlusProvider<C> {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn supports_address_chain(&self, chain: Chain) -> bool {
        mapper::map_address_chain(chain).is_ok()
    }

    fn supports_token_chain(&self, chain: Chain) -> bool {
        mapper::map_token_chain(chain).is_ok()
    }

    async fn scan_address(&self, target: &AddressTarget) -> Result<ScanResult<AddressTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let path = format!("/api/v1/address_security/{}", target.address);
        let query = vec![("chain_id", mapper::map_address_chain(target.chain)?)];
        let url = build_path_with_query(&path, &query)?;
        let response = self.client.get_with_headers::<Response<Option<SecurityAddress>>>(&url, self.headers().await?).await?;
        if response.code != 1 && response.code != 2 {
            return Err(response.message.into());
        }
        let security = response.result;
        let is_partial = response.code == 2;
        let is_malicious = security.as_ref().is_some_and(SecurityAddress::is_malicious);
        if is_partial && !is_malicious {
            return Err(response.message.into());
        }

        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason: if is_partial {
                Some(response.message)
            } else {
                security.is_none().then(|| "No address data found".to_string())
            },
            provider: self.name().into(),
        })
    }

    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn std::error::Error + Send + Sync>> {
        let path = format!("/api/v1/token_security/{}", mapper::map_token_chain(target.chain)?);
        let token_id = if target.chain == Chain::Tron {
            target.token_id.clone()
        } else {
            target.token_id.to_lowercase()
        };
        let query = vec![("contract_addresses", token_id.as_str())];
        let url = build_path_with_query(&path, &query)?;
        let response = self
            .client
            .get_with_headers::<Response<Option<HashMap<String, SecurityToken>>>>(&url, self.headers().await?)
            .await?;
        if response.code != 1 && response.code != 2 {
            return Err(response.message.into());
        }
        let security_token = response.result.as_ref().and_then(|tokens| tokens.get(&token_id)).cloned();
        let is_partial = response.code == 2;
        let is_malicious = security_token.as_ref().is_some_and(SecurityToken::is_malicious);
        if is_partial && !is_malicious {
            return Err(response.message.into());
        }

        let reason = if is_partial {
            Some(response.message)
        } else if is_malicious {
            Some("Token security risk detected".to_string())
        } else {
            security_token.is_none().then(|| "No token data found".to_string())
        };

        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason,
            provider: self.name().into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[test]
    fn test_sign_access_token_request() {
        assert_eq!(
            GoPlusProvider::<MockClient>::sign("mBOMg20QW11BbtyH4Zh0", 1_647_847_498, "V6aRfxlPJwN3ViJSIFSCdxPvneajuJsh"),
            "7293d385b9225b3c3f232b76ba97255d0e21063e"
        );
    }

    #[tokio::test]
    async fn test_refresh_access_token() {
        let client = MockClient::new().with_post(|path, body| {
            assert_eq!(path, "/api/v1/token");
            let request: serde_json::Value = serde_json::from_slice(body).unwrap();
            let time = request["time"].as_u64().unwrap();
            assert_eq!(request["app_key"], "app_key");
            assert_eq!(request["sign"], GoPlusProvider::<MockClient>::sign("app_key", time, "app_secret"));
            Ok(br#"{"code":1,"message":"OK","result":{"access_token":"token","expires_in":3600}}"#.to_vec())
        });
        let provider = GoPlusProvider::new(client, "app_key", "app_secret", None);

        assert_eq!(provider.refresh_access_token().await.unwrap(), ("token".to_string(), Duration::from_secs(3600)));
    }

    #[tokio::test]
    async fn test_scan_partial_token_without_risk_is_inconclusive() {
        let client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/api/v1/token_security/56?contract_addresses=0xabc");
            Ok(br#"{"code":2,"message":"Partial data obtained","result":{"0xabc":{}}}"#.to_vec())
        });
        let provider = GoPlusProvider::mock(client);

        let result = provider
            .scan_token(&TokenTarget {
                token_id: "0xAbC".to_string(),
                chain: Chain::SmartChain,
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scan_partial_token_with_risk_is_malicious() {
        let client = MockClient::new().with_get(|_| Ok(br#"{"code":2,"message":"Partial data obtained","result":{"0xabc":{"is_honeypot":"1"}}}"#.to_vec()));
        let provider = GoPlusProvider::mock(client);

        let result = provider
            .scan_token(&TokenTarget {
                token_id: "0xAbC".to_string(),
                chain: Chain::SmartChain,
            })
            .await
            .unwrap();

        assert!(result.is_malicious);
    }

    #[tokio::test]
    async fn test_scan_token_without_risks_is_safe() {
        let client = MockClient::new().with_get(|_| Ok(br#"{"code":1,"message":"OK","result":{"0xabc":{}}}"#.to_vec()));
        let provider = GoPlusProvider::mock(client);

        let result = provider
            .scan_token(&TokenTarget {
                token_id: "0xAbC".to_string(),
                chain: Chain::SmartChain,
            })
            .await
            .unwrap();

        assert!(!result.is_malicious);
        assert!(result.reason.is_none());
    }

    #[tokio::test]
    async fn test_scan_token_returns_api_error() {
        let client = MockClient::new().with_get(|_| Ok(br#"{"code":4012,"message":"Wrong Signature","result":null}"#.to_vec()));
        let provider = GoPlusProvider::mock(client);

        let error = provider
            .scan_token(&TokenTarget {
                token_id: "0xAbC".to_string(),
                chain: Chain::SmartChain,
            })
            .await
            .unwrap_err();

        assert_eq!(error.to_string(), "Wrong Signature");
    }
}
