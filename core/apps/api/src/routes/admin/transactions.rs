use std::sync::Arc;

use axum::extract::State;
use primitives::{Transaction, TransactionId, TransactionIdRequest};
use services::indexer::IndexerClient;
use services::transactions::TransactionsClient;

use crate::auth::api_client::{AdminWrite, DeviceTransactionsRead};
use crate::error::ApiError;
use crate::request::{Json, Path};
use crate::response::ApiResponse;

pub async fn get_transactions_by_hash(_permission: DeviceTransactionsRead, Path(hash): Path<String>, State(client): State<Arc<TransactionsClient>>) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.get_transactions_by_hash(&hash).await?.into())
}

pub async fn add_transaction(_permission: AdminWrite, State(client): State<Arc<IndexerClient>>, Json(request): Json<TransactionIdRequest>) -> Result<ApiResponse<TransactionId>, ApiError> {
    let transaction_id = TransactionId::new(request.chain, request.hash.clone());
    client.refresh_transaction(request).await?;
    Ok(transaction_id.into())
}
