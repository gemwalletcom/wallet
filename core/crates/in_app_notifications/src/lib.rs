use links::Deeplink;
use localizer::LanguageLocalizer;
use number_formatter::{ValueFormatter, ValueStyle};
use primitives::{CoreEmoji, CoreListItem, CoreListItemBadge, CoreListItemIcon, InAppNotification, JsonDecode, NotificationData, NotificationRewardsMetadata, NotificationRewardsRedeemMetadata, NotificationType, WalletId};

pub fn points_value(points: i32) -> String {
    format!("{} {}", ValueFormatter::format_f64(ValueStyle::Full, f64::from(points.unsigned_abs())), CoreEmoji::Gem.glyph())
}

pub fn map_notification(notification: NotificationData, localizer: &LanguageLocalizer) -> Option<InAppNotification> {
    let wallet_id = WalletId::from_id(&notification.wallet_id)?;
    let item = map_to_list_item(&notification, localizer);

    Some(InAppNotification {
        wallet_id,
        read_at: notification.read_at,
        created_at: notification.created_at,
        item,
    })
}

fn notification_item(id: String, title: String, subtitle: Option<String>, value: Option<String>, subvalue: Option<String>, emoji: CoreEmoji, badge: Option<CoreListItemBadge>, url: Option<String>) -> CoreListItem {
    CoreListItem {
        id,
        title,
        subtitle,
        value,
        subvalue,
        icon: Some(CoreListItemIcon::Emoji(emoji)),
        badge,
        url,
    }
}

fn map_to_list_item(notification: &NotificationData, localizer: &LanguageLocalizer) -> CoreListItem {
    let id = notification.id.to_string();
    let url = Some(Deeplink::Rewards { code: None }.to_gem_url());

    match notification.notification_type {
        NotificationType::ReferralJoined => {
            let points = notification.metadata.decode::<NotificationRewardsMetadata>().and_then(|m| m.points).unwrap_or(0);
            notification_item(
                id,
                localizer.notification_reward_pending_title(),
                Some(localizer.notification_reward_invite_description()),
                Some(format!("+{}", points_value(points))),
                None,
                CoreEmoji::Party,
                Some(CoreListItemBadge::New),
                url,
            )
        }
        NotificationType::RewardsEnabled => notification_item(
            id,
            localizer.notification_rewards_enabled_title(),
            Some(localizer.notification_rewards_enabled_description()),
            None,
            None,
            CoreEmoji::Gift,
            None,
            url,
        ),
        NotificationType::RewardsCodeDisabled => notification_item(
            id,
            localizer.notification_rewards_disabled_title(),
            Some(localizer.notification_rewards_disabled_description()),
            None,
            None,
            CoreEmoji::Warning,
            Some(CoreListItemBadge::New),
            url,
        ),
        NotificationType::RewardsRedeemed => {
            let redeem = notification.metadata.decode::<NotificationRewardsRedeemMetadata>();
            let points = redeem.as_ref().map(|m| m.points).unwrap_or(0);
            let value = redeem.as_ref().and_then(|m| {
                let asset = notification.asset.as_ref()?;
                ValueFormatter::format_with_symbol(ValueStyle::Auto, &m.value, asset.decimals, &asset.symbol).ok()
            });
            let subtitle = Some(localizer.notification_reward_redeemed_description(&points_value(points), value.as_deref()));
            let subvalue = Some(format!("-{}", points_value(points)));
            notification_item(
                id,
                localizer.notification_rewards_redeem_points_title(),
                subtitle,
                value.map(|value| format!("+{}", value)),
                subvalue,
                CoreEmoji::Gift,
                None,
                url,
            )
        }
        NotificationType::RewardsCreateUsername => {
            let points = notification.metadata.decode::<NotificationRewardsMetadata>().and_then(|m| m.points).unwrap_or(0);
            notification_item(
                id,
                localizer.notification_reward_title(&points_value(points)),
                Some(localizer.notification_reward_create_username_description()),
                Some(format!("+{}", points_value(points))),
                None,
                CoreEmoji::Gem,
                Some(CoreListItemBadge::New),
                url,
            )
        }
        NotificationType::RewardsInvite => {
            let points = notification.metadata.decode::<NotificationRewardsMetadata>().and_then(|m| m.points).unwrap_or(0);
            notification_item(
                id,
                localizer.notification_reward_title(&points_value(points)),
                Some(localizer.notification_reward_invite_description()),
                Some(format!("+{}", points_value(points))),
                None,
                CoreEmoji::Party,
                Some(CoreListItemBadge::New),
                url,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_points_value() {
        assert_eq!(points_value(1575), "1,575 💎");
        assert_eq!(points_value(-650), "650 💎");
        assert_eq!(points_value(0), "0 💎");
    }
}
