use primitives::{Transaction, TransactionId};
use rocket::serde::json::Json;
use rocket::{State, get, post};
use services::indexer::IndexerClient;
use services::transactions::TransactionsClient;

use crate::api_clients::{PermissionAdminWrite, PermissionDeviceTransactionsRead};
use crate::responders::{ApiError, ApiResponse};

#[get("/transactions/<hash>")]
pub async fn get_transactions_by_hash(_permission: PermissionDeviceTransactionsRead, hash: &str, client: &State<TransactionsClient>) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.get_transactions_by_hash(hash).await?.into())
}

#[post("/transactions/add", format = "json", data = "<transaction_id>")]
pub async fn add_transaction(_permission: PermissionAdminWrite, transaction_id: Json<TransactionId>, client: &State<IndexerClient>) -> Result<ApiResponse<TransactionId>, ApiError> {
    let transaction_id = transaction_id.into_inner();
    client.refresh_transaction(transaction_id.clone()).await?;
    Ok(transaction_id.into())
}
