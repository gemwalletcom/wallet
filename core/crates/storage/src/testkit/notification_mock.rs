use primitives::NotificationType;

use crate::NewNotification;

impl NewNotification {
    pub fn mock(wallet_id: i32) -> Self {
        Self {
            wallet_id,
            asset_id: None,
            notification_type: NotificationType::RewardsEnabled,
            metadata: None,
        }
    }
}
