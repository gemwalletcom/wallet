use crate::error::FiatQuoteError;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
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
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    validation.required_spec_claims.clear();

    let token = decode::<TransakWebhookClaims>(jwt, &DecodingKey::from_secret(access_token.as_bytes()), &validation)
        .map_err(|_| FiatQuoteError::InvalidRequest("Invalid Transak webhook signature".to_string()))?;

    if token.claims.event_id.as_deref().is_some_and(|event_id| event_id.starts_with("KYC_")) {
        return Ok(None);
    }

    let Some(webhook_data) = token.claims.webhook_data else {
        return Ok(None);
    };
    Ok(Some(serde_json::from_value(webhook_data)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    fn encode_claims(claims: Value, access_token: &str) -> String {
        encode(&Header::default(), &claims, &EncodingKey::from_secret(access_token.as_bytes())).unwrap()
    }

    #[test]
    fn test_decode_webhook_order() {
        let claims = serde_json::from_str(include_str!("../../../testdata/transak/webhook_transaction_completed.json")).unwrap();
        let jwt = encode_claims(claims, "access_token");

        assert_eq!(decode_webhook_order(&jwt, "access_token").unwrap().unwrap().id, "order-id");
        assert!(decode_webhook_order(&jwt, "wrong_access_token").is_err());

        let claims = serde_json::from_str(include_str!("../../../testdata/transak/webhook_kyc_approved.json")).unwrap();
        let jwt = encode_claims(claims, "access_token");
        assert!(decode_webhook_order(&jwt, "access_token").unwrap().is_none());
    }
}
