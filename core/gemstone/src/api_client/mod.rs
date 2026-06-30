use crate::alien::{AlienHttpMethod, AlienProvider, AlienTarget};
use gem_client::{CONTENT_TYPE, ContentType};
use primitives::{ScanTransaction, ScanTransactionPayload};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone)]
pub struct GemApiClient {
    api_url: String,
    provider: Arc<dyn AlienProvider>,
}

impl GemApiClient {
    pub fn new(api_url: String, provider: Arc<dyn AlienProvider>) -> Self {
        Self { api_url, provider }
    }

    pub async fn scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, String> {
        let url = format!("{}/v1/scan/transaction", self.api_url);
        let target = AlienTarget {
            url,
            method: AlienHttpMethod::Post,
            headers: Some(HashMap::from([(CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string())])),
            body: Some(serde_json::to_vec(&payload).map_err(|e| e.to_string())?),
        };
        let response = self.provider.request(target).await.map_err(|e| e.to_string())?;
        serde_json::from_slice(&response.data).map_err(|e| format!("Failed to parse response: {}", e))
    }
}
