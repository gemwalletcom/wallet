use primitives::rewards::{RewardEventType, RewardRedemptionOption, RewardStatus};

pub fn invite_reward_points(status: RewardStatus) -> i32 {
    match status {
        RewardStatus::Attribution => 0,
        RewardStatus::Unverified | RewardStatus::Pending | RewardStatus::Verified | RewardStatus::Trusted | RewardStatus::Disabled => RewardEventType::InviteNew.points(),
    }
}

pub fn offers_redemptions(status: RewardStatus) -> bool {
    match status {
        RewardStatus::Attribution => false,
        RewardStatus::Unverified | RewardStatus::Pending | RewardStatus::Verified | RewardStatus::Trusted | RewardStatus::Disabled => true,
    }
}

pub fn available_redemption_options(options: Vec<RewardRedemptionOption>) -> Vec<RewardRedemptionOption> {
    options.into_iter().filter(|option| option.remaining.unwrap_or_default() > 0).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invite_reward_points() {
        assert_eq!(invite_reward_points(RewardStatus::Verified), 100);
        assert_eq!(invite_reward_points(RewardStatus::Unverified), 100);
        assert_eq!(invite_reward_points(RewardStatus::Attribution), 0);
    }

    #[test]
    fn test_offers_redemptions() {
        assert!(offers_redemptions(RewardStatus::Verified));
        assert!(offers_redemptions(RewardStatus::Disabled));
        assert!(!offers_redemptions(RewardStatus::Attribution));
    }

    #[test]
    fn test_available_redemption_options() {
        let available = RewardRedemptionOption {
            remaining: Some(3),
            ..RewardRedemptionOption::mock(None)
        };
        let options = vec![
            available.clone(),
            RewardRedemptionOption {
                remaining: Some(0),
                ..RewardRedemptionOption::mock(None)
            },
            RewardRedemptionOption {
                remaining: None,
                ..RewardRedemptionOption::mock(None)
            },
        ];

        assert_eq!(available_redemption_options(options), vec![available]);
    }
}
