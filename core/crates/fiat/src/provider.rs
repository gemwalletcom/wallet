use crate::model::{FiatMapping, FiatProviderAsset};
use crate::webhook::FiatWebhookRequest;
use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteUrl, FiatQuoteUrlData, FiatTransactionUpdate, FiatWebhook, PaymentType};

pub const DEFAULT_PAYMENT_METHODS: [PaymentType; 3] = [PaymentType::Card, PaymentType::ApplePay, PaymentType::GooglePay];

pub(crate) fn generate_quote_id() -> String {
    gem_uuid::v4()
}

#[async_trait]
pub trait FiatProvider: Send + Sync {
    fn name(&self) -> FiatProviderName;

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_order_status(&self, _order_id: &str) -> Result<FiatTransactionUpdate, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!("provider {} does not support order status fetch", self.name().id()).into())
    }

    async fn parse_webhook(&self, request: FiatWebhookRequest) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>>;

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_quote_sell(&self, _request: FiatQuoteRequest, _request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        Err(format!("provider {} does not support sell quotes", self.name().id()).into())
    }
    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn std::error::Error + Send + Sync>>;

    async fn payment_methods(&self) -> Vec<PaymentType> {
        DEFAULT_PAYMENT_METHODS.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_quote_id_returns_uuid() {
        let quote_id = generate_quote_id();
        assert_eq!(gem_uuid::version(&quote_id), Some(4));
    }
}
