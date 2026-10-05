use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::WalletListItem;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct WalletAddressItem {
    pub wallet: WalletListItem,
    pub address: String,
}
