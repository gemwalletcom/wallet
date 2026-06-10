use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::{Account, WalletId, WalletType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct WalletImport {
    pub wallet_id: WalletId,
    pub wallet_type: WalletType,
    pub accounts: Vec<Account>,
}
