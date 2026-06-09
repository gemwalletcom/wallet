use serde::Deserialize;

pub type TokenStandard = String;

#[derive(Deserialize, Debug)]
pub struct Contract {
    pub address: String,
    pub chain: String,
    pub collection: String,
    pub contract_standard: String,
    pub name: String,
    pub total_supply: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct NftsResponse {
    pub nfts: Vec<NftAsset>,
}

#[derive(Deserialize, Debug)]
pub struct NftResponse {
    pub nft: Nft,
}

#[derive(Deserialize, Clone, Debug)]
pub struct NftAsset {
    pub identifier: String,
    pub contract: String,
    pub token_standard: TokenStandard,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Nft {
    pub identifier: String,
    pub collection: String,
    pub contract: String,
    pub token_standard: TokenStandard,
    pub name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub display_image_url: Option<String>,
    pub original_image_url: Option<String>,
    pub traits: Option<Vec<Trait>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Trait {
    pub trait_type: String,
    pub display_type: Option<String>,
    pub value: serde_json::Value,
}

#[derive(Deserialize)]
pub struct Collection {
    pub collection: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub safelist_status: Option<String>,
    pub opensea_url: Option<String>,
    pub project_url: Option<String>,
    pub discord_url: Option<String>,
    pub telegram_url: Option<String>,
    pub twitter_username: Option<String>,
    pub instagram_username: Option<String>,
}
