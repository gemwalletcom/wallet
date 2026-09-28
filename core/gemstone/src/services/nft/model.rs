use crate::services::localization::GemLocalizedText;
use chrono::{DateTime, Utc};
use primitives::{NFTAssetData, NFTData, VerificationStatus};

use crate::config::social::GemSocialLink;
use crate::models::list::{GemListRow, GemListSectionTitle};
use crate::services::assets::model::GemHeaderActions;
use crate::services::empty_state::GemEmptyState;

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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemNftEntry {
    pub item: GemNftItem,
    pub row: GemNftRow,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemNftListScreen {
    pub title: GemLocalizedText,
    pub offers_receive: bool,
    pub empty_state: GemEmptyState,
    pub syncs_on_appear: bool,
    pub items: Vec<GemNftEntry>,
    pub unverified_row: Option<GemNftUnverifiedRow>,
    pub has_content: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNftRow {
    pub id: String,
    pub title: String,
    pub image_url: String,
    pub count_text: Option<String>,
    pub is_verified: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCollectibleDetails {
    pub is_verified: bool,
    pub header: GemHeaderActions,
    pub actions: Vec<GemCollectibleMenuRow>,
    pub image_actions: Vec<GemCollectibleAction>,
    pub sections: Vec<GemCollectibleSectionGroup>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCollectibleSectionGroup {
    pub title: GemListSectionTitle,
    pub section: GemCollectibleSection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCollectibleAction {
    SaveImage,
    SetAvatar,
    Refresh,
    Report,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemCollectibleMenuRow {
    pub action: GemCollectibleAction,
    pub is_destructive: bool,
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
