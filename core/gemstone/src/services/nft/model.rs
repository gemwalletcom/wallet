use chrono::{DateTime, Utc};
use primitives::{NFTAssetData, NFTData, VerificationStatus};

use crate::config::social::GemSocialLink;
use crate::models::list::GemListRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemNftList {
    Collections,
    Unverified,
    Collection,
    Avatar,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemNftItem {
    Collection { data: NFTData },
    Asset { data: NFTAssetData },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNftUnverifiedRow {
    pub count_text: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNftRow {
    pub id: String,
    pub title: String,
    pub image_url: String,
    pub count_text: Option<String>,
    pub is_verified: bool,
}

#[uniffi::export]
pub fn nft_rows(items: Vec<GemNftItem>) -> Vec<GemNftRow> {
    items.iter().map(super::rules::row).collect()
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCollectibleDetails {
    pub can_send: bool,
    pub actions: Vec<GemCollectibleAction>,
    pub sections: Vec<GemCollectibleSection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCollectibleAction {
    SaveImage,
    SetAvatar,
    Refresh,
    Report,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemCollectibleSection {
    Status { status: VerificationStatus },
    Info { rows: Vec<GemListRow> },
    Attributes { attributes: Vec<GemCollectibleAttribute> },
    Links { links: Vec<GemSocialLink> },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCollectibleAttribute {
    pub name: String,
    pub value: GemCollectibleAttributeValue,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemCollectibleAttributeValue {
    Text { value: String },
    Date { date: DateTime<Utc> },
}
