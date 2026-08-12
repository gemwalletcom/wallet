pub mod client;
pub mod near_intents;
pub mod okx;
pub mod swaps_xyz;

pub use client::SwapClient;
pub use near_intents::NearIntentsProxyClient;
pub use swaps_xyz::SwapsXyzProxyClient;

use crate::responders::{ApiError, ApiResponse};
use primitives::FiatAssets;
use rocket::{State, get, post, serde::json::Json};
use swapper::swaps_xyz::{ActionRequest, ActionResponse};

#[get("/swap/assets")]
pub async fn get_swap_assets(client: &State<SwapClient>) -> Result<ApiResponse<FiatAssets>, ApiError> {
    Ok(client.get_swap_assets().await?.into())
}

#[post("/swaps/near_intents/quote", data = "<body>")]
pub async fn post_near_intents_quote(body: Json<serde_json::Value>, client: &State<NearIntentsProxyClient>) -> Result<Json<serde_json::Value>, ApiError> {
    let response = client.quote(body.0).await?;
    Ok(Json(response))
}

#[post("/swaps/swaps_xyz/action", data = "<body>")]
pub async fn post_swaps_action(body: Json<ActionRequest>, client: &State<SwapsXyzProxyClient>) -> Result<Json<ActionResponse>, ApiError> {
    Ok(Json(client.action(&body).await?))
}
