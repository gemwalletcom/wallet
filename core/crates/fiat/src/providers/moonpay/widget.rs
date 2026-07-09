use crate::hmac_signature::generate_hmac_signature;
use primitives::FiatQuoteType;
use url::Url;

const MOONPAY_BUY_REDIRECT_URL: &str = "https://buy.moonpay.com";
const MOONPAY_SELL_REDIRECT_URL: &str = "https://sell.moonpay.com";

pub struct MoonPayWidget {
    api_key: String,
    secret_key: String,
}

impl MoonPayWidget {
    pub fn new(api_key: String, secret_key: String) -> Self {
        Self { api_key, secret_key }
    }

    pub fn redirect_url(&self, quote_type: FiatQuoteType, amount: f64, symbol: &str, wallet_address: &str, external_transaction_id: &str, ip_address: &str) -> String {
        let mut url = Url::parse(Self::base_url(&quote_type)).unwrap();
        url.query_pairs_mut()
            .append_pair("apiKey", &self.api_key)
            .append_pair("externalTransactionId", external_transaction_id);

        match quote_type {
            FiatQuoteType::Buy => {
                url.query_pairs_mut()
                    .append_pair("baseCurrencyAmount", &amount.to_string())
                    .append_pair("currencyCode", symbol)
                    .append_pair("walletAddress", wallet_address)
                    .append_pair("allowedIpAddress", &self.sign(ip_address));
            }
            FiatQuoteType::Sell => {
                url.query_pairs_mut()
                    .append_pair("baseCurrencyCode", symbol)
                    .append_pair("baseCurrencyAmount", &amount.to_string())
                    .append_pair("refundWalletAddress", wallet_address);
            }
        };

        self.sign_url(url)
    }

    fn base_url(quote_type: &FiatQuoteType) -> &'static str {
        match quote_type {
            FiatQuoteType::Buy => MOONPAY_BUY_REDIRECT_URL,
            FiatQuoteType::Sell => MOONPAY_SELL_REDIRECT_URL,
        }
    }

    fn sign_url(&self, mut url: Url) -> String {
        let query = url.query().unwrap();
        let signature = self.sign(&format!("?{query}"));
        url.query_pairs_mut().append_pair("signature", &signature);
        url.as_str().to_string()
    }

    fn sign(&self, message: &str) -> String {
        generate_hmac_signature(&self.secret_key, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redirect_url_includes_allowed_ip_address_for_buy() {
        let ip_address = "203.0.113.1";
        let secret_key = "test_secret_key";
        let url = MoonPayWidget::new("test_api_key".to_string(), secret_key.to_string()).redirect_url(FiatQuoteType::Buy, 100.0, "eth", "0x123", "quote_id", ip_address);
        let url = Url::parse(&url).unwrap();
        let query = url.query().unwrap();
        let (unsigned_query, _) = query.rsplit_once("&signature=").unwrap();
        let signed_query = format!("?{unsigned_query}");
        let pairs = url.query_pairs().collect::<Vec<_>>();
        let allowed_ip_address = generate_hmac_signature(secret_key, ip_address);
        let signature = generate_hmac_signature(secret_key, &signed_query);

        assert_eq!(
            pairs.iter().find(|(key, _)| key == "allowedIpAddress").map(|(_, value)| value.to_string()),
            Some(allowed_ip_address)
        );
        assert_eq!(pairs.iter().find(|(key, _)| key == "signature").map(|(_, value)| value.to_string()), Some(signature));
        assert_eq!(pairs.last().map(|(key, _)| key.as_ref()), Some("signature"));
    }

    #[test]
    fn test_redirect_url_allowed_ip_address_depends_on_ip_address() {
        let widget = MoonPayWidget::new("test_api_key".to_string(), "test_secret_key".to_string());
        let first_url = widget.redirect_url(FiatQuoteType::Buy, 100.0, "eth", "0x123", "quote_id", "203.0.113.1");
        let second_url = widget.redirect_url(FiatQuoteType::Buy, 100.0, "eth", "0x123", "quote_id", "198.51.100.7");
        let first_url = Url::parse(&first_url).unwrap();
        let second_url = Url::parse(&second_url).unwrap();
        let first_allowed_ip_address = first_url.query_pairs().find(|(key, _)| key == "allowedIpAddress").map(|(_, value)| value.to_string());
        let second_allowed_ip_address = second_url.query_pairs().find(|(key, _)| key == "allowedIpAddress").map(|(_, value)| value.to_string());

        assert_ne!(first_allowed_ip_address, second_allowed_ip_address);
    }

    #[test]
    fn test_redirect_url_skips_allowed_ip_address_for_sell() {
        let url = MoonPayWidget::new("test_api_key".to_string(), "test_secret_key".to_string()).redirect_url(FiatQuoteType::Sell, 0.5, "eth", "0x123", "quote_id", "203.0.113.1");
        let url = Url::parse(&url).unwrap();
        let allowed_ip_address = url.query_pairs().find(|(key, _)| key == "allowedIpAddress").map(|(_, value)| value.to_string());

        assert_eq!(allowed_ip_address, None);
    }
}
