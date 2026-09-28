use crate::responders::{ApiError, ApiResponse};
use primitives::ReferralLeaderboard;
use rocket::{State, get};
use services::rewards::RewardsClient;

#[get("/rewards/leaderboard")]
pub async fn get_rewards_leaderboard(client: &State<RewardsClient>) -> Result<ApiResponse<ReferralLeaderboard>, ApiError> {
    Ok(client.get_rewards_leaderboard().await?.into())
}
