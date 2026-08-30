use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response<T> {
    pub code: i32,
    pub message: String,
    pub result: T,
}

#[derive(Serialize)]
pub struct AccessTokenRequest {
    pub app_key: String,
    pub sign: String,
    pub time: u64,
}

#[derive(Debug, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAddress {
    pub cybercrime: String,
    pub money_laundering: String,
    pub financial_crime: String,
    pub blacklist_doubt: String,
    pub stealing_attack: String,
}

impl SecurityAddress {
    pub fn is_malicious(&self) -> bool {
        self.cybercrime == "1" || self.money_laundering == "1" || self.financial_crime == "1" || self.blacklist_doubt == "1" || self.stealing_attack == "1"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FakeToken {
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B20Token {
    #[serde(default)]
    pub is_b20: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityToken {
    #[serde(default)]
    pub is_honeypot: Option<String>,
    #[serde(default)]
    pub fake_token: Option<FakeToken>,
    #[serde(default)]
    pub b20_token: Option<B20Token>,
    #[serde(default)]
    pub is_airdrop_scam: Option<String>,
    #[serde(default)]
    pub cannot_buy: Option<String>,
    #[serde(default)]
    pub cannot_sell_all: Option<String>,
}

impl SecurityToken {
    pub fn malicious_reason(&self) -> Option<&'static str> {
        if self.is_honeypot.as_deref() == Some("1") {
            Some("is_honeypot")
        } else if self.fake_token.as_ref().is_some_and(|token| token.value == 1) {
            Some("fake_token")
        } else if self.b20_token.as_ref().is_some_and(|token| token.is_b20 == "1") {
            Some("b20_token")
        } else if self.is_airdrop_scam.as_deref() == Some("1") {
            Some("is_airdrop_scam")
        } else if self.cannot_buy.as_deref() == Some("1") {
            Some("cannot_buy")
        } else if self.cannot_sell_all.as_deref() == Some("1") {
            Some("cannot_sell_all")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_accepts_missing_address_data() {
        let response: Response<Option<SecurityAddress>> = serde_json::from_str(r#"{"code":1,"message":"OK","result":null}"#).unwrap();

        assert!(response.result.is_none());
    }

    #[test]
    fn test_fake_token_object_is_malicious() {
        let token: SecurityToken = serde_json::from_str(
            r#"{
                "fake_token": {
                    "true_token_address": "0x55d398326f99059ff775485246999027b3197955",
                    "value": 1
                }
            }"#,
        )
        .unwrap();

        assert_eq!(token.malicious_reason(), Some("fake_token"));
    }

    #[test]
    fn test_string_risk_flags_remain_supported() {
        let token: SecurityToken = serde_json::from_str(r#"{"is_honeypot":"1"}"#).unwrap();

        assert_eq!(token.malicious_reason(), Some("is_honeypot"));
    }

    #[test]
    fn test_b20_token_is_malicious() {
        let token: SecurityToken = serde_json::from_str(r#"{"b20_token":{"is_b20":"1"}}"#).unwrap();

        assert_eq!(token.malicious_reason(), Some("b20_token"));
    }
}
