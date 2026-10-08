use std::sync::Arc;

use axum::extract::State;
use chain_providers::TransactionIdRequest;
use chrono::{DateTime, Utc};
use primitives::{Transaction, TransactionStateRequest, TransactionUpdate};
use serde::Deserialize;
use services::chain::ChainClient;

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{ChainParam, Path, Query, lenient};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct TransactionQuery {
    #[serde(default, deserialize_with = "lenient")]
    block_number: Option<u64>,
}

#[derive(Deserialize)]
pub struct TransactionStatusQuery {
    sender_address: Option<String>,
    #[serde(default, deserialize_with = "lenient")]
    created_at: Option<u64>,
    #[serde(default, deserialize_with = "lenient")]
    from_timestamp: Option<u64>,
    #[serde(default, deserialize_with = "lenient")]
    block_number: Option<u64>,
}

pub async fn get_transaction(_permission: ChainRead, Path((chain, hash)): Path<(ChainParam, String)>, Query(query): Query<TransactionQuery>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<Option<Transaction>>, ApiError> {
    Ok(client.get_transaction_by_hash(TransactionIdRequest::new(chain.0, hash, query.block_number)).await?.into())
}

pub async fn get_transaction_status(
    _permission: ChainRead,
    Path((chain, hash)): Path<(ChainParam, String)>,
    Query(query): Query<TransactionStatusQuery>,
    State(client): State<Arc<ChainClient>>,
) -> Result<ApiResponse<TransactionUpdate>, ApiError> {
    let created_at = query
        .created_at
        .or(query.from_timestamp)
        .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp as i64, 0))
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH);
    let request = TransactionStateRequest {
        id: hash,
        sender_address: query.sender_address.unwrap_or_default(),
        created_at,
        block_number: query.block_number.unwrap_or_default(),
    };
    Ok(client.get_transaction_status(chain.0, request).await?.into())
}
