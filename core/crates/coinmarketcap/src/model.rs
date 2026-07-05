use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct ListingsResponse {
    pub data: Vec<Listing>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Listing {
    pub id: u64,
    #[serde(default)]
    pub platform: Option<Value>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Info {
    pub logo: String,
    #[serde(default)]
    pub platform: Option<Value>,
    #[serde(default)]
    pub contract_address: Vec<ContractAddress>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ContractAddress {
    pub contract_address: String,
    pub platform: Platform,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Platform {
    pub name: String,
    pub coin: PlatformCoin,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PlatformCoin {
    pub slug: String,
}

impl Listing {
    pub fn is_token(&self) -> bool {
        self.platform.is_some()
    }
}

impl Info {
    pub fn is_token(&self) -> bool {
        self.platform.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_presence_marks_coinmarketcap_assets_as_tokens() {
        let response: Value = serde_json::from_str(include_str!("../testdata/cryptocurrency_info.json")).unwrap();
        let eth: Info = serde_json::from_value(response["data"]["1027"].clone()).unwrap();
        let usdt: Vec<Info> = serde_json::from_value(response["data"]["USDT"].clone()).unwrap();

        assert!(!eth.is_token());
        assert!(usdt[0].is_token());
    }
}
