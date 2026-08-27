use std::error::Error;

use fiat::{FiatClient, FiatDeviceContext, FiatWebhookRequest};
use primitives::{FiatAssets, FiatQuoteRequest, FiatQuoteType, FiatQuoteUrl, FiatQuotes, FiatTransactionData};
use storage::{Database, DevicesRepository, FiatRepository};

pub struct FiatQuotesClient {
    database: Database,
    fiat_client: FiatClient,
}

impl FiatQuotesClient {
    pub fn new(database: Database, fiat_client: FiatClient) -> Self {
        Self { database, fiat_client }
    }

    pub async fn get_quotes(&self, request: FiatQuoteRequest) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_quotes(request).await
    }

    pub async fn get_device_quotes(&self, request: FiatQuoteRequest, context: &FiatDeviceContext) -> Result<FiatQuotes, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_device_quotes(request, context).await
    }

    pub async fn get_quote_url(&self, quote_id: &str, context: &FiatDeviceContext, locale: &str) -> Result<FiatQuoteUrl, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_quote_url(quote_id, context, locale).await
    }

    pub async fn get_assets(&self, quote_type: FiatQuoteType) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        match quote_type {
            FiatQuoteType::Buy => self.get_on_ramp_assets().await,
            FiatQuoteType::Sell => self.get_off_ramp_assets().await,
        }
    }

    pub async fn get_on_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_on_ramp_assets().await
    }

    pub async fn get_off_ramp_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        self.fiat_client.get_off_ramp_assets().await
    }

    pub async fn process_and_publish_webhook(&self, request: FiatWebhookRequest, provider: &str) -> Result<streamer::FiatWebhookPayload, Box<dyn Error + Send + Sync>> {
        self.fiat_client.process_and_publish_webhook(request, provider).await
    }

    pub fn get_transactions_by_device_wallet_id(&self, device_row_id: i32, wallet_id: i32) -> Result<Vec<FiatTransactionData>, Box<dyn Error + Send + Sync>> {
        let transactions = FiatRepository::get_fiat_transactions_by_device_and_wallet_id(&mut self.database.fiat()?, device_row_id, wallet_id)?;

        Ok(transactions.into_iter().map(fiat::fiat_transaction_info).collect())
    }

    pub fn get_transactions_by_device_id(&self, device_id: &str) -> Result<Vec<FiatTransactionData>, Box<dyn Error + Send + Sync>> {
        let device_row_id = self.database.devices()?.get_device_row_id(device_id)?;
        let transactions = FiatRepository::get_fiat_transactions_by_device_id(&mut self.database.fiat()?, device_row_id)?;

        Ok(transactions.into_iter().map(fiat::fiat_transaction_info).collect())
    }
}
