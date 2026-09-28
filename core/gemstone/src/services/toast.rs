use crate::services::localization::GemLocalizedText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemToastIcon {
    Pin,
    Unpin,
    PriceAlert,
}

/// What a state change tells the user once it lands.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemToast {
    pub text: GemLocalizedText,
    pub icon: GemToastIcon,
}

impl GemToast {
    pub fn pinned(name: String, pinned: bool) -> Self {
        Self {
            text: GemLocalizedText::Pinned { name, pinned },
            icon: match pinned {
                true => GemToastIcon::Pin,
                false => GemToastIcon::Unpin,
            },
        }
    }

    pub fn price_alerts(name: String, enabled: bool) -> Self {
        Self {
            text: GemLocalizedText::PriceAlertsToggled { name, enabled },
            icon: GemToastIcon::PriceAlert,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_pin_toast_names_what_was_pinned_and_shows_the_matching_icon() {
        assert_eq!(GemToast::pinned("Bitcoin".to_string(), true).icon, GemToastIcon::Pin);
        assert_eq!(
            GemToast::pinned("Bitcoin".to_string(), false),
            GemToast {
                text: GemLocalizedText::Pinned { name: "Bitcoin".to_string(), pinned: false },
                icon: GemToastIcon::Unpin,
            }
        );
        assert_eq!(GemToast::price_alerts("Ether".to_string(), true).icon, GemToastIcon::PriceAlert);
    }
}
