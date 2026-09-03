use std::error::Error;

use async_trait::async_trait;
use gem_client::Client;
use jupiter::client::JupiterClient;
use primitives::Chain;

use crate::{ScanResult, TokenScanProvider, TokenTarget};

pub struct JupiterProvider<C: Client> {
    client: JupiterClient<C>,
}

impl<C: Client> JupiterProvider<C> {
    pub const NAME: &'static str = "Jupiter";

    pub fn new(client: C, api_key: &str) -> Self {
        Self {
            client: JupiterClient::new_with_client_and_api_key(client, api_key.to_string()),
        }
    }
}

#[async_trait]
impl<C: Client> TokenScanProvider for JupiterProvider<C> {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn supports_chain(&self, chain: Chain) -> bool {
        chain == Chain::Solana
    }

    async fn scan_token(&self, target: &TokenTarget) -> Result<ScanResult<TokenTarget>, Box<dyn Error + Send + Sync>> {
        if !self.supports_chain(target.chain) {
            return Err(format!("Unsupported Jupiter token chain: {}", target.chain).into());
        }

        let token = self
            .client
            .get_token(&target.token_id)
            .await?
            .ok_or_else(|| format!("Jupiter token not found: {}", target.token_id))?;
        let is_malicious = token.is_suspicious();

        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason: is_malicious.then(|| "audit.isSus".to_string()),
            provider: Self::NAME.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;

    use super::*;

    #[tokio::test]
    async fn test_scan_token() {
        let target = TokenTarget {
            token_id: "mint".to_string(),
            chain: Chain::Solana,
        };
        let suspicious_client = MockClient::new().with_get(|path| {
            assert_eq!(path, "/tokens/v2/search");
            Ok(br#"[{"id":"mint","audit":{"isSus":false}}]"#.to_vec())
        });
        let suspicious = JupiterProvider::new(suspicious_client, "").scan_token(&target).await.unwrap();

        assert!(suspicious.is_malicious);
        assert_eq!(suspicious.reason.as_deref(), Some("audit.isSus"));
        assert_eq!(suspicious.provider, JupiterProvider::<MockClient>::NAME);

        let safe_client = MockClient::new().with_get(|_| Ok(br#"[{"id":"mint"}]"#.to_vec()));
        let safe = JupiterProvider::new(safe_client, "").scan_token(&target).await.unwrap();

        assert!(!safe.is_malicious);
        assert_eq!(safe.reason, None);

        let missing_client = MockClient::new().with_get(|_| Ok(br#"[]"#.to_vec()));
        let missing_error = JupiterProvider::new(missing_client, "").scan_token(&target).await.unwrap_err();

        assert_eq!(missing_error.to_string(), "Jupiter token not found: mint");

        let unsupported_target = TokenTarget {
            token_id: "mint".to_string(),
            chain: Chain::Ethereum,
        };
        let unsupported_error = JupiterProvider::new(MockClient::new(), "").scan_token(&unsupported_target).await.unwrap_err();

        assert_eq!(unsupported_error.to_string(), "Unsupported Jupiter token chain: ethereum");
    }
}
