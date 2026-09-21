use serde::{Deserialize, Serialize};

use crate::{ChainAddress, WalletId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WalletConfiguration {
    // TODO: remove after 2026-12-18, once shipped apps read externally_controlled_accounts (docs/TODO.md X168)
    pub multi_signature_accounts: Vec<ChainAddress>,
    #[serde(default)]
    pub externally_controlled_accounts: Vec<ChainAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WalletConfigurationResult {
    pub wallet_id: WalletId,
    pub configuration: WalletConfiguration,
}
