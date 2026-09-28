use std::error::Error;

use primitives::rewards::{RedemptionResponse, RedemptionResult, RewardRedemptionOption};
use rewards::RewardsRedemptionError;
use storage::{DatabaseClient, RewardsRedemptionsRepository};

pub fn redeem_points(client: &mut DatabaseClient, username: &str, points: i32, option_id: &str, device_id: i32, wallet_id: i32) -> Result<RedemptionResponse, Box<dyn Error + Send + Sync>> {
    if let Some(error) = redemption_rejection(points, &client.get_redemption_option(option_id)?) {
        return Err(error.into());
    }
    let redemption = client.add_redemption(username, option_id, device_id, wallet_id)?;
    let redemption_id = redemption.id;
    Ok(RedemptionResponse {
        result: RedemptionResult { redemption },
        redemption_id,
    })
}

fn redemption_rejection(points: i32, option: &RewardRedemptionOption) -> Option<RewardsRedemptionError> {
    if points < option.points {
        return Some(RewardsRedemptionError::NotEnoughPoints);
    }
    if option.remaining == Some(0) {
        return Some(RewardsRedemptionError::OptionNotAvailable);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redemption_rejection() {
        let option = RewardRedemptionOption::mock(None);

        assert!(redemption_rejection(option.points, &option).is_none());
        assert!(matches!(redemption_rejection(option.points - 1, &option), Some(RewardsRedemptionError::NotEnoughPoints)));
        assert!(matches!(
            redemption_rejection(option.points, &RewardRedemptionOption { remaining: Some(0), ..option.clone() }),
            Some(RewardsRedemptionError::OptionNotAvailable)
        ));
        assert!(redemption_rejection(option.points, &RewardRedemptionOption { remaining: None, ..option }).is_none());
    }
}
