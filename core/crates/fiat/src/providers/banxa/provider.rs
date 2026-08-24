use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData, FiatTransactionUpdate};
use streamer::FiatWebhook;

use crate::{
    FiatProvider, FiatWebhookRequest,
    error::FiatQuoteError,
    model::{FiatMapping, FiatProviderAsset},
    provider::generate_quote_id,
    providers::banxa::mapper::map_asset_with_limits,
};

use super::{
    client::BanxaClient,
    mapper::{map_order, map_webhook_data},
};

#[async_trait]
impl FiatProvider for BanxaClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let (assets, buy_fiat_currencies) = tokio::try_join!(self.get_assets_buy(), self.get_fiat_currencies_buy())?;
        Ok(assets.into_iter().flat_map(|asset| map_asset_with_limits(asset, &buy_fiat_currencies, &[])).collect())
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .into_iter()
            .map(|country| FiatProviderCountry {
                provider: Self::NAME,
                alpha2: country.id,
                is_allowed: true,
            })
            .collect())
    }

    async fn get_order_status(&self, order_id: &str) -> Result<FiatTransactionUpdate, Box<dyn std::error::Error + Send + Sync>> {
        let order = self.get_order(order_id).await?;
        map_order(order)
    }

    async fn process_webhook(&self, request: FiatWebhookRequest) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        self.verify_webhook(&request)?;
        let order_id = map_webhook_data(request.data).map_err(|_| FiatQuoteError::InvalidWebhook)?;
        Ok(FiatWebhook::OrderId(order_id))
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        let network = FiatMapping::get_network(request_map.asset_symbol.network)?;
        let quote = self.get_quote_buy(&request_map.asset_symbol.symbol, &network, &request.currency, request.amount).await?;

        Ok(FiatQuoteResponse::new(generate_quote_id(), request.amount, quote.crypto_amount))
    }

    async fn get_quote_sell(&self, _request: FiatQuoteRequest, _request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn std::error::Error + Send + Sync>> {
        Err("not supported".into())
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn std::error::Error + Send + Sync>> {
        match data.quote.quote_type {
            FiatQuoteType::Buy => {
                let network = FiatMapping::get_network(data.asset_symbol.network)?;
                let order = self
                    .create_buy_order(
                        data.quote.id,
                        data.quote.fiat_amount,
                        data.quote.fiat_currency,
                        data.asset_symbol.symbol,
                        network,
                        data.wallet_address,
                    )
                    .await?;

                Ok(FiatQuoteUrl {
                    redirect_url: order.checkout_url,
                    provider_transaction_id: Some(order.id),
                })
            }
            FiatQuoteType::Sell => Err("not supported".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FiatProvider, FiatWebhookRequest, providers::banxa::client::BanxaClient};
    use streamer::FiatWebhook;

    #[tokio::test]
    async fn test_process_webhook_accepts_signed_order() {
        let request = FiatWebhookRequest::mock_banxa_signed(include_str!("../../../testdata/banxa/webhook_order_complete.json"));
        let result = BanxaClient::mock().process_webhook(request).await.unwrap();

        assert!(matches!(result, FiatWebhook::OrderId(order_id) if order_id == "banxa_order_123"));
    }

    #[tokio::test]
    async fn test_process_webhook_rejects_missing_signature() {
        let request = FiatWebhookRequest::mock(include_str!("../../../testdata/banxa/webhook_order_complete.json"));

        assert!(BanxaClient::mock().process_webhook(request).await.is_err());
    }

    #[test]
    fn test_map_webhook_data_extracts_order_id() {
        let order_id = super::map_webhook_data(serde_json::from_str(include_str!("../../../testdata/banxa/webhook_order_complete.json")).unwrap()).unwrap();

        assert_eq!(order_id, "banxa_order_123");
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::create_banxa_test_client;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::{FiatProviderName, FiatQuoteRequest};

    #[tokio::test]
    async fn test_banxa_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_banxa_test_client();
        let request = FiatQuoteRequest::mock();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.network = Some("BTC".to_string());

        let quote = FiatProvider::get_quote_buy(&client, request.clone(), mapping).await?;

        assert!(!quote.quote_id.is_empty());
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, request.amount);

        Ok(())
    }

    #[tokio::test]
    async fn test_banxa_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_banxa_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());

        let assets_with_buy_limits = assets.iter().filter(|asset| !asset.buy_limits.is_empty()).count();
        assert!(assets_with_buy_limits > 0);

        let asset_with_limits = assets.iter().find(|asset| !asset.buy_limits.is_empty()).unwrap();
        let first_limit = asset_with_limits.buy_limits.first().unwrap();
        assert!(first_limit.min_amount.is_some() || first_limit.max_amount.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_banxa_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_banxa_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());

        let country = countries.first().unwrap();
        assert_eq!(country.provider, FiatProviderName::Banxa);
        assert!(!country.alpha2.is_empty());

        Ok(())
    }
}
