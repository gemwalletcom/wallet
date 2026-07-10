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
}
