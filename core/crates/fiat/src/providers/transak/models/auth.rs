use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct CachedToken {
    pub access_token: String,
    pub expires_at: SystemTime,
}

impl CachedToken {
    pub fn is_valid(&self) -> bool {
        SystemTime::now() < self.expires_at
    }
}

impl From<TokenResponse> for CachedToken {
    fn from(token: TokenResponse) -> Self {
        Self {
            access_token: token.access_token,
            expires_at: UNIX_EPOCH + Duration::from_secs(token.expires_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_uses_provider_expiration() {
        let valid: CachedToken = serde_json::from_str::<TokenResponse>(r#"{"accessToken":"valid","expiresAt":4102444800}"#).unwrap().into();
        let expired: CachedToken = serde_json::from_str::<TokenResponse>(r#"{"accessToken":"expired","expiresAt":1}"#).unwrap().into();

        assert!(valid.is_valid());
        assert!(!expired.is_valid());
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWidgetUrlRequest {
    #[serde(rename = "widgetParams")]
    pub params: HashMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWidgetUrlResponse {
    pub widget_url: String,
}
