use std::sync::Arc;

use axum::extract::State;
use primitives::{Transaction, TransactionsResponse};
use serde::Deserialize;
use services::transactions::TransactionsClient;

use crate::auth::device::{AuthenticatedDevice, AuthenticatedDeviceWallet};
use crate::error::ApiError;
use crate::request::{AssetIdParam, Path, Query, QueryLimitParam, TransactionIdParam, lenient};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct TransactionsQuery {
    #[serde(default, deserialize_with = "lenient")]
    asset_id: Option<AssetIdParam>,
    #[serde(default, deserialize_with = "lenient")]
    from_timestamp: Option<u64>,
    #[serde(default)]
    limit: QueryLimitParam,
    #[serde(default, deserialize_with = "lenient")]
    offset: Option<usize>,
}

pub async fn get_transactions(device: AuthenticatedDeviceWallet, Query(query): Query<TransactionsQuery>, State(client): State<Arc<TransactionsClient>>) -> Result<ApiResponse<TransactionsResponse>, ApiError> {
    Ok(client
        .get_transactions_by_wallet_id(
            &device.record.device.id,
            device.record.id,
            device.wallet_id,
            query.asset_id.map(|asset_id| asset_id.0),
            query.from_timestamp,
            query.limit.0,
            query.offset.unwrap_or_default(),
        )
        .await?
        .into())
}

pub async fn get_transaction_by_wallet(device: AuthenticatedDeviceWallet, Path(id): Path<TransactionIdParam>, State(client): State<Arc<TransactionsClient>>) -> Result<ApiResponse<Transaction>, ApiError> {
    Ok(client.get_transaction_by_wallet_id(device.record.id, device.wallet_id, &id.0).await?.into())
}

pub async fn get_transaction(_device: AuthenticatedDevice, Path(id): Path<TransactionIdParam>, State(client): State<Arc<TransactionsClient>>) -> Result<ApiResponse<Transaction>, ApiError> {
    Ok(client.get_transaction_by_id(&id.0).await?.into())
}
