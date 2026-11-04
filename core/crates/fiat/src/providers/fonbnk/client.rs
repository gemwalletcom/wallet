use std::collections::HashMap;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use gem_client::{ClientExt, ReqwestClient, Target};
use gem_encoding::{EncodingError, decode_base64};
use primitives::FiatProviderName;

use super::model::{FonbnkCurrency, FonbnkOrderLimits, FonbnkOrderLimitsQuery, FonbnkQuote, FonbnkQuoteLeg, FonbnkQuoteRequest};
use super::target::FonbnkTarget;
use super::widget::FonbnkWidget;
use crate::hmac_signature::generate_hmac_signature_with_key;

pub struct FonbnkClient {
    client: ReqwestClient,
    client_id: String,
    api_secret: String,
    pub(super) widget: FonbnkWidget,
    pub(super) webhook_secret: String,
}

impl FonbnkClient {
    pub const NAME: FiatProviderName = FiatProviderName::Fonbnk;

    pub fn new(client: ReqwestClient, client_id: String, api_secret: String, widget_url: String, widget_secret: String, webhook_secret: String) -> Self {
        Self {
            client,
            widget: FonbnkWidget::new(widget_url, client_id.clone(), widget_secret),
            client_id,
            api_secret,
            webhook_secret,
        }
    }

    pub async fn get_currencies(&self) -> Result<Vec<FonbnkCurrency>, Box<dyn Error + Send + Sync>> {
        let target = FonbnkTarget::Currencies;
        let headers = self.signed_headers(&target.path())?;
        Ok(self.client.get(target).headers(headers).await?)
    }

    pub async fn get_order_limits(&self, deposit: &FonbnkQuoteLeg, payout: &FonbnkQuoteLeg) -> Result<FonbnkOrderLimits, Box<dyn Error + Send + Sync>> {
        let target = FonbnkTarget::OrderLimits {
            query: FonbnkOrderLimitsQuery::new(deposit, payout),
        };
        let headers = self.signed_headers(&target.path())?;
        Ok(self.client.get(target).headers(headers).await?)
    }

    pub async fn create_quote(&self, request: &FonbnkQuoteRequest) -> Result<FonbnkQuote, Box<dyn Error + Send + Sync>> {
        let target = FonbnkTarget::Quote;
        let headers = self.signed_headers(&target.path())?;
        Ok(self.client.post(target, request).headers(headers).await?)
    }

    fn signed_headers(&self, endpoint: &str) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis().to_string();
        let signature = sign_request(&self.api_secret, &timestamp, endpoint)?;
        Ok(HashMap::from([
            ("x-client-id".to_string(), self.client_id.clone()),
            ("x-timestamp".to_string(), timestamp),
            ("x-signature".to_string(), signature),
        ]))
    }
}

fn sign_request(api_secret: &str, timestamp: &str, endpoint: &str) -> Result<String, EncodingError> {
    let key = decode_base64(api_secret)?;
    Ok(generate_hmac_signature_with_key(&key, &format!("{timestamp}:{endpoint}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_request() {
        let signature = sign_request("dGVzdF9hcGlfc2VjcmV0", "1700000000000", "/api/v2/order-limits?depositPaymentChannel=bank&depositCurrencyType=fiat").unwrap();

        assert_eq!(signature, "S9y30BkY8f6ROMNLrsnriI0TRurhtg6MFTUA9iZQOKA=");
    }
}
