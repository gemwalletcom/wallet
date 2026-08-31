use serde::{Deserialize, Serialize};

use crate::{Account, WalletId, WalletType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletImport {
    pub wallet_id: WalletId,
    pub wallet_type: WalletType,
    pub accounts: Vec<Account>,
}
