use primitives::{ApplicationMetadata, ChainAddress, PaymentRequest, TransactionType};

#[derive(Debug, Clone, PartialEq)]
pub struct PaymentTransaction {
    pub merchant: ApplicationMetadata,
    pub account: ChainAddress,
    pub transaction: String,
    pub transaction_type: TransactionType,
    pub memo: Option<String>,
    pub request: Option<PaymentRequest>,
}
