use std::error::Error;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AddressSecurity {
    pub send_ad_by_memo: bool,
    pub has_fraud_transaction: bool,
    pub fraud_token_creator: bool,
    pub is_black_list: bool,
}

impl AddressSecurity {
    pub fn malicious_reason(&self) -> Option<String> {
        let reasons = [
            (self.send_ad_by_memo, "send_ad_by_memo"),
            (self.has_fraud_transaction, "has_fraud_transaction"),
            (self.fraud_token_creator, "fraud_token_creator"),
            (self.is_black_list, "is_black_list"),
        ]
        .into_iter()
        .filter_map(|(flag, reason)| flag.then_some(reason))
        .collect::<Vec<_>>();
        (!reasons.is_empty()).then(|| reasons.join(","))
    }
}

#[derive(Debug, Deserialize)]
pub struct TokenSecurity {
    pub token_level: TokenLevel,
}

#[derive(Debug, Deserialize)]
pub enum TokenLevel {
    #[serde(rename = "0")]
    Unknown,
    #[serde(rename = "1")]
    Neutral,
    #[serde(rename = "2")]
    Normal,
    #[serde(rename = "3")]
    Suspicious,
    #[serde(rename = "4")]
    Unsafe,
}

impl TokenSecurity {
    pub fn malicious_reason(&self) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        match self.token_level {
            TokenLevel::Unknown => Err("Tronscan token security is unknown".into()),
            TokenLevel::Neutral | TokenLevel::Normal => Ok(None),
            TokenLevel::Suspicious => Ok(Some("token_level:suspicious".to_string())),
            TokenLevel::Unsafe => Ok(Some("token_level:unsafe".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_malicious_reason() {
        for (response, expected) in [
            (
                r#"{"send_ad_by_memo":false,"has_fraud_transaction":false,"fraud_token_creator":false,"is_black_list":false}"#,
                None,
            ),
            (
                r#"{"send_ad_by_memo":true,"has_fraud_transaction":false,"fraud_token_creator":false,"is_black_list":false}"#,
                Some("send_ad_by_memo"),
            ),
            (
                r#"{"send_ad_by_memo":false,"has_fraud_transaction":true,"fraud_token_creator":false,"is_black_list":false}"#,
                Some("has_fraud_transaction"),
            ),
            (
                r#"{"send_ad_by_memo":false,"has_fraud_transaction":false,"fraud_token_creator":true,"is_black_list":false}"#,
                Some("fraud_token_creator"),
            ),
            (
                r#"{"send_ad_by_memo":false,"has_fraud_transaction":false,"fraud_token_creator":false,"is_black_list":true}"#,
                Some("is_black_list"),
            ),
            (
                r#"{"send_ad_by_memo":true,"has_fraud_transaction":true,"fraud_token_creator":false,"is_black_list":false}"#,
                Some("send_ad_by_memo,has_fraud_transaction"),
            ),
        ] {
            assert_eq!(serde_json::from_str::<AddressSecurity>(response).unwrap().malicious_reason().as_deref(), expected);
        }
        assert!(serde_json::from_str::<AddressSecurity>(r#"{"has_fraud_transaction":false}"#).is_err());
    }

    #[test]
    fn test_token_malicious_reason() {
        for (level, expected) in [("1", None), ("2", None), ("3", Some("token_level:suspicious")), ("4", Some("token_level:unsafe"))] {
            let response = format!(r#"{{"token_level":"{level}","black_list_type":1,"increase_total_supply":1,"is_proxy":true}}"#);
            assert_eq!(serde_json::from_str::<TokenSecurity>(&response).unwrap().malicious_reason().unwrap().as_deref(), expected);
        }
        let unknown = serde_json::from_str::<TokenSecurity>(r#"{"token_level":"0"}"#).unwrap();
        assert_eq!(unknown.malicious_reason().unwrap_err().to_string(), "Tronscan token security is unknown");
        for response in [r#"{"token_level":"5"}"#, r#"{"token_level":2}"#, r#"{}"#] {
            assert!(serde_json::from_str::<TokenSecurity>(response).is_err());
        }
    }
}
