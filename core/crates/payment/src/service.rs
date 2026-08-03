use std::sync::Arc;

use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{ChainAddress, PaymentLink, PaymentOptions, PaymentOutcome, PaymentProviderName, PaymentQuote, PaymentQuotes};

use crate::action::PreparedPayment;
use crate::config::PaymentConfig;
use crate::error::PaymentError;
use crate::provider::PaymentProvider;
use crate::wallet_connect_pay::{WALLET_CONNECT_PAY_API_URL, WalletConnectPayService};

pub struct PaymentService {
    providers: Vec<Box<dyn PaymentProvider>>,
}

impl PaymentService {
    pub fn new(provider: Arc<dyn RpcProvider>, config: PaymentConfig) -> Self {
        Self {
            providers: vec![Box::new(WalletConnectPayService::new(
                RpcClient::new(WALLET_CONNECT_PAY_API_URL.to_string(), provider),
                config.wallet_connect_pay,
            ))],
        }
    }

    pub async fn get_options(&self, link: &PaymentLink, addresses: &[ChainAddress]) -> Result<PaymentOptions, PaymentError> {
        self.provider(link.provider)?.get_options(&link.id, addresses).await
    }

    pub async fn get_prepared_payment(
        &self,
        provider: PaymentProviderName,
        quotes: &PaymentQuotes,
        quote: &PaymentQuote,
        addresses: &[ChainAddress],
    ) -> Result<PreparedPayment, PaymentError> {
        let payment = self.provider(provider)?.get_prepared_payment(quotes, quote, addresses).await?;
        payment.validate(addresses)?;
        Ok(payment)
    }

    pub async fn confirm(&self, provider: PaymentProviderName, quote: &PaymentQuote, action_results: Vec<String>) -> Result<PaymentOutcome, PaymentError> {
        self.provider(provider)?.confirm(quote, action_results).await
    }

    pub async fn get_status(&self, provider: PaymentProviderName, payment_id: &str) -> Result<PaymentOutcome, PaymentError> {
        self.provider(provider)?.get_status(payment_id).await
    }

    pub async fn cancel(&self, provider: PaymentProviderName, payment_id: &str) -> Result<(), PaymentError> {
        self.provider(provider)?.cancel(payment_id).await
    }

    fn provider(&self, name: PaymentProviderName) -> Result<&dyn PaymentProvider, PaymentError> {
        self.providers
            .iter()
            .find(|provider| provider.name() == name)
            .map(|provider| provider.as_ref())
            .ok_or(PaymentError::NotSupported)
    }
}
