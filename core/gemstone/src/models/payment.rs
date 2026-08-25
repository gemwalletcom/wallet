use crate::models::GemApplicationMetadata;
use crate::models::custom_types::GemBigUint;
pub use payment::PaymentTransaction;
pub use primitives::payment::{Payment, PaymentAmount, PaymentLink, PaymentRequest};
use primitives::{AssetId, ChainAddress, TransactionType};

pub type GemPayment = Payment;
pub type GemPaymentAmount = PaymentAmount;
pub type GemPaymentLink = PaymentLink;
pub type GemPaymentRequest = PaymentRequest;
pub type GemPaymentTransaction = PaymentTransaction;

#[uniffi::remote(Enum)]
pub enum GemPayment {
    Request(GemPaymentRequest),
    Link(GemPaymentLink),
}

#[uniffi::remote(Enum)]
pub enum GemPaymentAmount {
    ExactValue(String),
    AtomicValue(GemBigUint),
}

#[uniffi::remote(Record)]
pub struct GemPaymentRequest {
    pub address: String,
    pub amount: Option<GemPaymentAmount>,
    pub memo: Option<String>,
    pub references: Option<Vec<String>>,
    pub asset_id: Option<AssetId>,
}

#[uniffi::remote(Enum)]
pub enum GemPaymentLink {
    SolanaPay { url: String },
}

#[uniffi::remote(Record)]
pub struct GemPaymentTransaction {
    pub merchant: GemApplicationMetadata,
    pub account: ChainAddress,
    pub transaction: String,
    pub transaction_type: TransactionType,
    pub memo: Option<String>,
    pub request: Option<GemPaymentRequest>,
}
