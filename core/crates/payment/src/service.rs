use std::sync::Arc;

use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{ChainAddress, PaymentLink, PaymentOptions, PaymentOutcome, PaymentProviderName, PaymentQuote, PaymentQuotes};

use crate::action::PreparedPayment;
use crate::config::PaymentConfig;
use crate::error::PaymentError;
use crate::wallet_connect_pay::{WALLET_CONNECT_PAY_API_URL, WalletConnectPayService};

pub struct PaymentService {
    wallet_connect_pay: WalletConnectPayService<RpcClient>,
}

impl PaymentService {
    pub fn new(provider: Arc<dyn RpcProvider>, config: PaymentConfig) -> Self {
        Self {
            wallet_connect_pay: WalletConnectPayService::new(RpcClient::new(WALLET_CONNECT_PAY_API_URL.to_string(), provider), config.wallet_connect_pay),
        }
    }

    pub async fn get_options(&self, link: &PaymentLink, addresses: &[ChainAddress]) -> Result<PaymentOptions, PaymentError> {
        match link.provider {
            PaymentProviderName::WalletConnectPay => self.wallet_connect_pay.payment_options(&link.id, addresses).await,
            PaymentProviderName::SolanaPay => Err(PaymentError::NotSupported),
        }
    }

    pub async fn get_prepared_payment(
        &self,
        provider: PaymentProviderName,
        quotes: &PaymentQuotes,
        quote: &PaymentQuote,
        addresses: &[ChainAddress],
    ) -> Result<PreparedPayment, PaymentError> {
        let payment = match provider {
            PaymentProviderName::WalletConnectPay => self.wallet_connect_pay.prepare_payment(quotes, quote, addresses).await?,
            PaymentProviderName::SolanaPay => return Err(PaymentError::NotSupported),
        };
        payment.validate(addresses)?;
        Ok(payment)
    }

    pub async fn confirm(&self, provider: PaymentProviderName, quote: &PaymentQuote, action_results: Vec<String>) -> Result<PaymentOutcome, PaymentError> {
        match provider {
            PaymentProviderName::WalletConnectPay => self.wallet_connect_pay.confirm_payment(quote, action_results).await,
            PaymentProviderName::SolanaPay => Err(PaymentError::NotSupported),
        }
    }

    pub async fn get_status(&self, provider: PaymentProviderName, payment_id: &str) -> Result<PaymentOutcome, PaymentError> {
        match provider {
            PaymentProviderName::WalletConnectPay => self.wallet_connect_pay.get_payment_status(payment_id).await,
            PaymentProviderName::SolanaPay => Err(PaymentError::NotSupported),
        }
    }

    pub async fn cancel(&self, provider: PaymentProviderName, payment_id: &str) -> Result<(), PaymentError> {
        match provider {
            PaymentProviderName::WalletConnectPay => self.wallet_connect_pay.cancel_payment(payment_id).await,
            PaymentProviderName::SolanaPay => Err(PaymentError::NotSupported),
        }
    }
}
