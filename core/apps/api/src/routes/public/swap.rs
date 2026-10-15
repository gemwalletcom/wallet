use std::sync::Arc;

use axum::extract::State;
use primitives::FiatAssets;
use services::swap::{BridgersProxyClient, ChainflipProxyClient, NearIntentsProxyClient, RelayProxyClient, SwapClient, SwapsXyzProxyClient};
use swapper::RpcClient;
use swapper::chainflip::broker::DepositChannelRequest;
use swapper::okx::{OkxProviderProxy, QuoteParams, SwapParams, error_response};
use swapper::swaps_xyz::ActionRequest;

use crate::error::ApiError;
use crate::request::Json;
use crate::response::{ApiResponse, JsonProxyResponse};

pub async fn get_swap_assets(State(client): State<Arc<SwapClient>>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.get_swap_assets().await?.into())
}

pub async fn post_near_intents_quote(State(client): State<Arc<NearIntentsProxyClient>>, Json(body): Json<serde_json::Value>) -> Result<JsonProxyResponse, ApiError> {
    Ok(client.quote(body).await?.into())
}

pub async fn post_relay_quote(State(client): State<Arc<RelayProxyClient>>, Json(body): Json<serde_json::Value>) -> Result<JsonProxyResponse, ApiError> {
    Ok(client.quote(body).await?.into())
}

pub async fn post_bridgers_swap(State(client): State<Arc<BridgersProxyClient>>, Json(body): Json<serde_json::Value>) -> Result<JsonProxyResponse, ApiError> {
    Ok(client.swap(body).await?.into())
}

pub async fn post_chainflip_deposit_channel(State(client): State<Arc<ChainflipProxyClient>>, Json(body): Json<DepositChannelRequest>) -> Result<JsonProxyResponse, ApiError> {
    Ok(client.deposit_channel(&body).await?.into())
}

pub async fn post_swaps_xyz_action(State(client): State<Arc<SwapsXyzProxyClient>>, Json(body): Json<ActionRequest>) -> Result<JsonProxyResponse, ApiError> {
    Ok(client.action(&body).await?.into())
}

pub async fn post_okx_quote_v6(State(provider): State<Arc<OkxProviderProxy<RpcClient>>>, Json(body): Json<QuoteParams>) -> axum::Json<serde_json::Value> {
    axum::Json(provider.get_quote(body).await.unwrap_or_else(error_response))
}

pub async fn post_okx_swap_v6(State(provider): State<Arc<OkxProviderProxy<RpcClient>>>, Json(body): Json<SwapParams>) -> axum::Json<serde_json::Value> {
    axum::Json(provider.get_swap(body).await.unwrap_or_else(error_response))
}
