use super::{
    client::TransakClient,
    mapper::map_order_from_response,
    models::{Data, TransakQuote},
};
use crate::{
    FiatProvider, FiatWebhookRequest,
    model::{FiatMapping, FiatProviderAsset},
    providers::transak::mapper::map_asset_with_limits,
};
use async_trait::async_trait;
use primitives::{FiatProviderCountry, FiatProviderName, FiatQuoteRequest, FiatQuoteResponse, FiatQuoteType, FiatQuoteUrl, FiatQuoteUrlData};
use std::error::Error;
use streamer::FiatWebhook;

#[async_trait]
impl FiatProvider for TransakClient {
    fn name(&self) -> FiatProviderName {
        Self::NAME
    }

    async fn get_assets(&self) -> Result<Vec<FiatProviderAsset>, Box<dyn std::error::Error + Send + Sync>> {
        let (assets, fiat_currencies) = tokio::try_join!(self.get_supported_assets(), self.get_fiat_currencies())?;
        Ok(assets
            .response
            .into_iter()
            .flat_map(|asset| map_asset_with_limits(asset, &fiat_currencies.response))
            .collect::<Vec<FiatProviderAsset>>())
    }

    async fn get_countries(&self) -> Result<Vec<FiatProviderCountry>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .get_countries()
            .await?
            .response
            .into_iter()
            .map(|x| FiatProviderCountry {
                provider: Self::NAME,
                alpha2: x.alpha2,
                is_allowed: x.is_allowed,
            })
            .collect())
    }

    async fn process_webhook(&self, request: FiatWebhookRequest) -> Result<FiatWebhook, Box<dyn std::error::Error + Send + Sync>> {
        let encrypted_data = serde_json::from_value::<Data<String>>(request.data)?;
        let Some(order) = self.decode_webhook_data(&encrypted_data.data).await? else {
            return Ok(FiatWebhook::None);
        };

        Ok(FiatWebhook::Transaction(map_order_from_response(order)))
    }

    async fn get_quote_buy(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let network = request_map.asset_symbol.network.unwrap_or_default();
        let quote = self
            .get_buy_quote(request_map.asset_symbol.symbol, request.currency.clone(), request.amount, network)
            .await?;

        Ok(FiatQuoteResponse::new(quote.quote_id, request.amount, quote.crypto_amount))
    }

    async fn get_quote_sell(&self, request: FiatQuoteRequest, request_map: FiatMapping) -> Result<FiatQuoteResponse, Box<dyn Error + Send + Sync>> {
        let network = request_map.asset_symbol.network.unwrap_or_default();
        let quote = self
            .get_sell_quote(request_map.asset_symbol.symbol, request.currency.clone(), request.amount, network)
            .await?;

        Ok(FiatQuoteResponse::new(quote.quote_id, request.amount, quote.crypto_amount))
    }

    async fn get_quote_url(&self, data: FiatQuoteUrlData) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        let network = data.asset_symbol.network.clone().unwrap_or_default();

        let transak_quote = match data.quote.quote_type {
            FiatQuoteType::Buy => TransakQuote {
                quote_id: data.quote.id.clone(),
                fiat_amount: data.quote.fiat_amount,
                fiat_currency: data.quote.fiat_currency.clone(),
                crypto_currency: data.asset_symbol.symbol.clone(),
                crypto_amount: data.quote.crypto_amount,
                network,
                conversion_price: 0.0,
                total_fee: 0.0,
            },
            FiatQuoteType::Sell => {
                self.get_sell_quote(data.asset_symbol.symbol.clone(), data.quote.fiat_currency.clone(), data.quote.fiat_amount, network)
                    .await?
            }
        };

        let redirect_url = self.redirect_url(transak_quote, &data).await?;

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
    async fn test_transak_get_buy_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_transak_test_client();

        let request = FiatQuoteRequest::mock();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.network = Some("mainnet".to_string());

        let quote = FiatProvider::get_quote_buy(&client, request.clone(), mapping).await?;

        println!("Transak buy quote: {:?}", quote);
        assert!(!quote.quote_id.is_empty());
        assert!(quote.crypto_amount > 0.0);
        assert_eq!(quote.fiat_amount, request.amount);

        Ok(())
    }

    #[tokio::test]
    async fn test_transak_get_assets() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_transak_test_client();
        let assets = FiatProvider::get_assets(&client).await?;

        assert!(!assets.is_empty());
        println!("Found {} Transak assets", assets.len());

        if let Some(asset) = assets.first() {
            assert!(!asset.id.is_empty());
            assert!(!asset.symbol.is_empty());
            println!("Sample Transak asset: {:?}", asset);
        }

        let assets_with_buy_limits = assets.iter().filter(|a| !a.buy_limits.is_empty()).count();
        let assets_with_sell_limits = assets.iter().filter(|a| !a.sell_limits.is_empty()).count();

        println!("Assets with buy limits: {}", assets_with_buy_limits);
        println!("Assets with sell limits: {}", assets_with_sell_limits);

        assert!(assets_with_buy_limits > 0, "Expected at least some assets to have buy limits");

        if let Some(asset_with_limits) = assets.iter().find(|a| !a.buy_limits.is_empty()) {
            println!("Asset with limits: {} has {} buy limits", asset_with_limits.symbol, asset_with_limits.buy_limits.len());

            let first_limit = &asset_with_limits.buy_limits[0];
            assert!(first_limit.min_amount.is_some() || first_limit.max_amount.is_some());
            println!("Sample limit: {:?}", first_limit);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_transak_get_countries() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_transak_test_client();
        let countries = FiatProvider::get_countries(&client).await?;

        assert!(!countries.is_empty());
        println!("Found {} Transak countries", countries.len());

        if let Some(country) = countries.first() {
            assert_eq!(country.provider, FiatProviderName::Transak);
            assert!(!country.alpha2.is_empty());
            println!("Sample Transak country: {:?}", country);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_transak_get_sell_quote() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_transak_test_client();

        let request = FiatQuoteRequest::mock_sell();
        let mut mapping = FiatMapping::mock();
        mapping.asset_symbol.symbol = "BTC".to_string();
        mapping.asset_symbol.network = Some("mainnet".to_string());

        let quote = FiatProvider::get_quote_sell(&client, request.clone(), mapping).await?;

        println!("Transak sell quote: {:?}", quote);
        assert!(!quote.quote_id.is_empty());
        assert_eq!(quote.fiat_amount, request.amount);
        assert!(quote.crypto_amount > 0.0);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{FiatProvider, FiatWebhookRequest, providers::transak::client::TransakClient};
    use primitives::{FiatTransactionStatus, FiatTransactionUpdate};
    use streamer::FiatWebhook;

    #[tokio::test]
    async fn test_process_webhook_accepts_signed_transaction() {
        let claims = serde_json::from_str(include_str!("../../../testdata/transak/webhook_transaction_completed.json")).unwrap();
        let request = FiatWebhookRequest::mock_transak_signed(claims);

        let result = TransakClient::mock().process_webhook(request).await.unwrap();
        let FiatWebhook::Transaction(transaction) = result else {
            panic!("expected transaction webhook");
        };

        assert_eq!(
            transaction,
            FiatTransactionUpdate {
                transaction_id: "quote-id".to_string(),
                provider_transaction_id: Some("order-id".to_string()),
                status: FiatTransactionStatus::Complete,
                transaction_hash: Some("0x123".to_string()),
                fiat_amount: Some(42.0),
                fiat_currency: Some("USD".to_string()),
            },
        );
    }

    #[tokio::test]
    async fn test_process_webhook_rejects_invalid_signature() {
        let claims = serde_json::from_str(include_str!("../../../testdata/transak/webhook_transaction_completed.json")).unwrap();
        let request = FiatWebhookRequest::mock_transak_signed(claims);

        assert!(TransakClient::mock_with_access_token("wrong_access_token").process_webhook(request).await.is_err());
    }

    #[tokio::test]
    async fn test_process_webhook_ignores_kyc_event() {
        let claims = serde_json::from_str(include_str!("../../../testdata/transak/webhook_kyc_approved.json")).unwrap();
        let request = FiatWebhookRequest::mock_transak_signed(claims);

        assert!(matches!(TransakClient::mock().process_webhook(request).await.unwrap(), FiatWebhook::None));
    }
}
