#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningNetwork {
    Mainnet,
    Regtest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningInvoice {
    pub invoice: String,
    pub network: LightningNetwork,
    pub amount_sats: Option<u64>,
    pub payment_hash: String,
    pub created_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningSendQuote {
    pub invoice: LightningInvoice,
    pub amount_sats: u64,
    pub fee_sats: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningPaymentDirection {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningPaymentStatus {
    Pending,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightningPayment {
    pub id: String,
    pub direction: LightningPaymentDirection,
    pub status: LightningPaymentStatus,
    pub amount_sats: u64,
    pub fee_sats: u64,
    pub created_at: u64,
    pub invoice: Option<String>,
}
