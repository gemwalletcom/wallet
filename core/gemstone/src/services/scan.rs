use std::sync::Arc;

use primitives::{ScanTransaction, ScanTransactionPayload};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemScanService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemScanService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn scan_transaction(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, GemApiError> {
        Ok(self.api.client.scan_transaction(payload).await?)
    }
}
