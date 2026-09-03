use crate::providers::hashdit::{
    auth::signed_headers,
    mapper,
    models::{DetectRequest, DetectResponse},
    target::HashDitTarget,
};
use crate::{AddressScanProvider, AddressTarget, ScanResult, TokenScanProvider, TokenTarget};
use async_trait::async_trait;
use gem_client::{Client, ClientError, ClientExt, Target};
use primitives::Chain;
use std::time::{SystemTime, UNIX_EPOCH};

const PROVIDER_NAME: &str = "HashDit";
const ADDRESS_DETECTION: &str = "gem_wallet_address_detection";
const TOKEN_DETECTION: &str = "gem_wallet_token_detection";

pub struct HashDitProvider<C: Client> {
    client: C,
    app_id: String,
    app_secret: String,
}

impl<C: Client> HashDitProvider<C> {
    pub fn new(client: C, app_id: &str, app_secret: &str) -> Self {
        HashDitProvider {
            client,
            app_id: app_id.to_string(),
            app_secret: app_secret.to_string(),
        }
    }

    async fn detect(&self, business: &'static str, body: &DetectRequest) -> Result<DetectResponse, ClientError> {
        let target = HashDitTarget::Detect { business };
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis().to_string();
        let nonce = uuid::Uuid::new_v4().to_string().replace('-', "");
        let headers = signed_headers(&self.app_id, &self.app_secret, &timestamp, &nonce, "POST", &target.path(), &serde_json::to_string(body)?);
        self.client.post(target, body).headers(headers).await
    }

    fn parse_response(response: DetectResponse) -> Result<(bool, Option<String>), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(error_data) = response.error_data {
            return Err(Box::from(error_data));
        }

        let mut is_malicious = false;
        let mut reason: Option<String> = None;

        if let Some(data) = response.data {
            let has_result = data.has_result.unwrap_or_else(|| data.risk_level.is_some());
            if has_result {
                let level = data.risk_level.unwrap_or(0);
                // 3 - Medium Risk
                is_malicious = level >= 3;
                reason = Some(format!("Risk level: {}", level));
            } else {
                is_malicious = false;
                reason = Some("No data found".to_string());
            }
        }

        Ok((is_malicious, reason))
    }

    async fn scan<T: Clone + Send + Sync + 'static>(
        &self,
        target: &T,
        business: &'static str,
        body: &DetectRequest,
    ) -> Result<ScanResult<T>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.detect(business, body).await?;
        let (is_malicious, reason) = Self::parse_response(response)?;
        Ok(ScanResult {
            target: target.clone(),
            is_malicious,
            reason,
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
        let body = DetectRequest {
            chain_id: mapper::map_chain(target.chain)?,
            address: target.address.clone(),
        };
        self.scan(target, ADDRESS_DETECTION, &body).await
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
        let body = DetectRequest {
            chain_id: mapper::map_chain(target.chain)?,
            address: target.token_id.clone(),
        };
        self.scan(target, TOKEN_DETECTION, &body).await
    }
}
