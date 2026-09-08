use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub chain_id: String,
    pub base_token: Token,
    pub info: Option<TokenInfo>,
}

#[derive(Deserialize)]
pub struct Token {
    pub address: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub image_url: Option<String>,
}
