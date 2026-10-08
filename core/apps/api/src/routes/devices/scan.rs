use std::sync::Arc;

use axum::extract::State;
use primitives::{ScanTransaction, ScanTransactionPayload};
use services::security::ScanClient;

use crate::auth::device::{AuthenticatedDevice, DeviceJson};
use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn scan_transaction(_device: AuthenticatedDevice, State(client): State<Arc<ScanClient>>, request: DeviceJson<ScanTransactionPayload>) -> Result<ApiResponse<ScanTransaction>, ApiError> {
    Ok(client.get_scan_transaction(request.into_inner()).await?.into())
}
