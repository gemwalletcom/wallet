use primitives::{AssetId, CoreListItemIcon, InAppNotification, UrlAction};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNotificationIcon {
    Emoji { glyph: String },
    Asset { asset_id: AssetId, icon: crate::services::assets::icon::GemAssetIcon },
    Image { url: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNotificationRow {
    pub title: String,
    pub subtitle: Option<String>,
    pub value: Option<String>,
    pub subvalue: Option<String>,
    pub destination: Option<GemNotificationDestination>,
    pub is_unread: bool,
    pub icon: Option<GemNotificationIcon>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNotificationDestination {
    InApp { action: UrlAction },
    Web { url: String },
}

fn destination(url: Option<&str>) -> Option<GemNotificationDestination> {
    let url = url.filter(|url| !url.is_empty())?;
    Some(match ::payment::classify_url(url) {
        Some(action) => GemNotificationDestination::InApp { action },
        None => GemNotificationDestination::Web { url: url.to_string() },
    })
}

#[uniffi::export]
pub fn notification_rows(notifications: Vec<InAppNotification>) -> Vec<GemNotificationRow> {
    notifications.into_iter().map(notification_row).collect()
}

pub fn notification_row(notification: InAppNotification) -> GemNotificationRow {
    GemNotificationRow {
        title: notification.item.title.clone(),
        subtitle: notification.item.subtitle.clone(),
        value: notification.item.value.clone(),
        subvalue: notification.item.subvalue.clone(),
        destination: destination(notification.item.url.as_deref()),
        is_unread: notification.read_at.is_none(),
        icon: notification.item.icon.map(|icon| match icon {
            CoreListItemIcon::Emoji(emoji) => GemNotificationIcon::Emoji { glyph: emoji.glyph().to_string() },
            CoreListItemIcon::Asset(asset_id) => GemNotificationIcon::Asset {
                icon: crate::services::assets::icon::asset_icon(&asset_id),
                asset_id,
            },
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
        let read = InAppNotification {
            read_at: Some(chrono::Utc::now()),
            ..unread.clone()
        };

        assert!(notification_row(unread.clone()).is_unread);
        assert!(!notification_row(read).is_unread);
        let row = notification_row(unread);
        assert_eq!(row.icon, Some(GemNotificationIcon::Emoji { glyph: "\u{1f381}".into() }));
        assert_eq!(row.title, "Reward", "the row carries the text the screen shows");
    }

    #[test]
    fn test_a_notification_opens_in_the_app_when_the_url_is_one_the_app_knows() {
        assert_eq!(destination(None), None);
        assert_eq!(destination(Some("")), None);
        assert_eq!(
            destination(Some("https://gemwallet.com/blog")),
            Some(GemNotificationDestination::Web {
                url: "https://gemwallet.com/blog".to_string()
            }),
            "a web page the app cannot route opens outside it on both apps"
        );
        assert!(matches!(destination(Some("gem://tokens/bitcoin")), Some(GemNotificationDestination::InApp { .. })));
    }
}
