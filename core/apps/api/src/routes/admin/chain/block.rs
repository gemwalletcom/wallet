use std::sync::Arc;

use axum::extract::State;
use primitives::Transaction;
use serde::Deserialize;
use services::chain::ChainClient;

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{AddressParam, ChainParam, Path, Query};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct BlockQuery {
    transaction_type: Option<String>,
}

#[derive(Deserialize)]
pub struct FinalizeQuery {
    address: AddressParam,
    transaction_type: Option<String>,
}

pub async fn get_latest_block_number(_permission: ChainRead, Path(chain): Path<ChainParam>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<i64>, ApiError> {
    Ok(client.get_latest_block(chain.0).await?.into())
}

pub async fn get_block_transactions(_permission: ChainRead, Path((chain, block_number)): Path<(ChainParam, i64)>, Query(query): Query<BlockQuery>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.get_block_transactions(chain.0, block_number, query.transaction_type.as_deref()).await?.into())
}

pub async fn get_block_transactions_finalize(
    _permission: ChainRead,
    Path((chain, block_number)): Path<(ChainParam, i64)>,
    Query(query): Query<FinalizeQuery>,
    State(client): State<Arc<ChainClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.get_block_transactions_finalize(chain.0, block_number, vec![query.address.0], query.transaction_type.as_deref()).await?.into())
}
