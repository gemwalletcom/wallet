use chrono::{DateTime, Utc};

use crate::models::custom_types::GemBigUint;
pub use primitives::payment::{
    Payment, PaymentAction, PaymentAmount, PaymentLink, PaymentMerchant, PaymentOptions, PaymentOutcome, PaymentPrice, PaymentQuote, PaymentQuoteData, PaymentQuotes,
    PaymentRequest, PaymentStatus,
};
use primitives::{AssetId, Chain};

pub type GemPayment = Payment;
pub type GemPaymentAction = PaymentAction;
pub type GemPaymentAmount = PaymentAmount;
pub type GemPaymentLink = PaymentLink;
pub type GemPaymentMerchant = PaymentMerchant;
pub type GemPaymentOptions = PaymentOptions;
pub type GemPaymentOutcome = PaymentOutcome;
pub type GemPaymentPrice = PaymentPrice;
pub type GemPaymentQuote = PaymentQuote;
pub type GemPaymentQuoteData = PaymentQuoteData;
pub type GemPaymentQuotes = PaymentQuotes;
pub type GemPaymentRequest = PaymentRequest;
pub type GemPaymentStatus = PaymentStatus;

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
    pub asset_id: Option<AssetId>,
}

#[uniffi::remote(Enum)]
pub enum GemPaymentLink {
    SolanaPay(String),
    WalletConnectPay(String),
}

#[uniffi::remote(Enum)]
pub enum GemPaymentOptions {
    Quotes(GemPaymentQuotes),
    Outcome(GemPaymentOutcome),
}

#[uniffi::remote(Record)]
pub struct GemPaymentQuotes {
    pub merchant: GemPaymentMerchant,
    pub price: Option<GemPaymentPrice>,
    pub expires_at: Option<DateTime<Utc>>,
    pub quotes: Vec<GemPaymentQuote>,
}

#[uniffi::remote(Record)]
pub struct GemPaymentQuote {
    pub id: String,
    pub link: GemPaymentLink,
    pub asset_id: AssetId,
    pub value: GemBigUint,
    pub expires_at: Option<DateTime<Utc>>,
    pub collect_data_url: Option<String>,
    pub provider_data: String,
}

#[uniffi::remote(Record)]
pub struct GemPaymentPrice {
    pub symbol: String,
    pub value: GemBigUint,
    pub decimals: i32,
}

#[uniffi::remote(Record)]
pub struct GemPaymentMerchant {
    pub name: String,
    pub icon_url: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum GemPaymentStatus {
    RequiresAction,
    Processing,
    Succeeded,
    Failed,
    Expired,
    Cancelled,
}

#[uniffi::remote(Record)]
pub struct GemPaymentOutcome {
    pub status: GemPaymentStatus,
    pub transaction_id: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum GemPaymentAction {
    Send {
        chain: Chain,
        recipient: String,
        value: GemBigUint,
        data: String,
    },
}

#[uniffi::remote(Record)]
pub struct GemPaymentQuoteData {
    pub quote: GemPaymentQuote,
    pub action: GemPaymentAction,
}
