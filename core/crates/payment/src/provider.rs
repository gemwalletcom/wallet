use async_trait::async_trait;
use primitives::{ChainAddress, PaymentOptions, PaymentOutcome, PaymentProviderName, PaymentQuote, PaymentQuotes};

use crate::action::PreparedPayment;
use crate::error::PaymentError;

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    fn name(&self) -> PaymentProviderName;

    async fn get_options(&self, payment_id: &str, addresses: &[ChainAddress]) -> Result<PaymentOptions, PaymentError>;
    async fn get_prepared_payment(&self, quotes: &PaymentQuotes, quote: &PaymentQuote, addresses: &[ChainAddress]) -> Result<PreparedPayment, PaymentError>;
    async fn confirm(&self, quote: &PaymentQuote, action_results: Vec<String>) -> Result<PaymentOutcome, PaymentError>;
    async fn get_status(&self, payment_id: &str) -> Result<PaymentOutcome, PaymentError>;
    async fn cancel(&self, payment_id: &str) -> Result<(), PaymentError>;
}
