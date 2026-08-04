pub mod error;
pub mod remote_types;

use std::sync::Arc;

use payment::PaymentConfig;
use payment::WalletConnectPayAuth;
use payment::{PaymentAction as CorePaymentAction, PaymentService};
use primitives::{Chain, ChainAddress, PaymentLink, PaymentOptions, PaymentOutcome, PaymentProviderName, PaymentQuote, PaymentQuotes};

use crate::alien::{AlienProvider, AlienProviderWrapper};
use crate::message::sign_type::SignMessage;
use crate::models::swap::GemApprovalData;
use crate::payment::error::PaymentError;
use crate::wallet_connect::SignableTransaction;

#[derive(Debug, uniffi::Record)]
pub struct GemPreparedPayment {
    pub quotes: PaymentQuotes,
    pub quote: PaymentQuote,
    pub actions: Vec<PaymentAction>,
}

#[derive(Debug, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum PaymentAction {
    SignMessage { message: SignMessage },
    SignTransaction { chain: Chain, transaction: SignableTransaction },
    SendTransaction { chain: Chain, transaction: SignableTransaction },
    ApproveToken { chain: Chain, approval: GemApprovalData },
}

impl From<CorePaymentAction> for PaymentAction {
    fn from(action: CorePaymentAction) -> Self {
        match action {
            CorePaymentAction::SignMessage { message } => Self::SignMessage { message: message.into() },
            CorePaymentAction::SignTransaction { chain, transaction } => Self::SignTransaction {
                chain,
                transaction: transaction.into(),
            },
            CorePaymentAction::SendTransaction { chain, transaction } => Self::SendTransaction {
                chain,
                transaction: transaction.into(),
            },
            CorePaymentAction::ApproveToken { chain, approval } => Self::ApproveToken { chain, approval },
        }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectPayAuth {
    pub app_id: String,
    pub client_id: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPaymentConfig {
    pub wallet_connect_pay: GemWalletConnectPayAuth,
}

impl From<GemPaymentConfig> for PaymentConfig {
    fn from(config: GemPaymentConfig) -> Self {
        PaymentConfig::new(WalletConnectPayAuth::new(
            config.wallet_connect_pay.app_id,
            config.wallet_connect_pay.client_id,
        ))
    }
}

#[derive(uniffi::Object)]
pub struct GemPaymentService {
    service: PaymentService,
}

#[uniffi::export]
impl GemPaymentService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, config: GemPaymentConfig) -> Self {
        Self {
            service: PaymentService::new(Arc::new(AlienProviderWrapper::new(provider)), config.into()),
        }
    }

    pub async fn get_payment_options(&self, link: PaymentLink, addresses: Vec<ChainAddress>) -> Result<PaymentOptions, PaymentError> {
        self.service.get_options(&link, &addresses).await
    }

    pub async fn get_prepared_payment(
        &self,
        provider: PaymentProviderName,
        quotes: PaymentQuotes,
        quote: PaymentQuote,
        addresses: Vec<ChainAddress>,
    ) -> Result<GemPreparedPayment, PaymentError> {
        let payment = self.service.get_prepared_payment(provider, &quotes, &quote, &addresses).await?;
        Ok(GemPreparedPayment {
            quotes: payment.quotes,
            quote: payment.quote,
            actions: payment.actions.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn confirm_payment(&self, provider: PaymentProviderName, quote: PaymentQuote, action_results: Vec<String>) -> Result<PaymentOutcome, PaymentError> {
        self.service.confirm(provider, &quote, action_results).await
    }

    pub async fn cancel_payment(&self, provider: PaymentProviderName, payment_id: String) -> Result<(), PaymentError> {
        self.service.cancel(provider, &payment_id).await
    }

    pub async fn get_payment_status(&self, provider: PaymentProviderName, payment_id: String) -> Result<PaymentOutcome, PaymentError> {
        self.service.get_status(provider, &payment_id).await
    }
}
