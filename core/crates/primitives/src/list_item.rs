use crate::AssetId;
use model_derive::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumString};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString, Model)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[model(swift = "Sendable, Equatable")]
pub enum CoreEmoji {
    Gift,
    Gem,
    Party,
    Warning,
}

impl CoreEmoji {
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Gift => "\u{1f381}",
            Self::Gem => "\u{1f48e}",
            Self::Party => "\u{1f389}",
            Self::Warning => "\u{26a0}\u{fe0f}",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Model)]
#[model(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum CoreListItemIcon {
    Emoji(CoreEmoji),
    Asset(AssetId),
    Image(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, AsRefStr, EnumString, Model)]
#[strum(serialize_all = "camelCase")]
#[serde(rename_all = "camelCase")]
#[model(swift = "Sendable, Equatable")]
pub enum CoreListItemBadge {
    New,
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Sendable, Equatable")]
#[serde(rename_all = "camelCase")]
pub struct CoreListItem {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subvalue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<CoreListItemIcon>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<CoreListItemBadge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
