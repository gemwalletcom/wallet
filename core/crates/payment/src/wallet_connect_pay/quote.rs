use chrono::{DateTime, Utc};
use primitives::{Chain, PaymentAmount, PaymentMerchant, PaymentPrice, PaymentStatus};

#[derive(Debug, Clone, PartialEq)]
pub struct QuotedPayment {
    pub status: PaymentStatus,
    pub expires_at: DateTime<Utc>,
    pub merchant: PaymentMerchant,
    pub price: PaymentPrice,
    pub options: Vec<QuotedOption>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuotedOption {
    pub id: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub chain: Chain,
    pub amount: PaymentAmount,
    pub collect_data_url: Option<String>,
    pub provider_data: String,
}
