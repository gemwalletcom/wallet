use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::wallet_connector::short_name;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct TransactionAppMetadata {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub icon: Option<String>,
}

impl TransactionAppMetadata {
    pub fn short_name(&self) -> String {
        short_name(&self.name)
    }
}
