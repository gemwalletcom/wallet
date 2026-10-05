use primitives::rewards::RewardRedemptionOption;
use rewards::RewardsRedemptionError;

pub(crate) fn redemption_rejection(points: i32, option: &RewardRedemptionOption) -> Option<RewardsRedemptionError> {
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
        assert_eq!(redemption_rejection(option.points - 1, &option), Some(RewardsRedemptionError::NotEnoughPoints));
        assert_eq!(
            redemption_rejection(option.points, &RewardRedemptionOption { remaining: Some(0), ..option.clone() }),
            Some(RewardsRedemptionError::OptionNotAvailable)
        );
        assert!(redemption_rejection(option.points, &RewardRedemptionOption { remaining: None, ..option }).is_none());
    }
}
