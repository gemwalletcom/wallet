pub mod rules;

use std::sync::Arc;

use primitives::{ScanTransaction, ScanTransactionPayload};

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::error::GemServiceError;

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
}

impl GemScanService {
    pub async fn scan(&self, payload: ScanTransactionPayload) -> Result<ScanTransaction, GemServiceError> {
        self.api.client.scan_transaction(payload).await.map_err(|error| GemServiceError::from(GemApiError::from(error)))
    }
}
