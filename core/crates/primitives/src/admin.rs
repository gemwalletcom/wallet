use serde::{Deserialize, Serialize};

use crate::{Chain, Device, WalletId, WalletSource};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminDevice {
    pub device: Device,
    pub price_alert_count: i64,
    pub wallets: Vec<AdminWalletOverview>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminWalletOverview {
    pub id: WalletId,
    pub source: WalletSource,
    pub username: Option<String>,
    pub subscription_count: usize,
    pub transaction_count: i64,
    pub fiat_transaction_count: i64,
    pub nft_count: i64,
    pub chains: Vec<Chain>,
}
