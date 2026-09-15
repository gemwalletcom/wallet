use primitives::{AssetId, ChainAddress, PaymentInvoice, PaymentRequest, TransactionType};

#[derive(Debug, Clone, PartialEq)]
pub struct PaymentTransaction {
    pub invoice: PaymentInvoice,
    pub account: ChainAddress,
    pub transaction: String,
    pub transaction_type: TransactionType,
    pub memo: Option<String>,
    pub request: Option<PaymentRequest>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentLoad {
    Sign { transaction: PaymentTransaction },
    Verify { invoice: PaymentInvoice, asset_id: AssetId, url: String },
}

impl PaymentLoad {
    pub(crate) fn account(&self) -> Option<&ChainAddress> {
        match self {
            Self::Sign { transaction } => Some(&transaction.account),
            Self::Verify { .. } => None,
        }
    }
}
