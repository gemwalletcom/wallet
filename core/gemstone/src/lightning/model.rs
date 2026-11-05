use gem_spark::{LightningInvoice, LightningNetwork, LightningPayment, LightningPaymentDirection, LightningPaymentStatus};

#[uniffi::remote(Enum)]
pub enum LightningNetwork {
    Mainnet,
    Regtest,
}

#[uniffi::remote(Record)]
pub struct LightningInvoice {
    pub invoice: String,
    pub network: LightningNetwork,
    pub amount_sats: Option<u64>,
    pub payment_hash: String,
    pub created_at: u64,
    pub expires_at: u64,
}

#[uniffi::remote(Enum)]
pub enum LightningPaymentDirection {
    Incoming,
    Outgoing,
}

#[uniffi::remote(Enum)]
pub enum LightningPaymentStatus {
    Pending,
    Completed,
    Failed,
}

#[uniffi::remote(Record)]
pub struct LightningPayment {
    pub id: String,
    pub direction: LightningPaymentDirection,
    pub status: LightningPaymentStatus,
    pub amount_sats: u64,
    pub fee_sats: u64,
    pub created_at: u64,
    pub invoice: Option<String>,
}
