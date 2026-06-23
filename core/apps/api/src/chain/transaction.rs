use chrono::{DateTime, Utc};
use rocket::{State, get, tokio::sync::Mutex};

use crate::api_clients::PermissionChainRead;
use crate::params::ChainParam;
use crate::responders::{ApiError, ApiResponse};
use primitives::{Transaction, TransactionStateRequest, TransactionUpdate};

use super::ChainClient;

#[get("/chain/transactions/<chain>/<hash>")]
pub async fn get_transaction(
    _permission: PermissionChainRead,
    chain: ChainParam,
    hash: &str,
    client: &State<Mutex<ChainClient>>,
) -> Result<ApiResponse<Option<Transaction>>, ApiError> {
    Ok(client.lock().await.get_transaction_by_hash(chain.0, hash.to_string()).await?.into())
}

#[get("/chain/transactions/<chain>/<hash>/status?<sender_address>&<created_at>&<from_timestamp>&<block_number>")]
pub async fn get_transaction_status(
    _permission: PermissionChainRead,
    chain: ChainParam,
    hash: &str,
    sender_address: Option<String>,
    created_at: Option<u64>,
    from_timestamp: Option<u64>,
    block_number: Option<u64>,
    client: &State<Mutex<ChainClient>>,
) -> Result<ApiResponse<TransactionUpdate>, ApiError> {
    let created_at = created_at
        .or(from_timestamp)
        .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp as i64, 0))
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let request = TransactionStateRequest {
        id: hash.to_string(),
        sender_address: sender_address.unwrap_or_default(),
        created_at,
        block_number: block_number.unwrap_or_default(),
    };
    Ok(client.lock().await.get_transaction_status(chain.0, request).await?.into())
}
