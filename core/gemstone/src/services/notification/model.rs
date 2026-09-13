use primitives::{AssetId, CoreListItemIcon, InAppNotification};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNotificationIcon {
    Emoji { glyph: String },
    Asset { asset_id: AssetId },
    Image { url: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNotificationRow {
    pub is_unread: bool,
    pub icon: Option<GemNotificationIcon>,
}

#[uniffi::export]
pub fn notification_row(notification: InAppNotification) -> GemNotificationRow {
    GemNotificationRow {
        is_unread: notification.read_at.is_none(),
        icon: notification.item.icon.map(|icon| match icon {
            CoreListItemIcon::Emoji(emoji) => GemNotificationIcon::Emoji { glyph: emoji.glyph().to_string() },
            CoreListItemIcon::Asset(asset_id) => GemNotificationIcon::Asset { asset_id },
            CoreListItemIcon::Image(url) => GemNotificationIcon::Image { url },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{CoreEmoji, CoreListItem, WalletId};

    #[test]
    fn test_a_notification_is_unread_until_it_is_stamped_and_carries_its_glyph() {
        let item = CoreListItem {
            id: "one".into(),
            title: "Reward".into(),
            subtitle: None,
            value: None,
            subvalue: None,
            icon: Some(CoreListItemIcon::Emoji(CoreEmoji::Gift)),
            badge: None,
            url: None,
        };
        let unread = InAppNotification {
            wallet_id: WalletId::Multicoin("wallet".into()),
            read_at: None,
            created_at: chrono::Utc::now(),
            item,
        };
        let read = InAppNotification { read_at: Some(chrono::Utc::now()), ..unread.clone() };

        assert!(notification_row(unread.clone()).is_unread);
        assert!(!notification_row(read).is_unread);
        assert_eq!(notification_row(unread).icon, Some(GemNotificationIcon::Emoji { glyph: "\u{1f381}".into() }));
    }
}
