use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub chain_id: String,
    pub base_token: Token,
    pub info: Option<TokenInfo>,
    pub liquidity: Option<Liquidity>,
    pub volume: Option<Volume>,
}

#[derive(Clone, Deserialize)]
pub struct Token {
    pub address: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub image_url: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct Liquidity {
    pub usd: Option<f64>,
}

#[derive(Clone, Deserialize)]
pub struct Volume {
    pub h24: Option<f64>,
}

#[derive(Deserialize)]
pub struct Meta {
    pub slug: String,
}

#[derive(Deserialize)]
pub struct MetaDetails {
    pub pairs: Vec<Pair>,
}
