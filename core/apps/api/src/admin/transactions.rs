use primitives::{Chain, SwapProvider, Transaction, TransactionId, TransactionState};
use rocket::serde::json::Json;
use rocket::{State, get, post};
use serde::Deserialize;
use services::indexer::IndexerClient;
use services::transactions::TransactionsClient;
use swapper::swapper::GemSwapper;

use crate::api_clients::{PermissionAdminWrite, PermissionDeviceTransactionsRead};
use crate::responders::{ApiError, ApiResponse};

#[get("/transactions/<hash>")]
pub async fn get_transactions_by_hash(_permission: PermissionDeviceTransactionsRead, hash: &str, client: &State<TransactionsClient>) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.get_transactions_by_hash(hash).await?.into())
}

#[derive(Debug, Deserialize)]
pub struct SwapTransactionRequest {
    chain: Chain,
    hash: String,
    provider: SwapProvider,
}

#[post("/transactions/swap", format = "json", data = "<request>")]
pub async fn update_swap_transaction(_permission: PermissionAdminWrite, request: Json<SwapTransactionRequest>, swapper: &State<GemSwapper>, client: &State<IndexerClient>) -> Result<ApiResponse<Option<TransactionState>>, ApiError> {
    let request = request.into_inner();
    let result = swapper.get_swap_result(request.chain, request.provider, &request.hash).await?;
    Ok(client.update_swap_transaction(TransactionId::new(request.chain, request.hash), result).await?.into())
}

#[post("/transactions/add", format = "json", data = "<transaction_id>")]
pub async fn add_transaction(_permission: PermissionAdminWrite, transaction_id: Json<TransactionId>, client: &State<IndexerClient>) -> Result<ApiResponse<TransactionId>, ApiError> {
    let transaction_id = transaction_id.into_inner();
    client.refresh_transaction(transaction_id.clone()).await?;
    Ok(transaction_id.into())
}
