use std::sync::Arc;

use axum::extract::State;
use primitives::{Markets, ReferralLeaderboard};
use services::prices::MarketsClient;
use services::rewards::RewardsClient;

use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn get_markets(State(client): State<Arc<MarketsClient>>) -> Result<ApiResponse<Markets>, ApiError> {
    Ok(client.get_markets().await?.into())
}

pub async fn get_rewards_leaderboard(State(client): State<Arc<RewardsClient>>) -> Result<ApiResponse<ReferralLeaderboard>, ApiError> {
    Ok(client.get_rewards_leaderboard().await?.into())
}
