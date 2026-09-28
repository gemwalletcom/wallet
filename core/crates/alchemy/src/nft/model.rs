use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OwnedNftsResponse {
    pub owned_nfts: Vec<OwnedNft>,
    pub page_key: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OwnedNft {
    pub contract_address: String,
    pub token_id: String,
    pub is_spam: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContractMetadata {
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub token_type: Option<String>,
    pub open_sea_metadata: Option<OpenSeaMetadata>,
    pub is_spam: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OpenSeaMetadata {
    pub collection_name: Option<String>,
    pub safelist_request_status: Option<String>,
    pub image_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NftMetadata {
    pub contract: ContractMetadata,
    pub token_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<Image>,
    pub raw: Option<RawMetadata>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub cached_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub png_url: Option<String>,
    pub original_url: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct RawMetadata {
    pub metadata: Option<TokenMetadata>,
}

#[derive(Deserialize, Debug)]
pub struct TokenMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub attributes: Option<Vec<Attribute>>,
}

#[derive(Deserialize, Debug)]
pub struct Attribute {
    pub trait_type: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedNftsQuery {
    pub owner: String,
    pub page_size: usize,
    pub with_metadata: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
}
