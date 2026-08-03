use crate::{PaymentError, PaymentProvider, PreparedPayment};
use async_trait::async_trait;
use gem_client::Client;
use primitives::{ChainAddress, PaymentOptions, PaymentOutcome, PaymentProviderName, PaymentQuote, PaymentQuotes};

use crate::wallet_connect_pay::error::WalletConnectPayError;
use crate::wallet_connect_pay::service::WalletConnectPayService;

#[async_trait]
impl<C: Client + Send + Sync> PaymentProvider for WalletConnectPayService<C> {
    fn name(&self) -> PaymentProviderName {
        PaymentProviderName::WalletConnectPay
    }

    async fn get_options(&self, payment_id: &str, addresses: &[ChainAddress]) -> Result<PaymentOptions, PaymentError> {
        Ok(self.payment_options(payment_id, addresses).await?)
    }

    async fn get_prepared_payment(&self, quotes: &PaymentQuotes, quote: &PaymentQuote, addresses: &[ChainAddress]) -> Result<PreparedPayment, PaymentError> {
        Ok(self.prepare_payment(quotes, quote, addresses).await?)
    }

    async fn confirm(&self, quote: &PaymentQuote, results: Vec<String>) -> Result<PaymentOutcome, PaymentError> {
        Ok(self.confirm_payment(quote, results).await?)
    }

    async fn get_status(&self, payment_id: &str) -> Result<PaymentOutcome, PaymentError> {
        Ok(self.get_payment_status(payment_id).await?)
    }

    async fn cancel(&self, payment_id: &str) -> Result<(), PaymentError> {
        Ok(self.cancel_payment(payment_id).await?)
    }
}

impl From<WalletConnectPayError> for PaymentError {
    fn from(error: WalletConnectPayError) -> Self {
        match error {
            WalletConnectPayError::PaymentNotFound => Self::PaymentNotFound,
            WalletConnectPayError::PaymentExpired => Self::PaymentExpired,
            WalletConnectPayError::QuoteExpired => Self::QuoteExpired,
            WalletConnectPayError::NoPaymentOptions => Self::NoPaymentOptions,
            WalletConnectPayError::UnsupportedAccounts => Self::UnsupportedAccounts,
            WalletConnectPayError::SanctionedUser => Self::Rejected,
            WalletConnectPayError::RateLimited => Self::RateLimited,
            WalletConnectPayError::Unsupported(_) => Self::NotSupported,
            WalletConnectPayError::InvalidRequest(message) => Self::InvalidRequest(message),
            WalletConnectPayError::Network(message) => Self::Network(message),
        }
    }
}
