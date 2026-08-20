use std::sync::Arc;

use gem_jsonrpc::alien::{RpcClient, RpcProvider};
use primitives::{ChainAddress, PaymentLink, PaymentOptions, PaymentOutcome, PaymentQuote, PaymentQuoteData};

use crate::action;
use crate::error::PaymentError;
use crate::wallet_connect_pay::{WALLET_CONNECT_PAY_API_URL, WalletConnectPayAuth, WalletConnectPayService};

pub struct PaymentService {
    wallet_connect_pay: WalletConnectPayService<RpcClient>,
}

impl PaymentService {
    pub fn new(provider: Arc<dyn RpcProvider>, wallet_connect_pay: WalletConnectPayAuth) -> Self {
        Self {
            wallet_connect_pay: WalletConnectPayService::new(RpcClient::new(WALLET_CONNECT_PAY_API_URL.to_string(), provider), wallet_connect_pay),
        }
    }

    pub async fn get_options(&self, link: &PaymentLink, addresses: &[ChainAddress]) -> Result<PaymentOptions, PaymentError> {
        match link {
            PaymentLink::WalletConnectPay(payment_id) => self.wallet_connect_pay.get_options(payment_id, addresses).await,
            PaymentLink::SolanaPay(_) => Err(PaymentError::NotSupported),
        }
    }

    pub async fn get_quote_data(&self, quote: &PaymentQuote, addresses: &[ChainAddress]) -> Result<PaymentQuoteData, PaymentError> {
        let payment = match &quote.link {
            PaymentLink::WalletConnectPay(payment_id) => self.wallet_connect_pay.get_quote_data(payment_id, quote, addresses).await?,
            PaymentLink::SolanaPay(_) => return Err(PaymentError::NotSupported),
        };
        action::validate(&payment, addresses)?;
        Ok(payment)
    }

    pub async fn confirm(&self, quote: &PaymentQuote, transaction_hash: String) -> Result<PaymentOutcome, PaymentError> {
        match &quote.link {
            PaymentLink::WalletConnectPay(payment_id) => self.wallet_connect_pay.confirm(payment_id, &quote.id, transaction_hash).await,
            PaymentLink::SolanaPay(_) => Err(PaymentError::NotSupported),
        }
    }
}
