pub use payment::PaymentTransaction;
use primitives::ApplicationMetadata;
pub use primitives::payment::{Payment, PaymentAmount, PaymentLink, PaymentRequest};
use primitives::{ChainAddress, TransactionType};

pub type GemPayment = Payment;
pub type GemPaymentAmount = PaymentAmount;
pub type GemPaymentLink = PaymentLink;
pub type GemPaymentRequest = PaymentRequest;
pub type GemPaymentTransaction = PaymentTransaction;

#[uniffi::remote(Record)]
pub struct GemPaymentTransaction {
    pub merchant: ApplicationMetadata,
    pub account: ChainAddress,
    pub transaction: String,
    pub transaction_type: TransactionType,
    pub memo: Option<String>,
    pub request: Option<GemPaymentRequest>,
}
