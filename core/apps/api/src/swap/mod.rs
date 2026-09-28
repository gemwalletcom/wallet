pub mod okx;

use crate::responders::{ApiError, ApiResponse, JsonProxyResponse};
use primitives::FiatAssets;
use rocket::{State, get, post, serde::json::Json};
use services::swap::{NearIntentsProxyClient, SwapClient, SwapsXyzProxyClient};
use swapper::swaps_xyz::ActionRequest;

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<SwapClient>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.get_swap_assets().await?.into())
}

#[post("/swaps/near_intents/quote", data = "<body>")]
pub async fn post_near_intents_quote(body: Json<serde_json::Value>, client: &State<NearIntentsProxyClient>) -> Result<JsonProxyResponse, ApiError> {
    Ok(client.quote(body.0).await?.into())
}

#[post("/swaps/swaps_xyz/action", data = "<body>")]
pub async fn post_swaps_xyz_action(body: Json<ActionRequest>, client: &State<SwapsXyzProxyClient>) -> Result<JsonProxyResponse, ApiError> {
    Ok(client.action(&body).await?.into())
}
