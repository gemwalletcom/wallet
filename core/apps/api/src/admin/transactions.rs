use cacher::{CacheKey, CacherClient};
use primitives::{Transaction, TransactionId};
use rocket::serde::json::Json;
use rocket::{State, get, post};
use streamer::{StreamProducer, StreamProducerQueue};

use crate::api_clients::{PermissionAdminWrite, PermissionDeviceTransactionsRead};
use crate::devices::TransactionsClient;
use crate::responders::{ApiError, ApiResponse};

#[get("/transactions/<hash>")]
pub async fn get_transactions_by_hash(
    _permission: PermissionDeviceTransactionsRead,
    hash: &str,
    client: &State<TransactionsClient>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.get_transactions_by_hash(hash)?.into())
}

#[post("/transactions/add", format = "json", data = "<transaction_id>")]
pub async fn add_transaction(
    _permission: PermissionAdminWrite,
    transaction_id: Json<TransactionId>,
    cacher: &State<CacherClient>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<TransactionId>, ApiError> {
    let transaction_id = transaction_id.into_inner();
    let cache_key = CacheKey::FetchTransaction(transaction_id.chain.as_ref(), &transaction_id.hash).key();
    cacher.delete(&cache_key).await?;
    stream_producer.publish_fetch_transactions(vec![transaction_id.clone().into()]).await?;
    Ok(transaction_id.into())
}
