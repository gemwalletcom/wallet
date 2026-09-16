use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub seqno: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressInformation {
    pub balance: String,
}
