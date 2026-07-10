use crate::error::FiatQuoteError;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde_json::Value;
use std::error::Error;

use super::{client::TransakClient, models::TransakOrderResponse};

impl TransakClient {
    pub async fn decode_webhook_data(&self, jwt: &str) -> Result<TransakOrderResponse, Box<dyn Error + Send + Sync>> {
        let access_token = self.get_access_token().await?;
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.required_spec_claims.clear();

        let token = decode::<Value>(jwt, &DecodingKey::from_secret(access_token.as_bytes()), &validation)
            .map_err(|_| FiatQuoteError::InvalidRequest("Invalid Transak webhook signature".to_string()))?;

        let webhook_data = token
            .claims
            .get("webhookData")
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Transak webhook data".to_string()))?;
        Ok(serde_json::from_value(webhook_data.clone())?)
    }
}
