use crate::error::FiatQuoteError;
use gem_jwt::{VerifyingKey, decode};
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

use super::{client::TransakClient, models::TransakOrderResponse};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransakWebhookClaims {
    webhook_data: Option<Value>,
    #[serde(rename = "eventID")]
    event_id: Option<String>,
}

impl TransakClient {
    pub async fn decode_webhook_data(&self, jwt: &str) -> Result<Option<TransakOrderResponse>, Box<dyn Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;
        decode_webhook_order(jwt, &access_token)
    }
}

fn decode_webhook_order(jwt: &str, access_token: &str) -> Result<Option<TransakOrderResponse>, Box<dyn Error + Send + Sync>> {
    let key = VerifyingKey::hs256(access_token.as_bytes())?;
    let claims = decode::<TransakWebhookClaims>(&key, jwt).map_err(|_| FiatQuoteError::InvalidRequest("Invalid Transak webhook signature".to_string()))?;

    if claims.event_id.as_deref().is_some_and(|event_id| event_id.starts_with("KYC_")) {
        return Ok(None);
    }

    let Some(webhook_data) = claims.webhook_data else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_value(webhook_data)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_jwt::{SigningKey, encode};

    #[test]
    fn test_decode_webhook_order() {
        let claims: Value = serde_json::from_str(include_str!("../../../testdata/transak/webhook_transaction_completed.json")).unwrap();
        let jwt = encode(&SigningKey::hs256(b"access_token").unwrap(), None, &claims).unwrap();

        assert_eq!(decode_webhook_order(&jwt, "access_token").unwrap().unwrap().id, "order-id");
        assert!(decode_webhook_order(&jwt, "wrong_access_token").is_err());

        let claims: Value = serde_json::from_str(include_str!("../../../testdata/transak/webhook_kyc_approved.json")).unwrap();
        let jwt = encode(&SigningKey::hs256(b"access_token").unwrap(), None, &claims).unwrap();
        assert!(decode_webhook_order(&jwt, "access_token").unwrap().is_none());
    }
}
