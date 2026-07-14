use crate::{
    FiatProvider, FiatWebhookRequest,
    model::{FiatMapping, FiatProviderAsset},
    provider::generate_quote_id,
};
use async_trait::async_trait;
use std::error::Error;
use streamer::FiatWebhook;

use super::{
    client::MoonPayClient,
    mapper::{map_order, map_webhook_data},
};
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData};

#[async_trait]
impl FiatProvider for MoonPayClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let assets = self.get_assets().await?.into_iter().flat_map(Self::map_asset).collect::<Vec<FiatProviderAsset>>();
        Ok(assets)
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME,
                alpha2: x.alpha2,
                is_allowed: x.is_allowed,
            })
            .collect())
    }

    async fn process_webhook(&self, request: FiatWebhookRequest) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        self.verify_webhook(&request)?;
        let payload = map_webhook_data(request.data)?;
        Ok(FiatWebhook::Transaction(map_order(payload)))
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_buy_quote(request_map.asset_symbol.symbol.to_lowercase(), request.currency.to_lowercase(), request.amount)
            .await?;

        Ok(FiatQuoteResponse::new(generate_quote_id(), request.amount, quote.quote_currency_amount))
    }

    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let quote = self
            .get_sell_quote(request_map.asset_symbol.symbol.to_lowercase(), request.currency.to_lowercase(), request.amount)
            .await?;

        Ok(FiatQuoteResponse::new(generate_quote_id(), quote.quote_currency_amount, quote.base_currency_amount))
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let amount = match data.quote.quote_type {
            FiatQuoteType::Buy => data.quote.fiat_amount,
            FiatQuoteType::Sell => data.quote.crypto_amount,
        };

        let redirect_url = self.quote_redirect_url(
            data.quote.quote_type,
            amount,
            &data.asset_symbol.symbol,
            &data.wallet_address,
            &data.quote.id,
            &data.ip_address,
        );

        Ok(FiatQuoteUrl {
            redirect_url,
            provider_transaction_id: None,
        })
    }
}

#[cfg(all(test, feature = "fiat_integration_tests"))]
mod fiat_integration_tests {
    use crate::testkit::*;
    use crate::{FiatProvider, model::FiatMapping};
    use primitives::{FiatProviderName, FiatQuoteRequest};

    #[tokio::test]
    async fn test_moonpay_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_moonpay_test_client();

        let request = FiatQuoteRequest::mock();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.network = Some("bitcoin".to_string());

        let quote = FiatProvider::get_quote_buy(&client, request.clone(), mapping).await?;

        println!("MoonPay buy quote: {:?}", quote);
        assert!(!quote.quote_id.is_empty());
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, request.amount);

        Ok(())
    }

    #[tokio::test]
    async fn test_moonpay_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_moonpay_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());
        println!("Found {} MoonPay assets", assets.len());

        if let Some(asset) = assets.first() {
            assert!(!asset.id.is_empty());
            assert!(!asset.symbol.is_empty());
            println!("Sample MoonPay asset: {:?}", asset);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_moonpay_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_moonpay_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());
        println!("Found {} MoonPay countries", countries.len());

        if let Some(country) = countries.first() {
            assert_eq!(country.provider, FiatProviderName::MoonPay);
            assert!(!country.alpha2.is_empty());
            println!("Sample MoonPay country: {:?}", country);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{FiatProvider, FiatWebhookRequest, providers::moonpay::client::MoonPayClient};

    #[tokio::test]
    async fn test_process_webhook_rejects_missing_signature() {
        let raw_body = include_str!("../../../testdata/moonpay/webhook_buy_complete.json");
        let request = FiatWebhookRequest::mock(raw_body);

        assert!(MoonPayClient::mock().process_webhook(request).await.is_err());
    }
}
