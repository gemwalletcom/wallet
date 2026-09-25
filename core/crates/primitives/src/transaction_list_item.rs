use crate::{AddressName, Asset, Transaction};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Sendable, Equatable, Hashable")]
pub struct TransactionListItem {
    pub transaction: Transaction,
    pub asset: Asset,
    pub assets: Vec<Asset>,
    #[serde(rename = "fromAddress")]
    pub from_address: Option<AddressName>,
    #[serde(rename = "toAddress")]
    pub to_address: Option<AddressName>,
}
