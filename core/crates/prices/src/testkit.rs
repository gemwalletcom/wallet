use std::time::Duration;

use crate::PriceAlertRules;

impl PriceAlertRules {
    pub fn mock() -> Self {
        Self {
            notification_cooldown: Duration::from_secs(86_400),
            price_change_threshold: 5.0,
            rank_divisor: 5.0,
            milestones: vec![],
        }
    }
}
