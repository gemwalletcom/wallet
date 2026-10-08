use std::sync::Arc;

use axum::extract::State;
use primitives::{AddressBalances, AssetBalance, ChainAddress, Transaction};
use serde::Deserialize;
use services::chain::ChainClient;

use crate::auth::api_client::ChainRead;
use crate::error::ApiError;
use crate::request::{AddressParam, ChainParam, Path, Query, QueryLimitParam, lenient};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct TransactionsQuery {
    #[serde(default, deserialize_with = "lenient")]
    from_timestamp: Option<u64>,
    #[serde(default)]
    limit: QueryLimitParam,
}

pub async fn get_balances(_permission: ChainRead, Path((chain, address)): Path<(ChainParam, AddressParam)>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<AddressBalances>, ApiError> {
    let request = ChainAddress::new(chain.0, address.0);
    let (coin, staking, assets) = futures::try_join!(client.get_balances_coin(request.clone()), client.get_balances_staking(request.clone()), client.get_balances_assets(request),)?;
    Ok(AddressBalances { coin, staking, assets }.into())
}

pub async fn get_assets(_permission: ChainRead, Path((chain, address)): Path<(ChainParam, AddressParam)>, State(client): State<Arc<ChainClient>>) -> Result<ApiResponse<Vec<AssetBalance>>, ApiError> {
    Ok(client.get_balances_assets(ChainAddress::new(chain.0, address.0)).await?.into())
}

pub async fn get_transactions(
    _permission: ChainRead,
    Path((chain, address)): Path<(ChainParam, AddressParam)>,
    Query(query): Query<TransactionsQuery>,
    State(client): State<Arc<ChainClient>>,
) -> Result<ApiResponse<Vec<Transaction>>, ApiError> {
    Ok(client.get_transactions(ChainAddress::new(chain.0, address.0), query.from_timestamp, query.limit.0).await?.into())
}
