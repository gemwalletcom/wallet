use chrono::{DateTime, Utc};
use primitives::{AssetLink, BlockExplorerLink, Chain, NFTAssetData, NFTData, VerificationStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemNftList {
    Collections,
    Unverified,
    Collection,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum GemNftItem {
    Collection { data: NFTData },
    Asset { data: NFTAssetData },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCollectibleDetails {
    pub can_send: bool,
    pub sections: Vec<GemCollectibleSection>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemCollectibleSection {
    Status { status: VerificationStatus },
    Info { rows: Vec<GemCollectibleRow> },
    Attributes { attributes: Vec<GemCollectibleAttribute> },
    Links { links: Vec<AssetLink> },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemCollectibleRow {
    Collection { name: String },
    Network { chain: Chain },
    Contract { identifier: GemCollectibleIdentifier },
    TokenId { identifier: GemCollectibleIdentifier },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCollectibleIdentifier {
    pub value: String,
    pub text: String,
    pub explorer: Option<BlockExplorerLink>,
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
