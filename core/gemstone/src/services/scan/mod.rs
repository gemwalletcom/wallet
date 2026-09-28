pub mod rules;

use std::sync::Arc;

use primitives::{ScanTransaction, ScanTransactionPayload};

use crate::api::GemDeviceApiClient;

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
    pub async fn scan(&self, payload: ScanTransactionPayload) -> Option<ScanTransaction> {
        self.api.client.scan_transaction(payload).await.ok()
    }
}
