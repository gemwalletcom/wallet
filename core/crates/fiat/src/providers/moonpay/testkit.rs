use super::models::{Asset, CurrencyMetadata, FiatCurrencyType};
use crate::{FiatWebhookRequest, hmac_signature::generate_hmac_signature_hex};
use gem_client::ReqwestClient;

use super::client::MoonPayClient;

pub const TEST_WEBHOOK_SIGNING_KEY: &str = "test_webhook_key";

impl MoonPayClient {
    pub fn mock() -> Self {
        Self::new(
            ReqwestClient::new(String::new(), gem_client::reqwest_client()),
            String::new(),
            String::new(),
            TEST_WEBHOOK_SIGNING_KEY.to_string(),
        )
    }
}

impl FiatWebhookRequest {
    pub fn mock_moonpay_signed(raw_body: &str) -> Self {
        let timestamp = "1492774577";
        let signed_payload = format!("{timestamp}.{raw_body}");
        let signature = generate_hmac_signature_hex(TEST_WEBHOOK_SIGNING_KEY, &signed_payload);
        Self::mock_moonpay_with_signature(raw_body, &format!("t={timestamp},s={signature}"))
    }

    pub fn mock_moonpay_with_signature(raw_body: &str, signature: &str) -> Self {
        Self::mock_with_header(raw_body, "moonpay-signature-v2", signature)
    }
}

impl Asset {
    pub fn mock(code: &str, network_code: &str, contract_address: Option<&str>, is_base_asset: bool) -> Self {
        Self {
            code: code.to_string(),
            metadata: Some(CurrencyMetadata {
                contract_address: contract_address.map(|s| s.to_string()),
                network_code: network_code.to_string(),
            }),
            is_suspended: Some(false),
            is_base_asset: Some(is_base_asset),
            is_sell_supported: Some(true),
            not_allowed_countries: None,
            currency_type: FiatCurrencyType::Crypto,
            min_buy_amount: None,
            max_buy_amount: None,
            min_sell_amount: None,
            max_sell_amount: None,
        }
    }
}
